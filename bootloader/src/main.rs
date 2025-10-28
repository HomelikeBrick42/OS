#![no_std]
#![no_main]
#![feature(
    sync_unsafe_cell,
    format_args_nl,
    abi_x86_interrupt,
    const_precise_live_drops
)]

use crate::{
    framebuffer::{framebuffer, init_framebuffer, Color, FramebufferColor},
    idt::disable_interrupts,
    kernel::kernel_main,
    page_allocator::{init_page_allocator, PAGE_ALLOCATOR},
    utils::{error_screen, hlt},
};
use core::{arch::asm, fmt::Write};
use core::{num::NonZeroUsize, panic::PanicInfo};

pub mod drivers;
pub mod efi;
pub mod framebuffer;
pub mod gdt;
pub mod idt;
pub mod interrupt_safe_mutex;
pub mod kernel;
pub mod page_allocator;
pub mod print;
pub mod rust_global_allocators;
pub mod text_writer;
pub mod utils;
pub mod screen;

extern crate alloc;

#[unsafe(no_mangle)]
unsafe extern "efiapi" fn efi_main(
    image_handle: efi::Handle,
    system_table: efi::SystemTable,
) -> efi::Status {
    unsafe { init_framebuffer(system_table)? };

    let framebuffer = framebuffer();
    let width = framebuffer.width();
    let height = framebuffer.height();

    let background = Color {
        r: 20,
        g: 20,
        b: 20,
    };
    framebuffer.fill(0, 0, width, height, FramebufferColor::new(background));

    // get memory map
    let mut memory_map_size = 0;
    let mut memory_map = core::ptr::null_mut();
    let mut map_key = 0;
    let mut memory_descriptor_size = 0;
    let mut memory_descritor_version = 0;
    loop {
        let old_size = memory_map_size;
        match unsafe {
            system_table.get_memory_map(
                &mut memory_map_size,
                memory_map,
                &mut map_key,
                &mut memory_descriptor_size,
                &mut memory_descritor_version,
            )
        } {
            Err(efi::Error::BUFFER_TOO_SMALL) => {
                if !memory_map.is_null() {
                    unsafe { system_table.free_pages(memory_map.cast(), old_size.div_ceil(4096))? };
                }
                memory_map = unsafe {
                    system_table
                        .allocate_pages(
                            efi::AllocateType::AnyPages,
                            efi::MemoryType::LoaderData,
                            memory_map_size.div_ceil(4096),
                        )?
                        .cast()
                };
                continue;
            }
            result => break result?,
        }
    }
    assert!(memory_map_size.is_multiple_of(memory_descriptor_size));

    // exit boot services
    unsafe { system_table.exit_boot_services(image_handle, map_key)? };

    unsafe { disable_interrupts() };

    unsafe { init_page_allocator(memory_map, memory_map_size, memory_descriptor_size) };

    unsafe {
        let stack_size = 4 * 1024 * 1024;
        let stack = PAGE_ALLOCATOR.with(|alloc| {
            alloc
                .allocate(const { NonZeroUsize::new(16).unwrap() }, stack_size)
                .expect("allocating the stack should succeed")
        });

        let stack_start = stack + stack_size;
        let _: unsafe extern "win64" fn() -> ! = kernel_main;
        asm!(
            "mov rsp, {stack_start}",
            "call {kernel_main}",
            stack_start = in(reg) stack_start,
            kernel_main = sym kernel_main,
            options(noreturn)
        )
    }
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    error_screen(|text_writer| {
        if let Some(location) = info.location() {
            _ = write!(text_writer, "{}: ", location);
        }
        _ = writeln!(text_writer, "{}", info.message());
    });

    loop {
        hlt();
    }
}
