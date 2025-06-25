use std::collections::{HashMap, VecDeque};
use std::sync::{Mutex, RwLock};
use std::thread;
use std::time::Duration;

use super::*;
use binary_serde::recursive_array::RecursiveArray;
use binary_serde::{BinarySerde, BitfieldBitOrder, Endianness, binary_serde_bitfield};
use constants::*;
use log::{debug, error, info, warn};

use rups::blocking::Connection;
use rups::{ClientError, ConfigBuilder, NutError};
use std::collections::HashSet;
use std::convert::TryInto;

const STRING_ID_MANUFACTURER: u8 = 0x01;
const STRING_ID_PRODUCT: u8 = 0x02;
const STRING_ID_SERIAL: u8 = 0x03;
const REPORT_ID_IDENTIFICAITON: u8 = 0x01; // FEATURE ONLY
const REPORT_ID_PRESENTSTATUS: u8 = 0x07; // INPUT OR FEATURE(required by Windows)
const REPORT_ID_MANUFACTUREDATE: u8 = 0x09;
const REPORT_ID_REMAININGCAPACITY: u8 = 0x0C; // 12 INPUT OR FEATURE(required by Windows)
const REPORT_ID_RUNTIMETOEMPTY: u8 = 0x0D;
const REPORT_ID_FULLCHRGECAPACITY: u8 = 0x0E; // 14 FEATURE ONLY. Last Full Charge Capacity 
const REPORT_ID_REMNCAPACITYLIMIT: u8 = 0x11;
const REPORT_ID_CAPACITYMODE: u8 = 0x16;
const REPORT_ID_DESIGNCAPACITY: u8 = 0x17;

