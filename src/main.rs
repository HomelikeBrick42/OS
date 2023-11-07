#![no_std]
#![no_main]
#![feature(try_trait_v2, arbitrary_self_types)]
#![allow(
    clippy::too_many_arguments,
    clippy::missing_safety_doc,
    clippy::not_unsafe_ptr_arg_deref
)]
#![deny(rust_2018_idioms, unsafe_op_in_unsafe_fn)]

pub mod efi;

use core::{arch::asm, panic::PanicInfo};
use utf16_lit::utf16;

#[no_mangle]
pub unsafe extern "system" fn efi_main(
    _image_handle: efi::Handle,
    system_table: *mut efi::SystemTable,
) -> efi::Status {
    unsafe {
        (*system_table)
            .console_out
            .output_string(utf16!("Hello, World!\n\0").as_ptr())?;

        loop {
            asm!("hlt");
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
