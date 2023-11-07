use crate::{efi::Status, wrap_self_function_pointer};

#[repr(C)]
pub struct Gop {
    query_mode: Option<
        unsafe extern "efiapi" fn(*mut Gop, u32, *mut usize, *mut *mut GopModeInfo) -> Status,
    >,
    set_mode: Option<unsafe extern "efiapi" fn(*mut Gop, u32) -> Status>,
    blt: Option<
        unsafe extern "efiapi" fn(
            *mut Gop,
            *mut Pixel,
            u32,
            usize,
            usize,
            usize,
            usize,
            usize,
            usize,
            usize,
        ) -> Status,
    >,
    pub mode: *mut Mode,
}

impl Gop {
    pub const GOT_RGBA8: u32 = 0;
    pub const GOT_BGRA8: u32 = 1;

    pub const BLT_VIDEO_FILL: u32 = 0;
    pub const BLT_VIDEO_TO_BLT_BUFFER: u32 = 1;
    pub const BLT_BUFFER_TO_VIDEO: u32 = 2;
    pub const BLT_VIDEO_TO_VIDEO: u32 = 3;

    wrap_self_function_pointer!(query_mode(mode_number: u32, size_of_info: *mut usize, info: *mut *mut GopModeInfo) -> Status);
    wrap_self_function_pointer!(set_mode(mode_number: u32) -> Status);
    wrap_self_function_pointer!(
        blt(
            buffer: *mut Pixel,
            operation: u32,
            sx: usize,
            sy: usize,
            dx: usize,
            dy: usize,
            width: usize,
            height: usize,
            delta: usize,
        ) -> Status
    );
}

#[repr(C)]
pub struct GopModeInfo {
    pub version: u32,
    pub width: u32,
    pub height: u32,
    pub pixel_format: u32,
    pub pixel_bitmask: [u32; 4],
    pub pixels_per_scanline: u32,
}

#[repr(C)]
pub struct Mode {
    pub max_mode: u32,
    pub mode: u32,
    pub info: *mut GopModeInfo,
    pub info_size: u32,
    pub fb_base: *mut (),
    pub fb_size: u32,
}

#[repr(C)]
pub struct Pixel {
    pub blue: u8,
    pub green: u8,
    pub red: u8,
    pub reserved: u8,
}
