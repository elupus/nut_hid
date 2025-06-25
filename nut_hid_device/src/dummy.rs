use log::info;
use std::collections::{HashMap, VecDeque};
use std::sync::{Mutex, RwLock};
use std::thread;
use std::time::Duration;

use super::*;
use binary_serde::recursive_array::RecursiveArray;
use binary_serde::{BinarySerde, BitfieldBitOrder, Endianness, binary_serde_bitfield};
use constants::*;

pub const STRING_ID_MANUFACTURER: u8 = 0x01;
pub const STRING_ID_PRODUCT: u8 = 0x02;
pub const STRING_ID_SERIAL: u8 = 0x03;
pub const STRING_ID_DEVICECHEMISTRY: u8 = 0x04;
pub const STRING_ID_OEMVENDOR: u8 = 0x05;

pub const REPORT_ID_IDENTIFICAITON: u8 = 0x01; // FEATURE ONLY
pub const REPORT_ID_RECHARGEABLE: u8 = 0x06; // FEATURE ONLY
pub const REPORT_ID_PRESENTSTATUS: u8 = 0x07; // INPUT OR FEATURE(required by Windows)
pub const REPORT_ID_REMAINTIMELIMIT: u8 = 0x08;
pub const REPORT_ID_MANUFACTUREDATE: u8 = 0x09;
pub const REPORT_ID_CONFIGVOLTAGE: u8 = 0x0A; // 10 FEATURE ONLY
pub const REPORT_ID_VOLTAGE: u8 = 0x0B; // 11 INPUT (NA) OR FEATURE(implemented)
pub const REPORT_ID_REMAININGCAPACITY: u8 = 0x0C; // 12 INPUT OR FEATURE(required by Windows)
pub const REPORT_ID_RUNTIMETOEMPTY: u8 = 0x0D;
pub const REPORT_ID_FULLCHRGECAPACITY: u8 = 0x0E; // 14 FEATURE ONLY. Last Full Charge Capacity 
pub const REPORT_ID_WARNCAPACITYLIMIT: u8 = 0x0F;
pub const REPORT_ID_CPCTYGRANULARITY1: u8 = 0x10;
pub const REPORT_ID_REMNCAPACITYLIMIT: u8 = 0x11;
pub const REPORT_ID_DELAYBE4SHUTDOWN: u8 = 0x12; // 18 FEATURE ONLY
pub const REPORT_ID_DELAYBE4REBOOT: u8 = 0x13;
pub const REPORT_ID_AUDIBLEALARMCTRL: u8 = 0x14; // 20 INPUT OR FEATURE
pub const REPORT_ID_CURRENT: u8 = 0x15; // 21 FEATURE ONLY
pub const REPORT_ID_CAPACITYMODE: u8 = 0x16;
pub const REPORT_ID_DESIGNCAPACITY: u8 = 0x17;
pub const REPORT_ID_CPCTYGRANULARITY2: u8 = 0x18;
pub const REPORT_ID_AVERAGETIME2FULL: u8 = 0x1A;
pub const REPORT_ID_AVERAGECURRENT: u8 = 0x1B;
pub const REPORT_ID_AVERAGETIME2EMPTY: u8 = 0x1C;

pub const REPORT_ID_IDEVICECHEMISTRY: u8 = 0x1F; // Feature
pub const REPORT_ID_IOEMINFORMATION: u8 = 0x20; // Feature

