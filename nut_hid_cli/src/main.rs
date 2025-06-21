use std::sync::Arc;
use std::sync::mpsc::{Sender, channel};
use std::{ffi::c_void, thread::sleep, time::Duration};

mod constants;
mod properties;
use constants::*;
use properties::*;

use windows::Win32::Devices::Enumeration::Pnp::SwDeviceClose;
use windows::Win32::Foundation::S_OK;
use windows::{
    Win32::Devices::Enumeration::Pnp::{
        HSWDEVICE, SW_DEVICE_CREATE_INFO, SWDeviceCapabilitiesDriverRequired,
        SWDeviceCapabilitiesRemovable, SWDeviceCapabilitiesSilentInstall, SwDeviceCreate,
    },
    core::HRESULT,
};

use windows_strings::{PCWSTR, w};

type CallbackData = Result<String, HRESULT>;

extern "system" fn create_callback(
    _device: HSWDEVICE,
    result: HRESULT,
    context: *const c_void,
    device_instance_id: PCWSTR,
) {
    println!("Device created");

    let sender = context as *const Sender<CallbackData>;
    unsafe {
        Arc::increment_strong_count(sender);
    }
    let sender = unsafe { Arc::from_raw(context as *const Sender<CallbackData>) };

    if result == S_OK {
        let id = unsafe { device_instance_id.to_string().unwrap() };
        sender.send(Ok(id)).unwrap();
    } else {
        sender.send(Err(result)).unwrap();
    }
}

use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Backend to use
    #[arg(long, default_value = "dummy")]
    backend: String,

    /// Host to connect to if supported
    #[arg(long, default_value = "localhost")]
    host: String,

    /// Port to connect to if supported
    #[arg(long, default_value_t = 3494)]
    port: u32,
}

fn main() {
    println!("Creating device");

    let args = Args::parse();

    let mut properties = PropertiesStore::new();

    properties.add_string(DEVPROP_NUTHID_GUID, DEVPROP_NUTHID_KEY_HOST, &args.host);
    properties.add_string(
        DEVPROP_NUTHID_GUID,
        DEVPROP_NUTHID_KEY_BACKEND,
        &args.backend,
    );
    properties.add_u32(DEVPROP_NUTHID_GUID, DEVPROP_NUTHID_KEY_PORT, args.port);

    println!("With properties {:?}", properties);

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
            Some(properties.get()),
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
    sleep(Duration::from_secs(30));

    println!("Closing device");
    unsafe {
        SwDeviceClose(device);
    }

    /* recover sender */
    drop(unsafe { Arc::from_raw(sender) })
}
