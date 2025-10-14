use core::arch::asm;

pub fn hlt() -> ! {
    loop {
        unsafe { asm!("hlt") };
    }
}

pub unsafe fn disable_interrupts() {
    unsafe { asm!("cli") };
}

pub unsafe fn enable_interrupts() {
    unsafe { asm!("sti") };
}
