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
#[derive(PartialEq, Eq)]
pub struct Status(usize);

impl Status {
    pub const ERROR_MASK: usize = 1 << (core::mem::size_of::<Status>() * 8 - 1);

    pub const SUCCESS: Status = Status(0);
    pub const LOAD_ERROR: Status = Status(Self::ERROR_MASK | 1);
    pub const INVALID_PARAMETER: Status = Status(Self::ERROR_MASK | 2);
    pub const UNSUPPORTED: Status = Status(Self::ERROR_MASK | 3);
    pub const BAD_BUFFER_SIZE: Status = Status(Self::ERROR_MASK | 4);
    pub const BUFFER_TOO_SMALL: Status = Status(Self::ERROR_MASK | 5);
    pub const NOT_READY: Status = Status(Self::ERROR_MASK | 6);
    pub const DEVICE_ERROR: Status = Status(Self::ERROR_MASK | 7);
    pub const WRITE_PROTECTED: Status = Status(Self::ERROR_MASK | 8);
    pub const OUT_OF_RESOURCES: Status = Status(Self::ERROR_MASK | 9);
    pub const VOLUME_CORRUPTED: Status = Status(Self::ERROR_MASK | 10);
    pub const VOLUME_FULL: Status = Status(Self::ERROR_MASK | 11);
    pub const NO_MEDIA: Status = Status(Self::ERROR_MASK | 12);
    pub const MEDIA_CHANGED: Status = Status(Self::ERROR_MASK | 13);
    pub const NOT_FOUND: Status = Status(Self::ERROR_MASK | 14);
    pub const ACCESS_DENIED: Status = Status(Self::ERROR_MASK | 15);
    pub const NO_RESPONSE: Status = Status(Self::ERROR_MASK | 16);
    pub const NO_MAPPING: Status = Status(Self::ERROR_MASK | 17);
    pub const TIMEOUT: Status = Status(Self::ERROR_MASK | 18);
    pub const NOT_STARTED: Status = Status(Self::ERROR_MASK | 19);
    pub const ALREADY_STARTED: Status = Status(Self::ERROR_MASK | 20);
    pub const ABORTED: Status = Status(Self::ERROR_MASK | 21);
    pub const ICMP_ERROR: Status = Status(Self::ERROR_MASK | 22);
    pub const TFTP_ERROR: Status = Status(Self::ERROR_MASK | 23);
    pub const PROTOCOL_ERROR: Status = Status(Self::ERROR_MASK | 24);
    pub const INCOMPATIBLE_VERSION: Status = Status(Self::ERROR_MASK | 25);
    pub const SECURITY_VIOLATION: Status = Status(Self::ERROR_MASK | 26);
    pub const CRC_ERROR: Status = Status(Self::ERROR_MASK | 27);
    pub const END_OF_MEDIA: Status = Status(Self::ERROR_MASK | 28);
    pub const END_OF_FILE: Status = Status(Self::ERROR_MASK | 31);
    pub const INVALID_LANGUAGE: Status = Status(Self::ERROR_MASK | 32);
    pub const COMPROMISED_DATA: Status = Status(Self::ERROR_MASK | 33);
    pub const IP_ADDRESS_CONFLICT: Status = Status(Self::ERROR_MASK | 34);
    pub const HTTP_ERROR: Status = Status(Self::ERROR_MASK | 35);
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
