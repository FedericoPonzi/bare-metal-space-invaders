use crate::kernel_init;
use core::arch::global_asm;
use cortex_a::asm;
use cortex_a::registers::{CNTVOFF_EL2, ELR_EL2, HCR_EL2, SPSR_EL2, SP_EL1};
use log::info;
use tock_registers::interfaces::Writeable;

#[no_mangle]
#[link_section = ".text._start_arguments"]
pub static BOOT_CORE_ID: u64 = 0;

global_asm!(
    include_str!("boot.s"),
    CONST_CORE_ID_MASK = const 0b11
);

/// The Rust entry of the `kernel` binary, switches to el1
#[no_mangle]
pub unsafe extern "C" fn _start_rust() -> ! {
    // No offset for reading the counters.
    CNTVOFF_EL2.set(0);

    // Set EL1 execution state to AArch64.
    HCR_EL2.write(HCR_EL2::RW::EL1IsAarch64);

    // Set up a simulated exception return.
    //
    // First, fake a saved program status where all interrupts were masked and SP_EL1 was used as a
    // stack pointer.
    SPSR_EL2.write(
        SPSR_EL2::D::Masked
            + SPSR_EL2::A::Masked
            + SPSR_EL2::I::Masked
            + SPSR_EL2::F::Masked
            + SPSR_EL2::M::EL1h,
    );

    // Second, let the link register point to runtime_init().
    ELR_EL2.set(kernel_init as *const () as u64);

    // Set up SP_EL1 (stack pointer), which will be used by EL1 once we "return" to it.
    SP_EL1.set(0x80_000 as u64);
    info!("Ereturning..");
    asm::eret();
}
