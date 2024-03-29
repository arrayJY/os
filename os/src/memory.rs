use alloc::boxed::Box;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::{
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PhysFrame, Size4KiB,
    },
    PhysAddr, VirtAddr,
};

use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::structures::paging::{PageTableFlags, Translate};

pub mod memory_set;

pub struct MemoryFrameAllocator {
    memory_map: Option<&'static MemoryMap>,
    next: usize,
}

pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        None
    }
}

impl MemoryFrameAllocator {
    pub fn new() -> Self {
        MemoryFrameAllocator {
            memory_map: None,
            next: 0,
        }
    }
    pub fn get_mut(&mut self) -> &mut Self {
        self
    }
    pub unsafe fn init(&mut self, memory_map: &'static MemoryMap) {
        self.memory_map = Some(memory_map);
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        self.memory_map
            .expect("Frame allocator not initialized.")
            .iter()
            .filter(|r| r.region_type == MemoryRegionType::Usable)
            .map(|r| r.range.start_addr()..r.range.end_addr())
            .flat_map(|v| v.step_by(1024 * 4)) //4K
            .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for MemoryFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

pub const PAGE_SIZE: usize = 4096; //4KiB
lazy_static! {
    pub static ref FRAME_ALLOCATOR: Mutex<MemoryFrameAllocator> =
        Mutex::new(MemoryFrameAllocator::new());
    static ref PHSYICAL_MEMORY_OFFSET: Mutex<u64> = Mutex::new(0);
}

pub fn physical_memory_offset() -> u64 {
    *PHSYICAL_MEMORY_OFFSET.lock()
}

//Must call after initializing heap.
pub fn empty_page_table() -> &'static mut PageTable {
    Box::leak(Box::new(PageTable::new()))
}

pub fn kernel_mapped_new_page_table() -> OffsetPageTable<'static> {
    use crate::allocator::{HEAP_SIZE, HEAP_START};
    use crate::memory::FRAME_ALLOCATOR;
    use x86_64::structures::paging::{
        mapper::{MappedFrame, TranslateResult},
        PageTableFlags,
    };

    let phys_offset = VirtAddr::new(physical_memory_offset());
    let new_page_table = empty_page_table();
    let mut offset_page_table = unsafe { OffsetPageTable::new(new_page_table, phys_offset) };
    let translator = unsafe { current_offset_page_table() };
    let page_range = {
        let kernel_start_addr = VirtAddr::new(0);
        let kernel_end_addr = VirtAddr::new((HEAP_START + HEAP_SIZE) as u64);
        let start_page = Page::containing_address(kernel_start_addr);
        let end_page = Page::containing_address(kernel_end_addr);
        Page::<Size4KiB>::range(start_page, end_page)
    };

    let offset = physical_memory_offset();
    let offset_page_range = {
        const MEMORY_SIZE: u64 = 0x8000000; // 128MiB
        let start_virt = VirtAddr::new(offset);
        let end_virt = VirtAddr::new(offset + MEMORY_SIZE);
        let start_page = Page::containing_address(start_virt);
        let end_page = Page::containing_address(end_virt);
        Page::<Size4KiB>::range(start_page, end_page)
    };
    let mut frame_allocator_lock = FRAME_ALLOCATOR.lock();
    let frame_allocator = frame_allocator_lock.get_mut();
    for page in page_range {
        if let TranslateResult::Mapped { frame, flags, .. } =
            translator.translate(page.start_address())
        {
            if let MappedFrame::Size4KiB(frame) = frame {
                unsafe { offset_page_table.map_to(page, frame, flags, frame_allocator) }
                    .expect("map_to kernel page range failed.")
                    .flush()
            }
        }
    }
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    for page in offset_page_range {
        let frame = PhysFrame::<Size4KiB>::containing_address(PhysAddr::new(
            page.start_address().as_u64() - offset,
        ));
        unsafe { offset_page_table.map_to(page, frame, flags, frame_allocator) }
            .expect("map_to kernel page range failed.")
            .flush()
    }
    offset_page_table
}

pub fn init_frame_allocator(memory_map: &'static MemoryMap) {
    unsafe { FRAME_ALLOCATOR.lock().init(memory_map) }
}

pub fn alloc_frame() -> Option<PhysFrame> {
    FRAME_ALLOCATOR.lock().allocate_frame()
}

pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;
    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;
    let map_to_result = unsafe { mapper.map_to(page, frame, flags, frame_allocator) };
    map_to_result.expect("map_to failed.").flush()
}

pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    *PHSYICAL_MEMORY_OFFSET.lock() = physical_memory_offset.as_u64();
    let lv4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(lv4_table, physical_memory_offset)
}

pub unsafe fn current_offset_page_table() -> OffsetPageTable<'static> {
    let physical_memory_offset = VirtAddr::new(physical_memory_offset());
    let lv4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(lv4_table, physical_memory_offset)
}

pub unsafe fn current_page_table_address() -> usize {
    use x86_64::registers::control::Cr3;
    let (lv4_table_frame, _) = Cr3::read();
    lv4_table_frame.start_address().as_u64() as usize + physical_memory_offset() as usize
}

pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;
    let (lv4_table_frame, _) = Cr3::read();
    let phys = lv4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
    &mut *page_table_ptr
}

pub const KERNEL_STACK_START: u64 = 0x4000000;
pub const KERNEL_STACK_END: u64 = 0x4100000;
pub const KERNEL_STACK_SIZE: u64 = PAGE_SIZE as u64 * 7; // 28KiB
pub const GUARD_SIZE: u64 = PAGE_SIZE as u64;

pub fn init_kernel_stack(mapper: &mut OffsetPageTable) {
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    let mut allocator = FRAME_ALLOCATOR.lock();
    let allocator = allocator.get_mut();
    //Left one page between each stack as guard.
    for start in
        (KERNEL_STACK_START..KERNEL_STACK_END).step_by((KERNEL_STACK_SIZE + GUARD_SIZE) as usize)
    {
        let page_range = Page::<Size4KiB>::range(
            Page::containing_address(VirtAddr::new(start + GUARD_SIZE)),
            Page::containing_address(VirtAddr::new(start + GUARD_SIZE + KERNEL_STACK_SIZE)),
        );
        for page in page_range {
            let frame = allocator.allocate_frame().unwrap();
            unsafe { mapper.map_to(page, frame, flags, allocator) }
                .expect("map_to failed.")
                .flush();
        }
    }
}

#[inline]
pub fn get_app_kernel_stack(app_id: u64) -> u64 {
    use crate::memory::{GUARD_SIZE, KERNEL_STACK_END, KERNEL_STACK_SIZE};
    let top = KERNEL_STACK_END - (app_id as u64) * (KERNEL_STACK_SIZE + GUARD_SIZE);
    top
}
