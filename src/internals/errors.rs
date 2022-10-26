
use core::borrow::{BorrowMut};


use crate::{InterruptStackFrame, font, println};
use crate::internals::WhyDoTheyCallItOvenWhenYouOfInTheColdFoodOfOutHotEatTheFood::{COMMUNIST_RED, CUM_WHITE, Colour};
use crate::serial::terminal::ST;

pub extern "x86-interrupt" fn breakpoint_exception(stack_frame: InterruptStackFrame) {
    println!("---KERNEL WARNING UWU---");
    println!("breakpoint exception");
    println!("stack frame: {:#?}", stack_frame);
    loop {}
}