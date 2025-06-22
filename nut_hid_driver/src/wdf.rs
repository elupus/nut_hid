use std::ffi::CStr;
use std::slice;
use std::sync::Arc;
use std::{ffi::c_void, ptr};

use log::{debug, warn};
use wdk_sys::{
    _WDF_EXECUTION_LEVEL::WdfExecutionLevelInheritFromParent,
    _WDF_SYNCHRONIZATION_SCOPE::WdfSynchronizationScopeInheritFromParent, NT_SUCCESS, NTSTATUS,
    ULONG, WDF_OBJECT_ATTRIBUTES, WDF_OBJECT_CONTEXT_TYPE_INFO, WDFDEVICE, WDFMEMORY, WDFMEMORY__,
    WDFOBJECT, WDFREQUEST, call_unsafe_wdf_function_binding,
};
use wdk_sys::{
    DEVPROP_TYPE_STRING, DEVPROP_TYPE_UINT32, DEVPROPKEY, DEVPROPTYPE, GUID, STATUS_BAD_DATA,
    STATUS_BUFFER_TOO_SMALL, STATUS_SUCCESS, WDF_DEVICE_PROPERTY_DATA,
};

use crate::backports::from_utf16le_lossy;

pub fn wdf_object_attributes_init() -> WDF_OBJECT_ATTRIBUTES {
    WDF_OBJECT_ATTRIBUTES {
        Size: core::mem::size_of::<WDF_OBJECT_ATTRIBUTES>() as ULONG,
        ExecutionLevel: WdfExecutionLevelInheritFromParent,
        SynchronizationScope: WdfSynchronizationScopeInheritFromParent,
        ..WDF_OBJECT_ATTRIBUTES::default()
    }
}

pub fn wdf_object_attributes_init_context_type(
    context_type: &'static WDF_OBJECT_CONTEXT_TYPE_INFO,
) -> WDF_OBJECT_ATTRIBUTES {
    let mut attributes = wdf_object_attributes_init();
    if context_type.UniqueType.is_null() {
        attributes.ContextTypeInfo = context_type
    } else {
        attributes.ContextTypeInfo = context_type.UniqueType
    }
    attributes
}

pub const fn wdf_object_context_type_info_get<T: WdfContext>(
    name: &'static CStr,
) -> WDF_OBJECT_CONTEXT_TYPE_INFO {
    WDF_OBJECT_CONTEXT_TYPE_INFO {
        Size: core::mem::size_of::<WDF_OBJECT_CONTEXT_TYPE_INFO>() as ULONG,
        ContextName: name.as_ptr(),
        ContextSize: core::mem::size_of::<Option<T::Type>>(),
        UniqueType: std::ptr::null(),
        EvtDriverGetUniqueContextType: None,
    }
}

pub trait WdfContext {
    type Type: Clone;

    unsafe extern "C" fn destroy(object: WDFOBJECT) {
        debug!("Destroy context!");
        unsafe {
            let ptr = Self::get_raw(object);
            drop(ptr.read());
        }
    }

    fn get_type_info() -> &'static WDF_OBJECT_CONTEXT_TYPE_INFO;

    fn get_object_attributes() -> WDF_OBJECT_ATTRIBUTES {
        let mut attributes = wdf_object_attributes_init_context_type(Self::get_type_info());
        attributes.EvtDestroyCallback = Some(Self::destroy);
        attributes
    }

    fn get_raw(object: WDFOBJECT) -> *mut Option<Self::Type> {
        let context_ptr: *mut c_void;
        unsafe {
            context_ptr = call_unsafe_wdf_function_binding!(
                WdfObjectGetTypedContextWorker,
                object,
                Self::get_type_info()
            );
        }
        assert!(!context_ptr.is_null());
        context_ptr as *mut Option<Self::Type>
    }

    fn from_object(object: WDFOBJECT) -> Self::Type {
        unsafe {
            let ptr = Self::get_raw(object);
            let val = (*ptr).as_ref();
            val.expect("Object was never initialized").clone()
        }
    }

    fn init(object: WDFOBJECT, data: Option<Self::Type>) {
        let ptr = Self::get_raw(object);
        unsafe {
            ptr.write(data);
        }
    }
}

pub struct WdfRequest(pub WDFREQUEST);

impl WdfRequest {
    pub fn get_input_memory(&self) -> Result<WdfMemory, NTSTATUS> {
        let mut memory: WDFMEMORY = ptr::null_mut::<WDFMEMORY__>();

        let status;
        unsafe {
            status = call_unsafe_wdf_function_binding!(
                WdfRequestRetrieveInputMemory,
                self.0,
                &mut memory
            )
        }
        if !NT_SUCCESS(status) {
            println!("Failed to get input memory {status}");
            return Err(status);
        }
        assert!(!memory.is_null());
        Ok(WdfMemory(memory))
    }

    pub fn get_output_memory(&self) -> Result<WdfMemory, NTSTATUS> {
        let mut memory: WDFMEMORY = ptr::null_mut::<WDFMEMORY__>();

        let status;
        unsafe {
            status = call_unsafe_wdf_function_binding!(
                WdfRequestRetrieveOutputMemory,
                self.0,
                &mut memory
            )
        }
        if !NT_SUCCESS(status) {
            println!("Failed to get output memory {status}");
            return Err(status);
        }
        assert!(!memory.is_null());
        Ok(WdfMemory(memory))
    }

