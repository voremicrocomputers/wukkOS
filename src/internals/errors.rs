
use core::borrow::{BorrowMut};
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::PageFaultErrorCode;


use crate::{InterruptStackFrame, font, println, print};
use crate::internals::WhyDoTheyCallItOvenWhenYouOfInTheColdFoodOfOutHotEatTheFood::{COMMUNIST_RED, CUM_WHITE, Colour};
use crate::serial::terminal::ST;

pub extern "x86-interrupt" fn breakpoint_exception(stack_frame: InterruptStackFrame) {
    println!("---KERNEL WARNING UWU---");
    println!("breakpoint exception");
    println!("stack frame: {:#?}", stack_frame);
}

pub extern "x86-interrupt" fn double_fault(stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    println!("---KERNEL FUCKY WUKKY UWU---");
    println!("double fault!");
    println!("stack frame: {:#?}", stack_frame);
    loop {}
}

pub extern "x86-interrupt" fn page_fault(stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode) -> ! {
    println!("---KERNEL FUCKY WUKKY UWU---");
    println!("page fault!");
    println!("accessed address: {:?}", Cr2::read());
    println!("error code: {:?}", error_code);
    println!("stack frame: {:#?}", stack_frame);
    loop {}
}