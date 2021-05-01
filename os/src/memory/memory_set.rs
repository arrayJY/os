use alloc::vec::Vec;
use x86_64::{
    structures::paging::{
        mapper::TranslateResult,
        page::{Page, PageRange, PageRangeInclusive},
        Mapper, OffsetPageTable, PageTableFlags, Size4KiB, Translate,
    },
    VirtAddr,
};

use super::{active_level_4_table, empty_page_table};
use crate::memory::{PAGE_SIZE, phsyical_memory_offset};

pub const KERNEL_START: usize = 0x0;
pub const USER_START: usize = 0x8000000;
pub const USER_STACK_SIZE: usize = 1024 * 1024; //1MB
pub struct VirtAddrRange {
    pub strat: VirtAddr,
    pub end: VirtAddr,
}

pub struct MapArea {
    page_range: PageRangeInclusive,
    start_virt_addr: VirtAddr,
    end_virt_addr: VirtAddr,
    flags: PageTableFlags,
}

impl MapArea {
    pub fn new(start_virt_addr: VirtAddr, end_virt_addr: VirtAddr, flags: PageTableFlags) -> Self {
        let start = Page::containing_address(start_virt_addr);
        let end = Page::containing_address(end_virt_addr);
        Self {
            page_range: PageRangeInclusive { start, end },
            start_virt_addr,
            end_virt_addr,
            flags,
        }
    }
    pub fn map(&mut self, page_table: &mut OffsetPageTable) {
        for page in self.page_range {
            self.map_one(page, page_table)
        }
    }

    pub fn copy_data(&mut self, page_table: &mut OffsetPageTable, data: &[u8]) {
        let len = data.len();
        let physical_memory_offset = phsyical_memory_offset();
        let mut start_virt = self.start_virt_addr;
        let end_virtual = self.end_virt_addr;
        for page in self.page_range {
            let start = page.start_address().as_u64() as usize
                - self.page_range.start.start_address().as_u64() as usize;
            let src = &data[start..len.min(start + 4096)];
            let dst = unsafe {
                let mut dst = page_table.translate_addr(start_virt).unwrap().as_u64();
                dst += physical_memory_offset;
                core::slice::from_raw_parts_mut(dst as usize as *mut u8, src.len())
            };
            dst.copy_from_slice(src);
            start_virt += PAGE_SIZE;
            if start_virt >= end_virtual {
                break;
            }
        }
    }
}

impl MapArea {
    pub fn map_one(&mut self, page: Page, page_table: &mut OffsetPageTable) {
        use crate::memory::alloc_frame;
        use crate::memory::FRAME_ALLOCATOR;
        if let Err(x86_64::structures::paging::mapper::TranslateError::PageNotMapped) =
            page_table.translate_page(page)
        {
            let frame = alloc_frame().unwrap();
            let map_result = unsafe {
                page_table.map_to(page, frame, self.flags, FRAME_ALLOCATOR.lock().get_mut())
            };
            map_result.expect("Map failed.").flush();
        }
    }
}

pub struct MemorySet {
    pub page_table: OffsetPageTable<'static>,
    pub areas: Vec<MapArea>,
}

impl MemorySet {
    pub fn new() -> Self {
        let physical_memory_offset = VirtAddr::new(phsyical_memory_offset());
        Self {
            page_table: unsafe {
                OffsetPageTable::new(
                    active_level_4_table(physical_memory_offset),
                    physical_memory_offset,
                )
            },
            areas: Vec::new(),
        }
    }
    fn push(&mut self, mut map_area: MapArea, data: Option<&[u8]>) {
        map_area.map(&mut self.page_table);
        if let Some(data) = data {
            map_area.copy_data(&mut self.page_table, data)
        }
        self.areas.push(map_area);
    }
    pub fn insert(
        &mut self,
        start_virt_addr: VirtAddr,
        end_virt_addr: VirtAddr,
        flags: PageTableFlags,
        data: Option<&[u8]>,
    ) {
        self.push(MapArea::new(start_virt_addr, end_virt_addr, flags), data)
    }
    /*
    pub fn map_kernel_space(&mut self, translator: &OffsetPageTable, memory_offset: u64) {
        let kernel_space = Page::<Size4KiB>::range(
            Page::from_start_address(VirtAddr::new(KERNEL_START as u64)).unwrap(),
            Page::from_start_address(VirtAddr::new(USER_START as u64)).unwrap(),
        );
        let offset_kernel_space = Page::<Size4KiB>::range(
            Page::from_start_address(VirtAddr::new(KERNEL_START as u64 + memory_offset)).unwrap(),
            Page::from_start_address(VirtAddr::new(USER_START as u64 + memory_offset)).unwrap(),
        );
        use crate::memory::FRAME_ALLOCATOR;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        let mut frame_allocator = FRAME_ALLOCATOR.lock();
        let frame_allocator = frame_allocator.get_mut();
        let page_table = &mut self.page_table;
        for page in kernel_space {
            if let Ok(frame) = translator.translate_page(page) {
                unsafe {
                    page_table
                        .map_to(page, frame, flags, frame_allocator)
                        .expect("map_to failed.")
                        .flush()
                }
            }
        }
        for page in offset_kernel_space {
            if let Ok(frame) = translator.translate_page(page) {
                unsafe {
                    page_table
                        .map_to(page, frame, flags, frame_allocator)
                        .expect("map_to failed.")
                        .flush()
                }
            }
        }
    }
    */
}
impl MemorySet {
    pub fn from_elf(elf_data: &[u8]) -> (Self, usize, usize) {
        let mut memory_set = Self::new();
        let elf = xmas_elf::ElfFile::new(elf_data).expect("invalid elf!");
        let elf_header = elf.header;
        assert_eq!(
            elf.header.pt1.magic,
            [0x7f, 0x45, 0x4c, 0x46],
            "invalid elf!"
        );
        let ph_count = elf_header.pt2.ph_count();
        let mut max_page = Page::containing_address(VirtAddr::new(0));
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                let start_virt_addr = VirtAddr::new(ph.virtual_addr());
                let end_virt_addr = VirtAddr::new(ph.virtual_addr() + ph.mem_size());
                let mut flags = PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE;
                let ph_flag = ph.flags();
                if !ph_flag.is_execute() {
                    flags |= PageTableFlags::NO_EXECUTE
                }
                if ph_flag.is_write() {
                    flags |= PageTableFlags::WRITABLE
                }
                let map_area = MapArea::new(start_virt_addr, end_virt_addr, flags);
                max_page = map_area.page_range.end;
                memory_set.push(
                    map_area,
                    Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize]),
                );
            }
        }
        //TODO: guard page
        let mut user_stack_bottom = max_page.start_address();
        user_stack_bottom += 4096u64; // Page size = 4Kib
        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        memory_set.push(
            MapArea::new(
                user_stack_bottom,
                user_stack_top,
                PageTableFlags::PRESENT
                    | PageTableFlags::WRITABLE
                    | PageTableFlags::USER_ACCESSIBLE,
            ),
            None,
        );

        (
            memory_set,
            user_stack_top.as_u64() as usize,
            elf.header.pt2.entry_point() as usize,
        )
    }
}
