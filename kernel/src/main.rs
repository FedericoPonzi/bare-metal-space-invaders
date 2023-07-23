#![no_std]
#![no_main]
#![allow(missing_docs)]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(asm_const)]
#![feature(return_position_impl_trait_in_trait)]

//extern crate alloc;

extern crate alloc;

use crate::logger::IrisLogger;
use core::panic::PanicInfo;
use cortex_a::asm;
use cortex_a::registers::SCTLR_EL1;

mod allocator;
mod boot;
mod framebuffer;
mod logger;
mod mailbox;
mod print;
mod time;
mod uart_pl011;

use crate::mailbox::{max_clock_speed, set_clock_speed};
use crate::mmio::PL011_UART_START;
use crate::uart_pl011::PL011Uart;
use log::{debug, error, info};
use tock_registers::interfaces::ReadWriteable;

static IRIS_LOGGER: IrisLogger = IrisLogger::new();
pub static PL011_UART: PL011Uart = unsafe { PL011Uart::new(PL011_UART_START) };

mod mmio {
    pub const IO_BASE: usize = 0x3F00_0000;
    pub const UART_OFFSET: usize = 0x0020_1000;
    pub const VIDEOCORE_MBOX_OFFSET: usize = 0x0000_B880;
    pub const TIME_OFFSET: usize = 0x0000_3000;
    pub const TIMER_REG_BASE: usize = IO_BASE + TIME_OFFSET;
    pub const PL011_UART_START: usize = IO_BASE + UART_OFFSET;
    pub const VIDEOCORE_MBOX_BASE: usize = IO_BASE + VIDEOCORE_MBOX_OFFSET;
}

#[inline]

unsafe fn kernel_init() -> ! {
    SCTLR_EL1.modify(SCTLR_EL1::C::Cacheable + SCTLR_EL1::I::Cacheable);
    unsafe {
        PL011_UART.init().unwrap();
    }
    println!("kernel_init");
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
    space_invaders::run_game(fb, &time::BcmGpuTimer::new());
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("PANIC!{}", info);
    loop {
        asm::wfe()
    }
}