    pub fn set_information(&mut self, len: usize) {
        unsafe {
            call_unsafe_wdf_function_binding!(WdfRequestSetInformation, self.0, len as u64);
        }
    }

    pub fn complete(self, status: NTSTATUS) {
        unsafe { call_unsafe_wdf_function_binding!(WdfRequestComplete, self.0, status) }
    }
}

unsafe impl Send for WdfRequest {}
unsafe impl Sync for WdfRequest {}

pub struct WdfMemory(pub WDFMEMORY);

impl WdfMemory {
    pub fn get_buffer(&self) -> &[u8] {
        unsafe {
            let mut len: usize = 0;
            let buf = call_unsafe_wdf_function_binding!(WdfMemoryGetBuffer, self.0, &mut len);
            slice::from_raw_parts(buf as *const u8, len)
        }
    }

    pub fn get_buffer_mut(&mut self) -> &mut [u8] {
        unsafe {
            let mut len: usize = 0;
            let buf = call_unsafe_wdf_function_binding!(WdfMemoryGetBuffer, self.0, &mut len);
            slice::from_raw_parts_mut(buf as *mut u8, len)
        }
    }

    pub fn copy_from_slice<T>(&mut self, data: &[T], offset: usize) -> Result<usize, NTSTATUS> {
        let len = size_of::<T>() * data.len();
        let status;
        let ptr = data.as_ptr();
        unsafe {
            status = call_unsafe_wdf_function_binding!(
                WdfMemoryCopyFromBuffer,
                self.0,
                offset,
                ptr as *mut c_void,
                len
            );
        }

        if !NT_SUCCESS(status) {
            println!("Failed to copy from buffer {status}");
            return Err(status);
        }

        return Ok(len);
    }
}

pub fn wdf_device_query_property_ex(
    device: WDFDEVICE,
    device_property_data: &mut WDF_DEVICE_PROPERTY_DATA,
) -> Result<(Vec<u8>, DEVPROPTYPE), NTSTATUS> {
    let mut size: ULONG = 0;
    let mut property_type: DEVPROPTYPE = 0;
    unsafe {
        let status = call_unsafe_wdf_function_binding!(
            WdfDeviceQueryPropertyEx,
            device,
            device_property_data,
            0,
            ptr::null_mut(),
            &mut size,
            &mut property_type,
        );
        if status != STATUS_BUFFER_TOO_SMALL {
            warn!(
                "Failed to get property {:?} -> {}",
                device_property_data, status
            );
            return Err(status);
        }
    }

    let mut data = Vec::<u8>::with_capacity(size as usize);
    let uninit = data.spare_capacity_mut();

    unsafe {
        let status = call_unsafe_wdf_function_binding!(
            WdfDeviceQueryPropertyEx,
            device,
            device_property_data,
            size,
            uninit.as_mut_ptr() as *mut c_void,
            &mut size,
            &mut property_type,
        );
        if status != STATUS_SUCCESS {
            warn!(
                "Failed to get property {:?} -> {}",
                device_property_data, status
            );
            return Err(status);
        }
        data.set_len(size as usize);
    }

    Ok((data, property_type))
}

fn wdf_device_query_property_data(
    device: *mut wdk_sys::WDFDEVICE__,
    fmtid: wdk_sys::GUID,
    pid: u32,
) -> Result<(Vec<u8>, u32), NTSTATUS> {
    let device_property_key = DEVPROPKEY { fmtid, pid };
    let mut device_property_data = WDF_DEVICE_PROPERTY_DATA {
        Size: size_of::<WDF_DEVICE_PROPERTY_DATA>() as ULONG,
        PropertyKey: &device_property_key,
        Lcid: 0, /*LOCALE_NEUTRAL*/
        Flags: 0,
        ..Default::default()
    };
    let (data, property_type) = wdf_device_query_property_ex(device, &mut device_property_data)?;
    Ok((data, property_type))
}

pub fn wdf_device_query_property_string(
    device: WDFDEVICE,
    fmtid: GUID,
    pid: u32,
) -> Result<String, NTSTATUS> {
    let (data, property_type) = wdf_device_query_property_data(device, fmtid, pid)?;
    if property_type != DEVPROP_TYPE_STRING {
        warn!("Unexpected device property type: {property_type}");
        return Err(STATUS_BAD_DATA);
    }

    let result = from_utf16le_lossy(&data);

    match result.split_once(char::from(0)) {
        Some((result, _)) => {
            return Ok(result.to_string());
        }
        None => {
            warn!("Unexpected device property data: {result}");
            return Err(STATUS_BAD_DATA);
        }
    }
}

pub fn wdf_device_query_property_u32(
    device: WDFDEVICE,
    fmtid: GUID,
    pid: u32,
) -> Result<u32, NTSTATUS> {
    let (data, property_type) = wdf_device_query_property_data(device, fmtid, pid)?;
    if property_type != DEVPROP_TYPE_UINT32 {
        warn!("Unexpected device property type: {property_type}");
        return Err(STATUS_BAD_DATA);
    }

    let value: [u8; 4] = data.try_into().map_err(|_| STATUS_BAD_DATA)?;
    Ok(u32::from_ne_bytes(value))
}
