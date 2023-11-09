use core::arch::asm;

#[repr(C, packed)]
pub struct Descriptor {
    pub size: u16,
    pub offset: u64,
}

#[repr(C, packed)]
pub struct Entry {
    pub limit0: u16,
    pub base0: u16,
    pub base1: u8,
    pub access_byte: u8,
    pub limit1_flags: u8,
    pub base2: u8,
}

#[repr(C, align(0x1000))]
pub struct GDT {
    pub null: Entry,
    pub code: Entry,
    pub data: Entry,
}

pub static DEFAULT_GDT: GDT = GDT {
    null: Entry {
        limit0: 0x0000,
        base0: 0x0000,
        base1: 0x00,
        access_byte: 0x00,
        limit1_flags: 0x00,
        base2: 0x00,
    },
    code: Entry {
        limit0: 0x0000,
        base0: 0x0000,
        base1: 0x00,
        access_byte: 0x9a,
        limit1_flags: 0xa0,
        base2: 0x00,
    },
    data: Entry {
        limit0: 0x0000,
        base0: 0x0000,
        base1: 0x00,
        access_byte: 0x92,
        limit1_flags: 0xa0,
        base2: 0x00,
    },
};

/// # Safety
/// this must be called from asm because the C calling convention on this function is a lie, pass a pointer to the gdt descriptor in rax
#[naked]
pub(super) unsafe extern "C" fn load_gdt() {
    unsafe {
        asm!(
            "
lgdt [rax]
mov ax, 0x10
mov ds, ax
mov es, ax
mov fs, ax
mov gs, ax
mov ss, ax
pop rax
push 0x08
push rax
retfq
",
            options(noreturn),
        );
    }
}
