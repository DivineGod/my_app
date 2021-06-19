#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m::asm::delay;
use my_app as _;

use embedded_hal::digital::v2::OutputPin;
use rtic::app;
use rtic::cyccnt::U32Ext;
use stm32f3xx_hal::gpio::{gpioa, gpioc, Input, Output, PullUp, PushPull};
use stm32f3xx_hal::prelude::*;
use stm32f3xx_hal::timer;
use stm32f3xx_hal::timer::Timer;
use stm32f3xx_hal::usb::Peripheral;
use stm32f3xx_hal::usb::UsbBus;
use usb_device::bus;
use usb_device::class::UsbClass as _;
use usb_device::device::UsbDevice;
use usb_device::device::UsbDeviceBuilder;
use usb_device::device::UsbVidPid;
use usbd_hid::descriptor::generator_prelude::*;
use usbd_hid::descriptor::KeyboardReport;
use usbd_hid::hid_class::HIDClass;

const PERIOD: u32 = 10_000_000;

// Generic keyboard from
// https://github.com/obdev/v-usb/blob/master/usbdrv/USB-IDs-for-free.txt
const PID: u16 = 0x27db;
const VID: u16 = 0x16c0;

// We need to pass monotonic = rtic::cyccnt::CYCCNT to use schedule feature fo RTIC
#[app(device = stm32f3xx_hal::pac, peripherals = true, monotonic = rtic::cyccnt::CYCCNT)]
const APP: () = {
    // Global resources (global variables) are defined here and initialized with the
    // `LateResources` struct in init
    struct Resources {
        usb_device: UsbDevice<'static, UsbBus<Peripheral>>,
        usb_class: HIDClass<'static, UsbBus<Peripheral>>,
        button: gpioa::PA6<Input<PullUp>>,
        // output: gpioa::PA5<Output<PushPull>>,
        led: gpioc::PC13<Output<PushPull>>,
        timer: Timer<stm32f3xx_hal::stm32::TIM3>,
    }

    #[init(schedule = [blinker])]
    fn init(cx: init::Context) -> init::LateResources {
        static mut USB_BUS: Option<bus::UsbBusAllocator<UsbBus<Peripheral>>> = None;

        defmt::info!("hi");
        // Enable cycle counter
        let mut core = cx.core;
        core.DWT.enable_cycle_counter();

        let device: stm32f3xx_hal::stm32::Peripherals = cx.device;

        // Setup clocks
        let mut flash = device.FLASH.constrain();
        let mut rcc = device.RCC.constrain();
        let clocks = rcc
            .cfgr
            .use_hse(8.mhz())
            .sysclk(72.mhz())
            .pclk1(36.mhz())
            .pclk2(36.mhz())
            .freeze(&mut flash.acr);
        /*
        let clocks = rcc
            .cfgr
            .use_hse(8.mhz())
            .sysclk(48.mhz())
            .pclk1(24.mhz())
            .pclk2(24.mhz())
            .freeze(&mut flash.acr);
        // */
        assert!(clocks.usbclk_valid());

        let mut gpioa = device.GPIOA.split(&mut rcc.ahb);
        let mut gpioc = device.GPIOC.split(&mut rcc.ahb);

        // Pull the D+ pin down to send a RESET condition to the USB bus.
        let mut usb_dp = gpioa
            .pa12
            .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);
        usb_dp.set_low().expect("Couldn't reset the USB bus!");
        delay(clocks.sysclk().0 / 100); // USB Startup time for STM32F303 is 1µs

        let usb_dm = gpioa.pa11.into_af14(&mut gpioa.moder, &mut gpioa.afrh);
        let usb_dp = usb_dp.into_af14(&mut gpioa.moder, &mut gpioa.afrh);

        let usb = Peripheral {
            usb: device.USB,
            pin_dm: usb_dm,
            pin_dp: usb_dp,
        };
        *USB_BUS = Some(UsbBus::new(usb));
        let usb_bus = USB_BUS
            .as_ref()
            .expect("Couldn't make the USB_BUS a static reference");

        let usb_class = HIDClass::new(usb_bus, KeyboardReport::desc(), 1);
        let usb_device = UsbDeviceBuilder::new(usb_bus, UsbVidPid(VID, PID))
            .manufacturer("ando")
            .product("nano")
            .serial_number(env!("CARGO_PKG_VERSION"))
            // .device_class(3) // Not having this will make the thing qwork?
            .build();

        let mut timer = timer::Timer::tim3(device.TIM3, 1.khz(), clocks, &mut rcc.apb1);
        timer.listen(timer::Event::Update);

        let mut output = gpioa
            .pa5
            .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);
        output.set_low().unwrap();
        let button = gpioa
            .pa6
            .into_pull_up_input(&mut gpioa.moder, &mut gpioa.pupdr);

        // Setup LED
        let mut led = gpioc
            .pc13
            .into_push_pull_output(&mut gpioc.moder, &mut gpioc.otyper);
        led.set_low().unwrap();

        // Schedule the blinking task
        cx.schedule.blinker(cx.start + PERIOD.cycles()).unwrap();

        init::LateResources {
            usb_device,
            usb_class,
            led,
            button,
            // output,
            timer,
        }
    }

    #[idle]
    fn idle(_ctx: idle::Context) -> ! {
        loop {}
    }

    #[task(binds=USB_HP_CAN_TX, priority = 2, resources = [usb_device, usb_class])]
    fn hp_handler(mut cx: hp_handler::Context) {
        defmt::info!("hp handler");
        usb_poll(&mut cx.resources.usb_device, &mut cx.resources.usb_class);
    }

    #[task(binds=USB_LP_CAN_RX0, priority = 2, resources = [usb_device, usb_class])]
    fn lp_handler(mut cx: lp_handler::Context) {
        defmt::info!("lp handler");
        usb_poll(&mut cx.resources.usb_device, &mut cx.resources.usb_class);
    }

    #[task(binds=USB_LP, priority=2, resources=[usb_device, usb_class])]
    fn usb_lp_handler(mut cx: usb_lp_handler::Context) {
        defmt::info!("usb lp handler");
        usb_poll(&mut cx.resources.usb_device, &mut cx.resources.usb_class);
    }

    #[task(resources = [led, button], schedule = [blinker])]
    fn blinker(cx: blinker::Context) {
        // Use the safe local `static mut` of RTIC
        static mut LED_STATE: bool = false;

        if cx.resources.button.is_low().unwrap() {
            if *LED_STATE {
                cx.resources.led.set_high().unwrap();
                *LED_STATE = false;
            } else {
                cx.resources.led.set_low().unwrap();
                *LED_STATE = true;
            }
        } else {
            cx.resources.led.set_low().unwrap();
            *LED_STATE = true;
        }
        cx.schedule.blinker(cx.scheduled + PERIOD.cycles()).unwrap();
    }

    #[task(binds=TIM3, priority=1, resources=[timer, button, usb_class])]
    fn tick(mut cx: tick::Context) {
        cx.resources.timer.clear_update_interrupt_flag();

        let key_pressed = cx
            .resources
            .button
            .is_low()
            .expect("Couldn't poll pressed keys!");
        cx.resources
            .usb_class
            .lock(|hid| {
                // defmt::info!("writing from tick");
                // Type the character `a`
                if key_pressed {
                    hid.push_input(&KeyboardReport {
                        modifier: 0,
                        leds: 0,
                        keycodes: [4, 0, 0, 0, 0, 0],
                    })
                } else {
                    hid.push_input(&KeyboardReport {
                        modifier: 0,
                        leds: 0,
                        keycodes: [4, 0, 0, 0, 0, 0],
                    })
                }
            })
            .expect("Couldn't get access to USB_CLASS!");
    }

    extern "C" {
        fn EXTI0();
    }
};

fn usb_poll(
    usb_dev: &mut UsbDevice<'static, UsbBus<Peripheral>>,
    keyboard: &mut HIDClass<'static, UsbBus<Peripheral>>,
) {
    if usb_dev.poll(&mut [keyboard]) {
        keyboard.poll();
    }
}
