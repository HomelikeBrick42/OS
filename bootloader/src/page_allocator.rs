use crate::{efi, hlt, interrupt_safe_mutex::InterruptSafeMutex, utils::error_screen};
use core::{fmt::Write, num::NonZeroUsize};

#[derive(Debug)]
pub struct Block {
    pub start_address: usize,
    pub page_count: usize,
    pub bitmap_start: usize,
}

pub struct PageAllocator {
    blocks: &'static [Block],
    bitmap: &'static mut [u8],
}

impl PageAllocator {
    pub fn blocks(&self) -> &'static [Block] {
        self.blocks
    }

    pub unsafe fn set_allocated(&mut self, address: usize, value: bool) {
        for block in self.blocks {
            if address < block.start_address
                || address >= block.start_address + block.page_count * 4096
            {
                continue;
            }

            let index = block.bitmap_start + (address - block.start_address) / 4096;
            let bitmap_index = index / u8::BITS as usize;
            let bit_index = index % u8::BITS as usize;
            if value {
                self.bitmap[bitmap_index] |= 1 << bit_index;
            } else {
                self.bitmap[bitmap_index] &= !(1 << bit_index);
            }
            return;
        }
    }

    pub fn get_allocated(&self, address: usize) -> Option<bool> {
        for block in self.blocks {
            if address < block.start_address
                || address >= block.start_address + block.page_count * 4096
            {
                continue;
            }

            let index = block.bitmap_start + (address - block.start_address) / 4096;
            let bitmap_index = index / u8::BITS as usize;
            let bit_index = index % u8::BITS as usize;
            return Some(self.bitmap[bitmap_index] & (1 << bit_index) != 0);
        }
        None
    }

    pub fn allocate(&mut self, alignment: NonZeroUsize, size: usize) -> Option<usize> {
        if size == 0 {
            return Some(alignment.get());
        }

        for block in self.blocks {
            let mut start = None;
            let mut seen_count = 0usize;
            for page in 0..block.page_count {
                {
                    let index = block.bitmap_start + page;
                    let bitmap_index = index / u8::BITS as usize;
                    let bit_index = index % u8::BITS as usize;
                    if self.bitmap[bitmap_index] & (1 << bit_index) != 0 {
                        start = None;
                        seen_count = 0;
                        continue;
                    }
                }

                if start.is_none() {
                    // make sure its aligned if the allocation is starting here
                    if (block.start_address + page * 4096) % alignment.get() != 0 {
                        continue;
                    }
                    start = Some(block.start_address + page * 4096);
                }
                seen_count += 4096;

                if let Some(start) = start
                    && seen_count >= size
                {
                    for i in 0..size.div_ceil(4096) {
                        let index = block.bitmap_start + (start - block.start_address) / 4096 + i;
                        let bitmap_index = index / u8::BITS as usize;
                        let bit_index = index % u8::BITS as usize;
                        self.bitmap[bitmap_index] |= 1 << bit_index;
                    }
                    return Some(start);
                }
            }
        }
        None
    }

    pub unsafe fn free(&mut self, address: usize, size: usize) {
        if size == 0 {
            return;
        }

        for i in 0..size.div_ceil(4096) {
            unsafe { self.set_allocated(address + i * 4096, false) };
        }
    }

    pub fn total_pages(&self) -> usize {
        self.blocks.iter().map(|block| block.page_count).sum()
    }

    pub fn allocated_pages(&self) -> usize {
        self.bitmap
            .iter()
            .map(|byte| byte.count_ones() as usize)
            .sum()
    }

    pub fn free_pages(&self) -> usize {
        self.bitmap
            .iter()
            .map(|byte| byte.count_zeros() as usize)
            .sum()
    }
}

pub static PAGE_ALLOCATOR: InterruptSafeMutex<PageAllocator> =
    InterruptSafeMutex::new(PageAllocator {
        blocks: &[],
        bitmap: &mut [],
    });

