use alloc::vec::Vec;
use crate::KernelArgs;

#[cfg(feature = "f_multiboot2")]
pub mod multiboot2;

pub enum MemoryType {
    Available,
    Reserved,
    AcpiReclaimable,
    Nvs,
    BadMemory,
    Kernel,
    Bootloader,
    Unknown(u32)
}

pub struct MemoryArea {
    pub start: usize,
    pub end: usize,
    pub area_type: MemoryType,
}

pub trait KernelInfo {
    fn init_from_kernel_args(&mut self, args: KernelArgs);
    fn get_memory_areas(&self) -> Vec<MemoryArea>;
}