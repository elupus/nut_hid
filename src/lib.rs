use std::{os::windows::ffi::OsStrExt, slice, string::String};

use wdk_sys::{
    _HID_DESCRIPTOR__HID_DESCRIPTOR_DESC_LIST, _WDF_IO_QUEUE_DISPATCH_TYPE,
    _WDF_TRI_STATE::WdfUseDefault, HID_DESCRIPTOR, HID_DEVICE_ATTRIBUTES, NT_ERROR, NT_SUCCESS,
    NTSTATUS, PCUNICODE_STRING, PDRIVER_OBJECT, STATUS_INVALID_BUFFER_SIZE,
    STATUS_INVALID_PARAMETER, STATUS_NOT_IMPLEMENTED, STATUS_SUCCESS, ULONG, UNICODE_STRING, WCHAR,
    WDF_DRIVER_CONFIG, WDF_IO_QUEUE_CONFIG, WDF_IO_QUEUE_DISPATCH_TYPE, WDF_NO_HANDLE,
    WDF_NO_OBJECT_ATTRIBUTES, WDF_OBJECT_ATTRIBUTES, WDF_OBJECT_CONTEXT_TYPE_INFO, WDFDEVICE,
    WDFDEVICE_INIT, WDFDRIVER, WDFOBJECT, WDFQUEUE, WDFREQUEST, call_unsafe_wdf_function_binding,
};

use wdk::println;
mod constants;
mod hid;
mod wdf;

use hid::*;
use nut_hid_device::*;
use wdf::*;

use std::ffi::OsStr;

struct DeviceContext {
    device: WDFDEVICE,
    queue: WDFQUEUE,
    hid_device: Device,
    hid_device_desc: HID_DESCRIPTOR,
    hid_device_attr: HID_DEVICE_ATTRIBUTES,
}

const DEVICE_CONTEXT_TYPE_INFO: WDF_OBJECT_CONTEXT_TYPE_INFO = WDF_OBJECT_CONTEXT_TYPE_INFO {
    Size: core::mem::size_of::<WDF_OBJECT_CONTEXT_TYPE_INFO>() as ULONG,
    ContextName: c"DeviceContext".as_ptr(),
    ContextSize: core::mem::size_of::<DeviceContext>(),
    UniqueType: std::ptr::null(),
    EvtDriverGetUniqueContextType: None,
};

impl WdfContext for DeviceContext {
    fn get_type_info() -> &'static WDF_OBJECT_CONTEXT_TYPE_INFO {
        &DEVICE_CONTEXT_TYPE_INFO
    }
}

struct QueueContext {
    queue: WDFQUEUE,
    device: WDFDEVICE,
}

const QUEUE_CONTEXT_TYPE_INFO: WDF_OBJECT_CONTEXT_TYPE_INFO = WDF_OBJECT_CONTEXT_TYPE_INFO {
    Size: core::mem::size_of::<WDF_OBJECT_CONTEXT_TYPE_INFO>() as ULONG,
    ContextName: c"QueueContext".as_ptr(),
    ContextSize: core::mem::size_of::<QueueContext>(),
    UniqueType: std::ptr::null(),
    EvtDriverGetUniqueContextType: None,
};

impl WdfContext for QueueContext {
    fn get_type_info() -> &'static WDF_OBJECT_CONTEXT_TYPE_INFO {
        &QUEUE_CONTEXT_TYPE_INFO
    }
}

unsafe fn unicode_string_to_rust(string: UNICODE_STRING) -> String {
    // Translate UTF16 string to rust string
    let number_of_slice_elements = {
        string.Length as usize
            / core::mem::size_of_val(
                // SAFETY: This dereference is safe since `Buffer` is:
                //         * provided by `DriverEntry` and is never null
                //         * a valid pointer to `Buffer`'s type
                &unsafe { *string.Buffer },
            )
    };

    String::from_utf16_lossy(unsafe {
        // SAFETY: This is safe because:
        //         1. `string.Buffer` is valid for reads for `number_of_slice_elements` *
        //            `core::mem::size_of::<WCHAR>()` bytes, and is guaranteed to be aligned and it
        //            must be properly aligned.
        //         2. `registry_path.Buffer` points to `number_of_slice_elements` consecutive
        //            properly initialized values of type `WCHAR`.
        //         3. Windows does not mutate the memory referenced by the returned slice for for
        //            its entire lifetime.
        //         4. The total size, `number_of_slice_elements` * `core::mem::size_of::<WCHAR>()`,
        //            of the slice must be no larger than `isize::MAX`. This is proven by the below
        //            `debug_assert!`.
        debug_assert!(
            isize::try_from(number_of_slice_elements * core::mem::size_of::<WCHAR>()).is_ok()
        );
        slice::from_raw_parts(string.Buffer, number_of_slice_elements)
    })
}

