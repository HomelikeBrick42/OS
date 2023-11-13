use core::arch::asm;

pub unsafe fn output_byte(port: u16, value: u8) {
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") value);
    }
}

pub unsafe fn output_word(port: u16, value: u16) {
    unsafe {
        asm!("out dx, ax", in("dx") port, in("ax") value);
    }
}

pub unsafe fn output_doubleword(port: u16, value: u32) {
    unsafe {
        asm!("out dx, eax", in("dx") port, in("eax") value);
    }
}

pub unsafe fn input_byte(port: u16) -> u8 {
    let value;
    unsafe {
        asm!("in al, dx", out("al") value, in("dx") port);
    }
    value
}

pub unsafe fn input_word(port: u16) -> u16 {
    let value;
    unsafe {
        asm!("in ax, dx", out("ax") value, in("dx") port);
    }
    value
}

pub unsafe fn input_doubleword(port: u16) -> u32 {
    let value;
    unsafe {
        asm!("in eax, dx", out("eax") value, in("dx") port);
    }
    value
}

pub unsafe fn io_wait() {
    unsafe {
        asm!("out 0x80, al", in("al") 0u8);
    }
}
