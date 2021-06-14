use crate::hid::{HidDevice, Protocol, ReportType, Subclass};

#[allow(dead_code)]
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
    0x91, 0x02,       //   Output (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
    0x95, 0x01,       //   Report Count (1)
    0x75, 0x03,       //   Report Size (3)
    0x91, 0x03,       //   Output (Const,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
    0xC0,             // End Collection
];

// Note: not actually used. Soarer's NRKO descriptor is *actually* sent.
#[allow(dead_code)]
#[rustfmt::skip]
pub const BOOT_DESC: &[u8] = &[
    0x05, 0x01,       // USAGE_PAGE (Generic Desktop)
    0x09, 0x06,       // USAGE (Keyboard)
    0xa1, 0x01,       // COLLECTION (Application)
    0x75, 0x01,       //   REPORT_SIZE (1)
    0x95, 0x08,       //   REPORT_COUNT (8)
    0x05, 0x07,       //   USAGE_PAGE (Keyboard)
    0x19, 0xe0,       //   USAGE_MINIMUM (Keyboard LeftControl)
    0x29, 0xe7,       //   USAGE_MAXIMUM (Keyboard Right GUI)
    0x15, 0x00,       //   LOGICAL_MINIMUM (0)
    0x25, 0x01,       //   LOGICAL_MAXIMUM (1)
    0x81, 0x02,       //   INPUT (Data,Var,Abs)
    0x95, 0x01,       //   REPORT_COUNT (1)
    0x75, 0x08,       //   REPORT_SIZE (8)
    0x81, 0x03,       //   INPUT (Cnst,Var,Abs)
    0x95, 0x05,       //   REPORT_COUNT (5)
    0x75, 0x01,       //   REPORT_SIZE (1)
    0x05, 0x08,       //   USAGE_PAGE (LEDs)
    0x19, 0x01,       //   USAGE_MINIMUM (Num Lock)
    0x29, 0x05,       //   USAGE_MAXIMUM (Kana)
    0x91, 0x02,       //   OUTPUT (Data,Var,Abs)
    0x95, 0x01,       //   REPORT_COUNT (1)
    0x75, 0x03,       //   REPORT_SIZE (3)
    0x91, 0x03,       //   OUTPUT (Cnst,Var,Abs)
    0x95, 0x06,       //   REPORT_COUNT (6)
    0x75, 0x08,       //   REPORT_SIZE (8)
    0x15, 0x00,       //   LOGICAL_MINIMUM (0)
    0x25, 0x68,       //   LOGICAL_MAXIMUM (104)
    0x05, 0x07,       //   USAGE_PAGE (Keyboard)
    0x19, 0x00,       //   USAGE_MINIMUM (Reserved (no event indicated))
    0x29, 0x68,       //   USAGE_MAXIMUM (Keyboard Application)
    0x81, 0x00,       //   INPUT (Data,Ary,Abs)
    0xc0              // END_COLLECTION
];

const USB_FIRST_KEY_BIT: u8 = 1;
const USB_LAST_KEY_BIT: u8 = 0xA4;
const USB_NUM_KEY_BITS: u8 = USB_LAST_KEY_BIT - USB_FIRST_KEY_BIT + 1;
const USB_NUM_KEY_BIT_BYTES: u8 = (USB_NUM_KEY_BITS + 7) / 8;
const USB_NUM_PADDING_KEY_BITS: u8 = 8 * USB_NUM_KEY_BIT_BYTES - USB_NUM_KEY_BITS;

// Main key bitfield + 1 byte for modifiers + 1 media byte + 6 boot desc bytes
pub const USB_REPORT_SIZE: u8 = USB_NUM_KEY_BIT_BYTES + 8;

