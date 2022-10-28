pub mod allocator;

use alloc::boxed::Box;
use alloc::sync::Arc;
use lazy_static::lazy_static;
use limine::LimineMemoryMapEntryType;
use x86_64::structures::paging::{FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB, Translate};
use x86_64::{PhysAddr, VirtAddr};

lazy_static!{
    pub static ref MEM_MAPPER: Mutex<Option<OffsetPageTable<'static>>> = Mutex::new(None);
    pub static ref FRAME_ALLOC: Mutex<Option<BootInfoFrameAllocator>> = Mutex::new(None);
}

pub const VIRT_MEM_OFFSET: u64 = 0xffffffff80000000;

pub type PageSize = Size4KiB;

pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

pub unsafe fn init(phys_mem_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(phys_mem_offset);
    OffsetPageTable::new(level_4_table, phys_mem_offset)
}

unsafe fn active_level_4_table(phys_mem_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = phys_mem_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe { &mut  *page_table_ptr } // unsafe
}

use spin::Mutex;
use crate::{debug, print, println};
use crate::boot::{KERNEL_ADDRESS, MEM_MAP};

pub struct BootInfoFrameAllocator {
    next: usize,
}

impl BootInfoFrameAllocator {
    pub unsafe fn init() -> Self {
        Self {
            next: 0,
        }
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        #[cfg(feature = "f_limine")] {
            let mmap = MEM_MAP.get_response().get().expect("failed to get memory map")
                .memmap();
            let mut usable_frames = mmap.iter()
                .filter(|entry| entry.typ == LimineMemoryMapEntryType::Usable)
                    .map(|area| {
                        let frame_addr = area.base;
                        let frame_end = area.base + area.len;
                        let frame_size = frame_end - frame_addr;
                        let num_frames = frame_size / 4096;
                        let start_frame = PhysFrame::containing_address(PhysAddr::new(frame_addr));
                        (0..num_frames).map(move |i| start_frame + i)
                    })
                    .flatten();
            let frame = usable_frames.nth(self.next).clone();
            self.next += 1;

            frame
        }
    }
}

pub fn read_phys_memory32(addr: u32) -> u32 {
    let initaladdr = VirtAddr::new(addr as u64);
    let addr = unsafe { MEM_MAPPER.lock().as_mut().unwrap().translate_addr(initaladdr) };
    if let Some(addr) = addr {
        let addr = addr.as_u64() as *const u32;
        unsafe { *addr }
    } else {
        debug!("read_phys_memory32: addr {:x} not mapped", initaladdr.as_u64());
        // map the page
        let frame = FRAME_ALLOC.lock().as_mut().unwrap().allocate_frame().unwrap();
        debug!("allocated frame: {:?}", frame);
        let flags = x86_64::structures::paging::PageTableFlags::PRESENT | x86_64::structures::paging::PageTableFlags::WRITABLE;
        let page = x86_64::structures::paging::Page::containing_address(initaladdr);
        debug!("mapped page: {:?}", page);
        let map_to_result = unsafe { MEM_MAPPER.lock().as_mut().unwrap().map_to(page, frame, flags, FRAME_ALLOC.lock().as_mut().unwrap()) };
        debug!("map_to_result: {:?}", map_to_result);
        if map_to_result.is_err() {
            panic!("Failed to map page");
        }
        let addr = unsafe { MEM_MAPPER.lock().as_mut().unwrap().translate_addr(initaladdr) };
        if let Some(addr) = addr {
            let addr = addr.as_u64() as *const u32;
            unsafe { *addr }
        } else {
            panic!("Failed to map page");
        }
    }
}

pub fn write_phys_memory32(addr: u32, value: u32) {
    let initaladdr = VirtAddr::new(addr as u64);
    let addr = unsafe { MEM_MAPPER.lock().as_mut().unwrap().translate_addr(initaladdr) };
    if let Some(addr) = addr {
        let addr = addr.as_u64() as *mut u32;
        unsafe { *addr = value };
    } else {
        debug!("write_phys_memory32: addr {:x} not mapped", initaladdr.as_u64());
        // map the page
        let frame = FRAME_ALLOC.lock().as_mut().unwrap().allocate_frame().unwrap();
        debug!("allocated frame: {:?}", frame);
        let flags = x86_64::structures::paging::PageTableFlags::PRESENT | x86_64::structures::paging::PageTableFlags::WRITABLE;
        let page = x86_64::structures::paging::Page::containing_address(initaladdr);
        debug!("mapped page: {:?}", page);
        let map_to_result = unsafe { MEM_MAPPER.lock().as_mut().unwrap().map_to(page, frame, flags, FRAME_ALLOC.lock().as_mut().unwrap()) };
        debug!("map_to_result: {:?}", map_to_result);
        if map_to_result.is_err() {
            panic!("Failed to map page");
        }
        let addr = unsafe { MEM_MAPPER.lock().as_mut().unwrap().translate_addr(initaladdr) };
        if let Some(addr) = addr {
            let addr = addr.as_u64() as *mut u32;
            unsafe { *addr = value };
        } else {
            panic!("Failed to map page");
        }
    }
}