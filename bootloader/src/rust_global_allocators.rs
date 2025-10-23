use crate::page_allocator::with_page_allocator;
use core::{alloc::GlobalAlloc, num::NonZeroUsize};

#[derive(Debug, Clone, Copy)]
struct GlobalAllocator;

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        with_page_allocator(|alloc| {
            alloc
                .allocate(
                    unsafe { NonZeroUsize::new_unchecked(layout.align()) },
                    layout.size(),
                )
                .map_or(core::ptr::null_mut(), |addr| addr as _)
        })
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        with_page_allocator(|alloc| unsafe { alloc.free(ptr as _, layout.size()) })
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: GlobalAllocator = GlobalAllocator;
