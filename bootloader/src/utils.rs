use core::arch::asm;

pub fn hlt() {
    unsafe { asm!("hlt", options(nomem, nostack)) };
}

pub unsafe fn inb<const PORT: u16>() -> u8 {
    let value;
    unsafe {
        asm!(
            "in al, {port}",
            port = const PORT,
            out("al") value,
            options(nomem, nostack)
        );
    }
    value
}

pub unsafe fn outb<const PORT: u16>(value: u8) {
    unsafe {
        asm!(
            "out {port}, al",
            port = const PORT,
            in("al") value,
            options(nomem, nostack)
        );
    }
}

pub fn io_wait() {
    unsafe { outb::<0x80>(0) };
}

pub fn get_flags() -> u64 {
    let flags;
    unsafe {
        asm!(
            "pushf",
            "pop rax",
            out("rax") flags,
            options(nomem)
        );
    }
    flags
}
