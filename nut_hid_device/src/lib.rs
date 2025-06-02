use std::collections::HashMap;
pub mod constants;
pub mod nut;
pub mod mini;

pub struct Device {
    pub reports: HashMap<u8, Vec<u8>>,
    pub strings: HashMap<u8, String>,
    pub vendor_id: u16,
    pub product_id: u16,
    pub version: u16,
    pub manufacturer: String,
    pub serial_number: String,
    pub product: String,
    pub report_descriptor: Vec<u8>,
}
