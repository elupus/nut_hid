use std::collections::HashMap;

use constants::*;
use super::*;

#[repr(C, packed(1))]
#[derive(Debug, Copy, Clone)]
#[allow(non_snake_case)]
pub struct HidMiniControlInfo {
    ReportId: ::core::ffi::c_uchar,
    ControlCode: ::core::ffi::c_uchar,

    Dummy1: ::core::ffi::c_ulong,
    Dummy2: ::core::ffi::c_ulong,
}

#[repr(C, packed(1))]
#[derive(Debug, Copy, Clone)]
#[allow(non_snake_case)]
pub struct HidMiniInputReport {
    ReportId: ::core::ffi::c_uchar,
    Data: ::core::ffi::c_uchar,
}

#[repr(C, packed(1))]
#[derive(Debug, Copy, Clone)]
#[allow(non_snake_case)]
pub struct HidMiniOutputReport {
    ReportId: ::core::ffi::c_uchar,
    Data: ::core::ffi::c_uchar,
    Pad1: ::core::ffi::c_ushort,
    Pad2: ::core::ffi::c_ulong,
}

const CONTROL_FEATURE_REPORT_ID: ::core::ffi::c_uchar = 0x01;
const FEATURE_REPORT_SIZE_CB: usize = core::mem::size_of::<HidMiniControlInfo>() - 1;
const INPUT_REPORT_SIZE_CB: usize = core::mem::size_of::<HidMiniControlInfo>() - 1;
const OUTPUT_REPORT_SIZE_CB: usize = core::mem::size_of::<HidMiniControlInfo>() - 1;

#[rustfmt::skip]
pub const HID_MINI_REPORT_DESCRIPTOR: &[u8] = &[
    0x06,0x00, 0xFF,                // USAGE_PAGE (Vender Defined Usagpe Page)
    0x09,0x01,                      // USAGE (Vendor Usage 0x01)
    0xA1,0x01,                      // COLLECTION (Application)
    0x85,CONTROL_FEATURE_REPORT_ID,    // REPORT_ID (1)
    0x09,0x01,                         // USAGE (Vendor Usage 0x01)
    0x15,0x00,                         // LOGICAL_MINIMUM(0)
    0x26,0xff, 0x00,                   // LOGICAL_MAXIMUM(255)
    0x75,0x08,                         // REPORT_SIZE (0x08)
    0x96,(FEATURE_REPORT_SIZE_CB & 0xff) as u8, (FEATURE_REPORT_SIZE_CB >> 8) as u8, // REPORT_COUNT
    0xB1,0x00,                         // FEATURE (Data,Ary,Abs)
    0x09,0x01,                         // USAGE (Vendor Usage 0x01)
    0x75,0x08,                         // REPORT_SIZE (0x08)
    0x96,(INPUT_REPORT_SIZE_CB & 0xff) as u8, (INPUT_REPORT_SIZE_CB >> 8) as u8, // REPORT_COUNT
    0x81,0x00,                         // INPUT (Data,Ary,Abs)
    0x09,0x01,                         // USAGE (Vendor Usage 0x01)
    0x75,0x08,                         // REPORT_SIZE (0x08)
    0x96,(OUTPUT_REPORT_SIZE_CB & 0xff) as u8, (OUTPUT_REPORT_SIZE_CB >> 8) as u8, // REPORT_COUNT
    0x91,0x00,                         // OUTPUT (Data,Ary,Abs)
    0xC0,                           // END_COLLECTION
];


pub fn new_mini_device() -> Device
{
    Device {
        reports: HashMap::new(),
        strings: HashMap::new(),
        vendor_id: 0xDEED,
        product_id: 0xFEED,
        version: 0x0101,
        manufacturer: NUT_HID_MANUFACTURER.into(),
        serial_number: NUT_HID_SERIALNUMBER.into(),
        product: NUT_HID_PRODUCT.into(),
        report_descriptor: HID_MINI_REPORT_DESCRIPTOR.into(),
    }
}