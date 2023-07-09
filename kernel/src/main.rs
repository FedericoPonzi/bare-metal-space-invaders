#![no_std]
#![no_main]
#![allow(missing_docs)]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(asm_const)]

extern crate alloc;

use crate::logger::IrisLogger;
use core::panic::PanicInfo;
use cortex_a::asm;

//mod allocator;
mod boot;
mod gpio;
mod mailbox;
mod print;
//mod uart;
mod logger;
mod uart_pl011;
//mod uart;
use log::{debug, error, info, warn};

static IRIS_LOGGER: IrisLogger = IrisLogger::new();

pub const UART_OFFSET: usize = 0x0020_1000;
pub const START: usize = 0x3F00_0000;
pub const PL011_UART_START: usize = START + UART_OFFSET;

use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[inline]
unsafe fn kernel_init() -> ! {
    IRIS_LOGGER.init().unwrap();
    extern "C" {
        static mut __text_end: usize;
    }
    let heap_start = __text_end;
    let heap_end = 1 << 30; // 1 GB by default;
    let heap_size = heap_end - heap_start;
    unsafe {
        ALLOCATOR.lock().init(heap_start as *mut u8, heap_size);
    }

    main();
    panic!()
}

fn main() {
    let fb = mailbox::lfb_init(0).unwrap();
    println!("Starting game...");
    space_invaders::run_game(fb);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("PANIC!{}", info);
    loop {
        asm::wfe()
    }
}
