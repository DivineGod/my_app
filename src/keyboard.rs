use crate::hid::{HidDevice, Protocol, ReportType, Subclass};
use stm32f3xx_hal::prelude::*;

#[rustfmt::skip]
const REPORT_DESCRIPTOR: &[u8] = &[
    0x05, 0x01,       // Usage Page (Generic Desktop Ctrls)
    0x09, 0x06,       // Usage (Keyboard)
    0xA1, 0x01,       // Collection (Application)
    0x05, 0x07,       //   Usage Page (Kbrd/Keypad)
    0x19, 0xE0,       //   Usage Minimum (0xE0)
    0x29, 0xE7,       //   Usage Maximum (0xE7)
    0x15, 0x00,       //   Logical Minimum (0)
    0x25, 0x01,       //   Logical Maximum (1)
    0x95, 0x08,       //   Report Count (8)
    0x75, 0x01,       //   Report Size (1)
    0x81, 0x02,       //   Input (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0x95, 0x01,       //   Report Count (1)
    0x75, 0x08,       //   Report Size (8)
    0x81, 0x03,       //   Input (Const,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0x05, 0x07,       //   Usage Page (Kbrd/Keypad)
    0x19, 0x00,       //   Usage Minimum (0x00)
    0x29, 0xFF,       //   Usage Maximum (0xFF)
    0x15, 0x00,       //   Logical Minimum (0)
    0x26, 0xFF, 0x00, //   Logical Maximum (255)
    0x95, 0x06,       //   Report Count (6)
    0x75, 0x08,       //   Report Size (8)
    0x81, 0x00,       //   Input (Data,Array,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0x05, 0x08,       //   Usage Page (LEDs)
    0x19, 0x01,       //   Usage Minimum (Num Lock)
    0x29, 0x05,       //   Usage Maximum (Kana)
    0x95, 0x05,       //   Report Count (5)
    0x75, 0x01,       //   Report Size (1)
    0x91,
    0x02,             //   Output (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
    0x95, 0x01,       //   Report Count (1)
    0x75, 0x03,       //   Report Size (3)
    0x91,
    0x03,             //   Output (Const,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
    0xC0,             // End Collection
];

pub struct Keyboard {
    report: [u8; 8],
}
impl Keyboard {
    pub fn new() -> Keyboard {
        Keyboard { report: [0; 8] }
    }
}

impl HidDevice for Keyboard {
    fn subclass(&self) -> Subclass {
        Subclass::BootInterface
    }

    fn protocol(&self) -> Protocol {
        Protocol::Keyboard
    }

    fn report_descriptor(&self) -> &[u8] {
        REPORT_DESCRIPTOR
    }

    fn get_report(&mut self, report_type: ReportType, _report_id: u8) -> Result<&[u8], ()> {
        match report_type {
            ReportType::Input => Ok(&self.report),
            _ => Err(()),
        }
    }

    fn set_report(
        &mut self,
        report_type: ReportType,
        report_id: u8,
        data: &[u8],
    ) -> Result<(), ()> {
        if report_type == ReportType::Output && report_id == 0 && data.len() == 1 {
            return Ok(());
            /*
            if data[0] & 1 << 1 != 0 {
                return self.led.set_low();
            } else {
                return self.led.set_high();
            }
            */
        }
        Err(())
    }
}