fn wdf_io_queue_config_init_default_queue(
    dispatch_type: WDF_IO_QUEUE_DISPATCH_TYPE,
) -> WDF_IO_QUEUE_CONFIG {
    let mut config = WDF_IO_QUEUE_CONFIG {
        Size: core::mem::size_of::<WDF_IO_QUEUE_CONFIG>() as ULONG,
        PowerManaged: WdfUseDefault,
        DefaultQueue: 1,
        DispatchType: dispatch_type,
        ..WDF_IO_QUEUE_CONFIG::default()
    };
    if dispatch_type == _WDF_IO_QUEUE_DISPATCH_TYPE::WdfIoQueueDispatchParallel {
        config.Settings.Parallel.NumberOfPresentedRequests = ULONG::MAX
    }
    config
}

unsafe extern "C" fn evt_wdf_device_context_cleanup(object: WDFOBJECT) {
    println!("EvtWdfDeviceContextCleanup Entered!");
    let _context = wdf_get_context::<DeviceContext>(object);
}

unsafe extern "C" fn evt_wdf_device_context_destroy(object: WDFOBJECT) {
    println!("EvtWdfDeviceContextDestroy Entered!");
    let _context = wdf_get_context::<DeviceContext>(object);
}

fn wdf_device_create(
    mut device_init: *mut WDFDEVICE_INIT,
    attributes: &mut WDF_OBJECT_ATTRIBUTES,
) -> Result<WDFDEVICE, NTSTATUS> {
    let ntstatus: NTSTATUS;
    let mut device: WDFDEVICE = WDF_NO_HANDLE.cast();

    unsafe {
        // SAFETY: This is safe because:
        //       1. `device_init` is provided by `EvtDriverDeviceAdd` and is never null
        //       2. the argument receiving `WDF_NO_OBJECT_ATTRIBUTES` is allowed to be
        //          null
        //       3. `device_handle_output` is expected to be null
        ntstatus = call_unsafe_wdf_function_binding!(
            WdfDeviceCreate,
            &mut device_init,
            attributes,
            &mut device,
        );

        if !NT_SUCCESS(ntstatus) {
            println!("WdfDeviceCreate failed: {ntstatus:#02x}");
            return Err(ntstatus);
        }
        assert!(!device.is_null());
    }
    Ok(device)
}

