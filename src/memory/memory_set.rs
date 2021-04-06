use alloc::vec::Vec;
use x86_64::{
    structures::paging::{
        page::{Page, PageRange},
        OffsetPageTable, PageTableFlags,
    },
    VirtAddr,
};

use super::empty_page_table;

pub struct VirtAddrRange {
    pub strat: VirtAddr,
    pub end: VirtAddr,
}

pub struct MapArea {
    page_range: PageRange,
    flags: PageTableFlags,
}

impl MapArea {
    pub fn new(start_virt_addr: VirtAddr, end_virt_addr: VirtAddr, flags: PageTableFlags) -> Self {
        let start = Page::containing_address(start_virt_addr);
        let end = Page::containing_address(end_virt_addr);
        Self {
            page_range: PageRange { start, end },
            flags,
        }
    }
    pub fn map(&mut self, page_table: &mut OffsetPageTable) {
        for page in self.page_range {
            self.map_one(page, page_table)
        }
    }
}

impl MapArea {
    pub fn map_one(&mut self, page: Page, page_table: &mut OffsetPageTable) {
        use crate::memory::alloc_frame;
        use crate::memory::FRAME_ALLOCATOR;
        use x86_64::structures::paging::Mapper;
        let frame = alloc_frame().unwrap();
        let map_result =
            unsafe { page_table.map_to(page, frame, self.flags, FRAME_ALLOCATOR.lock().get_mut()) };
        map_result.expect("Map failed.").flush();
    }
}

pub struct MemorySet {
    page_table: OffsetPageTable<'static>,
    areas: Vec<MapArea>,
}

impl MemorySet {
    pub fn new(physical_memory_offset: VirtAddr) -> Self {
        Self {
            page_table: unsafe { OffsetPageTable::new(empty_page_table(), physical_memory_offset) },
            areas: Vec::new(),
        }
    }
    fn push(&mut self, mut map_area: MapArea, _data: Option<&[u8]>) {
        map_area.map(&mut self.page_table);
        self.areas.push(map_area);
    }
    pub fn insert(
        &mut self,
        start_virt_addr: VirtAddr,
        end_virt_addr: VirtAddr,
        flags: PageTableFlags,
    ) {
        self.push(MapArea::new(start_virt_addr, end_virt_addr, flags), None)
    }
}
