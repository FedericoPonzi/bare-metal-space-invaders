pub const IO_BASE: usize = 0x3F00_0000;
pub const UART_OFFSET: usize = 0x0020_1000;
const GPIO_OFFSET: usize = 0x200000;

const GPFSEL0: usize = IO_BASE + 0x200000;
const GPSET0: usize = IO_BASE + 0x20001C;
const GPCLR0: usize = IO_BASE + 0x200028;
const GPPUPPDN0: usize = IO_BASE + 0x2000E4;


static UART_ADDRESS: usize = IO_BASE + UART_OFFSET;

struct MiniUart {
    irq:
}

const fn uart_init() -> MiniUart {}