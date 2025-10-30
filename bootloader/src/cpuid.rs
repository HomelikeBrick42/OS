use core::{arch::asm, mem::MaybeUninit};

pub struct CpuidRegisters {
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
}

pub fn is_cpuid_supported() -> bool {
    let changed_flags: u32;
    unsafe {
        asm!(
            "pushfq",
            "pushfq",
            "xor dword ptr [rsp], 0x00200000",
            "popfq",
            "pushfq",
            "pop rax",
            "xor eax, [rsp]",
            "popfq",
            out("eax") changed_flags,
            options(nomem)
        );
    }
    changed_flags & 0x00200000 != 0
}

pub unsafe fn cpuid(in_eax: u32, in_ecx: MaybeUninit<u32>) -> CpuidRegisters {
    let (eax, ebx, ecx, edx);
    unsafe {
        asm!(
            "push rbx",
            "cpuid",
            "mov r8, rbx",
            "pop rbx",
            inout("eax") in_eax => eax,
            out("r8") ebx, // using r8 because an llvm bug prevents using ebx
            inout("ecx") in_ecx => ecx,
            out("edx") edx,
        );
    }
    CpuidRegisters { eax, ebx, ecx, edx }
}
