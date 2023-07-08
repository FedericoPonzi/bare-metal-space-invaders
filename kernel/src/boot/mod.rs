use core::arch::global_asm;
// Assembly counterpart to this file.
global_asm!(include_str!("boot.s"));

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// The Rust entry of the `kernel` binary.
///
/// The function is called from the assembly `_start` function.
#[no_mangle]
pub unsafe fn _start_rust() -> ! {
    crate::kernel_init()
}
