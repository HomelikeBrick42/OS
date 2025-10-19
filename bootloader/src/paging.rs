pub unsafe fn init_paging_and_identity_map_all_pages_from_page_allocator() {}

pub unsafe fn map_page(physical_address: usize, virtual_address: usize) {
    _ = physical_address;
    _ = virtual_address;
}

pub unsafe fn enable_paging() {}
