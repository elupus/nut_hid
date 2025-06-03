use std::collections::HashMap;
pub mod constants;
pub mod nut;
pub mod mini;

#[derive(Default)]

pub struct DeviceData {
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

pub trait Device {
    fn data_mut(&mut self) -> &mut DeviceData;
    fn data(&self) -> &DeviceData;
    fn read(&mut self) -> Option<(u8, Vec<u8>)>;
}
