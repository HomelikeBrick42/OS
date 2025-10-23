use crate::utils::{inb, io_wait, outb};

pub const PIC1_COMMAND: u16 = 0x20;
pub const PIC1_DATA: u16 = 0x21;
pub const PIC2_COMMAND: u16 = 0xA0;
pub const PIC2_DATA: u16 = 0xA1;

pub const PIC_EOI: u8 = 0x20;
pub const ICW1_INIT: u8 = 0x10;
pub const ICW1_ICW4: u8 = 0x01;
pub const ICW4_8086: u8 = 0x01;

pub unsafe fn remap_pic(offset1: u8, offset2: u8) {
    let a1 = unsafe { inb(PIC1_DATA) };
    io_wait();
    let a2 = unsafe { inb(PIC2_DATA) };
    io_wait();

    unsafe { outb(PIC1_COMMAND, ICW1_INIT | ICW1_ICW4) };
    io_wait();
    unsafe { outb(PIC2_COMMAND, ICW1_INIT | ICW1_ICW4) };
    io_wait();

    unsafe { outb(PIC1_DATA, offset1) };
    io_wait();
    unsafe { outb(PIC2_DATA, offset2) };
    io_wait();

    unsafe { outb(PIC1_DATA, 4) };
    io_wait();
    unsafe { outb(PIC2_DATA, 2) };
    io_wait();

    unsafe { outb(PIC1_DATA, ICW4_8086) };
    io_wait();
    unsafe { outb(PIC2_DATA, ICW4_8086) };
    io_wait();

    unsafe { outb(PIC1_DATA, a1) };
    io_wait();
    unsafe { outb(PIC2_DATA, a2) };
    io_wait();
}

pub unsafe fn pic1_end() {
    unsafe { outb(PIC1_COMMAND, PIC_EOI) };
    io_wait();
}

pub unsafe fn pic2_end() {
    unsafe { outb(PIC2_COMMAND, PIC_EOI) };
    io_wait();
    unsafe { outb(PIC1_COMMAND, PIC_EOI) };
    io_wait();
}
