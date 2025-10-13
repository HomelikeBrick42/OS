use spin::Mutex;

use crate::{efi, hlt, text_writer::TextWriter};
use core::fmt::Write;

#[derive(Debug)]
pub struct Block {
    pub start_address: usize,
    pub pages_count: usize,
    pub bitmap_start: usize,
}

pub struct PageAllocator {
    pub blocks: &'static [Block],
    bitmap: &'static mut [u8],
}

impl PageAllocator {
    pub unsafe fn set_allocated(&mut self, address: usize, value: bool) {
        for block in self.blocks {
            if address < block.start_address
                || address >= block.start_address + block.pages_count * 4096
            {
                continue;
            }

            let index = block.bitmap_start + (address - block.start_address) / 4096;
            let bitmap_index = index / u8::BITS as usize;
            let bit_index = bitmap_index % u8::BITS as usize;
            if value {
                self.bitmap[bitmap_index] |= 1 << bit_index;
            } else {
                self.bitmap[bitmap_index] &= !(1 << bit_index);
            }
        }
    }

    pub fn get_allocated(&self, address: usize) -> Option<bool> {
        for block in self.blocks {
            if address < block.start_address
                || address >= block.start_address + block.pages_count * 4096
            {
                continue;
            }

            let index = block.bitmap_start + (address - block.start_address) / 4096;
            let bitmap_index = index / u8::BITS as usize;
            let bit_index = bitmap_index % u8::BITS as usize;
            return Some(self.bitmap[bitmap_index] & (1 << bit_index) != 0);
        }
        None
    }
}

static PAGE_ALLOCATOR: Mutex<PageAllocator> = Mutex::new(PageAllocator {
    blocks: &[],
    bitmap: &mut [],
});

pub fn with_page_allocator<R>(f: impl FnOnce(&mut PageAllocator) -> R) -> R {
    f(&mut PAGE_ALLOCATOR.lock())
}

pub unsafe fn init_page_allocator(
    memory_map: *mut efi::MemoryDescriptor,
    memory_map_size: usize,
    memory_descriptor_size: usize,
    text_writer: &mut TextWriter<'_>,
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

            if memory_descriptor.physical_start + memory_descriptor.number_of_pages as usize * 4096
                != next_memory_descriptor.physical_start
            {
                block_count += 1;
            }
        }

        required_page_bits += memory_descriptor.number_of_pages as usize;
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
                | efi::MemoryType::BootServicesCode
                | efi::MemoryType::BootServicesData
        );

        if can_use_page {
            if ptr.is_null() {
                ptr = core::ptr::with_exposed_provenance_mut(memory_descriptor.physical_start);
            }

            size_so_far += memory_descriptor.number_of_pages as usize * 4096;

            if size_so_far >= required_allocator_size {
                break;
            }
        }

        if i + 1 < memory_map_count {
            let next_memory_descriptor =
                unsafe { &*memory_map.byte_add((i + 1) * memory_descriptor_size) };

            if memory_descriptor.physical_start + memory_descriptor.number_of_pages as usize * 4096
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
    if size_so_far >= required_allocator_size {
        writeln!(text_writer, "Allocating page allocator state at {:p}", ptr).unwrap();
    } else {
        writeln!(
            text_writer,
            "Cannot find enough memory to store page allocator state"
        )
        .unwrap();
        hlt()
    }

    unsafe { core::ptr::write_bytes(ptr, 0, required_allocator_size) };
    let blocks = unsafe { core::slice::from_raw_parts_mut(ptr.cast::<Block>(), block_count) };
    let bitmap = unsafe {
        core::slice::from_raw_parts_mut(ptr.byte_add(blocks_size).cast::<u8>(), bitmap_size)
    };

    {
        let mut block_index = 0usize;
        for i in 0..memory_map_count {
            let memory_descriptor = unsafe { &*memory_map.byte_add(i * memory_descriptor_size) };

            let block = &mut blocks[block_index];
            if block.pages_count == 0 {
                block.start_address = memory_descriptor.physical_start;
                block.bitmap_start = i;
            }

            if !matches!(
                memory_descriptor.memory_type,
                efi::MemoryType::ConventionalMemory
                    | efi::MemoryType::BootServicesCode
                    | efi::MemoryType::BootServicesData
            ) {
                let index = i / u8::BITS as usize;
                let bit_index = i % u8::BITS as usize;
                bitmap[index] |= 1 << bit_index;
            }

            block.pages_count += 1;

            if i + 1 < memory_map_count {
                let next_memory_descriptor =
                    unsafe { &*memory_map.byte_add((i + 1) * memory_descriptor_size) };

                if memory_descriptor.physical_start
                    + memory_descriptor.number_of_pages as usize * 4096
                    != next_memory_descriptor.physical_start
                {
                    block_index += 1;
                }
            }
        }
    }

    let mut page_allocator = PageAllocator { blocks, bitmap };
    for index in 0..required_allocator_size.div_ceil(4096) {
        let base_address = ptr.addr();
        unsafe { page_allocator.set_allocated(base_address + index * 4096, true) };
    }
    *PAGE_ALLOCATOR.lock() = page_allocator;
}