#[rustfmt::skip]
pub const UPS_REPORT_DESCRIPTOR: &[u8] = &[
    0x05, 0x84, // USAGE_PAGE (Power Device)
    0x09, 0x04, // USAGE (UPS)
    0xA1, 0x01, // COLLECTION (Application)
    0x09, 0x24, //   USAGE (PowerSummary)
    0xA1, 0x02, //   COLLECTION (Logical)
    0x75, 0x08, //     REPORT_SIZE (8)
    0x95, 0x01, //     REPORT_COUNT (1)
    0x15, 0x00, //     LOGICAL_MINIMUM (0)
    0x26, 0xFF, 0x00, //     LOGICAL_MAXIMUM (255)

    0x85, REPORT_ID_IDENTIFICAITON, //     REPORT_ID (4)

    0x09, 0xFE, //     USAGE (iProduct)
    0x79, STRING_ID_PRODUCT, //     STRING INDEX (2)
    0xB1, 0x23, //     FEATURE (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Nonvolatile, Bitfield)

    0x09, 0xFF, //     USAGE (iSerialNumber)
    0x79, STRING_ID_SERIAL, //  STRING INDEX (3)
    0xB1, 0x23, //     FEATURE (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Nonvolatile, Bitfield)

    0x09, 0xFD, //     USAGE (iManufacturer)
    0x79, STRING_ID_MANUFACTURER, //     STRING INDEX (1)
    0xB1, 0x23, //     FEATURE (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Nonvolatile, Bitfield)


    0x05, 0x85, //     USAGE_PAGE (Battery System) ====================
    0x85, REPORT_ID_RECHARGEABLE, //     REPORT_ID (6)
    0x09, 0x8B, //     USAGE (Rechargable)                  
    0xB1, 0x23, //     FEATURE (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Nonvolatile, Bitfield)
    0x85, REPORT_ID_IDEVICECHEMISTRY, //     REPORT_ID (31)
    0x09, 0x89, //     USAGE (iDeviceChemistry)
    0x79, STRING_ID_DEVICECHEMISTRY, //     STRING INDEX (4)
    0xB1, 0x23, //     FEATURE (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Nonvolatile, Bitfield)
    0x85, REPORT_ID_IOEMINFORMATION,  //     REPORT_ID (32)
    0x09, 0x8F, //     USAGE (iOEMInformation)
    0x79, STRING_ID_OEMVENDOR, //     STRING INDEX (5)
    0xB1, 0x23, //     FEATURE (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Nonvolatile, Bitfield)
    0x85, REPORT_ID_CAPACITYMODE, //     REPORT_ID (22)
    0x09, 0x2C, //     USAGE (CapacityMode)
    0xB1, 0x23, //     FEATURE (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Nonvolatile, Bitfield)
    0x85, REPORT_ID_CPCTYGRANULARITY1, //     REPORT_ID (16)
    0x09, 0x8D, //     USAGE (CapacityGranularity1)
    0x26, 0x64,0x00, //     LOGICAL_MAXIMUM (100)    
    0xB1, 0x22, //     FEATURE (Data, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Nonvolatile, Bitfield)
    0x85, REPORT_ID_CPCTYGRANULARITY2, //     REPORT_ID (24)
    0x09, 0x8E, //     USAGE (CapacityGranularity2)
    0xB1, 0x23, //     FEATURE (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Nonvolatile, Bitfield)
    0x85, REPORT_ID_FULLCHRGECAPACITY, //     REPORT_ID (14)        
    0x09, 0x67, //     USAGE (FullChargeCapacity)
    0xB1, 0x83, //     FEATURE (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Volatile, Bitfield)
    0x85, REPORT_ID_DESIGNCAPACITY, //     REPORT_ID (23)
    0x09, 0x83, //     USAGE (DesignCapacity)
    0xB1, 0x83, //     FEATURE (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Volatile, Bitfield)
    0x85, REPORT_ID_REMAININGCAPACITY, //     REPORT_ID (12)
    0x09, 0x66, //     USAGE (RemainingCapacity)
    0x81, 0xA3, //     INPUT (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Bitfield)
    0x09, 0x66, //     USAGE (RemainingCapacity)
    0xB1, 0xA3, //     FEATURE (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Volatile, Bitfield)
    0x85, REPORT_ID_WARNCAPACITYLIMIT, //     REPORT_ID (15)
    0x09, 0x8C, //     USAGE (WarningCapacityLimit)
    0xB1, 0xA2, //     FEATURE (Data, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Volatile, Bitfield)
    0x85, REPORT_ID_REMNCAPACITYLIMIT, //     REPORT_ID (17)
    0x09, 0x29, //     USAGE (RemainingCapacityLimit)
    0xB1, 0xA2, //     FEATURE (Data, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Volatile, Bitfield)
    0x85, REPORT_ID_MANUFACTUREDATE, //     REPORT_ID (9)
    0x09, 0x85, //     USAGE (ManufacturerDate)
    0x75, 0x10, //     REPORT_SIZE (16)
    0x27, 0xFF, 0xFF, 0x00, 0x00, //     LOGICAL_MAXIMUM (65534)
    0xB1, 0xA3, //     FEATURE (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Volatile, Bitfield)
    0x85, REPORT_ID_AVERAGETIME2FULL, //     REPORT_ID (26)
    0x09, 0x6A, //     USAGE (AverageTimeToFull)
    0x27, 0xFF, 0xFF, 0x00, 0x00, //     LOGICAL_MAXIMUM (65534)
    0x66, 0x01, 0x10, //     UNIT (Seconds)
    0x55, 0x00, //     UNIT_EXPONENT (0)
    0xB1, 0xA3, //     FEATURE (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Volatile, Bitfield) 
    0x85, REPORT_ID_AVERAGETIME2EMPTY, //     REPORT_ID (28)
    0x09, 0x69, //     USAGE (AverageTimeToEmpty)  
    0x81, 0xA3, //     INPUT (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Bitfield)
    0x09, 0x69, //     USAGE (AverageTimeToEmpty)
    0xB1, 0xA3, //     FEATURE (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Volatile, Bitfield)
    0x85, REPORT_ID_RUNTIMETOEMPTY, //     REPORT_ID (13)    
    0x09, 0x68, //     USAGE (RunTimeToEmpty)  
    0x81, 0xA3, //     INPUT (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Bitfield)
    0x09, 0x68, //     USAGE (RunTimeToEmpty)
    0xB1, 0xA3, //     FEATURE (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Volatile, Bitfield)      
    0x85, REPORT_ID_REMAINTIMELIMIT, //     REPORT_ID (8)
    0x09, 0x2A, //     USAGE (RemainingTimeLimit)
    0x75, 0x10, //     REPORT_SIZE (16)
    0x27, 0x64, 0x05, 0x00, 0x00, //     LOGICAL_MAXIMUM (1380)
    0x16, 0x78, 0x00, //     LOGICAL_MINIMUM (120)
    0x81, 0x22, //     INPUT (Data, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Bitfield)
    0x09, 0x2A, //     USAGE (RemainingTimeLimit)
    0xB1, 0xA2, //     FEATURE (Data, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Volatile, Bitfield)
    0x05, 0x84, //     USAGE_PAGE (Power Device) ====================
    0x85, REPORT_ID_DELAYBE4SHUTDOWN, //     REPORT_ID (18)
    0x09, 0x57, //     USAGE (DelayBeforeShutdown)
    0x16, 0x00, 0x80, //     LOGICAL_MINIMUM (-32768)
    0x27, 0xFF, 0x7F, 0x00, 0x00, //     LOGICAL_MAXIMUM (32767)
    0xB1, 0xA2, //     FEATURE (Data, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Volatile, Bitfield)
    0x85, REPORT_ID_DELAYBE4REBOOT, //     REPORT_ID (19)
    0x09, 0x55, //     USAGE (DelayBeforeReboot)
    0xB1, 0xA2, //     FEATURE (Data, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Volatile, Bitfield)
    0x85, REPORT_ID_CONFIGVOLTAGE, //     REPORT_ID (10)
    0x09, 0x40, //     USAGE (ConfigVoltage)
    0x15, 0x00, //     LOGICAL_MINIMUM (0)
    0x27, 0xFF, 0xFF, 0x00, 0x00, //     LOGICAL_MAXIMUM (65535)
    0x67, 0x21, 0xD1, 0xF0, 0x00, //     UNIT (Centivolts)
    0x55, 0x05, //     UNIT_EXPONENT (5)
    0xB1, 0x23, //     FEATURE (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Nonvolatile, Bitfield)
    0x85, REPORT_ID_VOLTAGE, //     REPORT_ID (11)
    0x09, 0x30, //     USAGE (Voltage)
    0x81, 0xA3, //     INPUT (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Bitfield)
    0x09, 0x30, //     USAGE (Voltage)
    0xB1, 0xA3, //     FEATURE (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Volatile, Bitfield)
    0x85, REPORT_ID_AUDIBLEALARMCTRL, //     REPORT_ID (20)
    0x09, 0x5A, //     USAGE (AudibleAlarmControl)
    0x75, 0x08, //     REPORT_SIZE (8)
    0x15, 0x01, //     LOGICAL_MINIMUM (1)
    0x25, 0x03, //     LOGICAL_MAXIMUM (3)
    0x65, 0x00, //     UNIT (0)
    0x55, 0x00, //     UNIT_EXPONENT (0)
    0x81, 0x22, //     INPUT (Data, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Bitfield)
    0x09, 0x5A, //     USAGE (AudibleAlarmControl)
    0xB1, 0xA2, //     FEATURE (Data, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Volatile, Bitfield)
    0x09, 0x02, //     USAGE (PresentStatus)
    0xA1, 0x02, //     COLLECTION (Logical)
    0x85, REPORT_ID_PRESENTSTATUS, //       REPORT_ID (7)
    0x05, 0x85, //       USAGE_PAGE (Battery System) =================
    0x09, 0x44, //       USAGE (Charging)
    0x75, 0x01, //       REPORT_SIZE (1)
    0x15, 0x00, //       LOGICAL_MINIMUM (0)
    0x25, 0x01, //       LOGICAL_MAXIMUM (1)
    0x81, 0xA3, //       INPUT (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Bitfield)
    0x09, 0x44, //       USAGE (Charging)
    0xB1, 0xA3, //       FEATURE (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Volatile, Bitfield)
    0x09, 0x45, //       USAGE (Discharging)
    0x81, 0xA3, //       INPUT (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Bitfield)
    0x09, 0x45, //       USAGE (Discharging)
    0xB1, 0xA3, //       FEATURE (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Volatile, Bitfield)
    0x09, 0xD0, //       USAGE (ACPresent)
    0x81, 0xA3, //       INPUT (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Bitfield)
    0x09, 0xD0, //       USAGE (ACPresent)
    0xB1, 0xA3, //       FEATURE (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Volatile, Bitfield)
    0x09, 0xD1, //       USAGE (BatteryPresent)
    0x81, 0xA3, //       INPUT (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Bitfield)
    0x09, 0xD1, //       USAGE (BatteryPresent)
    0xB1, 0xA3, //       FEATURE (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Volatile, Bitfield)
    0x09, 0x42, //       USAGE (BelowRemainingCapacityLimit)
    0x81, 0xA3, //       INPUT (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Bitfield)
    0x09, 0x42, //       USAGE (BelowRemainingCapacityLimit)
    0xB1, 0xA3, //       FEATURE (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Volatile, Bitfield)
    0x09, 0x43, //       USAGE (RemainingTimeLimitExpired)
    0x81, 0xA2, //       INPUT (Data, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Bitfield)
    0x09, 0x43, //       USAGE (RemainingTimeLimitExpired)
    0xB1, 0xA2, //       FEATURE (Data, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Volatile, Bitfield)  
    0x09, 0x4B, //       USAGE (NeedReplacement)
    0x81, 0xA3, //       INPUT (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Bitfield)
    0x09, 0x4B, //       USAGE (NeedReplacement)
    0xB1, 0xA3, //       FEATURE (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Volatile, Bitfield)    
    0x09, 0xDB, //       USAGE (VoltageNotRegulated)
    0x81, 0xA3, //       INPUT (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Bitfield)
    0x09, 0xDB, //       USAGE (VoltageNotRegulated)
    0xB1, 0xA3, //       FEATURE (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Volatile, Bitfield)
    0x09, 0x46, //       USAGE (FullyCharged)
    0x81, 0xA3, //       INPUT (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Bitfield)
    0x09, 0x46, //       USAGE (FullyCharged)
    0xB1, 0xA3, //       FEATURE (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Volatile, Bitfield)
    0x09, 0x47, //       USAGE (FullyDischarged)
    0x81, 0xA3, //       INPUT (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Bitfield)
    0x09, 0x47, //       USAGE (FullyDischarged)
    0xB1, 0xA3, //       FEATURE (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Volatile, Bitfield)    
    0x05, 0x84, //       USAGE_PAGE (Power Device) =================
    0x09, 0x68, //       USAGE (ShutdownRequested)
    0x81, 0xA2, //       INPUT (Data, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Bitfield)
    0x09, 0x68, //       USAGE (ShutdownRequested)
    0xB1, 0xA2, //       FEATURE (Data, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Volatile, Bitfield)
    0x09, 0x69, //       USAGE (ShutdownImminent)
    0x81, 0xA3, //       INPUT (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Bitfield)
    0x09, 0x69, //       USAGE (ShutdownImminent)
    0xB1, 0xA3, //       FEATURE (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Volatile, Bitfield)
    0x09, 0x73, //       USAGE (CommunicationLost)
    0x81, 0xA3, //       INPUT (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Bitfield)
    0x09, 0x73, //       USAGE (CommunicationLost)
    0xB1, 0xA3, //       FEATURE (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Volatile, Bitfield)
    0x09, 0x65, //       USAGE (Overload)
    0x81, 0xA3, //       INPUT (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Bitfield)
    0x09, 0x65, //       USAGE (Overload)
    0xB1, 0xA3, //       FEATURE (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Volatile, Bitfield)
    0x95, 0x02, //       REPORT_COUNT (2) // padding bits to make the report byte aligned
    0x81, 0x01, //       INPUT (Constant, Array, Absolute)
    0xB1, 0x01, //       FEATURE (Constant, Array, Absolute, No Wrap, Linear, Preferred State, No Null Position, Nonvolatile, Bitfield)
    0xC0,       //     END_COLLECTION
    0xC0,       //   END_COLLECTION
    0xC0        // END_COLLECTION
];