#[rustfmt::skip]
const UPS_REPORT_DESCRIPTOR: &[u8] = &[
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
    0x85, REPORT_ID_CAPACITYMODE, //     REPORT_ID (22)
    0x09, 0x2C, //     USAGE (CapacityMode)
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
    0x85, REPORT_ID_REMNCAPACITYLIMIT, //     REPORT_ID (17)
    0x09, 0x29, //     USAGE (RemainingCapacityLimit)
    0xB1, 0xA2, //     FEATURE (Data, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Volatile, Bitfield)
    0x85, REPORT_ID_MANUFACTUREDATE, //     REPORT_ID (9)
    0x09, 0x85, //     USAGE (ManufacturerDate)
    0x75, 0x10, //     REPORT_SIZE (16)
    0x27, 0xFF, 0xFF, 0x00, 0x00, //     LOGICAL_MAXIMUM (65534)
    0xB1, 0xA3, //     FEATURE (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Volatile, Bitfield)
    0x85, REPORT_ID_RUNTIMETOEMPTY, //     REPORT_ID (13)    
    0x09, 0x68, //     USAGE (RunTimeToEmpty)  
    0x81, 0xA3, //     INPUT (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Bitfield)
    0x09, 0x68, //     USAGE (RunTimeToEmpty)
    0xB1, 0xA3, //     FEATURE (Constant, Variable, Absolute, No Wrap, Linear, No Preferred, No Null Position, Volatile, Bitfield)      
    0x05, 0x84, //     USAGE_PAGE (Power Device)
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
    remaining_time_limit_expired: bool, // bit 0x05
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

pub struct NutState {
    pending: VecDeque<(u8, Vec<u8>)>,
    connection: Option<Connection>,
    name: String,
}

pub struct NutDevice {
    device: RwLock<DeviceData>,
    device_config: DeviceConfig,
    state: Mutex<NutState>,
}

fn connect(config: &DeviceConfig) -> Result<(Connection, String), ClientError> {
    let config = ConfigBuilder::new()
        .with_host((config.host.clone(), config.port as u16).try_into()?)
        .with_debug(false)
        .build();

    debug!("Connecting: {:?}", &config);
    let mut connection = Connection::new(&config)?;

    let ups = connection.list_ups()?;
    let (name, description) = ups
        .first()
        .ok_or(nut::ClientError::generic("No ups found"))?;

    debug!("Using ups {name} - {description}");

    Ok((connection, name.clone()))
}

impl NutState {
    fn get_u8(&mut self, variable: &str) -> Result<Option<u8>, ClientError> {
        if let Some(value) = self.get_str(variable)? {
            return Ok(u8::from_str_radix(&value, 10).ok());
        }
        Ok(None)
    }

    fn get_str(&mut self, variable: &str) -> Result<Option<String>, ClientError> {
        let connection = self.connection.as_mut().unwrap();
        match connection.get_var(&self.name, variable) {
            Err(ClientError::Nut(NutError::VarNotSupported)) => {
                debug!("Variable {} not supported", variable);
                return Ok(None);
            }
            Err(ClientError::Nut(NutError::Generic(err))) if err.contains("VAR-NOT-SUPPORTED") => {
                debug!("Variable {} not supported", variable);
                return Ok(None);
            }
            Err(err) => {
                error!("Failed to get {}: {}", variable, err);
                return Err(err);
            }
            Ok(status) => {
                return Ok(Some(status.value()));
            }
        };
    }

    fn update(&mut self) -> Result<(), ClientError> {
        if let Some(value) = self.get_u8("battery.charge")? {
            self.pending
                .push_back((REPORT_ID_REMAININGCAPACITY, vec![value]));
        }

        if let Some(value) = self.get_u8("battery.charge.low")? {
            self.pending
                .push_back((REPORT_ID_REMNCAPACITYLIMIT, vec![value]));
        }

        if let Some(value) = self.get_u8("battery.runtime")? {
            self.pending
                .push_back((REPORT_ID_RUNTIMETOEMPTY, vec![value / 60]));
        }

        let ups_status = self.get_str("ups.status")?.or(Some("".into())).unwrap();
        let battery_charger_status = self
            .get_str("battery_charger_status")?
            .or(Some("".into()))
            .unwrap();

        let present_status = PresentStatus::from_status(&ups_status, &battery_charger_status);
        debug!("Present status: {:?}", present_status);

        self.pending
            .push_back((REPORT_ID_PRESENTSTATUS, struct_to_vec(present_status)));
        Ok(())
    }
}

impl PresentStatus {
    fn from_status(ups_status: &str, battery_charger_status: &str) -> PresentStatus {
        let fields = ups_status.split(' ');
        let values = HashSet::<&str>::from_iter(fields);

        PresentStatus {
            charging: values.contains("CHRG") || battery_charger_status == "charging",
            discharging: values.contains("DISCHRG") || battery_charger_status == "discharging",
            ac_present: values.contains("OL"),
            overload: values.contains("OVER"),
            fully_charged: values.contains("HB"),
            below_remaining_capacity_limit: values.contains("LB"),
            communication_lost: values.contains("OFF")
                || values.contains("WAIT")
                || ups_status == "",
            battery_present: !values.contains("BYPASS"),
            need_replacement: values.contains("RB"),
            ..Default::default()
        }
    }
}

impl NutDevice {
    fn lost_connection_report() -> (u8, Vec<u8>) {
        (
            REPORT_ID_PRESENTSTATUS,
            struct_to_vec(PresentStatus {
                communication_lost: true,
                ..Default::default()
            }),
        )
    }
}

impl Device for NutDevice {
    fn data(&self) -> &RwLock<DeviceData> {
        &self.device
    }

    fn read(&self) -> Option<(u8, Vec<u8>)> {
        /* get all pending */
        let mut state = self.state.lock().unwrap();
        if let Some(report) = state.pending.pop_front() {
            return Some(report);
        }

        /* only update periodically */
        thread::sleep(Duration::from_secs(2));

        if state.connection.is_none() {
            let (connection, name) = match connect(&self.device_config) {
                Err(err) => {
                    error!("Failed to connect {:?}", err);
                    return Some(NutDevice::lost_connection_report());
                }
                Ok((connection, name)) => (connection, name),
            };
            state.connection = Some(connection);
            state.name = name;
        }

        if let Err(err) = state.update() {
            error!("Failed to update state: {}", err);
            let connection = state.connection.take().unwrap();

            if let Err(err) = connection.close() {
                warn!("Failed to close connection: {}", err);
            }

            state.pending.clear();
            return Some(NutDevice::lost_connection_report());
        }

        state.pending.pop_front()
    }
}

fn struct_to_vec<T: BinarySerde>(data: T) -> Vec<u8> {
    data.binary_serialize_to_array(Endianness::Little)
        .as_slice()
        .into()
}

pub fn new_nut_device(device_config: DeviceConfig) -> NutDevice {
    info!("Creating NUT backend");
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
        communication_lost: true,
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

    NutDevice {
        device: RwLock::new(device),
        device_config: device_config,
        state: NutState {
            connection: None,
            name: "".into(),
            pending: VecDeque::new(),
        }
        .into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn present_status_to_bytes() {
        let status = PresentStatus {
            discharging: true,
            shutdown_imminent: true,
            ..Default::default()
        };

        let data = struct_to_vec(status);

        assert_eq!(data, [0x02, 0x08]);
    }

    #[test]
    fn present_status_from_status() {
        let status = PresentStatus::from_status("CHRG DISCHRG OL", "");
        assert_eq!(
            status,
            PresentStatus {
                charging: true,
                discharging: true,
                ac_present: true,
                battery_present: true,
                ..Default::default()
            }
        );

        let status = PresentStatus::from_status("OL", "");
        assert_eq!(
            status,
            PresentStatus {
                ac_present: true,
                battery_present: true,
                ..Default::default()
            }
        );

        let status = PresentStatus::from_status("OL RB", "charging");
        assert_eq!(
            status,
            PresentStatus {
                charging: true,
                ac_present: true,
                need_replacement: true,
                battery_present: true,
                ..Default::default()
            }
        );

        let status = PresentStatus::from_status("OB OVER", "discharging");
        assert_eq!(
            status,
            PresentStatus {
                discharging: true,
                ac_present: false,
                overload: true,
                battery_present: true,
                ..Default::default()
            }
        );
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
