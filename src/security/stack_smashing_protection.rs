pub const STACK_CHK_GUARD: usize = usize::MAX;

pub const STACK_CHK_PTR: *const usize = &STACK_CHK_GUARD;

pub extern "C" fn __stack_chk_fail() -> ! {
    panic!("stack fucking or uh smashing detected!");
    loop {}
}