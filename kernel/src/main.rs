#![no_std]
#![no_main]
#![feature(global_asm)]
#![allow(missing_docs)]
use core::panic::PanicInfo;
use cortex_a::asm;

mod boot;
mod framebuffer;
mod mailbox;
mod print;
mod uart;

unsafe fn kernel_init() -> ! {
    init_fb();
    main();
    panic!()
}
fn main() {
    let b = console::read_bytes();
    if b.to_char() == "N" {
        play()
    }
}
fn play() {
    loop {
        clean_screen();
        draw_aliens();
        draw_spaceship();
        draw_shots();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        asm::wfe()
    }
}
