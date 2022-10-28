use lazy_static::lazy_static;
use pc_keyboard::{Keyboard, layouts, ScancodeSet1, HandleControl};
use pc_keyboard::DecodedKey::Unicode;
use spin::Mutex;
use crate::print;

lazy_static!{
    static ref KBD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(Keyboard::new(HandleControl::MapLettersToUnicode));
}

pub fn handle_scancode(scancode: u8) {
    let mut kbd = KBD.lock();
    if let Ok(Some(key_event)) = kbd.add_byte(scancode) {
        if let Some(key) = kbd.process_keyevent(key_event) {
            if let Unicode(c) = key {
                print!("{}", c);
            }
        }
    }
}