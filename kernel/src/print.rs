use crate::uart_pl011::PL011Uart;
use crate::PL011_UART_START;
use core::fmt;

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    let uart = unsafe { PL011Uart::new(PL011_UART_START) };
    unsafe {
        uart.init().unwrap();
    }
    uart.write_fmt(args).unwrap();
}

/// Prints without a newline.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::print::_print(format_args!($($arg)*)));
}

/// Prints with a newline.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ({
        $crate::print::_print(format_args_nl!($($arg)*));
    })
}