#[derive(Debug, Default, PartialEq, Eq)]
#[binary_serde_bitfield(order = BitfieldBitOrder::LsbFirst)]
struct PresentStatus {
    #[bits(1)]
    charging: bool, // bit 0x00
    #[bits(1)]
    discharging: bool, // bit 0x01
    #[bits(1)]
    ac_present: bool, // bit 0x02
    #[bits(1)]
    battery_present: bool, // bit 0x03
    #[bits(1)]
    below_remaining_capacity_limit: bool, // bit 0x04
    #[bits(1)]
    temaining_time_limit_expired: bool, // bit 0x05
    #[bits(1)]
    need_replacement: bool, // bit 0x06
    #[bits(1)]
    voltage_not_regulated: bool, // bit 0x07

    #[bits(1)]
    fully_charged: bool, // bit 0x08
    #[bits(1)]
    fully_discharged: bool, // bit 0x09
    #[bits(1)]
    shutdown_requested: bool, // bit 0x0A
    #[bits(1)]
    shutdown_imminent: bool, // bit 0x0B
    #[bits(1)]
    communication_lost: bool, // bit 0x0C
    #[bits(1)]
    overload: bool, // bit 0x0D
    #[bits(1)]
    unused1: bool,
    #[bits(1)]
    unused2: bool,
}

