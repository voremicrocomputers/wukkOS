use alloc::sync::Arc;
use core::marker::PhantomData;
use acpi::{AcpiHandler, PhysicalMapping};
use crate::{debug, KernelArgs};

#[cfg(feature = "f_multiboot2")]
use multiboot2::{load, MemoryMapTag, BootInformation};
use x86_64::structures::paging::{FrameAllocator, OffsetPageTable};
use crate::memory::BootInfoFrameAllocator;

pub struct KernelInfo {
    kernel_start: u64,
    kernel_end: u64,
    safe_mem_start: u64,
    #[cfg(feature = "f_multiboot2")]
    boot_info: BootInformation,
}

impl KernelInfo {
    pub fn init_from_kernel_args(args: KernelArgs) -> Self {
        #[cfg(feature = "f_multiboot2")]
        {
            let boot_info = unsafe { load(args.multiboot_information_address) }.expect("failed to load multiboot2 information");
            let elf_sections = boot_info.elf_sections_tag().expect("no elf sections tag");
            let kernel_start = elf_sections.sections().map(|s| s.start_address()).min().unwrap();
            let kernel_end = elf_sections.sections().map(|s| s.end_address()).max().unwrap();
            // get end of multiboot for safe memory
            let safe_mem_start = boot_info.start_address() + boot_info.total_size();
            let kernel_info = KernelInfo {
                kernel_start,
                kernel_end,
                safe_mem_start: safe_mem_start as u64,
                boot_info,
            };
            kernel_info
        }
    }

    #[cfg(feature = "f_multiboot2")]
    pub fn get_memory_tag(&self) -> &MemoryMapTag {
        let mm_tag = self.boot_info.memory_map_tag().expect("no memory map tag").clone();
        mm_tag
    }

    #[cfg(feature = "f_multiboot2")]
    pub fn memory_areas(&self) -> impl Iterator<Item = &multiboot2::MemoryArea> {
        let mm_tag = self.boot_info.memory_map_tag().expect("ERR NO MEM MAP TAG!");
        mm_tag.all_memory_areas()
    }

    pub fn is_safe_memory(&self, addr: u64) -> bool {
        addr >= self.safe_mem_start && addr >= self.kernel_end
    }

    pub fn safe_memory_start(&self) -> u64 {
        self.safe_mem_start
    }

    pub fn acpi_get_ioapic_addr(&self, mem_mapper: &mut OffsetPageTable, frame_allocator: &mut BootInfoFrameAllocator) -> u64 {
        #[cfg(feature = "f_multiboot2")]
        {
            let acpi_tag = self.boot_info.rsdp_v1_tag().expect("no acpi tag");
            let rsdp = acpi_tag;
            let rsdp = unsafe { &*rsdp };
            let rsdt = rsdp.rsdt_address();
            #[derive(Clone)]
            struct Handler<'a> {
                mem_mapper: Arc<&'a mut OffsetPageTable<'a>>,
                frame_allocator: Arc<&'a mut BootInfoFrameAllocator>,
            }
            impl<'a> AcpiHandler for Handler<'a> {
                unsafe fn map_physical_region<T>(&self, physical_address: usize, size: usize) -> PhysicalMapping<Self, T> {
                    let frame_allocator = self.frame_allocator.clone();
                    debug!("read_phys_memory32: addr {:x} not mapped", physical_address);
                    // map the page
                    let frame = frame_allocator.allocate_frame().unwrap();
                    debug!("allocated frame: {:?}", frame);
                    let flags = x86_64::structures::paging::PageTableFlags::PRESENT | x86_64::structures::paging::PageTableFlags::WRITABLE;
                    let page = x86_64::structures::paging::Page::containing_address(initaladdr);
                    debug!("mapped page: {:?}", page);
                    let map_to_result = unsafe { mem_mapper.map_to(page, frame, flags, frame_allocator) };
                    debug!("map_to_result: {:?}", map_to_result);
                    if map_to_result.is_err() {
                        panic!("Failed to map page");
                    }
                    let addr = unsafe { mem_mapper.translate_addr(initaladdr) };
                    if let Some(addr) = addr {
                        let addr = addr.as_u64() as *const u32;
                        unsafe { *addr }
                    } else {
                        panic!("Failed to map page");
                    }
                }

                fn unmap_physical_region<T>(region: &PhysicalMapping<Self, T>) {
                    todo!()
                }
            }
            let rsdt = acpi::AcpiTables::from_rsdt(rsdt).expect("failed to get acpi tables");
            let mut ioapic_addr = 0;
            for entry in rsdt.entries() {
                let entry = unsafe { &*entry };
                if entry.signature() == *b"APIC" {
                    let apic = entry.as_apic();
                    let apic = unsafe { &*apic };
                    for entry in apic.entries() {
                        let entry = unsafe { &*entry };
                        if entry.signature() == *b"IOAP" {
                            let ioapic = entry.as_ioapic();
                            let ioapic = unsafe { &*ioapic };
                            ioapic_addr = ioapic.address();
                        }
                    }
                }
            }
            ioapic_addr
        }
    }
}