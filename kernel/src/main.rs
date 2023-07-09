#![no_std]
#![no_main]
#![feature(global_asm)]
#![allow(missing_docs)]
use core::panic::PanicInfo;
use cortex_a::asm;

mod boot;
mod mailbox;
mod print;
mod uart;

unsafe fn kernel_init() -> ! {
    main();
    panic!()
}
fn main() {
    space_invaders::run_game();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        asm::wfe()
    }
}
