use core::arch::asm;

pub unsafe fn output_byte(port: u16, value: u8) {
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") value);
    }
}

pub unsafe fn input_byte(port: u16) -> u8 {
    let value;
    unsafe {
        asm!("in al, dx", out("al") value, in("dx") port);
    }
    value
}

pub unsafe fn io_wait() {
    unsafe {
        asm!("out 0x80, al", in("al") 0u8);
    }
}

pub const PIC1_COMMAND_PORT: u16 = 0x20;
pub const PIC1_DATA_PORT: u16 = 0x21;
pub const PIC2_COMMAND_PORT: u16 = 0xA0;
pub const PIC2_DATA_PORT: u16 = 0xA1;

pub const PIC_EOI: u8 = 0x20;
pub const ICW1_INIT: u8 = 0x10;
pub const ICW1_ICW4: u8 = 0x01;
pub const ICW1_8086: u8 = 0x01;

pub unsafe fn remap_pic() {
    unsafe {
        let a1 = input_byte(PIC1_DATA_PORT);
        io_wait();
        let a2 = input_byte(PIC2_DATA_PORT);
        io_wait();

        output_byte(PIC1_COMMAND_PORT, ICW1_INIT | ICW1_ICW4);
        io_wait();
        output_byte(PIC2_COMMAND_PORT, ICW1_INIT | ICW1_ICW4);
        io_wait();

        output_byte(PIC1_DATA_PORT, 0x20);
        io_wait();
        output_byte(PIC2_DATA_PORT, 0x28);
        io_wait();

        output_byte(PIC1_DATA_PORT, 4);
        io_wait();
        output_byte(PIC2_DATA_PORT, 2);
        io_wait();

        output_byte(PIC1_DATA_PORT, ICW1_8086);
        io_wait();
        output_byte(PIC2_DATA_PORT, ICW1_8086);
        io_wait();

        output_byte(PIC1_DATA_PORT, a1);
        io_wait();
        output_byte(PIC2_DATA_PORT, a2);
        io_wait();
    }
}
