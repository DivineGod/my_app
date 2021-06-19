#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use core::convert::Infallible;
use cortex_m::asm::delay;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use generic_array::typenum::{U5, U6};
// use keyberon::action::{k, Action::*};
use keyberon::action::k;
use keyberon::debounce::Debouncer;
use keyberon::impl_heterogenous_array;
use keyberon::key_code::KbHidReport;
use keyberon::key_code::KeyCode::{self, *};
use keyberon::layout::Layout;
use keyberon::matrix::{Matrix, PressedKeys};
use my_app as _;
use rtic::app;
use stm32f3xx_hal::gpio::{gpiob, gpioc, Input, Output, PullUp, PushPull};
use stm32f3xx_hal::prelude::*;
use stm32f3xx_hal::timer;
use stm32f3xx_hal::timer::Timer;
use stm32f3xx_hal::usb::Peripheral;
use stm32f3xx_hal::usb::UsbBus;
use usb_device::bus::UsbBusAllocator;
use usb_device::class::UsbClass as _;
use usb_device::device::UsbDeviceBuilder;
use usb_device::device::UsbVidPid;

type UsbClass = keyberon::Class<'static, UsbBus<Peripheral>, Leds>;
type UsbDevice = usb_device::device::UsbDevice<'static, UsbBus<Peripheral>>;

pub struct Cols(
    gpiob::PB0<Input<PullUp>>,
    gpiob::PB1<Input<PullUp>>,
    gpiob::PB2<Input<PullUp>>,
    gpiob::PB3<Input<PullUp>>,
    gpiob::PB4<Input<PullUp>>,
);
impl_heterogenous_array! {
    Cols,
    dyn InputPin<Error = Infallible>,
    U5,
    [0, 1, 2, 3, 4]
}

pub struct Rows(
    gpiob::PB13<Output<PushPull>>,
    gpiob::PB14<Output<PushPull>>,
    gpiob::PB15<Output<PushPull>>,
    gpiob::PB10<Output<PushPull>>,
    gpiob::PB11<Output<PushPull>>,
    gpiob::PB12<Output<PushPull>>,
);
impl_heterogenous_array! {
    Rows,
    dyn OutputPin<Error = Infallible>,
    U6,
    [0, 1, 2, 3, 4, 5]
}

pub static LAYERS: keyberon::layout::Layers<()> = &[&[
    &[k(Kb1), k(Kb1), k(Kb1), k(Kb1), k(Kb1)],
    &[k(Kb2), k(Kb2), k(Kb2), k(Kb2), k(Kb2)],
    &[k(Kb3), k(Kb3), k(Kb3), k(Kb3), k(Kb3)],
    &[k(Kb4), k(Kb4), k(Kb4), k(Kb4), k(Kb4)],
    &[k(Kb5), k(Kb5), k(Kb5), k(Kb5), k(Kb5)],
    &[k(Kb6), k(Kb6), k(Kb6), k(Kb6), k(Kb6)],
]];

pub struct Leds {
    caps_lock: gpioc::PC13<Output<PushPull>>,
}
impl keyberon::keyboard::Leds for Leds {
    fn caps_lock(&mut self, status: bool) {
        if status {
            self.caps_lock.set_low().unwrap()
        } else {
            self.caps_lock.set_high().unwrap()
        }
    }
}

