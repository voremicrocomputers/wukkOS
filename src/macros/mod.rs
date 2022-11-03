use core::fmt;
use core::fmt::Write;
#[cfg(target_arch = "x86_64")]
use crate::boot::x86_64::LimineWriter;
use crate::internals::cpu::ppc32::PROMHNDL;
#[cfg(target_arch = "x86_64")]
use crate::serial::x86_64::terminal::ST;

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
    #[cfg(target_arch = "powerpc")]
    {
        use core::fmt::Write;
        use crate::internals::cpu::ppc32::PROMHNDL;
        struct OFPrinter;
        impl Write for OFPrinter {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                unsafe { PROMHNDL.lock().get().write_line(s) };
                Ok(())
            }
        }
        OFPrinter.write_fmt(args).unwrap();
    }

    #[cfg(target_arch = "apic")]
    {
        use core::fmt::Write;
        ST.writer.lock().write_fmt(args).unwrap();

        let mut limine_writer = LimineWriter;
        limine_writer.write_fmt(args).unwrap();
    }
}

#[doc(hidden)]
pub fn _debug(args: fmt::Arguments) {
    #[cfg(target_arch = "powerpc")]
    {
        use core::fmt::Write;
        use crate::internals::cpu::ppc32::PROMHNDL;
        struct OFPrinter;
        impl Write for OFPrinter {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                unsafe { PROMHNDL.lock().get().write_line(s) };
                Ok(())
            }
        }
        OFPrinter.write_str("[debug] ").unwrap();
        OFPrinter.write_fmt(args).unwrap();
        OFPrinter.write_str("\n").unwrap();
    }

    #[cfg(target_arch = "apic")]
    {
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
}