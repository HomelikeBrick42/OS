use crate::{halt, println};
use core::arch::asm;

pub unsafe fn enable_interrupts() {
    unsafe {
        asm!("sti");
    }
}

pub unsafe fn disable_interrupts() {
    unsafe {
        asm!("cli");
    }
}

macro_rules! interrupt_handler {
    ($name:ident, $to_call:path $(,)?) => {
        const _: unsafe extern "win64" fn() = $to_call; // make sure the signature is correct

        #[naked]
        pub unsafe extern "C" fn $name() {
            use ::core::arch::asm;
            unsafe {
                asm!(
                    "
sub rsp, 128
push rax
push rcx
push rdx
push r8
push r9
push r10
push r11
sub rsp, 8
call {to_call}
add rsp, 8
pop r11
pop r10
pop r9
pop r8
pop rdx
pop rcx
pop rax
add rsp, 128
iretq
",
                    to_call = sym $to_call,
                    options(noreturn),
                );
            }
        }
    };
}

pub(crate) use interrupt_handler;

interrupt_handler!(page_fault_handler, page_fault);
unsafe extern "win64" fn page_fault() {
    println!("page fault detected");
    halt()
}

interrupt_handler!(double_fault_handler, double_fault);
unsafe extern "win64" fn double_fault() {
    println!("double fault detected");
    halt()
}

interrupt_handler!(general_protection_fault_handler, general_protection);
unsafe extern "win64" fn general_protection() {
    println!("general protection fault detected");
    halt()
}