// We need to pass monotonic = rtic::cyccnt::CYCCNT to use schedule feature fo RTIC
#[app(device = stm32f3xx_hal::pac, peripherals = true)]
const APP: () = {
    // Global resources (global variables) are defined here and initialized with the
    // `LateResources` struct in init
    struct Resources {
        usb_device: UsbDevice,
        usb_class: UsbClass,
        matrix: Matrix<Cols, Rows>,
        debouncer: Debouncer<PressedKeys<U6, U5>>,
        layout: Layout<()>,
        timer: Timer<stm32f3xx_hal::stm32::TIM3>,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        static mut USB_BUS: Option<UsbBusAllocator<UsbBus<Peripheral>>> = None;

        defmt::info!("hi");

        let device: stm32f3xx_hal::stm32::Peripherals = cx.device;

        // Setup clocks
        let mut flash = device.FLASH.constrain();
        let mut rcc = device.RCC.constrain();
        /*
        let clocks = rcc
            .cfgr
            .use_hse(8.mhz())
            .sysclk(72.mhz())
            .pclk1(36.mhz())
            .pclk2(36.mhz())
            .freeze(&mut flash.acr);
        // */
        //*
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
        let mut gpiob = device.GPIOB.split(&mut rcc.ahb);
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

        // Setup LED
        let mut led = gpioc
            .pc13
            .into_push_pull_output(&mut gpioc.moder, &mut gpioc.otyper);
        led.set_low().unwrap();
        let leds = Leds { caps_lock: led };

        let usb_class = keyberon::new_class(usb_bus, leds);
        // let usb_device = keyberon::new_device(usb_bus);
        let usb_device = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x16c0, 0x27db))
            .manufacturer("ando")
            .product("nano")
            .serial_number(env!("CARGO_PKG_VERSION"))
            .build();

        let mut timer = timer::Timer::tim3(device.TIM3, 1.khz(), clocks, &mut rcc.apb1);
        timer.listen(timer::Event::Update);

        let matrix = Matrix::new(
            Cols(
                gpiob
                    .pb0
                    .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr),
                gpiob
                    .pb1
                    .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr),
                gpiob
                    .pb2
                    .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr),
                gpiob
                    .pb3
                    .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr),
                gpiob
                    .pb4
                    .into_pull_up_input(&mut gpiob.moder, &mut gpiob.pupdr),
            ),
            Rows(
                gpiob
                    .pb13
                    .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper),
                gpiob
                    .pb14
                    .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper),
                gpiob
                    .pb15
                    .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper),
                gpiob
                    .pb10
                    .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper),
                gpiob
                    .pb11
                    .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper),
                gpiob
                    .pb12
                    .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper),
            ),
        );

        init::LateResources {
            usb_device,
            usb_class,
            timer,
            debouncer: Debouncer::new(PressedKeys::default(), PressedKeys::default(), 5),
            matrix: matrix.unwrap(),
            layout: Layout::new(LAYERS),
        }
    }

    #[idle]
    fn idle(_ctx: idle::Context) -> ! {
        loop {}
    }

    #[task(binds=USB_HP_CAN_TX, priority = 2, resources = [usb_device, usb_class])]
    fn hp_handler(mut cx: hp_handler::Context) {
        // defmt::info!("hp handler");
        usb_poll(&mut cx.resources.usb_device, &mut cx.resources.usb_class);
    }

    #[task(binds=USB_LP_CAN_RX0, priority = 2, resources = [usb_device, usb_class])]
    fn lp_handler(mut cx: lp_handler::Context) {
        // defmt::info!("lp handler");
        usb_poll(&mut cx.resources.usb_device, &mut cx.resources.usb_class);
    }

    #[task(binds=USB_LP, priority=2, resources=[usb_device, usb_class])]
    fn usb_lp_handler(mut cx: usb_lp_handler::Context) {
        // defmt::info!("usb lp handler");
        usb_poll(&mut cx.resources.usb_device, &mut cx.resources.usb_class);
    }

    #[task(binds=TIM3, priority=1, resources=[timer, usb_class, matrix, debouncer, layout])]
    fn tick(mut cx: tick::Context) {
        cx.resources.timer.clear_update_interrupt_flag();

        for event in cx
            .resources
            .debouncer
            .events(cx.resources.matrix.get().unwrap())
        {
            // use keyberon::layout::Event;
            defmt::info!("got an event");
            cx.resources.layout.event(event);
        }
        match cx.resources.layout.tick() {
            keyberon::layout::CustomEvent::Release(()) => cortex_m::peripheral::SCB::sys_reset(),
            _ => (),
        }
        send_report(cx.resources.layout.keycodes(), &mut cx.resources.usb_class);
    }

    extern "C" {
        fn EXTI0();
    }
};

fn send_report(iter: impl Iterator<Item = KeyCode>, usb_class: &mut resources::usb_class<'_>) {
    use rtic::Mutex;
    let report: KbHidReport = iter.collect();
    if usb_class.lock(|k| k.device_mut().set_keyboard_report(report.clone())) {
        while let Ok(0) = usb_class.lock(|k| k.write(report.as_bytes())) {}
    }
}

fn usb_poll(usb_device: &mut UsbDevice, keyboard: &mut UsbClass) {
    if usb_device.poll(&mut [keyboard]) {
        keyboard.poll();
    }
}