extern "C" fn evt_driver_device_add(
    _driver: WDFDRIVER,
    device_init: *mut WDFDEVICE_INIT,
) -> NTSTATUS {
    println!("EvtDriverDeviceAdd Entered!");

    let mut attributes = DeviceContext::get_object_attributes();
    attributes.EvtCleanupCallback = Some(evt_wdf_device_context_cleanup);
    attributes.EvtDestroyCallback = Some(evt_wdf_device_context_destroy);

    unsafe {
        call_unsafe_wdf_function_binding!(WdfFdoInitSetFilter, device_init);
    }

    println!("Creating device");
    let device = match wdf_device_create(device_init, &mut attributes) {
        Err(status) => return status,
        Ok(device) => device,
    };

    println!("Creating default queue");
    let queue = match create_default_queue(device) {
        Err(status) => return status,
        Ok(queue) => queue,
    };

    println!("Build hid descriptors");
    let hid_device = nut_hid_device::nut::new_nut_device();

    let hid_report_desc = &hid_device.report_descriptor;

    let hid_device_desc = HID_DESCRIPTOR {
        bLength: 0x09,
        bDescriptorType: 0x21,
        bcdHID: 0x0100,
        bCountry: 0x00,
        bNumDescriptors: 0x01,
        DescriptorList: [_HID_DESCRIPTOR__HID_DESCRIPTOR_DESC_LIST {
            bReportType: 0x22,
            wReportLength: hid_report_desc.len() as ::core::ffi::c_ushort,
        }],
        ..HID_DESCRIPTOR::default()
    };

    let hid_device_attr = HID_DEVICE_ATTRIBUTES {
        Size: size_of::<HID_DEVICE_ATTRIBUTES>() as u32,
        VendorID: hid_device.vendor_id,
        ProductID: hid_device.product_id,
        VersionNumber: hid_device.version,
        ..HID_DEVICE_ATTRIBUTES::default()
    };

    println!("Creating device context for: {:#?}, {:#?}", hid_device_desc, hid_device_attr);
    let context = DeviceContext {
        device: device,
        queue: queue,
        hid_device: hid_device,
        hid_device_desc: hid_device_desc,
        hid_device_attr: hid_device_attr,
    };
    wdf_init_context::<DeviceContext>(device as WDFOBJECT, context);

    STATUS_SUCCESS
}

extern "C" fn evt_driver_unload(_driver: WDFDRIVER) {
    println!("Goodbye World!");
    println!("Driver Exit Complete!");
}

#[unsafe(export_name = "DriverEntry")]
pub unsafe extern "system" fn driver_entry(
    driver: PDRIVER_OBJECT,
    registry_path: PCUNICODE_STRING,
) -> NTSTATUS {
    println!("Starting NUT HID Driver");

    let mut driver_config = WDF_DRIVER_CONFIG {
        Size: core::mem::size_of::<WDF_DRIVER_CONFIG>() as ULONG,
        EvtDriverDeviceAdd: Some(evt_driver_device_add),
        EvtDriverUnload: Some(evt_driver_unload),
        ..WDF_DRIVER_CONFIG::default()
    };

    let registry_path_rust = unsafe {
        // SAFETY: This dereference is safe since `registry_path` is:
        //         * provided by `DriverEntry` and is never null
        //         * a valid pointer to a `UNICODE_STRING`
        unicode_string_to_rust(*registry_path)
    };

    println!("Registry Parameter Key: {registry_path_rust}");

    let status;
    unsafe {
        // SAFETY: This is safe because:
        //         1. `driver` is provided by `DriverEntry` and is never null
        //         2. `registry_path` is provided by `DriverEntry` and is never null
        //         3. `driver_attributes` is allowed to be null
        //         4. `driver_config` is a valid pointer to a valid `WDF_DRIVER_CONFIG`
        //         5. `driver_handle_output` is expected to be null
        status = call_unsafe_wdf_function_binding!(
            WdfDriverCreate,
            driver,
            registry_path,
            WDF_NO_OBJECT_ATTRIBUTES,
            &mut driver_config,
            WDF_NO_HANDLE.cast::<WDFDRIVER>(),
        );
    }

    println!("Driver Status: {status}");

    status
}

extern "C" fn evt_io_device_control(
    queue: WDFQUEUE,
    request: WDFREQUEST,
    _output_buffer_length: usize,
    _input_buffer_length: usize,
    io_control_code: ULONG,
) {
    let device;
    unsafe {
        device = call_unsafe_wdf_function_binding!(WdfIoQueueGetDevice, queue);
    }

    let device_context = wdf_get_context::<DeviceContext>(device.cast());
    let queue_context = wdf_get_context::<QueueContext>(queue.cast());
    let status: NTSTATUS;
    let mut request = WdfRequest(request);

    match evt_io_device_control_internal(
        &mut request,
        io_control_code,
        device_context,
        queue_context,
    ) {
        Ok(()) => status = STATUS_SUCCESS,
        Err(e) => status = e,
    }

    request.complete(status);
}

fn get_report(memory: &WdfMemory) -> Result<(u8, &[u8]), NTSTATUS> {
    let buffer = memory.get_buffer();
    let len = buffer.len();
    if len < 1 {
        println!("invalid input buffer length {len}");
        return Err(STATUS_INVALID_BUFFER_SIZE);
    }
    Ok((buffer[0], &buffer[1..]))
}

