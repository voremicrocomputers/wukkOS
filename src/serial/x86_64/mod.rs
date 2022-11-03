

use core::arch::asm;
use core::borrow::{Borrow, BorrowMut};
use core::ops::Deref;

pub mod terminal_helpers;
pub mod terminal;
pub mod simplifiers;

#[derive(Clone, Copy, PartialEq)]
pub enum potential_serial_ports {
    COM1 = 0x3F8,
    COM2 = 0x2F8,
    COM3 = 0x3E8,
    COM4 = 0x2E8,
    COM5 = 0x5F8,
    COM6 = 0x4F8,
    COM7 = 0x5E8,
    COM8 = 0x4E8,
}

impl potential_serial_ports {
    pub fn to_string<'a>(&self) -> &'a str {
        match self {
            potential_serial_ports::COM1 => "COM1",
            potential_serial_ports::COM2 => "COM2",
            potential_serial_ports::COM3 => "COM3",
            potential_serial_ports::COM4 => "COM4",
            potential_serial_ports::COM5 => "COM5",
            potential_serial_ports::COM6 => "COM6",
            potential_serial_ports::COM7 => "COM7",
            potential_serial_ports::COM8 => "COM8",
            _ => {
                panic!("Invalid potential_serial_ports");
            }
        }
    }
}

enum serial_offsets {
    DATA = 0,
    INTERRUPT_ID = 1,
    FIFO_CTRL = 2,
    LINE_CTRL = 3,
    MODEM_CTRL = 4,
    LINE_STATUS = 5,
    MODEM_STATUS = 6,
    SCRATCH = 7,
}

#[derive(Copy, Clone)]
pub struct Port {
    pub base: potential_serial_ports,
}

pub struct SerialPorts {
    pub ports_enabled: [bool; 8],
    pub ports: [Port; 8],
}

#[cfg(any(target_arch="x86", target_arch="apic"))]
pub fn command(port: u16, data: u8) {
    unsafe {
        asm!("out dx, al", in("al") data, in("dx") port);
    }
}

#[cfg(any(target_arch="x86", target_arch="apic"))]
pub fn read(port: u16) -> u8 {
    let mut data: u8;
    unsafe {
        asm!("in al, dx", out("al") data, in("dx") port);
    }
    data
}

// dummy functions for non-x86
#[cfg(not(any(target_arch="x86", target_arch="apic")))]
pub fn command(port: u16, data: u8) {
    unimplemented!()
}

#[cfg(not(any(target_arch="x86", target_arch="apic")))]
pub fn read(port: u16) -> u8 {
    unimplemented!()
}

impl Port {
    fn is_transmit_empty(&self) -> bool {
        let status = read(self.base as u16 + serial_offsets::LINE_STATUS as u16);
        status & 0x20 == 0x20
    }

    pub fn transmit(&self, data: u8) {
        while !self.is_transmit_empty() {}
        command(self.base as u16 + serial_offsets::DATA as u16, data);
    }

    pub fn transmit_string(&self, data: &str) {
        for c in data.chars() {
            self.transmit(c as u8);
        }
    }

    fn is_recv_full(&self) -> bool {
        let status = read(self.base as u16 + serial_offsets::LINE_STATUS as u16);
        status & 0x01 == 0x01
    }

    pub fn receive(&self, mut timeout: u16) -> u8 {
        if timeout != 0 {
            while !self.is_recv_full() {
                timeout -= 1;
                if timeout == 0 {
                    return 0;
                }
            }
        } else {
            while !self.is_recv_full() {}
        }
        read(self.base as u16 + serial_offsets::DATA as u16)
    }

    pub fn transrecv(&self, data: u8) -> u8 {
        self.transmit(data);
        self.receive(0)
    }
}

pub fn test_port(port: potential_serial_ports) -> bool {
    let mut port: u16 = port as u16;
    command(port + serial_offsets::INTERRUPT_ID as u16, 0x00); // disable interrupts
    command(port + serial_offsets::LINE_CTRL as u16, 0x80); // enable DLAB
    command(port + serial_offsets::DATA as u16, 0x03); // set divisor to 3 (lo byte)
    command(port + serial_offsets::LINE_CTRL as u16, 0x03); // set divisor to 3 (hi byte)
    command(port + serial_offsets::FIFO_CTRL as u16, 0xC7); // enable FIFO, clear them, with 14-byte threshold
    command(port + serial_offsets::MODEM_CTRL as u16, 0x0B); // IRQs enabled, RTS/DSR set
    command(port + serial_offsets::MODEM_CTRL as u16, 0x1E); // loopback mode

    // test serial
    command(port + serial_offsets::DATA as u16, 0xAE);
    // check if we received the correct byte
    if read(port + serial_offsets::DATA as u16) != 0xAE {
        return false;
    } else {
        // set stuffz idk
        command(port + serial_offsets::MODEM_CTRL as u16, 0x0F);
        return true;
    }
}

pub fn init_serial() -> SerialPorts {
    // this is so fucking cursed
    let mut ports_tmp : [Port; 8] = [
        Port { base: potential_serial_ports::COM1 },
        Port { base: potential_serial_ports::COM2 },
        Port { base: potential_serial_ports::COM3 },
        Port { base: potential_serial_ports::COM4 },
        Port { base: potential_serial_ports::COM5 },
        Port { base: potential_serial_ports::COM6 },
        Port { base: potential_serial_ports::COM7 },
        Port { base: potential_serial_ports::COM8 }];
    let mut ports_enabled_tmp : [bool; 8] = [false; 8];
    for i in 0..8 {
        if test_port(ports_tmp[i].base) {
            ports_enabled_tmp[i] = true;
        }
    }
    SerialPorts {
        ports_enabled: ports_enabled_tmp,
        ports: ports_tmp,
    }
}