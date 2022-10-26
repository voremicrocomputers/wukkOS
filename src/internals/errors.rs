
use core::borrow::{BorrowMut};


use crate::{InterruptStackFrame, font};
use crate::internals::WhyDoTheyCallItOvenWhenYouOfInTheColdFoodOfOutHotEatTheFood::{COMMUNIST_RED, CUM_WHITE, Colour};

pub extern "x86-interrupt" fn breakpoint_exception(stack_frame: InterruptStackFrame) {
    /*
    // cover the screen in a nice communist red (:
    let mut fb = FACEBOOK.fb_mutex.lock();
    let fb_width = FACEBOOK.fb_width.lock();
    let fb_height = FACEBOOK.fb_height.lock();

    draw_box(0,0,*fb_width,*fb_height, COMMUNIST_RED, fb.borrow_mut());
    // draw our funny text
    draw_horizcentre_string(*fb_width,(*fb_height / 2) - (14 * (8/2)), "OOPSY WOOPSY, THE KERNEL HAD A FUCKY WUCKY UWU", CUM_WHITE, fb.borrow_mut());
    draw_horizcentre_string(*fb_width,(*fb_height / 2) - (10 * (8/2)), "WHOEVER WAS PROGRAMMING THE KERNEL DECIDED TO LEAVE A BREAKPOINT IN IT, OOPS (:", CUM_WHITE, fb.borrow_mut());
    draw_horizcentre_string(*fb_width,(*fb_height / 2) - (4 * (8/2)), "THE KERNEL IS NOW HALTED, YOU CAN'T DO ANYTHING UNTIL YOU RESTART THE KERNEL", CUM_WHITE, fb.borrow_mut());

    drop(fb_width);
    drop(fb_height);
    drop(fb);
     */
}