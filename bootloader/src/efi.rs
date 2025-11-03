use core::{fmt::Debug, num::NonZeroIsize, ptr::NonNull};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Error(pub NonZeroIsize);

impl Error {
    pub const UNSUPPORTED: Self = Self(NonZeroIsize::new(isize::MIN | 3).unwrap());
    pub const BUFFER_TOO_SMALL: Self = Self(NonZeroIsize::new(isize::MIN | 5).unwrap());
}

pub type Status = Result<(), Error>;

const _: () = assert!(size_of::<Status>() == size_of::<Error>());

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Handle(*const ());

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct SystemTable(*const SystemTableData);

impl SystemTable {
    pub unsafe fn con_out_print(self, string: *const u16) -> Status {
        unsafe {
            let con_out = (*self.0).con_out;
            ((*con_out).output_string)(con_out, string)
        }
    }

    pub unsafe fn locate_protocol(self, guid: Guid) -> Result<*mut (), Error> {
        let mut ptr = core::ptr::null_mut();
        unsafe {
            ((*(*self.0).boot_services).locate_protocol)(
                &raw const guid,
                core::ptr::null(),
                &raw mut ptr,
            )?;
        }
        Ok(ptr)
    }

    pub unsafe fn locate_gop(self) -> Result<GOP, Error> {
        unsafe {
            let protocol = self.locate_protocol(Guid::GRAPHICS_OUTPUT_PROTOCOL)?;
            Ok(GOP(protocol.cast()))
        }
    }

    pub unsafe fn allocate_pages(
        self,
        allocate_type: AllocateType,
        memory_type: MemoryType,
        pages: usize,
    ) -> Result<*mut (), Error> {
        let mut ptr = core::ptr::null_mut();
        unsafe {
            ((*(*self.0).boot_services).allocate_pages)(
                allocate_type,
                memory_type,
                pages,
                &mut ptr,
            )?;
        }
        Ok(ptr)
    }

    pub unsafe fn free_pages(self, memory: *mut (), pages: usize) -> Status {
        unsafe { ((*(*self.0).boot_services).free_pages)(memory, pages) }
    }

    pub unsafe fn get_memory_map(
        self,
        memory_map_size: &mut usize,
        memory_map: *mut MemoryDescriptor,
        map_key: &mut usize,
        descriptor_size: &mut usize,
        descriptor_version: &mut u32,
    ) -> Status {
        unsafe {
            ((*(*self.0).boot_services).get_memory_map)(
                memory_map_size,
                memory_map,
                map_key,
                descriptor_size,
                descriptor_version,
            )
        }
    }

    pub unsafe fn exit_boot_services(self, image_handle: Handle, map_key: usize) -> Status {
        unsafe { ((*(*self.0).boot_services).exit_boot_services)(image_handle, map_key) }
    }
}

#[repr(C)]
struct TableHeader {
    signature: u64,
    revision: u32,
    header_size: u32,
    crc32: u32,
    reserved: u32,
}

#[repr(C)]
struct SystemTableData {
    hdr: TableHeader,
    firmware_vendor: *const u16,
    firmware_revision: u32,
    console_in_handle: Handle,
    con_in: *const SimpleTextInputProtocol,
    console_out_handle: Handle,
    con_out: *const SimpleTextOutputProtocol,
    standard_error_handle: Handle,
    std_err: *const SimpleTextOutputProtocol,
    runtime_services: *const RuntimeServices,
    boot_services: *const BootServices,
    number_of_table_entries: usize,
    configuration_table: *const ConfigurationTable,
}

#[repr(C)]
struct SimpleTextInputProtocol {}

#[repr(C)]
struct SimpleTextOutputProtocol {
    reset: unsafe extern "efiapi" fn(),
    output_string: unsafe extern "efiapi" fn(
        this: *const SimpleTextOutputProtocol,
        string: *const u16,
    ) -> Status,
    test_string: unsafe extern "efiapi" fn(),
    query_mode: unsafe extern "efiapi" fn(),
    set_mode: unsafe extern "efiapi" fn(),
    set_attribute: unsafe extern "efiapi" fn(),
    clear_screen: unsafe extern "efiapi" fn(),
    set_cursor_position: unsafe extern "efiapi" fn(),
    enable_cursor: unsafe extern "efiapi" fn(),
    mode: *const SimpleTextOutputMode,
}

#[repr(C)]
struct SimpleTextOutputMode {
    max_mode: i32,
    mode: i32,
    attribute: i32,
    cursor_column: i32,
    cursor_row: i32,
    cursor_visible: bool,
}

