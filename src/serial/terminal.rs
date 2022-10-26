use core::fmt;
use core::ops::Deref;
use lazy_static::lazy_static;
use spin::Mutex;
use crate::serial::Port;

pub struct SerialTerminal {
    pub port: Mutex<Option<Port>>,
    pub writer: Mutex<SerialTerminalWriter>,
}

pub struct SerialTerminalWriter {
    pub port: Mutex<Option<Port>>,
}

lazy_static! {
    pub static ref ST: SerialTerminal = {
        let serial_terminal: SerialTerminal = SerialTerminal {
            port: Mutex::new(None),
            writer: Mutex::new(SerialTerminalWriter {
                port: Mutex::new(None),
            }),
        };
        serial_terminal
    };
}

impl SerialTerminal {
    pub fn init_from_port(&self, port: Port) {
        self.port.lock().replace(port);
        self.writer.lock().port.lock().replace(port);
    }

    pub fn log(&self, message: &str) {
        if let Some(port) = self.port.lock().deref() {
            port.transmit_string(message);
        }
    }

    pub fn logln(&self, message: &str) {
        if let Some(port) = self.port.lock().deref() {
            port.transmit_string(message);
            port.transmit_string("\r\n");
        }
    }
}

impl fmt::Write for SerialTerminalWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if let Some(port) = self.port.lock().deref() {
            port.transmit_string(s);
        }
        Ok(())
    }
}