use core::ffi::c_void;

use crate::{
    efi::{Guid, Handle, Status},
    wrap_self_function_pointer,
};

#[repr(C)]
pub struct TableHeader {
    pub signature: u64,
    pub revision: u32,
    pub headersize: u32,
    pub crc32: u32,
    pub reserved: u32,
}

#[repr(C)]
pub struct SystemTable {
    pub header: TableHeader,
    pub fw_vendor: *mut u16,
    pub fw_revision: u32,
    pub console_in_handle: Handle,
    pub console_in: *mut (),
    pub console_out_handle: Handle,
    pub console_out: *mut SimpleTextOutputProtocol,
    pub stderr_handle: Handle,
    pub std_err: *mut (),
    pub runtime: *mut (),
    pub boottime: *const BootServices,
    pub nr_tables: usize,
    pub tables: *mut (),
}

#[repr(C)]
pub struct SimpleTextOutputProtocol {
    reset: Option<unsafe extern "efiapi" fn(*mut SimpleTextOutputProtocol, u8) -> Status>,
    output_string:
        Option<unsafe extern "efiapi" fn(*mut SimpleTextOutputProtocol, *const u16) -> Status>,
    test_string:
        Option<unsafe extern "efiapi" fn(*mut SimpleTextOutputProtocol, *const u16) -> Status>,
    query_mode: Option<
        unsafe extern "efiapi" fn(*mut SimpleTextOutputProtocol, u32, *mut u32, *mut u32) -> Status,
    >,
    set_mode: Option<unsafe extern "efiapi" fn(*mut SimpleTextOutputProtocol, u32) -> Status>,
    set_attribute: Option<unsafe extern "efiapi" fn(*mut SimpleTextOutputProtocol, u32) -> Status>,
    clear_screen: Option<unsafe extern "efiapi" fn(*mut SimpleTextOutputProtocol) -> Status>,
    set_cursor_position:
        Option<unsafe extern "efiapi" fn(*mut SimpleTextOutputProtocol, u32, u32) -> Status>,
    enable_cursor: Option<unsafe extern "efiapi" fn(*mut SimpleTextOutputProtocol, bool) -> Status>,
    mode: *mut (),
}

impl SimpleTextOutputProtocol {
    wrap_self_function_pointer!(reset(extended_verification: u8) -> Status);
    wrap_self_function_pointer!(output_string(string: *const u16) -> Status);
    wrap_self_function_pointer!(test_string(string: *const u16) -> Status);
    wrap_self_function_pointer!(query_mode(mode_number: u32, columns: *mut u32, rows: *mut u32) -> Status);
    wrap_self_function_pointer!(set_mode(mode_number: u32) -> Status);
    wrap_self_function_pointer!(set_attribute(attribute: u32) -> Status);
    wrap_self_function_pointer!(clear_screen() -> Status);
    wrap_self_function_pointer!(set_cursor_position(column: u32, row: u32) -> Status);
    wrap_self_function_pointer!(enable_cursor(enable: bool) -> Status);
}

#[repr(C)]
pub enum MemoryType {
    ReservedMemoryType,
    LoaderCode,
    LoaderData,
    BootServicesCode,
    BootServicesData,
    RuntimeServicesCode,
    RuntimeServicesData,
    ConventionalMemory,
    UnusableMemory,
    ACPIReclaimMemory,
    ACPIMemoryNVS,
    MemoryMappedIO,
    MemoryMappedIOPortSpace,
    PalCode,
    PersistentMemory,
    UnacceptedMemoryType,
    MaxMemoryType,
}

#[repr(C)]
pub struct MemoryDescriptor {}

