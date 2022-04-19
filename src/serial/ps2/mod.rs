use super::*;
use crate::internals::WhyDoTheyCallItOvenWhenYouOfInTheColdFoodOfOutHotEatTheFood::{KernelError, ErrorKind, ErrorLevel};

pub enum PS2Type {
    AncientKeyboard,
    Keyboard,
    Mouse,
    FiveButtonMouse,
    ScrollWheelMouse,
    Unknown,
}

enum Messages {
    MSGDisableScanning = 0xF5,
    MSGIdentify = 0xF2,
    MSGACK = 0xFA,
}

pub fn probePort(port : Port) -> Result<PS2Type, KernelError> {
    let mut response : u8;
    response = port.transrecv(Messages::MSGDisableScanning as u8);
    // check for ACK
    if response != Messages::MSGACK as u8 {
        return Err(KernelError::new(ErrorKind::HardwareFuckUp, ErrorLevel::Warning, "Did not receive ACK", port.base.to_string()));
    }
    response = port.transrecv(Messages::MSGIdentify as u8);
    // check for ACK
    if response != Messages::MSGACK as u8 {
        return Err(KernelError::new(ErrorKind::HardwareFuckUp, ErrorLevel::MinorFuckUp, "Did not receive ACK after asking port to identify", port.base.to_string()));
    }
    // read the response with a timeout of x cpu cycles, this can be quite low as it will only matter on slow machines
    response = port.receive(0xFFFF);
    // if there's no response, it's a really old keyboard (IBM or whoever, why did you have to do this??)\
    if response == 0 {
        return Ok(PS2Type::AncientKeyboard);
    } else {
        // switch on the response
        match response {
            0x00 => { // standard mouse
                return Ok(PS2Type::Mouse);
            },
            0x03 => { // mouse with scroll wheel
                return Ok(PS2Type::ScrollWheelMouse);
            },
            0x04 => { // 5 button mouse
                return Ok(PS2Type::FiveButtonMouse);
            },
            0xAB => { // some type of keyboard, we'll need to read another byte to be sure
                response = port.receive(0xFFFF);
                match response {
                    0x41 => { // some keyboard idfk
                        return Ok(PS2Type::Keyboard);
                    },
                    0xC1 => { // same as above
                        return Ok(PS2Type::Keyboard);
                    },
                    0x83 => { // keyboard after taking normal pills
                        return Ok(PS2Type::Keyboard);
                    },
                    _ => { // what?
                        return Ok(PS2Type::Unknown);
                    }
                }
            },
            _ => { // some other device idk
                return Ok(PS2Type::Unknown);
            }
        }
    }
}