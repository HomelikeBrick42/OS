mod gop;
mod guid;
mod tables;

pub use gop::*;
pub use guid::*;
pub use tables::*;

pub struct Handle_;
pub type Handle = *mut Handle_;

#[repr(transparent)]
#[must_use]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Status(usize);

impl Status {
    pub const ERROR_MASK: usize = 1 << (core::mem::size_of::<Self>() * 8 - 1);

    pub const SUCCESS: Self = Self(0);
    pub const LOAD_ERROR: Self = Self(Self::ERROR_MASK | 1);
    pub const INVALID_PARAMETER: Self = Self(Self::ERROR_MASK | 2);
    pub const UNSUPPORTED: Self = Self(Self::ERROR_MASK | 3);
    pub const BAD_BUFFER_SIZE: Self = Self(Self::ERROR_MASK | 4);
    pub const BUFFER_TOO_SMALL: Self = Self(Self::ERROR_MASK | 5);
    pub const NOT_READY: Self = Self(Self::ERROR_MASK | 6);
    pub const DEVICE_ERROR: Self = Self(Self::ERROR_MASK | 7);
    pub const WRITE_PROTECTED: Self = Self(Self::ERROR_MASK | 8);
    pub const OUT_OF_RESOURCES: Self = Self(Self::ERROR_MASK | 9);
    pub const VOLUME_CORRUPTED: Self = Self(Self::ERROR_MASK | 10);
    pub const VOLUME_FULL: Self = Self(Self::ERROR_MASK | 11);
    pub const NO_MEDIA: Self = Self(Self::ERROR_MASK | 12);
    pub const MEDIA_CHANGED: Self = Self(Self::ERROR_MASK | 13);
    pub const NOT_FOUND: Self = Self(Self::ERROR_MASK | 14);
    pub const ACCESS_DENIED: Self = Self(Self::ERROR_MASK | 15);
    pub const NO_RESPONSE: Self = Self(Self::ERROR_MASK | 16);
    pub const NO_MAPPING: Self = Self(Self::ERROR_MASK | 17);
    pub const TIMEOUT: Self = Self(Self::ERROR_MASK | 18);
    pub const NOT_STARTED: Self = Self(Self::ERROR_MASK | 19);
    pub const ALREADY_STARTED: Self = Self(Self::ERROR_MASK | 20);
    pub const ABORTED: Self = Self(Self::ERROR_MASK | 21);
    pub const ICMP_ERROR: Self = Self(Self::ERROR_MASK | 22);
    pub const TFTP_ERROR: Self = Self(Self::ERROR_MASK | 23);
    pub const PROTOCOL_ERROR: Self = Self(Self::ERROR_MASK | 24);
    pub const INCOMPATIBLE_VERSION: Self = Self(Self::ERROR_MASK | 25);
    pub const SECURITY_VIOLATION: Self = Self(Self::ERROR_MASK | 26);
    pub const CRC_ERROR: Self = Self(Self::ERROR_MASK | 27);
    pub const END_OF_MEDIA: Self = Self(Self::ERROR_MASK | 28);
    pub const END_OF_FILE: Self = Self(Self::ERROR_MASK | 31);
    pub const INVALID_LANGUAGE: Self = Self(Self::ERROR_MASK | 32);
    pub const COMPROMISED_DATA: Self = Self(Self::ERROR_MASK | 33);
    pub const IP_ADDRESS_CONFLICT: Self = Self(Self::ERROR_MASK | 34);
    pub const HTTP_ERROR: Self = Self(Self::ERROR_MASK | 35);
}