#[repr(C)]
pub struct BootServices {
    pub header: TableHeader,
    raise_tpl: Option<unsafe extern "efiapi" fn()>,
    restore_tpl: Option<unsafe extern "efiapi" fn()>,
    allocate_pages: Option<unsafe extern "efiapi" fn()>,
    free_pages: Option<unsafe extern "efiapi" fn()>,
    get_memory_map: Option<
        unsafe extern "efiapi" fn(
            memory_map_size: *mut usize,
            memory_map: *mut MemoryDescriptor,
            map_key: *mut usize,
            descriptor_size: *mut usize,
            descriptor_version: *mut u32,
        ) -> Status,
    >,
    allocate_pool: Option<
        unsafe extern "efiapi" fn(
            pool_type: MemoryType,
            size: usize,
            buffer: *mut *mut c_void,
        ) -> Status,
    >,
    free_pool: Option<unsafe extern "efiapi" fn(buffer: *mut c_void) -> Status>,
    create_event: Option<unsafe extern "efiapi" fn()>,
    set_timer: Option<unsafe extern "efiapi" fn()>,
    wait_for_event: Option<unsafe extern "efiapi" fn()>,
    signal_event: Option<unsafe extern "efiapi" fn()>,
    close_event: Option<unsafe extern "efiapi" fn()>,
    check_event: Option<unsafe extern "efiapi" fn()>,
    install_protocol_interface: Option<unsafe extern "efiapi" fn()>,
    reinstall_protocol_interface: Option<unsafe extern "efiapi" fn()>,
    uninstall_protocol_interface: Option<unsafe extern "efiapi" fn()>,
    handle_protocol: Option<unsafe extern "efiapi" fn()>,
    reserved: *mut (),
    register_protocol_notify: Option<unsafe extern "efiapi" fn()>,
    locate_handle: Option<unsafe extern "efiapi" fn()>,
    locate_device_path: Option<unsafe extern "efiapi" fn()>,
    install_configuration_table: Option<unsafe extern "efiapi" fn()>,
    load_image: Option<unsafe extern "efiapi" fn()>,
    start_image: Option<unsafe extern "efiapi" fn()>,
    exit: Option<unsafe extern "efiapi" fn()>,
    unload_image: Option<unsafe extern "efiapi" fn()>,
    exit_boot_services:
        Option<unsafe extern "efiapi" fn(image_handle: Handle, map_key: usize) -> Status>,
    get_next_monotonic_count: Option<unsafe extern "efiapi" fn()>,
    stall: Option<unsafe extern "efiapi" fn()>,
    set_watchdog_timer: Option<unsafe extern "efiapi" fn()>,
    connect_controller: Option<unsafe extern "efiapi" fn()>,
    disconnect_controller: Option<unsafe extern "efiapi" fn()>,
    open_protocol: Option<unsafe extern "efiapi" fn()>,
    close_protocol: Option<unsafe extern "efiapi" fn()>,
    open_protocol_information: Option<unsafe extern "efiapi" fn()>,
    protocols_per_handle: Option<unsafe extern "efiapi" fn()>,
    locate_handle_buffer: Option<unsafe extern "efiapi" fn()>,
    locate_protocol:
        Option<unsafe extern "efiapi" fn(*const Guid, *mut c_void, *mut *mut c_void) -> Status>,
    install_multiple_protocol_interfaces: Option<unsafe extern "efiapi" fn()>,
    uninstall_multiple_protocol_interfaces: Option<unsafe extern "efiapi" fn()>,
    calculate_crc32: Option<unsafe extern "efiapi" fn()>,
    copy_mem: Option<unsafe extern "efiapi" fn()>,
    set_mem: Option<unsafe extern "efiapi" fn()>,
    create_event_ex: Option<unsafe extern "efiapi" fn()>,
}

#[macro_export]
macro_rules! wrap_function_pointer {
    ($name:ident($($arg:ident: $typ:ty),* $(,)?) $(-> $ret_typ:ty)?) => {
        #[inline]
        pub unsafe fn $name(self: *const Self, $($arg: $typ),*) $(-> $ret_typ)? {
            unsafe { (*self).$name.unwrap_unchecked()($($arg),*) }
        }
    };
}

impl BootServices {
    wrap_function_pointer!(get_memory_map(
        memory_map_size: *mut usize,
        memory_map: *mut MemoryDescriptor,
        map_key: *mut usize,
        descriptor_size: *mut usize,
        descriptor_version: *mut u32,
    ) -> Status);
    wrap_function_pointer!(exit_boot_services(image_handle: Handle, map_key: usize) -> Status);
    wrap_function_pointer!(allocate_pool(pool_type: MemoryType, size: usize, buffer: *mut *mut c_void) -> Status);
    wrap_function_pointer!(free_pool(buffer: *mut c_void) -> Status);
    wrap_function_pointer!(locate_protocol(protocol: *const Guid, registration: *mut c_void, protocol_interface: *mut *mut c_void) -> Status);
}
