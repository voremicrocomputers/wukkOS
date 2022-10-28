use alloc::sync::Arc;
use core::marker::PhantomData;
use core::ptr::NonNull;
use acpi::{AcpiHandler, InterruptModel, PhysicalMapping};
use limine::LimineTerminalResponse;
use crate::{debug, println, TERMINAL_REQUEST};

#[cfg(feature = "f_multiboot2")]
use multiboot2::{load, MemoryMapTag, BootInformation};
use x86_64::structures::paging::{FrameAllocator, Mapper, OffsetPageTable, Page, Translate};
use x86_64::VirtAddr;
use crate::memory::{BootInfoFrameAllocator, FRAME_ALLOC, MEM_MAPPER, PageSize, read_phys_memory32};
use crate::serial::terminal::ST;

#[derive(Clone)]
struct Handler;
impl AcpiHandler for Handler {
    unsafe fn map_physical_region<T>(&self, physical_address: usize, size: usize) -> PhysicalMapping<Self, T> {
        // todo! check if size is too big
        debug!("read_phys_memory32: addr {:x} not mapped", physical_address);
        let mut i = 0;
        while i < size {
            let _ = read_phys_memory32(physical_address as u32 + i as u32);
            i += 4;
        }
        let addr = unsafe { MEM_MAPPER.lock().as_mut().unwrap().translate_addr(VirtAddr::new(physical_address as u64)) };
        if let Some(addr) = addr.clone() {
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