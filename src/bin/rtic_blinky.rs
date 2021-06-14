#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use my_app as _;

use embedded_hal::digital::v2::OutputPin;
use rtic::app;
use rtic::cyccnt::U32Ext;
use stm32f3xx_hal::gpio::{gpioa, gpioc, Input, Output, PullUp, PushPull};
use stm32f3xx_hal::prelude::*;

const PERIOD: u32 = 10_000_000;

// We need to pass monotonic = rtic::cyccnt::CYCCNT to use schedule feature fo RTIC
#[app(device = stm32f3xx_hal::pac, peripherals = true, monotonic = rtic::cyccnt::CYCCNT)]
const APP: () = {
    // Global resources (global variables) are defined here and initialized with the
    // `LateResources` struct in init
    struct Resources {
        button: gpioa::PA4<Input<PullUp>>,
        led: gpioc::PC13<Output<PushPull>>,
    }

    #[init(schedule = [blinker])]
    fn init(cx: init::Context) -> init::LateResources {
        defmt::info!("hi");
        // Enable cycle counter
        let mut core = cx.core;
        core.DWT.enable_cycle_counter();

        let device: stm32f3xx_hal::stm32::Peripherals = cx.device;

        // Setup clocks
        let mut flash = device.FLASH.constrain();
        let mut rcc = device.RCC.constrain();
        let _clocks = rcc
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

        let mut gpioa = device.GPIOA.split(&mut rcc.ahb);
        let mut gpioc = device.GPIOC.split(&mut rcc.ahb);

        let mut output = gpioa
            .pa5
            .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);
        output.set_low().unwrap();
        let button = gpioa
            .pa4
            .into_pull_up_input(&mut gpioa.moder, &mut gpioa.pupdr);

        // Setup LED
        let mut led = gpioc
            .pc13
            .into_push_pull_output(&mut gpioc.moder, &mut gpioc.otyper);
        led.set_low().unwrap();

        // Schedule the blinking task
        cx.schedule.blinker(cx.start + PERIOD.cycles()).unwrap();

        init::LateResources { led, button }
    }

    #[idle]
    fn idle(_ctx: idle::Context) -> ! {
        loop {}
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

    extern "C" {
        fn EXTI0();
    }
};
