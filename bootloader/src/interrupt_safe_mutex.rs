use crate::idt::with_disabled_interrupts;
use core::{
    cell::UnsafeCell,
    sync::atomic::{AtomicBool, Ordering},
};

pub struct InterruptSafeMutex<T: ?Sized> {
    locked: AtomicBool,
    value: UnsafeCell<T>,
}

unsafe impl<T: Send + ?Sized> Send for InterruptSafeMutex<T> {}
unsafe impl<T: Send + ?Sized> Sync for InterruptSafeMutex<T> {}

impl<T> InterruptSafeMutex<T> {
    pub const fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    pub const fn into_inner(self) -> T {
        self.value.into_inner()
    }
}

impl<T: ?Sized> InterruptSafeMutex<T> {
    pub const fn get_mut(&mut self) -> &mut T {
        self.value.get_mut()
    }

    pub fn with<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        with_disabled_interrupts(|| {
            while self
                .locked
                .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
                .is_err()
            {
                core::hint::spin_loop();
            }

            let value = f(unsafe { &mut *self.value.get() });

            self.locked.store(false, Ordering::Release);

            value
        })
    }
}
