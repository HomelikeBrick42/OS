#![no_std]
#![no_main]
#![feature(
    try_trait_v2,
    arbitrary_self_types,
    panic_info_message,
    allocator_api,
    naked_functions
)]
#![allow(
    clippy::too_many_arguments,
    clippy::missing_safety_doc,
    clippy::not_unsafe_ptr_arg_deref
)]
#![deny(rust_2018_idioms, unsafe_op_in_unsafe_fn)]

extern crate alloc;

pub mod allocator;
pub mod efi;
pub mod framebuffer;
pub mod gdt;
pub mod idt;
pub mod interrupts;
pub mod text_writer;

use crate::framebuffer::PixelFormat;
use alloc::alloc::Global;
use allocator::{LinkedListAllocator, GLOBAL_LINKED_LIST_ALLOCATOR};
use core::{
    alloc::{Allocator, Layout},
    arch::asm,
    ffi::c_void,
    fmt::Write,
    panic::PanicInfo,
};
use framebuffer::Framebuffer;
use gdt::load_gdt;
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

        FRAMEBUFFER.call_once(|| {
            Framebuffer::new(
                (*(*gop).mode).fb_base,
                width,
                height,
                pixels_per_scanline,
                pixel_format,
            )
        });
    }

    // Get memory map and exit boot services
    let (memory_map, memory_descriptor_size, memory_descriptor_count) = unsafe {
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

        assert_eq!(memory_map_size % descriptor_size, 0);
        (
            memory_map,
            descriptor_size,
            memory_map_size / descriptor_size,
        )
    };

    let framebuffer = get_screen_framebuffer();
    framebuffer.draw_rect(
        0,
        0,
        framebuffer.width(),
        framebuffer.height(),
        (51, 51, 51),
    );

    // Create allocator
    GLOBAL_LINKED_LIST_ALLOCATOR.call_once(|| unsafe {
        LinkedListAllocator::from_efi_memory_map(
            memory_map,
            memory_descriptor_size,
            memory_descriptor_count,
        )
    });

    let font = psf2::Font::new(include_bytes!("./zap-light24.psf") as &[u8]).unwrap();
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

    if false {
        {
            GLOBAL_LINKED_LIST_ALLOCATOR
                .get()
                .unwrap()
                .print_allocation_headers(&mut writer)
                .unwrap();

            let a = alloc::boxed::Box::new(5);
            writeln!(writer, "Allocated value: {a}").unwrap();

            GLOBAL_LINKED_LIST_ALLOCATOR
                .get()
                .unwrap()
                .print_allocation_headers(&mut writer)
                .unwrap();
        }

        GLOBAL_LINKED_LIST_ALLOCATOR
            .get()
            .unwrap()
            .print_allocation_headers(&mut writer)
            .unwrap();
    }

    #[allow(clippy::fn_to_numeric_cast)]
    unsafe {
        static IDTR: spin::Once<idt::Descriptor> = spin::Once::new();
        let idtr = IDTR.call_once(|| idt::Descriptor {
            limit: 0x0FFF,
            offset: Global
                .allocate_zeroed(Layout::from_size_align(4096, 4096).unwrap())
                .unwrap()
                .as_ptr() as *mut u8 as u64,
        });

        (idtr.offset as *mut idt::DescriptorEntry).add(0xE).write({
            let mut descriptor = idt::DescriptorEntry {
                offset0: 0,
                selector: 0x08,
                ist: 0x00,
                type_attr: idt::TA_INTERRUPT_GATE,
                offset1: 0,
                offset2: 0,
                ignore: 0,
            };
            descriptor.set_offset(interrupts::page_fault_thunk as unsafe extern "C" fn() as u64);
            descriptor
        });

        asm!("lidt [rax]", in("rax") idtr as *const idt::Descriptor)
    }

    unsafe {
        let descriptor = gdt::Descriptor {
            size: (core::mem::size_of::<gdt::GDT>() - 1) as u16,
            offset: &gdt::DEFAULT_GDT as *const gdt::GDT as u64,
        };
        asm!(
            "call {load_gdt}",
            load_gdt = sym load_gdt,
            inout("ax") &descriptor as *const gdt::Descriptor => _,
        );
    }

    main(writer)
}

fn main(mut writer: TextWriter<&'static [u8]>) -> ! {
    writeln!(writer, "OS started successfully").unwrap();

    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

static FRAMEBUFFER: spin::Once<Framebuffer> = spin::Once::new();

pub fn get_screen_framebuffer() -> Framebuffer {
    FRAMEBUFFER
        .get()
        .copied()
        .expect("this can only be called once the kernel is initialized")
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
