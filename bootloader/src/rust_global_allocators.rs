use crate::page_allocator::PAGE_ALLOCATOR;
use core::alloc::GlobalAlloc;

#[derive(Debug, Clone, Copy)]
struct GlobalAllocator;

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        PAGE_ALLOCATOR
            .with(|alloc| alloc.allocate(layout))
            .map_or(core::ptr::null_mut(), |addr| addr as _)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        PAGE_ALLOCATOR.with(|alloc| unsafe { alloc.free(ptr as _, layout) })
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: GlobalAllocator = GlobalAllocator;
