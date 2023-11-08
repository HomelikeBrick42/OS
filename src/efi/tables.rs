use bitflags::bitflags;
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

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MemoryType(pub i32);

impl MemoryType {
    pub const RESERVED_MEMORY_TYPE: MemoryType = MemoryType(0);
    pub const LOADER_CODE: MemoryType = MemoryType(1);
    pub const LOADER_DATA: MemoryType = MemoryType(2);
    pub const BOOT_SERVICES_CODE: MemoryType = MemoryType(3);
    pub const BOOT_SERVICES_DATA: MemoryType = MemoryType(4);
    pub const RUNTIME_SERVICES_CODE: MemoryType = MemoryType(5);
    pub const RUNTIME_SERVICES_DATA: MemoryType = MemoryType(6);
    pub const CONVENTIONAL_MEMORY: MemoryType = MemoryType(7);
    pub const UNUSABLE_MEMORY: MemoryType = MemoryType(8);
    pub const ACPI_RECLAIM_MEMORY: MemoryType = MemoryType(9);
    pub const ACPI_MEMORY_NVS: MemoryType = MemoryType(10);
    pub const MEMORY_MAPPED_IO: MemoryType = MemoryType(11);
    pub const MEMORY_MAPPED_IOPORT_SPACE: MemoryType = MemoryType(12);
    pub const PAL_CODE: MemoryType = MemoryType(13);
    pub const PERSISTENT_MEMORY: MemoryType = MemoryType(14);
    pub const UNACCEPTED_MEMORY_TYPE: MemoryType = MemoryType(15);
    pub const MAX_MEMORY_TYPE: MemoryType = MemoryType(16);
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PhysicalAddress(pub u64);

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct VirtualAddress(pub u64);

bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy)]
    pub struct Attribute: u64 {
        const UC = 0x0000000000000001;
        const WC = 0x0000000000000002;
        const WT = 0x0000000000000004;
        const WB = 0x0000000000000008;
        const UCE = 0x0000000000000010;
        const WP = 0x0000000000001000;
        const RP = 0x0000000000002000;
        const XP = 0x0000000000004000;
        const NV = 0x0000000000008000;
        const MORE_RELIABLE = 0x0000000000010000;
        const RO = 0x0000000000020000;
        const SP = 0x0000000000040000;
        const CPU_CRYPTO = 0x0000000000080000;
        const RUNTIME = 0x8000000000000000;
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MemoryDescriptor {
    pub type_: MemoryType,
    pub physical_start: PhysicalAddress,
    pub virtual_start: VirtualAddress,
    pub number_of_pages: u64,
    pub attribute: Attribute,
}

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
