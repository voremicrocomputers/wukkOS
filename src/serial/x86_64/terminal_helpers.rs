pub fn clear_screen() -> &'static str {
    //"\033[2J"
    "\x1B[2J"
}

pub fn init_cursor() -> &'static str {
    //"\033[?25l"
    "\x1B[H"
}