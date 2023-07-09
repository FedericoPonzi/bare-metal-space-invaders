use tock_registers::register_bitfields;
use tock_registers::registers::ReadWrite;

pub const MMIO_BASE: u32 = 0x3F00_0000;

// Descriptions taken from
// https://github.com/raspberrypi/documentation/files/1888662/BCM2837-ARM-Peripherals.-.Revised.-.V2-1.pdf
register_bitfields! {
    u32,

    /// GPIO Function Select 1
    pub GPFSEL1 [
        /// Pin 15
        FSEL15 OFFSET(15) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            RXD1 = 0b010  // Mini UART - Alternate function 5

        ],

        /// Pin 14
        FSEL14 OFFSET(12) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            TXD1 = 0b010  // Mini UART - Alternate function 5
        ]
    ],

    /// GPIO Pull-up/down Clock Register 0
    pub GPPUDCLK0 [
        /// Pin 15
        PUDCLK15 OFFSET(15) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 14
        PUDCLK14 OFFSET(14) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ]
    ]
}

pub const GPFSEL1: *const ReadWrite<u32, GPFSEL1::Register> =
    (MMIO_BASE + 0x0020_0004) as *const ReadWrite<u32, GPFSEL1::Register>;

pub const GPPUD: *const ReadWrite<u32> = (MMIO_BASE + 0x0020_0094) as *const ReadWrite<u32>;

pub const GPPUDCLK0: *const ReadWrite<u32, GPPUDCLK0::Register> =
    (MMIO_BASE + 0x0020_0098) as *const ReadWrite<u32, GPPUDCLK0::Register>;