#[repr(C)]
struct RuntimeServices {
    hdr: TableHeader,
    get_time: unsafe extern "efiapi" fn(),
    set_time: unsafe extern "efiapi" fn(),
    get_wakeup_time: unsafe extern "efiapi" fn(),
    set_wakeup_time: unsafe extern "efiapi" fn(),
    set_virtual_address_map: unsafe extern "efiapi" fn(),
    convert_pointer: unsafe extern "efiapi" fn(),
    get_variable: unsafe extern "efiapi" fn(),
    get_next_variable_name: unsafe extern "efiapi" fn(),
    set_variable: unsafe extern "efiapi" fn(),
    get_next_high_monotonic_count: unsafe extern "efiapi" fn(),
    reset_system: unsafe extern "efiapi" fn(),
    update_capsule: unsafe extern "efiapi" fn(),
    query_capsule_capabilities: unsafe extern "efiapi" fn(),
    query_variable_info: unsafe extern "efiapi" fn(),
}

#[repr(C)]
struct BootServices {
    hdr: TableHeader,
    raise_tpl: unsafe extern "efiapi" fn(),
    restore_tpl: unsafe extern "efiapi" fn(),
    allocate_pages: unsafe extern "efiapi" fn(
        allocate_type: AllocateType,
        memory_type: MemoryType,
        pages: usize,
        memory: *mut *mut (),
    ) -> Status,
    free_pages: unsafe extern "efiapi" fn(memory: *mut (), pages: usize) -> Status,
    get_memory_map: unsafe extern "efiapi" fn(
        memory_map_size: *mut usize,
        memory_map: *mut MemoryDescriptor,
        map_key: *mut usize,
        descriptor_size: *mut usize,
        descriptor_version: *mut u32,
    ) -> Status,
    allocate_pool: unsafe extern "efiapi" fn(),
    free_pool: unsafe extern "efiapi" fn(),
    create_event: unsafe extern "efiapi" fn(),
    set_timer: unsafe extern "efiapi" fn(),
    wait_for_event: unsafe extern "efiapi" fn(),
    signal_event: unsafe extern "efiapi" fn(),
    close_event: unsafe extern "efiapi" fn(),
    check_event: unsafe extern "efiapi" fn(),
    install_protocol_interface: unsafe extern "efiapi" fn(),
    reinstall_protocol_interface: unsafe extern "efiapi" fn(),
    uninstall_protocol_interface: unsafe extern "efiapi" fn(),
    handle_protocol: unsafe extern "efiapi" fn(),
    reserved: *const (),
    register_protocol_notify: unsafe extern "efiapi" fn(),
    locate_handle: unsafe extern "efiapi" fn(),
    locate_device_path: unsafe extern "efiapi" fn(),
    install_configuration_table: unsafe extern "efiapi" fn(),
    load_image: unsafe extern "efiapi" fn(),
    start_image: unsafe extern "efiapi" fn(),
    exit: unsafe extern "efiapi" fn(),
    unload_image: unsafe extern "efiapi" fn(),
    exit_boot_services: unsafe extern "efiapi" fn(image_handle: Handle, map_key: usize) -> Status,
    get_next_monotonic_count: unsafe extern "efiapi" fn(),
    stall: unsafe extern "efiapi" fn(),
    set_watchdog_timer: unsafe extern "efiapi" fn(),
    connect_controller: unsafe extern "efiapi" fn(),
    disconnect_controller: unsafe extern "efiapi" fn(),
    open_protocol: unsafe extern "efiapi" fn(),
    close_protocol: unsafe extern "efiapi" fn(),
    open_protocol_information: unsafe extern "efiapi" fn(),
    protocols_per_handle: unsafe extern "efiapi" fn(),
    locate_handle_buffer: unsafe extern "efiapi" fn(),
    locate_protocol: unsafe extern "efiapi" fn(
        protocol: *const Guid,
        registration: *const (),
        interface: *mut *mut (),
    ) -> Status,
    install_multiple_protocol_interfaces: unsafe extern "efiapi" fn(),
    uninstall_multiple_protocol_interfaces: unsafe extern "efiapi" fn(),
    calculate_crc32: unsafe extern "efiapi" fn(),
    copy_mem: unsafe extern "efiapi" fn(),
    set_mem: unsafe extern "efiapi" fn(),
    create_event_ex: unsafe extern "efiapi" fn(),
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub enum AllocateType {
    AnyPages,
    MaxAddress,
    Address,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct MemoryType(u32);

#[allow(non_upper_case_globals)]
impl MemoryType {
    pub const ReservedMemoryType: Self = Self(0);
    pub const LoaderCode: Self = Self(1);
    pub const LoaderData: Self = Self(2);
    pub const BootServicesCode: Self = Self(3);
    pub const BootServicesData: Self = Self(4);
    pub const RuntimeServicesCode: Self = Self(5);
    pub const RuntimeServicesData: Self = Self(6);
    pub const ConventionalMemory: Self = Self(7);
    pub const UnusableMemory: Self = Self(8);
    pub const ACPIReclaimMemory: Self = Self(9);
    pub const ACPIMemoryNVS: Self = Self(10);
    pub const MemoryMappedIO: Self = Self(11);
    pub const MemoryMappedIOPortSpace: Self = Self(12);
    pub const PalCode: Self = Self(13);
    pub const PersistentMemory: Self = Self(14);
}

impl Debug for MemoryType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            Self::ReservedMemoryType => write!(f, "EfiReservedMemoryType"),
            Self::LoaderCode => write!(f, "EfiLoaderCode"),
            Self::LoaderData => write!(f, "EfiLoaderData"),
            Self::BootServicesCode => write!(f, "EfiBootServicesCode"),
            Self::BootServicesData => write!(f, "EfiBootServicesData"),
            Self::RuntimeServicesCode => write!(f, "EfiRuntimeServicesCode"),
            Self::RuntimeServicesData => write!(f, "EfiRuntimeServicesData"),
            Self::ConventionalMemory => write!(f, "EfiConventionalMemory"),
            Self::UnusableMemory => write!(f, "EfiUnusableMemory"),
            Self::ACPIReclaimMemory => write!(f, "EfiACPIReclaimMemory"),
            Self::ACPIMemoryNVS => write!(f, "EfiACPIMemoryNVS"),
            Self::MemoryMappedIO => write!(f, "EfiMemoryMappedIO"),
            Self::MemoryMappedIOPortSpace => write!(f, "EfiMemoryMappedIOPortSpace"),
            Self::PalCode => write!(f, "EfiPalCode"),
            Self::PersistentMemory => write!(f, "EfiPersistentMemory"),
            Self(x) => write!(f, "EfiUnknownMemoryType({x:#X})"),
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct MemoryDescriptor {
    pub memory_type: MemoryType,
    pub physical_start: usize,
    pub virtual_start: usize,
    pub number_of_pages: usize,
    pub attribute: u64,
}

#[repr(C)]
struct ConfigurationTable {
    vendor_guid: Guid,
    vendor_table: *const (),
}

#[derive(Debug, Clone, Copy)]
#[repr(C, align(8))]
pub struct Guid {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}

impl Guid {
    pub const GRAPHICS_OUTPUT_PROTOCOL: Self = Self {
        data1: 0x9042a9de,
        data2: 0x23dc,
        data3: 0x4a38,
        data4: [0x96, 0xfb, 0x7a, 0xde, 0xd0, 0x80, 0x51, 0x6a],
    };
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct GOP(*const GOPData);

impl GOP {
    pub fn query_mode(self, mode_number: u32) -> Result<(usize, GOPModeInformation), Error> {
        let mut size = 0;
        let mut info = core::ptr::null();
        unsafe {
            ((*self.0).query_mode)(self.0, mode_number, &raw mut size, &raw mut info)?;
        }
        Ok((size, unsafe { *info }))
    }

    pub fn set_mode(self, mode_number: u32) -> Status {
        unsafe { ((*self.0).set_mode)(self.0, mode_number) }
    }

    pub fn mode(self) -> Option<NonNull<GOPMode>> {
        unsafe { NonNull::new((*self.0).mode.cast_mut()) }
    }
}

#[repr(C)]
struct GOPData {
    query_mode: unsafe extern "efiapi" fn(
        this: *const GOPData,
        mode_number: u32,
        size_of_info: *mut usize,
        info: *mut *const GOPModeInformation,
    ) -> Status,
    set_mode: unsafe extern "efiapi" fn(this: *const GOPData, mode_number: u32) -> Status,
    blt: unsafe extern "efiapi" fn(),
    mode: *const GOPMode,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct GOPMode {
    pub max_mode: u32,
    pub mode: u32,
    pub info: *const GOPModeInformation,
    pub size_of_info: usize,
    pub frame_buffer_base: *mut (),
    pub frame_buffer_size: usize,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct GOPModeInformation {
    pub version: u32,
    pub horizontal_resolution: u32,
    pub vertical_resolution: u32,
    pub pixel_format: GraphicsPixelFormat,
    pub pixel_information: PixelBitmask,
    pub pixels_per_scan_line: u32,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub enum GraphicsPixelFormat {
    RedGreenBlueReserved8BitPerColor,
    BlueGreenRedReserved8BitPerColor,
    BitMask,
    BltOnly,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct PixelBitmask {
    pub red_mask: u32,
    pub green_mask: u32,
    pub blue_mask: u32,
    pub reserved_mask: u32,
}
