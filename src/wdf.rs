use std::slice;
use std::{ffi::c_void, ptr};

use wdk_sys::{
    _WDF_EXECUTION_LEVEL::WdfExecutionLevelInheritFromParent,
    _WDF_SYNCHRONIZATION_SCOPE::WdfSynchronizationScopeInheritFromParent, NT_SUCCESS, NTSTATUS,
    ULONG, WDF_OBJECT_ATTRIBUTES, WDF_OBJECT_CONTEXT_TYPE_INFO, WDFMEMORY, WDFMEMORY__, WDFOBJECT,
    WDFREQUEST, WDFREQUEST__, call_unsafe_wdf_function_binding,
};

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

pub trait WdfContext {
    fn get_type_info() -> &'static WDF_OBJECT_CONTEXT_TYPE_INFO;
    fn get_object_attributes() -> WDF_OBJECT_ATTRIBUTES
    {
        wdf_object_attributes_init_context_type(Self::get_type_info())
    }
}

pub fn wdf_get_context_raw<T>(object: WDFOBJECT) -> *mut T
where
    T: WdfContext,
{
    let context_ptr;
    unsafe {
        context_ptr = call_unsafe_wdf_function_binding!(
            WdfObjectGetTypedContextWorker,
            object,
            T::get_type_info()
        );
    }
    assert!(!context_ptr.is_null());
    context_ptr as *mut T
}

pub fn wdf_get_context<T>(object: WDFOBJECT) -> &'static mut T
where
    T: WdfContext,
{
    let context_ptr = wdf_get_context_raw::<T>(object);
    unsafe { &mut *context_ptr }
}

pub fn wdf_init_context<T>(object: WDFOBJECT, context: T)
where
    T: WdfContext,
{
    let context_raw = wdf_get_context_raw::<T>(object);
    unsafe {
        context_raw.write(context);
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

    pub fn complete(&mut self, status: NTSTATUS) {
        unsafe { call_unsafe_wdf_function_binding!(WdfRequestComplete, self.0, status) }
    }
}

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
