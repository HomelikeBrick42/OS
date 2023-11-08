#![no_std]
#![no_main]
#![feature(
    try_trait_v2,
    arbitrary_self_types,
    sync_unsafe_cell,
    panic_info_message
)]
#![allow(
    clippy::too_many_arguments,
    clippy::missing_safety_doc,
    clippy::not_unsafe_ptr_arg_deref
)]
#![deny(rust_2018_idioms, unsafe_op_in_unsafe_fn)]

pub mod efi;
pub mod framebuffer;
pub mod text_writer;

use crate::framebuffer::PixelFormat;
use core::{arch::asm, cell::SyncUnsafeCell, ffi::c_void, fmt::Write, panic::PanicInfo};
use framebuffer::Framebuffer;
use text_writer::TextWriter;
use utf16_lit::utf16_null;

#[no_mangle]
pub unsafe extern "system" fn efi_main(
    image_handle: efi::Handle,
    system_table: *mut efi::SystemTable,
) -> efi::Status {
    // Load gop for text rendering
    unsafe {
        let mut gop: *mut efi::Gop = core::ptr::null_mut();
        (*system_table).boottime.locate_protocol(
            &efi::Guid::GRAPHICS_OUTPUT_PROTOCOL,
            core::ptr::null_mut(),
            &mut gop as *mut *mut _ as *mut *mut c_void,
        )?;

        let mut info = core::ptr::null_mut();
        let mut size_of_info = 0;
        gop.query_mode(
            if !(*gop).mode.is_null() {
                (*(*gop).mode).mode
            } else {
                0
            },
            &mut size_of_info,
            &mut info,
        )?;
        gop.set_mode(if !(*gop).mode.is_null() {
            (*(*gop).mode).mode
        } else {
            0
        })?;

        let (width, height, pixels_per_scanline) = (
            (*info).width as usize,
            (*info).height as usize,
            (*info).pixels_per_scanline as usize,
        );
        let pixel_format = match (*info).pixel_format {
            efi::Gop::GOT_RGBA8 => PixelFormat::Rgb,
            efi::Gop::GOT_BGRA8 => PixelFormat::Bgr,
            _ => {
                (*system_table)
                    .console_out
                    .output_string(utf16_null!("Unsupported pixel format!\r\n").as_ptr())?;
                return efi::Status::UNSUPPORTED;
            }
        };

        // Sound, because nothing is trying to get a reference to the framebuffer before here
        FRAMEBUFFER.get().write(Framebuffer::new(
            (*(*gop).mode).fb_base,
            width,
            height,
            pixels_per_scanline,
            pixel_format,
        ));
    }

    // Exit boot services
    let (memory_map, memory_map_size, memory_descriptor_size) = unsafe {
        let mut memory_map_size = 0;
        let mut memory_map: *mut efi::MemoryDescriptor = core::ptr::null_mut();
        let mut map_key = 0;
        let mut descriptor_size = 0;
        let mut descriptor_version = 0;

        while {
            if memory_map_size > 0 {
                if !memory_map.is_null() {
                    (*system_table)
                        .boottime
                        .free_pool(memory_map as *mut c_void)?;
                }
                (*system_table).boottime.allocate_pool(
                    efi::MemoryType::LOADER_DATA,
                    memory_map_size,
                    &mut memory_map as *mut *mut _ as *mut *mut c_void,
                )?;
            }

            let status = (*system_table).boottime.get_memory_map(
                &mut memory_map_size,
                memory_map,
                &mut map_key,
                &mut descriptor_size,
                &mut descriptor_version,
            );

            if status == efi::Status::BUFFER_TOO_SMALL {
                true
            } else if status == efi::Status::SUCCESS {
                false
            } else {
                return status;
            }
        } {}

        (*system_table)
            .boottime
            .exit_boot_services(image_handle, map_key)?;

        (memory_map, memory_map_size, descriptor_size)
    };

    assert_eq!(memory_map_size % memory_descriptor_size, 0);

    let mut available_memory = 0;

    let mut offset = 0;
    while offset < memory_map_size {
        let memory_descriptor = unsafe {
            *memory_map
                .cast::<u8>()
                .add(offset)
                .cast::<efi::MemoryDescriptor>()
        };

        if let efi::MemoryType::CONVENTIONAL_MEMORY
        | efi::MemoryType::BOOT_SERVICES_CODE
        | efi::MemoryType::BOOT_SERVICES_DATA
        | efi::MemoryType::RUNTIME_SERVICES_CODE
        | efi::MemoryType::RUNTIME_SERVICES_DATA = memory_descriptor.type_
        {
            available_memory += memory_descriptor.number_of_pages * 4096;
        }

        offset += memory_descriptor_size;
    }

    main(available_memory)
}

fn main(available_memory: u64) -> ! {
    let framebuffer = get_screen_framebuffer();
    framebuffer.draw_rect(
        0,
        0,
        framebuffer.width(),
        framebuffer.height(),
        (51, 51, 51),
    );

    let font = psf2::Font::new(include_bytes!("./zap-light24.psf")).unwrap();
    let mut writer = TextWriter {
        framebuffer,
        font,
        cursor_x: 0,
        cursor_x_begin: 0,
        cursor_x_end: Some(framebuffer.width()),
        cursor_y: 0,
        foreground_color: (255, 255, 255),
        background_color: None,
    };

    writeln!(
        writer,
        "There is {}MiB of avaiable memory",
        available_memory / 1024 / 1024
    )
    .unwrap();

    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

static FRAMEBUFFER: SyncUnsafeCell<Framebuffer> = unsafe {
    SyncUnsafeCell::new(Framebuffer::new(
        core::ptr::null_mut(),
        0,
        0,
        0,
        PixelFormat::Rgb,
    ))
};

/// this can be called after GOP is initialized, which is basically anytime except at the start of efi_main
pub fn get_screen_framebuffer() -> Framebuffer {
    unsafe { *FRAMEBUFFER.get() }
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    if let Some(message) = info.message() {
        if let Ok(font) = psf2::Font::new(include_bytes!("./zap-light24.psf")) {
            let framebuffer = get_screen_framebuffer();
            let mut writer = TextWriter {
                framebuffer,
                font,
                cursor_x: 0,
                cursor_x_begin: 0,
                cursor_x_end: Some(framebuffer.width()),
                cursor_y: 0,
                foreground_color: (255, 0, 0),
                background_color: Some((0, 0, 0)),
            };
            if let Some(location) = info.location() {
                _ = write!(writer, "{}: ", location);
            }
            _ = write!(writer, "{}", message);
        }
    }

    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
