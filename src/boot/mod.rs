use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::marker::PhantomData;
use core::ptr::NonNull;
use acpi::{AcpiHandler, AcpiTables, InterruptModel, PhysicalMapping};
use acpi::platform::interrupt::InterruptSourceOverride;
use cstr_core::CString;
use limine::{LimineBootInfoRequest, LimineKernelAddressRequest, LimineMemmapRequest, LimineTerminalRequest, LimineTerminalResponse, LimineRsdpRequest, LimineSmpRequest, LimineModuleRequest};
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
pub static SMP_REQUEST: LimineSmpRequest = LimineSmpRequest::new(0);
pub static MOD_REQUEST: LimineModuleRequest = LimineModuleRequest::new(0);

#[derive(Clone)]
struct Handler;
impl AcpiHandler for Handler {
    unsafe fn map_physical_region<T>(&self, physical_address: usize, size: usize) -> PhysicalMapping<Self, T> { // todo! check if size is too big
        // only get lower 32 bits of physical address
        let physical_address = physical_address as u32;
        debug!("mapping physical region: {:x} - {:x}", physical_address, physical_address + size as u32);
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

        if let Err(e) = res {
            debug!("(THIS IS NORMAL) failed to unmap physical region: {:?}", e);
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

pub fn get_ioapic_info() -> (u32, Vec<InterruptSourceOverride>) {
    let rsdp = RSDP_REQUEST.get_response().get().unwrap();
    let rsdp_ptr = rsdp.address.get().unwrap() as *const u8;
    let tables = unsafe { AcpiTables::from_rsdp(Handler, rsdp_ptr as usize).unwrap() };
    let platform_info = tables.platform_info().expect("no platform info");
    let interrupt_model = platform_info.interrupt_model;
    let apic = match interrupt_model {
        InterruptModel::Apic(apic) => apic,
        _ => panic!("unsupported interrupt model"),
    };
    let ioapic = apic.io_apics.first().expect("no ioapic");
    let address = ioapic.address;
    let overrides = apic.interrupt_source_overrides;
    (address, overrides)
}

pub fn get_initwukko() -> Vec<u8> {
    let mut response = MOD_REQUEST.get_response().get_mut().unwrap();
    let module = &response.modules()[0];
    let path_cstr = module.path.as_ptr().unwrap();
    let path = unsafe { CString::from_raw(path_cstr as *mut _) };
    debug!("initwukko path: {}", path.to_str().unwrap());
    let start = module.base.get().unwrap() as *const _ as usize;
    let size = module.length as usize;
    let end = start + size;
    let mut data = Vec::new();
    for i in start..end {
        let byte = unsafe { *(i as *const u8) };
        data.push(byte);
    }
    data
}