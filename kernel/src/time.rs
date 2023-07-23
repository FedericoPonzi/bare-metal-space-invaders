use crate::mmio::TIMER_REG_BASE;
use core::time::Duration;

#[repr(C)]
//TODO: Wrap this regiseters in register ds.
// https://tc.gts3.org/cs3210/2020/spring/r/BCM2837-ARM-Peripherals.pdf cap 12 - pag 172
struct ArmTimeRegisters {
    /// CS register
    /// This register is used to record and clear timer channel comparator matches.
    /// The system timer match bits are routed to the interrupt controller where they can generate an interrupt.
    /// The M0-3 fields contain the free-running counter match status.
    /// Write a one to the relevant bit to clear the match detect status bit and the corresponding interrupt request line.
    controller_status: u32,
    /// CLO: counter lower 32 bits
    counter_lower: u32,
    /// CHI: System Timer Counter Higher 32 bits
    counter_higher: u32,
    /// system Timer compare registers - 4 in total
    compare: [u32; 4],
}

/// BCM's system timer.
/// Note the GPU uses timers 0 and 2, so they're reserved.
pub struct BcmGpuTimer;
impl BcmGpuTimer {
    #[inline(always)]
    pub const fn new() -> Self {
        Self
    }
}
impl space_invaders::TimeManagerInterface for BcmGpuTimer {
    #[inline(always)]
    fn now(&self) -> Duration {
        let ptr = TIMER_REG_BASE as *mut ArmTimeRegisters;
        let registers = unsafe { &mut *ptr };
        let lower = unsafe { core::ptr::read_volatile(&registers.counter_lower) } as u64;
        let upper = unsafe { core::ptr::read_volatile(&registers.counter_higher) } as u64;
        let microseconds = (upper << 32) | lower;
        Duration::from_micros(microseconds)
    }
}

pub static TIME_MANAGER: BcmGpuTimer = BcmGpuTimer::new();
