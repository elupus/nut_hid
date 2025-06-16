use windows::core::GUID;
use windows_strings::{w, PCWSTR};

/* 53c0d411-cfb1-4d29-8f81-e705f3ac17a1 */
pub const DEVPROP_NUTHID_GUID: GUID = GUID {
    data1: 0x53c0d411,
    data2: 0xcfb1,
    data3: 0x4d29,
    data4: [0x8f, 0x81, 0xe7, 0x05, 0xf3, 0xac, 0x17, 0xa1],
};

pub const DEVPROP_NUTHID_KEY_HOST: u32 = 2;
pub const DEVPROP_NUTHID_KEY_PORT: u32 = 3;


pub const ENUMERATOR_NAME: PCWSTR = w!("NutHidEnumerator");
pub const HARDWARE_IDS: PCWSTR = w!("root\\NutHidDevice\0");
pub const INSTANCE_ID: PCWSTR = w!("NutHidInstance");
pub const DEVICE_DESCRIPTION: PCWSTR = w!("NUT Hid Device");
pub const PARENT_DEVICE_INSTANCE: PCWSTR = w!("HTREE\\ROOT\\0");
