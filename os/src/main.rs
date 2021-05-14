#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(global_asm)]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use os::{allocator::heap_init, exec::user_init, memory, system_call};
extern crate alloc;
#[allow(unused_imports)]
use os::println;
use x86_64::{VirtAddr};
global_asm!(include_str!("link_app.S"));

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    os::init();
    println!("[kernel] Kernel initialized.");
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { os::memory::init(phys_mem_offset) };
    memory::init_frame_allocator(&boot_info.memory_map);
    println!("[kernel] Frame allocator initialized.");
    heap_init(&mut mapper).expect("Initialize heap failed.");
    println!("[kernel] Heap initialized.");
    memory::init_kernel_stack(&mut mapper);
    println!("[kernel] Kernel stack initialized.");
    system_call::trap_init();

    println!("----------");
    println!("[user programs]");
    user_init();

    #[cfg(test)]
    test_main();

    os::hlt_loop();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    os::test_panic_handler(_info)
}
