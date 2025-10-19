use core::{arch::asm, mem::offset_of};

#[repr(C, packed)]
pub struct GdtDescriptor {
    pub size: u16,
    pub offset: usize,
}

const _: () = assert!(size_of::<GdtDescriptor>() == 10);

pub struct Entry {
    pub limit0: u16,
    pub base0: u16,
    pub base1: u8,
    pub access_byte: u8,
    pub limit1_flags: u8,
    pub base2: u8,
}

const _: () = assert!(size_of::<Entry>() == 8);

#[repr(C, align(0x1000))]
struct Gdt {
    null: Entry,
    kernel_code: Entry,
    kernel_data: Entry,
}

#[allow(clippy::unusual_byte_groupings)]
static GDT: Gdt = Gdt {
    null: Entry {
        limit0: 0x0000,
        base0: 0x0000,
        base1: 0x00,
        access_byte: 0x00,
        limit1_flags: 0x00,
        base2: 0x00,
    },
    kernel_code: Entry {
        limit0: 0xFFFF,
        base0: 0x0000,
        base1: 0x00,
        access_byte: 0b1_00_1_1_0_1_1,
        limit1_flags: 0xF0 | 0b1010,
        base2: 0x00,
    },
    kernel_data: Entry {
        limit0: 0xFFFF,
        base0: 0x0000,
        base1: 0x00,
        access_byte: 0b1_00_1_0_0_1_1,
        limit1_flags: 0xF0 | 0b1010,
        base2: 0x00,
    },
};

pub unsafe fn setup_gdt() {
    let descriptor = GdtDescriptor {
        size: (size_of::<Gdt>() - 1) as _,
        offset: (&raw const GDT).addr(),
    };

    // load the gdt into the gdtr resgister
    unsafe { asm!("lgdt [{}]", in(reg) &raw const descriptor) };

    unsafe { reload_kernel_segments() };
}

unsafe fn reload_kernel_segments() {
    unsafe {
        asm!(
            "push {kernel_code}",
            "lea rax, [2f]",
            "push rax",
            "retfq",
            "2:",
            "mov ax, {kernel_data}",
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            "mov ss, ax",
            out("ax") _, // mark rax/ax as a clobber register
            kernel_code = const offset_of!(Gdt, kernel_code),
            kernel_data = const offset_of!(Gdt, kernel_data),
        );
    }
}
