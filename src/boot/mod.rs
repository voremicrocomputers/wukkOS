use alloc::sync::Arc;
use core::marker::PhantomData;
use core::ptr::NonNull;
use acpi::{AcpiHandler, AcpiTables, InterruptModel, PhysicalMapping};
use limine::{LimineBootInfoRequest, LimineKernelAddressRequest, LimineMemmapRequest, LimineTerminalRequest, LimineTerminalResponse, LimineRsdpRequest};
use crate::{debug, println};

#[cfg(feature = "f_multiboot2")]
use multiboot2::{load, MemoryMapTag, BootInformation};
use x86_64::structures::paging::{FrameAllocator, Mapper, OffsetPageTable, Page, Translate};
use x86_64::VirtAddr;
use crate::memory::{BootInfoFrameAllocator, FRAME_ALLOC, MEM_MAPPER, PageSize, read_phys_memory32, VIRT_MEM_OFFSET};
use crate::serial::terminal::ST;

pub static BOOTLOADER_INFO: LimineBootInfoRequest = LimineBootInfoRequest::new(0);
pub static TERMINAL_REQUEST: LimineTerminalRequest = LimineTerminalRequest::new(0);
pub static MEM_MAP: LimineMemmapRequest = LimineMemmapRequest::new(0);
pub static RSDP_REQUEST: LimineRsdpRequest = LimineRsdpRequest::new(0);
pub static KERNEL_ADDRESS: LimineKernelAddressRequest = LimineKernelAddressRequest::new(0);

#[derive(Clone)]
struct Handler;
impl AcpiHandler for Handler {
    unsafe fn map_physical_region<T>(&self, physical_address: usize, size: usize) -> PhysicalMapping<Self, T> { // todo! check if size is too big
        // only get lower 32 bits of physical address
        let physical_address = physical_address as u32;
        debug!("Mapping physical region: {:x} - {:x}", physical_address, physical_address + size as u32);
        let _ = read_phys_memory32(physical_address as u32) as usize;
        PhysicalMapping::new(
            physical_address as usize,
            NonNull::new_unchecked(physical_address as *mut T),
            size, size,
            Self)
    }

    fn unmap_physical_region<T>(region: &PhysicalMapping<Self, T>) {
        // get page
        let page: Page<PageSize> = Page::containing_address(VirtAddr::new(region.physical_start() as u64));
        // unmap page
        let res = unsafe { MEM_MAPPER.lock().as_mut().unwrap().unmap(page) };
        // it isn't *that* important if we don't unmap successfully at the moment, so just write a warning if we fail

        if res.is_err() {
            println!("[WARN] failed to unmap page (this is normal)");
        }
    }
}

pub struct LimineWriter;

impl core::fmt::Write for LimineWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        static mut CACHED: Option<&'static LimineTerminalResponse> = None;
        unsafe {
            if let Some(writer) = CACHED {
                let terminal = &writer.terminals()[0];
                writer.write().unwrap()(terminal, s);
            } else {
                let response = TERMINAL_REQUEST.get_response().get().unwrap();
                let terminal = &response.terminals()[0];
                let writer = response.write().unwrap();
                writer(terminal, s);

                CACHED = Some(response);
            }
        }
        Ok(())
    }
}

pub fn get_ioapic_addr() -> u32 {
    let rsdp = RSDP_REQUEST.get_response().get().unwrap();
    let rsdp_ptr = rsdp.address.get().unwrap() as *const u8;
    let tables = unsafe { AcpiTables::from_rsdp(Handler, rsdp_ptr as usize).unwrap() };
    let platform_info = tables.platform_info().expect("no platform info");
    let interrupt_model = platform_info.interrupt_model;
    let ioapic_addr = match interrupt_model {
        InterruptModel::Apic(apic) => apic.io_apics[0].address,
        _ => panic!("unsupported interrupt model"),
    };
    ioapic_addr
}