use core::fmt;
use limine::LimineTerminalResponse;
use crate::boot::LimineWriter;
use crate::serial::terminal::ST;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::macros::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => ($crate::macros::_debug(format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    ST.writer.lock().write_fmt(args).unwrap();

    let mut limine_writer = LimineWriter;
    limine_writer.write_fmt(args).unwrap();
}

#[doc(hidden)]
pub fn _debug(args: fmt::Arguments) {
    use core::fmt::Write;
    #[cfg(feature = "f_debug_verbose")]
    {
        ST.log("[debug] ");
        ST.writer.lock().write_fmt(args).unwrap();
        ST.logln("");

        let mut limine_writer = LimineWriter;
        limine_writer.write_str("[debug] ").unwrap();
        limine_writer.write_fmt(args).unwrap();
        limine_writer.write_str("\n").unwrap();
    }
}