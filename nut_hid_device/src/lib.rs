use std::{collections::HashMap, sync::RwLock};

use crate::{dummy::DummyDevice, mini::MiniDevice, nut::NutDevice};
pub mod constants;
pub mod mini;
pub mod nut;
pub mod dummy;

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

#[derive(Default, Debug)]
pub struct DeviceConfig {
    pub host: String,
    pub port: u32,
    pub backend: String,
}

pub trait Device {
    fn data(&self) -> &RwLock<DeviceData>;
    fn read(&self) -> Option<(u8, Vec<u8>)>;
}

pub enum DeviceEnum
{
    NutDevice(NutDevice),
    DummyDevice(DummyDevice),
    MiniDevice(MiniDevice),
}

impl Device for DeviceEnum
{
    fn data(&self) -> &RwLock<DeviceData>
    {
        match self {
            DeviceEnum::NutDevice(device) => device.data(),
            DeviceEnum::DummyDevice(device) => device.data(),
            DeviceEnum::MiniDevice(device) => device.data()
        }
    }

    fn read(&self) -> Option<(u8, Vec<u8>)>
    {
        match self {
            DeviceEnum::NutDevice(device) => device.read(),
            DeviceEnum::DummyDevice(device) => device.read(),
            DeviceEnum::MiniDevice(device) => device.read()
        }
    }
}

impl DeviceEnum
{
    pub fn from_config(config: DeviceConfig) -> Result<DeviceEnum, DeviceError>
    {
        match config.backend.as_str() {
            "nut" => Ok(Self::NutDevice(nut::new_nut_device(config))),
            "dummy" => Ok(Self::DummyDevice(dummy::new_dummy_device(config))),
            "mini" => Ok(Self::MiniDevice(mini::new_mini_device())),
            _ => Err(DeviceError::InvalidBackend),
        }
    }
}

pub enum DeviceError
{
    InvalidBackend
}
