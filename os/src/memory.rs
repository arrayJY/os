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
use x86_64::structures::paging::PageTableFlags;
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

    fn usable_frames(&self) -> impl Iterator<Item=PhysFrame> {
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
pub unsafe fn empty_page_table() -> &'static mut PageTable {
    Box::leak(Box::new(PageTable::new()))
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
pub const KERNEL_STACK_SIZE: u64 = PAGE_SIZE as u64;

pub fn init_kernel_stack(mapper: &mut OffsetPageTable) {
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    let mut allocator = FRAME_ALLOCATOR.lock();
    let allocator = allocator.get_mut();
    //Left one page between each stack as guard.
    for start in (KERNEL_STACK_START..KERNEL_STACK_END)
        .step_by(KERNEL_STACK_SIZE as usize + PAGE_SIZE)
    {
        let page_range = Page::<Size4KiB>::range(
            Page::containing_address(VirtAddr::new(start)),
            Page::containing_address(VirtAddr::new(start + KERNEL_STACK_SIZE)),
        );
        for page in page_range {
            let frame = allocator.allocate_frame().unwrap();
            unsafe { mapper.map_to(page, frame, flags, allocator) }
                .expect("map_to failed.")
                .flush();
        }
    }
    let page_range = Page::<Size4KiB>::range(
        Page::containing_address(VirtAddr::new(KERNEL_STACK_START)),
        Page::containing_address(VirtAddr::new(KERNEL_STACK_END)),
    );
    for page in page_range {
        crate::println!("{:?} -> {:?}", page, mapper.translate_page(page));
    }
}

#[inline]
pub fn get_app_kernel_stack(app_id: u64) -> u64 {
    KERNEL_STACK_START + (app_id + 1) * KERNEL_STACK_SIZE
}