#[derive(Debug, Default, BinarySerde, PartialEq, Eq)]
struct Identification {
    i_product: u8,
    i_manufacturer: u8,
    i_serial: u8,
}

pub struct DummyDevice {
    device: RwLock<DeviceData>,
    pending: Mutex<VecDeque<(u8, Vec<u8>)>>,
}

impl Device for DummyDevice {
    fn data(&self) -> &RwLock<DeviceData> {
        &self.device
    }

    fn read(&self) -> Option<(u8, Vec<u8>)> {
        /* get all pending */
        let mut pending = self.pending.lock().unwrap();
        if let Some(report) = pending.pop_front() {
            thread::sleep(Duration::from_secs(2));
            return Some(report);
        }

        pending.push_back((REPORT_ID_REMAININGCAPACITY, vec![80]));
        pending.push_back((REPORT_ID_REMAININGCAPACITY, vec![70]));
        pending.push_back((REPORT_ID_REMAININGCAPACITY, vec![60]));
        pending.push_back((REPORT_ID_REMAININGCAPACITY, vec![50]));
        pending.push_back((REPORT_ID_REMAININGCAPACITY, vec![60]));
        pending.push_back((REPORT_ID_REMAININGCAPACITY, vec![70]));
        pending.pop_front()
    }
}

