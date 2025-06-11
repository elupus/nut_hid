use std::sync::Arc;
use std::sync::mpsc::{Sender, channel};
use std::{ffi::c_void, thread::sleep, time::Duration};

use windows::Win32::Devices::Enumeration::Pnp::SwDeviceClose;
use windows::Win32::Foundation::S_OK;
use windows::{
    Win32::{
        Devices::{
            Enumeration::Pnp::{
                HSWDEVICE, SW_DEVICE_CREATE_INFO, SWDeviceCapabilitiesDriverRequired,
                SWDeviceCapabilitiesRemovable, SWDeviceCapabilitiesSilentInstall, SwDeviceCreate,
            },
            Properties::{DEVPROP_STORE_SYSTEM, DEVPROP_TYPE_STRING, DEVPROPCOMPKEY, DEVPROPERTY},
        },
        Foundation::DEVPROPKEY,
    },
    core::{GUID, HRESULT},
};

use windows_strings::{PCWSTR, w};

/* 53c0d411-cfb1-4d29-8f81-e705f3ac17a1 */
pub const DEVPROP_NUTHID_GUID: GUID = GUID {
    data1: 0x53c0d411,
    data2: 0xcfb1,
    data3: 0x4d29,
    data4: [0x8f, 0x81, 0xe7, 0x05, 0xf3, 0xac, 0x17, 0xa1],
};

pub const DEVPROP_NUTHID_KEY_HOST: DEVPROPKEY = DEVPROPKEY {
    fmtid: DEVPROP_NUTHID_GUID,
    pid: 2,
};

pub const DEVPROP_NUTHID_COMPKEY_HOST: DEVPROPCOMPKEY = DEVPROPCOMPKEY {
    Store: DEVPROP_STORE_SYSTEM,
    Key: DEVPROP_NUTHID_KEY_HOST,
    LocaleName: PCWSTR::null(),
};

const ENUMERATOR_NAME: PCWSTR = w!("NutHidEnumerator");
const HARDWARE_IDS: PCWSTR = w!("root\\NutHidDevice\0");
const INSTANCE_ID: PCWSTR = w!("NutHidInstance");
const DEVICE_DESCRIPTION: PCWSTR = w!("NUT Hid Device");
const PARENT_DEVICE_INSTANCE: PCWSTR = w!("HTREE\\ROOT\\0");

type CallbackData = Result<String, HRESULT>;

extern "system" fn create_callback(
    _device: HSWDEVICE,
    result: HRESULT,
    context: *const c_void,
    device_instance_id: PCWSTR,
) {
    println!("Device created");

    let sender = context as *const Sender<CallbackData>;
    unsafe { Arc::increment_strong_count(sender); }
    let sender = unsafe { Arc::from_raw(context as *const Sender<CallbackData>) };

    if result == S_OK {
        let id = unsafe { device_instance_id.to_string().unwrap() };
        sender.send(Ok(id)).unwrap();
    } else {
        sender.send(Err(result)).unwrap();
    }
}

fn main() {
    println!("Creating device");

    let hostname: PCWSTR = w!("nuthost");
    let hostname_len = unsafe { hostname.len() + 1 } * size_of::<u16>();

    let property_hostname = DEVPROPERTY {
        Type: DEVPROP_TYPE_STRING,
        CompKey: DEVPROP_NUTHID_COMPKEY_HOST,
        BufferSize: hostname_len as u32,
        Buffer: hostname.as_ptr() as *mut c_void,
    };

    let info = SW_DEVICE_CREATE_INFO {
        cbSize: size_of::<SW_DEVICE_CREATE_INFO>() as u32,
        pszInstanceId: INSTANCE_ID,
        pszzHardwareIds: HARDWARE_IDS,
        pszzCompatibleIds: w!(""),
        pszDeviceDescription: DEVICE_DESCRIPTION,
        CapabilityFlags: (SWDeviceCapabilitiesRemovable.0
            + SWDeviceCapabilitiesSilentInstall.0
            + SWDeviceCapabilitiesDriverRequired.0) as u32,
        ..Default::default()
    };

    let (sender, receiver): (Sender<CallbackData>, _) = channel();

    /* convert to raw ptr that need to live until we close the device */
    let sender = Arc::into_raw(sender.into());

    let device = unsafe {
        SwDeviceCreate(
            ENUMERATOR_NAME,
            PARENT_DEVICE_INSTANCE,
            &info,
            Some(&[property_hostname]),
            Some(create_callback),
            Some(sender as *const c_void),
        )
        .unwrap()
    };

    println!("Waiting for device");
    let device_instance_id = receiver
        .recv_timeout(Duration::from_secs(5))
        .unwrap()
        .unwrap();

    println!("Waiting for use of device {device_instance_id}");
    sleep(Duration::from_secs(5));

    println!("Closing device");
    unsafe {
        SwDeviceClose(device);
    }

    /* recover sender */
    drop(unsafe { Arc::from_raw(sender) })
}