impl core::fmt::Display for Status {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            Self::SUCCESS => write!(f, "EFI_SUCCESS"),
            Self::LOAD_ERROR => write!(f, "EFI_LOAD_ERROR"),
            Self::INVALID_PARAMETER => write!(f, "EFI_INVALID_PARAMETER"),
            Self::UNSUPPORTED => write!(f, "EFI_UNSUPPORTED"),
            Self::BAD_BUFFER_SIZE => write!(f, "EFI_BAD_BUFFER_SIZE"),
            Self::BUFFER_TOO_SMALL => write!(f, "EFI_BUFFER_TOO_SMALL"),
            Self::NOT_READY => write!(f, "EFI_NOT_READY"),
            Self::DEVICE_ERROR => write!(f, "EFI_DEVICE_ERROR"),
            Self::WRITE_PROTECTED => write!(f, "EFI_WRITE_PROTECTED"),
            Self::OUT_OF_RESOURCES => write!(f, "EFI_OUT_OF_RESOURCES"),
            Self::VOLUME_CORRUPTED => write!(f, "EFI_VOLUME_CORRUPTED"),
            Self::VOLUME_FULL => write!(f, "EFI_VOLUME_FULL"),
            Self::NO_MEDIA => write!(f, "EFI_NO_MEDIA"),
            Self::MEDIA_CHANGED => write!(f, "EFI_MEDIA_CHANGED"),
            Self::NOT_FOUND => write!(f, "EFI_NOT_FOUND"),
            Self::ACCESS_DENIED => write!(f, "EFI_ACCESS_DENIED"),
            Self::NO_RESPONSE => write!(f, "EFI_NO_RESPONSE"),
            Self::NO_MAPPING => write!(f, "EFI_NO_MAPPING"),
            Self::TIMEOUT => write!(f, "EFI_TIMEOUT"),
            Self::NOT_STARTED => write!(f, "EFI_NOT_STARTED"),
            Self::ALREADY_STARTED => write!(f, "EFI_ALREADY_STARTED"),
            Self::ABORTED => write!(f, "EFI_ABORTED"),
            Self::ICMP_ERROR => write!(f, "EFI_ICMP_ERROR"),
            Self::TFTP_ERROR => write!(f, "EFI_TFTP_ERROR"),
            Self::PROTOCOL_ERROR => write!(f, "EFI_PROTOCOL_ERROR"),
            Self::INCOMPATIBLE_VERSION => write!(f, "EFI_INCOMPATIBLE_VERSION"),
            Self::SECURITY_VIOLATION => write!(f, "EFI_SECURITY_VIOLATION"),
            Self::CRC_ERROR => write!(f, "EFI_CRC_ERROR"),
            Self::END_OF_MEDIA => write!(f, "EFI_END_OF_MEDIA"),
            Self::END_OF_FILE => write!(f, "EFI_END_OF_FILE"),
            Self::INVALID_LANGUAGE => write!(f, "EFI_INVALID_LANGUAGE"),
            Self::COMPROMISED_DATA => write!(f, "EFI_COMPROMISED_DATA"),
            Self::IP_ADDRESS_CONFLICT => write!(f, "EFI_IP_ADDRESS_CONFLICT"),
            Self::HTTP_ERROR => write!(f, "EFI_HTTP_ERROR"),
            _ => write!(f, "unknown efi error"),
        }
    }
}

impl core::ops::FromResidual for Status {
    fn from_residual(residual: <Self as core::ops::Try>::Residual) -> Self {
        residual
    }
}

// Make ? operator work
impl core::ops::Try for Status {
    type Output = ();
    type Residual = Status;

    fn from_output((): Self::Output) -> Self {
        Self::SUCCESS
    }

    fn branch(self) -> core::ops::ControlFlow<Self::Residual, Self::Output> {
        let Status(code) = self;
        if code & Self::ERROR_MASK != 0 {
            core::ops::ControlFlow::Break(self)
        } else {
            core::ops::ControlFlow::Continue(())
        }
    }
}

#[macro_export]
macro_rules! wrap_self_function_pointer {
    ($name:ident($($arg:ident: $typ:ty),* $(,)?) $(-> $ret_typ:ty)?) => {
        #[inline]
        pub unsafe fn $name(self: *mut Self, $($arg: $typ),*) $(-> $ret_typ)? {
            unsafe { (*self).$name.unwrap_unchecked()(self, $($arg),*) }
        }
    };
}
