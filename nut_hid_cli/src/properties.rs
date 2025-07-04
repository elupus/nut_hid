use std::os::raw::c_void;

use windows::{
    Win32::{
        Devices::Properties::{
            DEVPROP_STORE_SYSTEM, DEVPROP_TYPE_STRING, DEVPROP_TYPE_UINT32, DEVPROPCOMPKEY,
            DEVPROPERTY,
        },
        Foundation::DEVPROPKEY,
    },
    core::GUID,
};
use windows_strings::PCWSTR;

pub const fn get_comp_key(fmtid: GUID, pid: u32) -> DEVPROPCOMPKEY {
    DEVPROPCOMPKEY {
        Store: DEVPROP_STORE_SYSTEM,
        Key: DEVPROPKEY {
            fmtid: fmtid,
            pid: pid,
        },
        LocaleName: PCWSTR::null(),
    }
}

#[derive(Debug)]
pub struct PropertiesStore {
    strings: Vec<Box<widestring::U16CStr>>,
    data: Vec<Box<Vec<u8>>>,
    properties: Vec<DEVPROPERTY>,
}

impl PropertiesStore {
    pub fn new() -> PropertiesStore {
        PropertiesStore {
            strings: Vec::new(),
            data: Vec::new(),
            properties: Vec::new(),
        }
    }

    pub fn add_string(&mut self, fmtid: GUID, pid: u32, value: &str) {
        let key = get_comp_key(fmtid, pid);
        let value = widestring::U16CString::from_str(value)
            .unwrap()
            .into_boxed_ucstr();

        let value_ptr = value.as_ptr();
        let value_len = (value.len() + 1) * size_of::<u16>();
        self.strings.push(value);

        let property = DEVPROPERTY {
            Type: DEVPROP_TYPE_STRING,
            CompKey: key,
            BufferSize: value_len as u32,
            Buffer: value_ptr as *mut c_void,
        };
        self.properties.push(property);
    }

    pub fn add_u32(&mut self, fmtid: GUID, pid: u32, value: u32) {
        let key = get_comp_key(fmtid, pid);

        let value: Vec<u8> = value.to_ne_bytes().into();
        self.data.push(value.into());
        let value = self.data.last().unwrap();

        let property = DEVPROPERTY {
            Type: DEVPROP_TYPE_UINT32,
            CompKey: key,
            BufferSize: value.len() as u32,
            Buffer: value.as_ptr() as *mut c_void,
        };
        self.properties.push(property);
    }

    pub fn get<'a>(&'a self) -> &'a Vec<DEVPROPERTY> {
        &self.properties
    }
}