fn get_string_id(memory: &WdfMemory) -> Result<(u32, u32), NTSTATUS> {
    let buffer = memory.get_buffer();

    if buffer.len() < size_of::<ULONG>() {
        return Err(STATUS_INVALID_BUFFER_SIZE);
    }

    let value;
    unsafe {
        value = *(buffer.as_ptr() as *const ULONG);
    }

    let string_id = value & 0xffff;
    let language_id = (value >> 16) & 0xffff;
    Ok((string_id, language_id))
}

fn request_copy_from_slice<T>(request: &mut WdfRequest, data: &[T]) -> Result<(), NTSTATUS> {
    let mut memory = request.get_output_memory()?;

    let len = memory.copy_from_slice(data, 0)?;
    request.set_information(len);
    return Ok(());
}

fn request_copy_from_string(request: &mut WdfRequest, data: &str) -> Result<(), NTSTATUS> {
    let value = OsStr::new(data);
    let encoded = value.encode_wide().chain(Some(0)).collect::<Vec<_>>();
    request_copy_from_slice(request, encoded.as_slice())?;
    Ok(())
}

fn get_string(request: &mut WdfRequest, device_contex: &mut DeviceContext) -> Result<(), NTSTATUS> {
    let (string_id, _) = get_string_id(&request.get_input_memory()?)?;

    println!("get_string {string_id}");

    let hid_device = &device_contex.hid_device;
    let value;
    match string_id {
        HID_STRING_ID_IMANUFACTURER => {
            value = &hid_device.manufacturer;
        }
        HID_STRING_ID_IPRODUCT => {
            value = &hid_device.product;
        }
        HID_STRING_ID_ISERIALNUMBER => {
            value = &hid_device.serial_number;
        }
        _ => return Err(STATUS_NOT_IMPLEMENTED),
    }

    request_copy_from_string(request, value)
}

fn get_indexed_string(
    request: &mut WdfRequest,
    device_contex: &mut DeviceContext,
) -> Result<(), NTSTATUS> {
    let (string_id, _) = get_string_id(&request.get_input_memory()?)?;

    println!("get_indexed_string {string_id}");

    let strings = &device_contex.hid_device.strings;
    let data = strings
        .get(&(string_id as u8))
        .ok_or(STATUS_INVALID_PARAMETER)?;

    request_copy_from_string(request, data)
}

// Read a pending report from device
fn read_report(
    _request: &mut WdfRequest,
    _device_contex: &mut DeviceContext,
) -> Result<(), NTSTATUS> {
    Err(STATUS_NOT_IMPLEMENTED)
}

fn write_report(
    _request: &mut WdfRequest,
    _device_contex: &mut DeviceContext,
) -> Result<(), NTSTATUS> {
    Err(STATUS_NOT_IMPLEMENTED)
}

fn get_report_internal(
    request: &mut WdfRequest,
    device_contex: &mut DeviceContext,
) -> Result<(), NTSTATUS> {
    let input_memory = request.get_input_memory()?;
    let (report_id, _) = get_report(&input_memory)?;

    println!("get_report_internal {report_id}");

    let mut output_memory = request.get_output_memory()?;
    let mut offset = 0;

    let reports = &device_contex.hid_device.reports;
    let data = reports.get(&report_id).ok_or(STATUS_INVALID_PARAMETER)?;
    offset += output_memory.copy_from_slice(slice::from_ref(&report_id), offset)?;
    offset += output_memory.copy_from_slice(data, offset)?;
    request.set_information(offset);

    Ok(())
}

fn get_feature(
    request: &mut WdfRequest,
    device_contex: &mut DeviceContext,
) -> Result<(), NTSTATUS> {
    println!("get_feature");
    get_report_internal(request, device_contex)
}

fn get_input_report(
    request: &mut WdfRequest,
    device_contex: &mut DeviceContext,
) -> Result<(), NTSTATUS> {
    println!("get_feature");
    get_report_internal(request, device_contex)
}

