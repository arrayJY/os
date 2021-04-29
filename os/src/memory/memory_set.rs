use alloc::vec::Vec;
use x86_64::{
    structures::paging::{
        page::{Page, PageRange},
        OffsetPageTable, PageTableFlags, Size4KiB, Translate,
    },
    VirtAddr,
};

use super::empty_page_table;
use crate::memory::phsyical_memory_offset;

pub const USER_STACK_SIZE: usize = 1024 * 1024; //1MB
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

    pub fn copy_data(&mut self, page_table: &mut OffsetPageTable, data: &[u8]) {
        let len = data.len();
        let physical_memory_offset = phsyical_memory_offset();
        for page in self.page_range {
            let start_virt = page.start_address();
            let start = page.start_address().as_u64() as usize;
            let src = &data[start..len.min(start + 4096)];
            let dst = unsafe {
                let mut dst = page_table.translate_addr(start_virt).unwrap().as_u64();
                dst += physical_memory_offset;
                core::slice::from_raw_parts_mut(dst as usize as *mut u8, 4096)
            };
            dst.copy_from_slice(src);
            if start >= len {
                break;
            }
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
}
impl MemorySet {
    pub fn from_elf(
        elf_data: &[u8],
        physical_memory_offset: VirtAddr,
    ) -> (Self, VirtAddr, VirtAddr) {
        let mut memory_set = Self::new(physical_memory_offset);
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
        let t = memory_set.push(
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
            user_stack_top,
            VirtAddr::new(elf.header.pt2.entry_point()),
        )
    }
}
