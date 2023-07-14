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
use cortex_a::registers::SCTLR_EL1;

//mod allocator;
mod boot;
mod gpio;
mod mailbox;
mod print;
//mod uart;
mod allocator;
mod framebuffer;
mod logger;
mod time;
mod uart_pl011;

//mod uart;
use crate::mailbox::{max_clock_speed, set_clock_speed};
use log::{debug, error, info, warn};
use tock_registers::interfaces::ReadWriteable;

static IRIS_LOGGER: IrisLogger = IrisLogger::new();

pub const UART_OFFSET: usize = 0x0020_1000;
pub const START: usize = 0x3F00_0000;
pub const PL011_UART_START: usize = START + UART_OFFSET;

#[inline]

unsafe fn kernel_init() -> ! {
    println!("kernel_init");
    SCTLR_EL1.modify(SCTLR_EL1::C::Cacheable + SCTLR_EL1::I::Cacheable);

    IRIS_LOGGER.init().unwrap();
    allocator::ALLOCATOR.initialize();
    let max_clock_speed = max_clock_speed();
    info!("Kernel speed: {:?}", max_clock_speed);
    set_clock_speed(max_clock_speed.unwrap());

    //panic!();
    info!("kernel_init");
    extern "C" {
        static mut __text_end: usize;
    }
    let heap_start = __text_end;
    let heap_end = 1 << 29; // 1 GB by default;
    info!("heap_end: {}", heap_end);
    let heap_size = heap_end - heap_start;

    info!("Allocator initiated");

    main();
    panic!()
}

fn main() {
    info!("main");
    let fb = mailbox::lfb_init(0).unwrap();
    println!("Starting game...");
    space_invaders::run_game(fb, time::BcmGpuTimer::new());
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("PANIC!{}", info);
    loop {
        asm::wfe()
    }
}
