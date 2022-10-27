use alloc::sync::Arc;
use core::marker::PhantomData;
use core::ptr::NonNull;
use acpi::{AcpiHandler, InterruptModel, PhysicalMapping};
use crate::{debug, KernelArgs, println};

#[cfg(feature = "f_multiboot2")]
use multiboot2::{load, MemoryMapTag, BootInformation};
use x86_64::structures::paging::{FrameAllocator, Mapper, OffsetPageTable, Page, Translate};
use x86_64::VirtAddr;
use crate::memory::{BootInfoFrameAllocator, FRAME_ALLOC, MEM_MAPPER, PageSize};

pub struct KernelInfo {
    kernel_start: u64,
    kernel_end: u64,
    safe_mem_start: u64,
    #[cfg(feature = "f_multiboot2")]
    boot_info: BootInformation,
}

#[derive(Clone)]
struct Handler;
impl AcpiHandler for Handler {
    unsafe fn map_physical_region<T>(&self, physical_address: usize, size: usize) -> PhysicalMapping<Self, T> {
        // todo! check if size is too big
        debug!("read_phys_memory32: addr {:x} not mapped", physical_address);
        // map the page
        let frame = FRAME_ALLOC.lock().as_mut().unwrap().allocate_frame().unwrap();
        debug!("allocated frame: {:?}", frame);
        let flags = x86_64::structures::paging::PageTableFlags::PRESENT | x86_64::structures::paging::PageTableFlags::WRITABLE;
        let page: Page<PageSize> = Page::containing_address(VirtAddr::new(physical_address as u64));
        debug!("mapped page: {:?}", page);
        let map_to_result = unsafe { MEM_MAPPER.lock().as_mut().unwrap().map_to(page, frame, flags, FRAME_ALLOC.lock().as_mut().unwrap()) };
        debug!("map_to_result: {:?}", map_to_result);
        if map_to_result.is_err() {
            panic!("Failed to map page");
        }
        let addr = unsafe { MEM_MAPPER.lock().as_mut().unwrap().translate_addr(VirtAddr::new(physical_address as u64)) };
        if let Some(addr) = addr {
            // physical start, virtual start, region length, mapped length, Self
            PhysicalMapping::new(
                physical_address,
                NonNull::new_unchecked(addr.as_u64() as *mut T),
                size, size,
                Self)
        } else {
            panic!("Failed to map page");
        }
    }

    fn unmap_physical_region<T>(region: &PhysicalMapping<Self, T>) {
        // get page
        let page: Page<PageSize> = Page::containing_address(VirtAddr::new(region.physical_start() as u64));
        // unmap page
        let res = unsafe { MEM_MAPPER.lock().as_mut().unwrap().unmap(page) };
        // it isn't *that* important if we don't unmap successfully at the moment, so just write a warning if we fail
        if res.is_err() {
            println!("[WARN] failed to unmap page");
        }
    }
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

    pub fn acpi_get_ioapic_addr(&self) -> u32 {
        #[cfg(feature = "f_multiboot2")]
        {
            let acpi_tag = self.boot_info.rsdp_v1_tag().expect("no acpi tag");
            let rsdp = acpi_tag;
            let rsdp = unsafe { &*rsdp };
            let rsdt = rsdp.rsdt_address();
            let rsdt = unsafe {
                acpi::AcpiTables::from_rsdt(
                    Handler, 0,
                    rsdt)
                    .expect("failed to get acpi tables")
            };
            let platform_info = rsdt.platform_info().expect("failed to get platform info");
            let interrupt_model = platform_info.interrupt_model;
            if let InterruptModel::Apic(apic) = interrupt_model {
                let ioapics = apic.io_apics;
                let ioapic = ioapics.first().expect("no ioapics");
                let ioapic_addr = ioapic.address;
                ioapic_addr
            } else {
                panic!("no ioapic");
            }
        }
    }
}