// Report descriptor by Soarer on geekhack
#[rustfmt::skip]
pub const SOARER_DESC: &[u8] = &[
    0x05, 0x01,          // Usage Page (Generic Desktop),
    0x09, 0x06,          // Usage (Keyboard),
    0xA1, 0x01,          // Collection (Application),

    // modifier byte
    0x75, 0x01,          //   Report Size (1),
    0x95, 0x08,          //   Report Count (8),
    0x05, 0x07,          //   Usage Page (Key Codes),
    0x19, 0xE0,          //   Usage Minimum (224),
    0x29, 0xE7,          //   Usage Maximum (231),
    0x15, 0x00,          //   Logical Minimum (0),
    0x25, 0x01,          //   Logical Maximum (1),
    0x81, 0x02,          //   Input (Data, Variable, Absolute), ;Modifier byte
    // 0xC0,                 // End Collection

    // // Media controls (constant in boot desc)
    // 0x05, 0x0C,        // Usage Page (Consumer)
    // 0x09, 0x01,        // Usage (Consumer Control)
    // 0xA1, 0x01,        // Collection (Application)
    0x05, 0x0C,        //   Usage Page (Consumer)
    0x15, 0x00,        //   Logical Minimum (0)
    0x25, 0x01,        //   Logical Maximum (1)
    0x75, 0x01,        //   Report Size (1)
    0x95, 0x08,        //   Report Count (8)
    0x09, 0xB5,        //   Usage (Scan Next Track)
    0x09, 0xB6,        //   Usage (Scan Previous Track)
    0x09, 0xB7,        //   Usage (Stop)
    0x09, 0xB8,        //   Usage (Eject)
    0x09, 0xCD,        //   Usage (Play/Pause)
    0x09, 0xE2,        //   Usage (Mute)
    0x09, 0xE9,        //   Usage (Volume Increment)
    0x09, 0xEA,        //   Usage (Volume Decrement)
    0x81, 0x02,        //   Input (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    // 0xC0,                 // End Collection

    // 0x05, 0x01,          // Usage Page (Generic Desktop),
    // 0x09, 0x06,          // Usage (Keyboard),
    // 0xA1, 0x01,          // Collection (Application),

    // LEDs
    0x95, 0x05,          //   Report Count (5),
    0x75, 0x01,          //   Report Size (1),
    0x05, 0x08,          //   Usage Page (LEDs),
    0x19, 0x01,          //   Usage Minimum (1),
    0x29, 0x05,          //   Usage Maximum (5),
    0x91, 0x02,          //   Output (Data, Variable, Absolute), ;LED report

    // LED padding
    0x95, 0x01,          //   Report Count (1),
    0x75, 0x03,          //   Report Size (3),
    0x91, 0x03,          //   Output (Constant),                 ;LED report padding

    // Boot Desc bytes
    0x95, 0x06,                     //   Report Count (6),
    0x75, 0x08,						//   Report Size (8),
    0x81, 0x03,						//   Input (Constant),                 ;Padding

    // Keys
    0x75, 0x01,					//   Report Size (1),
    0x95, USB_NUM_KEY_BITS,		//   Report Count (),
    0x05, 0x07,					//   Usage Page (Key Codes),
    0x19, USB_FIRST_KEY_BIT,    //   Usage Minimum (),
    0x29, USB_LAST_KEY_BIT,     //   Usage Maximum (),
    0x15, 0x00,					//   Logical Minimum (0),
    0x25, 0x01,					//   Logical Maximum (1),
    0x81, 0x02,					//   Input (Data, Variable, Absolute), ;keys bit array

    // Might not be needed if USB_NUM_PADDING_KEY_BITS is 0, check the math
    0x95, USB_NUM_PADDING_KEY_BITS, //   Report Count (4),
    0x75, 0x01,						//   Report Size (1),
    0x81, 0x03,						//   Input (Constant),                 ;Padding

    0xC0,                 // End Collection
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
        Subclass::None
    }

    fn protocol(&self) -> Protocol {
        Protocol::Keyboard
    }

    fn report_descriptor(&self) -> &[u8] {
        BOOT_DESC
        // SOARER_DESC
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
            defmt::info!("report {:?}, data {:?}", report_type, data);
            return Ok(());
        }
        Err(())
    }
}