fn set_report_internal(
    request: &mut WdfRequest,
    device_contex: &mut DeviceContext,
) -> Result<(), NTSTATUS> {
    let input_memory = request.get_input_memory()?;
    let (report_id, report) = get_report(&input_memory)?;

    println!("set_report_internal {report_id}");

    let reports = &mut device_contex.hid_device.reports;
    reports.remove(&report_id);
    reports.insert(report_id, report.to_vec());

    Ok(())
}

fn set_output_report(
    request: &mut WdfRequest,
    device_contex: &mut DeviceContext,
) -> Result<(), NTSTATUS> {
    println!("set_output_report");
    set_report_internal(request, device_contex)
}

fn set_feature(
    request: &mut WdfRequest,
    device_contex: &mut DeviceContext,
) -> Result<(), NTSTATUS> {
    println!("set_feature");
    set_report_internal(request, device_contex)
}

fn evt_io_device_control_internal(
    request: &mut WdfRequest,
    io_control_code: ULONG,
    device_context: &mut DeviceContext,
    _queue_context: &mut QueueContext,
) -> Result<(), NTSTATUS> {
    println!("io device control {io_control_code}");

    match io_control_code {
        IOCTL_HID_GET_DEVICE_DESCRIPTOR => {
            request_copy_from_slice(request, slice::from_ref(&device_context.hid_device_desc))?;
        }
        IOCTL_HID_GET_DEVICE_ATTRIBUTES => {
            request_copy_from_slice(request, slice::from_ref(&device_context.hid_device_attr))?
        }
        IOCTL_HID_GET_REPORT_DESCRIPTOR => {
            request_copy_from_slice(request, &device_context.hid_device.report_descriptor)?
        }
        IOCTL_HID_GET_STRING => {
            get_string(request, device_context)?;
        }
        IOCTL_HID_GET_INDEXED_STRING => {
            get_indexed_string(request, device_context)?;
        }
        IOCTL_HID_READ_REPORT => {
            read_report(request, device_context)?;
        }
        IOCTL_HID_WRITE_REPORT => {
            write_report(request, device_context)?;
        }
        IOCTL_UMDF_HID_GET_FEATURE => {
            get_feature(request, device_context)?;
        }
        IOCTL_UMDF_HID_SET_FEATURE => {
            set_feature(request, device_context)?;
        }
        IOCTL_UMDF_HID_GET_INPUT_REPORT => {
            get_input_report(request, device_context)?;
        }
        IOCTL_UMDF_HID_SET_OUTPUT_REPORT => {
            set_output_report(request, device_context)?;
        }
        _ => {
            println!("Unsupported control");
            return Err(STATUS_NOT_IMPLEMENTED);
        }
    }
    Ok(())
}

fn create_default_queue(device: WDFDEVICE) -> Result<WDFQUEUE, NTSTATUS> {
    let mut config = wdf_io_queue_config_init_default_queue(
        _WDF_IO_QUEUE_DISPATCH_TYPE::WdfIoQueueDispatchParallel,
    );
    let mut attributes = QueueContext::get_object_attributes();
    let mut queue: WDFQUEUE = WDF_NO_HANDLE.cast();

    config.EvtIoDeviceControl = Some(evt_io_device_control);

    let status;
    unsafe {
        // SAFETY: This is safe because:
        //         1. `driver` is provided by `DriverEntry` and is never null
        //         2. `registry_path` is provided by `DriverEntry` and is never null
        //         3. `driver_attributes` is allowed to be null
        //         4. `driver_config` is a valid pointer to a valid `WDF_DRIVER_CONFIG`
        //         5. `driver_handle_output` is expected to be null
        status = call_unsafe_wdf_function_binding!(
            WdfIoQueueCreate,
            device,
            &mut config,
            &mut attributes,
            &mut queue,
        );
        if NT_ERROR(status) {
            println!("Failed to create queue {status}");
            return Err(status);
        }
        assert!(!queue.is_null());
    }

    println!("Create queue context");
    let context = QueueContext {
        queue: queue,
        device: device,
    };
    wdf_init_context::<QueueContext>(queue.cast(), context);

    return Ok(queue);
}
