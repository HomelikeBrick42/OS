use core::{
    alloc::{Allocator, GlobalAlloc, Layout},
    cell::SyncUnsafeCell,
    ptr::NonNull,
};

use crate::efi;

struct GlobalAllocator;

#[global_allocator]
static GLOBAL_ALLOCATOR: GlobalAllocator = GlobalAllocator;

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        unsafe {
            (*GLOBAL_LINKED_LIST_ALLOCATOR.get())
                .allocate(layout)
                .map_or(core::ptr::null_mut(), |mut mem| mem.as_mut().as_mut_ptr())
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        unsafe {
            (*GLOBAL_LINKED_LIST_ALLOCATOR.get()).deallocate(NonNull::new_unchecked(ptr), layout)
        }
    }

    unsafe fn alloc_zeroed(&self, layout: core::alloc::Layout) -> *mut u8 {
        unsafe {
            (*GLOBAL_LINKED_LIST_ALLOCATOR.get())
                .allocate_zeroed(layout)
                .map_or(core::ptr::null_mut(), |mut mem| mem.as_mut().as_mut_ptr())
        }
    }

    unsafe fn realloc(
        &self,
        ptr: *mut u8,
        layout: core::alloc::Layout,
        new_size: usize,
    ) -> *mut u8 {
        match new_size.cmp(&layout.size()) {
            core::cmp::Ordering::Less => unsafe {
                (*GLOBAL_LINKED_LIST_ALLOCATOR.get())
                    .shrink(
                        NonNull::new_unchecked(ptr),
                        layout,
                        Layout::from_size_align_unchecked(new_size, layout.align()),
                    )
                    .map_or(core::ptr::null_mut(), |mut mem| mem.as_mut().as_mut_ptr())
            },
            core::cmp::Ordering::Equal => ptr,
            core::cmp::Ordering::Greater => unsafe {
                (*GLOBAL_LINKED_LIST_ALLOCATOR.get())
                    .grow(
                        NonNull::new_unchecked(ptr),
                        layout,
                        Layout::from_size_align_unchecked(new_size, layout.align()),
                    )
                    .map_or(core::ptr::null_mut(), |mut mem| mem.as_mut().as_mut_ptr())
            },
        }
    }
}

pub(super) static GLOBAL_LINKED_LIST_ALLOCATOR: SyncUnsafeCell<LinkedListAllocator> =
    SyncUnsafeCell::new(LinkedListAllocator {
        first_allocation: spin::Mutex::new(core::ptr::null_mut()),
    });

#[repr(C, packed)]
struct AllocationHeader {
    next: *mut AllocationHeader,
    prev: *mut AllocationHeader,
    size: usize,
    allocated: bool,
}

pub struct LinkedListAllocator {
    first_allocation: spin::Mutex<*mut AllocationHeader>,
}

unsafe impl Send for LinkedListAllocator {}
unsafe impl Sync for LinkedListAllocator {}

impl LinkedListAllocator {
    pub unsafe fn from_efi_memory_map(
        memory_map: *mut efi::MemoryDescriptor,
        memory_descriptor_size: usize,
        memory_descriptor_count: usize,
    ) -> Self {
        let mut first = core::ptr::null_mut::<AllocationHeader>();
        let mut prev = first;
        for i in 0..memory_descriptor_count {
            let memory_descriptor = unsafe {
                *memory_map
                    .cast::<u8>()
                    .add(i * memory_descriptor_size)
                    .cast::<efi::MemoryDescriptor>()
            };

            if let efi::MemoryType::CONVENTIONAL_MEMORY = memory_descriptor.type_ {
                assert_ne!(memory_descriptor.number_of_pages, 0);

                let current = memory_descriptor.physical_start.0 as *mut AllocationHeader;
                if first.is_null() {
                    first = current;
                }

                unsafe {
                    current.write(AllocationHeader {
                        next: core::ptr::null_mut(),
                        prev,
                        allocated: false,
                        size: memory_descriptor.number_of_pages as usize * 4096
                            - core::mem::size_of::<AllocationHeader>(),
                    });
                    if !prev.is_null() {
                        (*prev).next = current;
                    }
                }

                prev = current;
            }
        }

        // Combine consecutive regions
        {
            let mut current = first;
            while !current.is_null() {
                unsafe {
                    if current.cast::<u8>().add((*current).size) == (*current).next.cast::<u8>() {
                        (*current).size +=
                            (*(*current).next).size + core::mem::size_of::<AllocationHeader>();
                        (*current).next = (*(*current).next).next;
                        (*(*current).next).prev = current;
                    } else {
                        current = (*current).next;
                    }
                }
            }
        }

        Self {
            first_allocation: spin::Mutex::new(first),
        }
    }
}

unsafe impl Allocator for LinkedListAllocator {
    #[inline]
    fn allocate(
        &self,
        layout: core::alloc::Layout,
    ) -> Result<core::ptr::NonNull<[u8]>, core::alloc::AllocError> {
        let first_allocation = self.first_allocation.lock();
        let mut allocation = *first_allocation;
        while !allocation.is_null() {
            unsafe {
                if !(*allocation).allocated && (*allocation).size >= layout.size() {
                    // TODO: alignment and not allocating the entire memory block

                    (*allocation).allocated = true;
                    return Ok(NonNull::new_unchecked(core::ptr::slice_from_raw_parts_mut(
                        allocation.add(1).cast::<u8>(),
                        (*allocation).size,
                    )));
                }
                allocation = (*allocation).next;
            }
        }
        Err(core::alloc::AllocError)
    }

    #[inline]
    unsafe fn deallocate(&self, ptr: core::ptr::NonNull<u8>, _layout: core::alloc::Layout) {
        let _lock = self.first_allocation.lock();
        unsafe {
            let allocation = ptr.as_ptr().cast::<AllocationHeader>().sub(1);
            (*allocation).allocated = false;
            if allocation.cast::<u8>().add((*allocation).size) == (*allocation).next.cast::<u8>() {
                (*allocation).size +=
                    (*(*allocation).next).size + core::mem::size_of::<AllocationHeader>();
                (*allocation).next = (*(*allocation).next).next;
                (*(*allocation).next).prev = allocation;
            }
            if !(*allocation).prev.is_null() {
                let current = (*allocation).prev;
                if current.cast::<u8>().add((*current).size) == (*current).next.cast::<u8>() {
                    (*current).size +=
                        (*(*current).next).size + core::mem::size_of::<AllocationHeader>();
                    (*current).next = (*(*current).next).next;
                    (*(*current).next).prev = current;
                }
            }
        }
    }
}
