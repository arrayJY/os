use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use linked_list_allocator::LockedHeap;
use x86_64::{
    structures::paging::{mapper::MapToError, Mapper, OffsetPageTable, Page, Size4KiB},
    VirtAddr,
};

pub const HEAP_START: usize = 0x5000000;
pub const HEAP_SIZE: usize = 0x0100000; // 16MiB;

pub struct Stupid;

unsafe impl GlobalAlloc for Stupid {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("dealloc Should be never called!")
    }
}

pub fn heap_init(mapper: &mut OffsetPageTable) -> Result<(), MapToError<Size4KiB>> {
    use x86_64::structures::paging::PageTableFlags;
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    use crate::memory::alloc_frame;
    use crate::memory::FRAME_ALLOCATOR;
    for page in page_range {
        let frame = alloc_frame().ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper
                .map_to(page, frame, flags, FRAME_ALLOCATOR.lock().get_mut())?
                .flush()
        };
    }

    unsafe { ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE) }

    Ok(())
}

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();
