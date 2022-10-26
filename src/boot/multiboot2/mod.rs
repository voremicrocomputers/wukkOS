extern crate multiboot2;

use alloc::vec;
use alloc::vec::Vec;
use multiboot2::{BootInformation, load, MemoryAreaType};
use crate::boot::{KernelInfo, MemoryArea, MemoryType};
use crate::KernelArgs;

#[derive(Default)]
pub struct Multiboot2Bootloader {
    pub boot_info: Option<BootInformation>,
}

impl KernelInfo for Multiboot2Bootloader {
    fn init_from_kernel_args(&mut self, args: KernelArgs) {
        let boot_info = unsafe {
            load(args.multiboot_information_address)
        }.expect("invalid kernel args!");
        self.boot_info = Some(boot_info);
    }

    fn get_memory_areas(&self) -> Vec<MemoryArea> {
        let mut memory_areas = vec![];
        let boot_info = self.boot_info.as_ref().unwrap();
        let memory_map_tag = boot_info.memory_map_tag().expect("memory map tag required but not found!");
        for area in memory_map_tag.memory_areas() {
            memory_areas.push(MemoryArea {
                start: area.start_address() as usize,
                end: area.end_address() as usize,
                area_type: match area.typ() {
                    MemoryAreaType::Available => MemoryType::Available,
                    MemoryAreaType::Reserved => MemoryType::Reserved,
                    MemoryAreaType::AcpiAvailable => MemoryType::AcpiReclaimable,
                    MemoryAreaType::ReservedHibernate => MemoryType::Reserved,
                    MemoryAreaType::Defective => MemoryType::BadMemory,
                }
            })
        }
        memory_areas
    }
}