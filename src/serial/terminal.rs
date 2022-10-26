use alloc::sync::Arc;
use core::ops::Deref;
use lazy_static::lazy_static;
use spin::Mutex;
use crate::serial::Port;

pub struct SerialTerminal {
    pub port: Arc<Mutex<Option<Port>>>,
}

lazy_static! {
    pub static ref ST: SerialTerminal = {
        let mut serial_terminal = SerialTerminal {
            port: Arc::new(Mutex::new(None)),
        };
        serial_terminal
    };
}

impl SerialTerminal {
    pub fn init_from_port(&self, port: Port) {
        self.port.lock().replace(port);
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