fn struct_to_vec<T: BinarySerde>(data: T) -> Vec<u8> {
    data.binary_serialize_to_array(Endianness::Little)
        .as_slice()
        .into()
}

pub fn new_dummy_device(_device_config: DeviceConfig) -> DummyDevice {
    info!("Creating Dummy backend");
    let mut device = DeviceData {
        reports: HashMap::new(),
        strings: HashMap::new(),
        vendor_id: NUT_HID_VID,
        product_id: NUT_HID_PID,
        version: NUT_HID_VERSION,
        manufacturer: NUT_HID_MANUFACTURER.into(),
        serial_number: NUT_HID_SERIALNUMBER.into(),
        product: NUT_HID_PRODUCT.into(),
        report_descriptor: UPS_REPORT_DESCRIPTOR.into(),
    };

    let identification = Identification {
        i_product: STRING_ID_PRODUCT,
        i_serial: STRING_ID_SERIAL,
        i_manufacturer: STRING_ID_MANUFACTURER,
    };

    device
        .reports
        .insert(REPORT_ID_IDENTIFICAITON, struct_to_vec(identification));

    let status = PresentStatus {
        ac_present: true,
        battery_present: true,
        ..Default::default()
    };

    device
        .reports
        .insert(REPORT_ID_PRESENTSTATUS, struct_to_vec(status));
    device.reports.insert(REPORT_ID_CAPACITYMODE, [2].into()); /* Percentage */

    device
        .reports
        .insert(REPORT_ID_DESIGNCAPACITY, [100].into());
    device
        .reports
        .insert(REPORT_ID_FULLCHRGECAPACITY, [100].into());
    device
        .reports
        .insert(REPORT_ID_REMAININGCAPACITY, [90].into());
    device
        .reports
        .insert(REPORT_ID_RUNTIMETOEMPTY, [121].into()); /* Minutes remaining */

    DummyDevice {
        device: RwLock::new(device),
        pending: Mutex::new(VecDeque::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn present_status() {
        let status = PresentStatus {
            discharging: true,
            shutdown_imminent: true,
            ..Default::default()
        };

        let data = struct_to_vec(status);

        assert_eq!(data, [0x02, 0x08]);
    }

    #[test]
    fn identification() {
        let value = Identification {
            i_product: 1,
            i_manufacturer: 2,
            i_serial: 3,
            ..Default::default()
        };

        let data = struct_to_vec(value);

        assert_eq!(data, [0x01, 0x02, 0x03]);
    }

    #[test]
    fn print_report() {
        println!("{:x?}", UPS_REPORT_DESCRIPTOR);
    }
}
