use alloc::vec::Vec;
use x86_64::{
    structures::paging::{
        page::{Page, PageRangeInclusive},
        Mapper, OffsetPageTable, PageTableFlags, Translate,
    },
    VirtAddr,
};

use super::active_level_4_table;
use crate::memory::{physical_memory_offset, PAGE_SIZE};
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::structures::paging::mapper::TranslateError::PageNotMapped;

pub const KERNEL_START: usize = 0x0;
pub const USER_START: usize = 0x8000000;
pub const USER_STACK_SIZE: usize = 1024 * 1024; //1MB

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

    pub fn from(other: &MapArea) -> Self {
        Self {
            page_range: other.page_range,
            start_virt_addr: other.start_virt_addr,
            end_virt_addr: other.end_virt_addr,
            flags: other.flags,
        }
    }
    pub fn map(&mut self, page_table: &mut OffsetPageTable) {
        for page in self.page_range {
            self.map_one(page, page_table)
        }
    }

    pub fn unmap(&mut self, page_table: &mut OffsetPageTable) {
        for page in self.page_range {
            page_table.unmap(page);
        }
    }

    pub fn copy_data(&mut self, page_table: &mut OffsetPageTable, data: &[u8]) {
        let len = data.len();
        let physical_memory_offset = physical_memory_offset();
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

        match page_table.translate_page(page) {
            Err(PageNotMapped) => {
                let frame = alloc_frame().unwrap();
                let map_result = unsafe {
                    page_table.map_to(page, frame, self.flags, FRAME_ALLOCATOR.lock().get_mut())
                };
                map_result.expect("Map failed.").flush();
            }
            /*
            Ok(_) => {
                page_table.unmap(page).unwrap();
                let frame = alloc_frame().unwrap();
                let map_result = unsafe {
                    page_table.map_to(page, frame, self.flags, FRAME_ALLOCATOR.lock().get_mut())
                };
                map_result.expect("Map failed.").flush();
            }
             */
            _ => {}
        }
    }
}

pub struct MemorySet {
    pub page_table: OffsetPageTable<'static>,
    pub areas: Vec<MapArea>,
}

impl MemorySet {
    pub fn new() -> Self {
        let physical_memory_offset = VirtAddr::new(physical_memory_offset());
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

    pub fn page_table_address(&mut self, translator: &OffsetPageTable) -> usize {
        use x86_64::structures::paging::PageTable;
        let lv4_table: *const PageTable = self.page_table.level_4_table();
        let page_table_phys_addr = translator
            .translate_addr(VirtAddr::new(lv4_table as u64))
            .unwrap();
        page_table_phys_addr.as_u64() as usize
    }

    pub fn remove_all_areas(&mut self) {
        let page_table = &mut self.page_table;
        self.areas
            .iter_mut()
            .rev()
            .for_each(|area| area.unmap(page_table));
        self.areas.clear();
    }
    pub fn remove_area_with_start_addr(&mut self, start_addr: VirtAddr) {
        if let Some((i, area)) = self
            .areas
            .iter_mut()
            .enumerate()
            .find(|(i, area)| area.page_range.start.start_address() == start_addr)
        {
            area.unmap(&mut self.page_table);
            self.areas.remove(i);
        }
    }
}

impl MemorySet {
    pub fn from(user_space: &MemorySet) -> Self {
        let mut memory_set = Self::new();
        let memory_offset = physical_memory_offset();
        for area in user_space.areas.iter() {
            let mut new_area = MapArea::from(area);
            let data = {
                let start = (user_space
                    .page_table
                    .translate_addr(area.start_virt_addr)
                    .unwrap()
                    .as_u64()
                    + memory_offset) as *const u8;
                let len = (area.end_virt_addr - area.start_virt_addr) as usize;
                unsafe { core::slice::from_raw_parts(start, len) }
            };
            memory_set.push(new_area, Some(data));
        }
        memory_set
    }
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
        let mut user_stack_bottom = max_page.start_address();
        user_stack_bottom += 4096u64; //Guard Page

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

lazy_static! {
    pub static ref KERNEL_SPACE: Mutex<MemorySet> = Mutex::new(MemorySet::new());
}