pub unsafe fn init_page_allocator(
    memory_map: *mut efi::MemoryDescriptor,
    memory_map_size: usize,
    memory_descriptor_size: usize,
) {
    let memory_map_count = memory_map_size / memory_descriptor_size;

    // sort memory map
    {
        let mut i = 1;
        while i < memory_map_count {
            let mut j = i;
            while j > 0
                && let left = unsafe { memory_map.byte_add((j - 1) * memory_descriptor_size) }
                && let right = unsafe { memory_map.byte_add(j * memory_descriptor_size) }
                && unsafe { (*left).physical_start > (*right).physical_start }
            {
                unsafe {
                    core::ptr::swap_nonoverlapping(
                        left.cast::<u8>(),
                        right.cast::<u8>(),
                        memory_descriptor_size,
                    );
                }
                j -= 1;
            }
            i += 1;
        }
    }

    let mut required_page_bits = 0usize;
    let mut block_count = 1usize;

    for i in 0..memory_map_count {
        let memory_descriptor = unsafe { &*memory_map.byte_add(i * memory_descriptor_size) };

        assert_eq!(
            memory_descriptor.virtual_start, 0,
            "im not considering non-identity virtual mapping currently"
        );

        if i + 1 < memory_map_count {
            let next_memory_descriptor =
                unsafe { &*memory_map.byte_add((i + 1) * memory_descriptor_size) };

            if memory_descriptor.physical_start + memory_descriptor.number_of_pages * 4096
                != next_memory_descriptor.physical_start
            {
                block_count += 1;
            }
        }

        required_page_bits += memory_descriptor.number_of_pages;
    }

    let blocks_size = block_count * size_of::<Block>();
    let bitmap_size = required_page_bits.div_ceil(u8::BITS as usize);
    let required_allocator_size = bitmap_size + bitmap_size;

    let mut ptr = core::ptr::null_mut::<()>();
    let mut size_so_far = 0;
    for i in 0..memory_map_count {
        let memory_descriptor = unsafe { &*memory_map.byte_add(i * memory_descriptor_size) };

        let mut can_use_page = matches!(
            memory_descriptor.memory_type,
            efi::MemoryType::ConventionalMemory
        );

        if can_use_page {
            if ptr.is_null() {
                ptr = core::ptr::with_exposed_provenance_mut(memory_descriptor.physical_start);
            }

            size_so_far += memory_descriptor.number_of_pages * 4096;

            if size_so_far >= required_allocator_size {
                break;
            }
        }

        if i + 1 < memory_map_count {
            let next_memory_descriptor =
                unsafe { &*memory_map.byte_add((i + 1) * memory_descriptor_size) };

            if memory_descriptor.physical_start + memory_descriptor.number_of_pages * 4096
                != next_memory_descriptor.physical_start
            {
                can_use_page = false;
            }
        }

        if !can_use_page {
            ptr = core::ptr::null_mut();
            size_so_far = 0;
        }
    }
    if size_so_far < required_allocator_size {
        error_screen(|writer| {
            writeln!(
                writer,
                "Cannot find enough memory to store page allocator state"
            )
            .unwrap();

            loop {
                hlt();
            }
        })
    }

    unsafe { core::ptr::write_bytes(ptr, 0, required_allocator_size) };
    let blocks = unsafe { core::slice::from_raw_parts_mut(ptr.cast::<Block>(), block_count) };
    let bitmap = unsafe {
        core::slice::from_raw_parts_mut(ptr.byte_add(blocks_size).cast::<u8>(), bitmap_size)
    };

    {
        let mut bitmap_index = 0;
        let mut block_index = 0usize;
        for i in 0..memory_map_count {
            let memory_descriptor = unsafe { &*memory_map.byte_add(i * memory_descriptor_size) };

            let block = &mut blocks[block_index];
            if block.page_count == 0 {
                block.start_address = memory_descriptor.physical_start;
                block.bitmap_start = bitmap_index;
            }

            block.page_count += memory_descriptor.number_of_pages;
            bitmap_index += memory_descriptor.number_of_pages;

            if i + 1 < memory_map_count {
                let next_memory_descriptor =
                    unsafe { &*memory_map.byte_add((i + 1) * memory_descriptor_size) };

                if memory_descriptor.physical_start + memory_descriptor.number_of_pages * 4096
                    != next_memory_descriptor.physical_start
                {
                    block_index += 1;
                }
            }
        }
    }

    let mut page_allocator = PageAllocator { blocks, bitmap };

    for i in 0..memory_map_count {
        let memory_descriptor = unsafe { &*memory_map.byte_add(i * memory_descriptor_size) };
        if !matches!(
            memory_descriptor.memory_type,
            efi::MemoryType::ConventionalMemory
                | efi::MemoryType::BootServicesCode
                | efi::MemoryType::BootServicesData
        ) {
            for index in 0..memory_descriptor.number_of_pages {
                unsafe {
                    page_allocator
                        .set_allocated(memory_descriptor.physical_start + index * 4096, true);
                }
            }
        }
    }

    for index in 0..required_allocator_size.div_ceil(4096) {
        let base_address = ptr.addr();
        unsafe { page_allocator.set_allocated(base_address + index * 4096, true) };
    }

    // make sure to not allocate the null page, just for now
    unsafe { page_allocator.set_allocated(0, true) };

    PAGE_ALLOCATOR.with(|alloc| *alloc = page_allocator);
}
