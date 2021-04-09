use crate::println;
use x86_64::{
    structures::paging::{
        frame::PhysFrameRangeInclusive, page::PageRangeInclusive, Mapper, OffsetPageTable, Page,
        PhysFrame, Size4KiB, Translate,
    },
    VirtAddr,
};

pub static mut USER_STACK: [u8; 1024 * 8] = [0u8; 1024 * 8];
pub fn create_user_stack_map(
    page_range: PageRangeInclusive,
    frame_range: PhysFrameRangeInclusive,
    mapper: &mut OffsetPageTable,
) {
    use crate::memory::FRAME_ALLOCATOR;
    use x86_64::structures::paging::PageTableFlags as Flags;
    let flags = Flags::PRESENT | Flags::WRITABLE | Flags::USER_ACCESSIBLE;
    for (page, frame) in page_range.zip(frame_range) {
        let map_to_result =
            unsafe { mapper.map_to(page, frame, flags, FRAME_ALLOCATOR.lock().get_mut()) };
        map_to_result.expect("map_to failed:").flush()
    }
}

pub fn init_user_stack(mapper: &mut OffsetPageTable) -> VirtAddr {
    let (start_frame, end_frame, offset) = {
        let start = VirtAddr::new(unsafe { USER_STACK.as_ptr() as u64 });
        let end = start + unsafe { USER_STACK.len() };
        let start_phys = mapper.translate_addr(start).expect("User stack error.");
        let end_phys = mapper.translate_addr(end).expect("User stack error.");
        let start_frame = PhysFrame::containing_address(start_phys);
        let end_frame = PhysFrame::containing_address(end_phys);
        println!("stack: {:?} -> {:?}", start, start_phys);
        (
            start_frame,
            end_frame,
            start_phys - start_frame.start_address(),
        )
    };
    let (start_page, end_page) = {
        let start_virt = VirtAddr::new(0x3000_0000_0000);
        let end_virt = start_virt + unsafe { USER_STACK.len() };
        (
            Page::containing_address(start_virt),
            Page::containing_address(end_virt),
        )
    };
    let page_range = Page::<Size4KiB>::range_inclusive(start_page, end_page);
    let frame_range = PhysFrame::<Size4KiB>::range_inclusive(start_frame, end_frame);
    create_user_stack_map(page_range, frame_range, mapper);
    start_page.start_address() + offset + unsafe { USER_STACK.len() }
}

pub fn user_entry_point(mapper: &mut OffsetPageTable) -> VirtAddr {
    let virt = VirtAddr::new(user_space_func as u64);
    let phsy = mapper.translate_addr(virt).expect("No user entry point.");
    println!("entry: {:?} -> {:?}", virt, phsy);

    let page = Page::<Size4KiB>::containing_address(VirtAddr::new(0x3000_4000_0000));
    let frame = PhysFrame::<Size4KiB>::containing_address(phsy);

    let offset = phsy - frame.start_address();

    use crate::memory::FRAME_ALLOCATOR;
    use x86_64::structures::paging::PageTableFlags as Flags;
    let flags = Flags::PRESENT | Flags::WRITABLE | Flags::USER_ACCESSIBLE;
    let map_to_result =
        unsafe { mapper.map_to(page, frame, flags, FRAME_ALLOCATOR.lock().get_mut()) };
    map_to_result.expect("map_to failed.").flush();

    page.start_address() + offset
}

pub fn user_init(mapper: &mut OffsetPageTable) {
    let user_stack = init_user_stack(mapper);
    let user_entry_point = user_entry_point(mapper);
    let stack = mapper.translate_addr(user_stack);
    let entry = mapper.translate_addr(user_entry_point);
    crate::println!("{:?} -> {:?}", user_stack, stack);
    crate::println!("{:?} -> {:?}", user_entry_point, entry);
    jump_to_user_space(user_stack, user_entry_point);
}

pub fn jump_to_user_space(user_stack: VirtAddr, user_entry_point: VirtAddr) {
    use crate::gdt::{Selectors, GDT};
    use x86_64::instructions::segmentation::{load_ds, load_es};
    use x86_64::registers::rflags::RFlags;
    // let (user_stack, user_entry_point) = (user_stack.as_u64(), user_entry_point.as_u64());
    let (user_stack, user_entry_point) = (
        unsafe { USER_STACK.as_ptr() as u64 + USER_STACK.len() as u64 - 4 },
        user_space_func as u64,
    );
    // );
    // crate::println!("{:x} {:x}", user_stack, user_entry_point);
    // let flags = (RFlags::INTERRUPT_FLAG | RFlags::IOPL_LOW).bits();
    let (
        _,
        Selectors {
            user_code_seg,
            user_data_seg,
            ..
        },
    ) = *GDT;
    crate::println!("{:x} {:x}", user_code_seg.0, user_data_seg.0);
    unsafe {
        load_ds(user_data_seg);
        load_es(user_data_seg);
        asm!(
            "mov rdx, {}",
            "mov rsi, {}",
            "push 0x1b", // User data segment
            "push rdx", // User space stack
            "push 0x202", // RFlags
            "push 0x23", // User code segment
            "push rsi", // User entry point
            "iretq",
            in (reg) user_stack,
            in (reg) user_entry_point
        );
        /*
        // asm!("mov rsi, {}", in (reg) user_entry_point);
        asm!("push 0x1b"); // User data segment
        asm!("push rdx"); // User space stack
        asm!("push 0x202"); // RFlags
        asm!("push 0x23"); // User code segment
        asm!("push rsi"); // User entry point
        asm!("iretq")
        */
    }
}

pub fn user_space_func() {
    unsafe {
        asm!("nop", "nop", "nop");
    }
}
