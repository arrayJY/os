use lazy_static::lazy_static;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::{
    structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
    VirtAddr,
};

pub struct Selectors {
    pub kernel_code_seg: SegmentSelector,
    pub task_state_seg: SegmentSelector,
    /* useless now
    pub kernel_data_seg: SegmentSelector,
    pub user_code_seg: SegmentSelector,
    pub user_data_seg: SegmentSelector,
    */
}

#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub enum ISTIndex {
    DoubleFault = 0,
}

impl ISTIndex {
    pub fn as_u16(self) -> u16{
        self as u16
    }
    pub fn as_usize(self) -> usize {
        self as usize
    }
}
lazy_static! {
    pub static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let kernel_code_seg = gdt.add_entry(Descriptor::kernel_code_segment());
        let task_state_seg = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (
            gdt,
            Selectors {
                kernel_code_seg,
                task_state_seg,
            },
        )
    };
    pub static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[ISTIndex::DoubleFault as usize] = {
            const STACK_SIZE: usize = 1024 * 16;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_bottom = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_top = stack_bottom + STACK_SIZE;
            stack_top
        };
        tss
    };
}

pub fn init() {
    use x86_64::instructions::segmentation::set_cs;
    use x86_64::instructions::tables::load_tss;
    let (
        ref gdt,
        Selectors {
            kernel_code_seg,
            task_state_seg,
        },
    ) = *GDT;
    gdt.load();
    unsafe {
        set_cs(kernel_code_seg);
        load_tss(task_state_seg);
    }
}
