#![feature(lang_items)]
#![feature(asm)]
#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(core_intrinsics)]
extern crate rlibc;
extern crate multiboot;
extern crate display;
extern crate memory;
extern crate hardware;
extern crate alloc;
extern crate malloc;
extern crate stdx_memory;
extern crate stdx;
extern crate pic8259_simple;
extern crate multiprocess;
extern crate setup;

use multiboot::multiboot_header::MultibootHeader;
use multiboot::multiboot_header::tags::{basic_memory_info, elf, memory_map};
use multiboot::multiboot_header::tags::memory_map::*;
use display::vga::writer::Writer;
use memory::allocator::bump::BumpAllocator;
use memory::allocator::bump::ConstSizeBumpAllocator;
use memory::frame::frame_allocator::*;
use memory::frame::Frame;
use memory::frame::FRAME_SIZE;
use memory::paging;
use memory::paging::page_table;
use memory::paging::page_table::P4Table;
use stdx_memory::MemoryAllocator;
use stdx_memory::MemoryAllocatorMeta;
use core::clone::Clone;
use core::fmt::Write;
use alloc::boxed::Box;
use alloc::vec::Vec;
use malloc::TestAllocator;
use stdx_memory::collections::immutable::double_linked_list::DoubleLinkedList;
use memory::allocator::slab::SlabAllocator;
use memory::allocator::slab::SlabHelp;
use memory::allocator::buddy::BuddyAllocator;

use hardware::x86_64::registers;
use hardware::x86_64::interrupts;
use hardware::x86_64::interrupts::idt::{InterruptTable, HardwareInterrupts};
use hardware::x86_64::interrupts::InterruptTableHelp;
use hardware::x86_64::interrupts::handler::{InterruptHandler, InterruptHandlerWithErrorCode, InterruptStackFrameValue};
use hardware::x86_64::interrupts::pic;
use core::ptr;
use core::ops::DerefMut;
use core::cell;
use alloc::alloc::Layout;
use alloc::rc::Rc;
use stdx_memory::heap;
use multiprocess::process::{Process, Message};
use multiprocess::executor;
use multiprocess::process;
use pic8259_simple::ChainedPics;

use setup::interrupts::handlers;
use setup::globals;
use setup::globals::{
    VGA_WRITER,
    PROCESS_EXECUTOR,
    INTERRUPT_TABLE,
    CHAINED_PICS,
    HEAP_ALLOCATOR
};

#[no_mangle]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub extern "C" fn rust_main(multiboot_header_address: usize) {
    unsafe {

        let multiboot_header = MultibootHeader::load(multiboot_header_address);

        VGA_WRITER = Some(Writer::new());

        //print_multiboot_data(multiboot_header, VGA_WRITERG.as_mut().unwrap());

        let mut frame_allocator = FrameAllocator::new(multiboot_header);

        paging::remap_kernel(&mut paging::p4_table(), &mut frame_allocator, multiboot_header);

        let mut slab_allocator = globals::initialize_memory_allocator(&multiboot_header);

        HEAP_ALLOCATOR.value = ptr::NonNull::new_unchecked(&mut slab_allocator as *mut SlabAllocator);

        memory_allocator_should_properly_allocate_and_free_memory();

        globals::initialize_interrupt_table();

        interrupts::load_interrupt_table(&INTERRUPT_TABLE);

        let mut executor = Rc::new(cell::UnsafeCell::new(executor::Executor::new()));

        PROCESS_EXECUTOR.value =  ptr::NonNull::new_unchecked(&mut executor as *mut executor::ExecutorRef);

        use core::mem;
        use core::ops::Deref;

        let dummy_process = DummyProcess { value : 1000 };
        let dummy_process_state_box = Box::new(dummy_process);
       
        PROCESS_EXECUTOR.create_process( dummy_process_state_box);
        PROCESS_EXECUTOR.post_message(0, Box::new(IncreaseCtr { some : 299}));

        /*let mut root_process = process::RootProcess::new(Rc::clone(&executor));

        let sample_process = SampleProcess {
            root :  process::ProcessRef::clone(&root_process),

            child : None,

            ctr : 0
        };

        let sample_process_box = Box::new(sample_process);

        let mut sample_ref = root_process.fork(sample_process_box);

        {
            sample_ref.post_message(Box::new(process::StartProcess {} ));
        }

        let sender_process = SenderProcess {
            root : process::ProcessRef::clone(&root_process),

            child : sample_ref,
        };

        let sender_process_box = Box::new(sender_process);

        {
            root_process.fork(sender_process_box);
        }*/

        hardware::x86_64::interrupts::enable_interrupts();

        // run pre-init tests
        let p4_table = paging::p4_table();

        /*paging_map_should_properly_map_pages(p4_table, slab_allocator.frame_allocator(), &mut vga_writer);
        paging_translate_page_should_properly_translate_pages(p4_table, slab_allocator.frame_allocator());
        paging_unmap_should_properly_unmap_elements(p4_table, slab_allocator.frame_allocator());
        paging_translate_address_should_properly_translate_virtual_address(p4_table, slab_allocator.frame_allocator());*/
        loop {
            unsafe { writeln!(VGA_WRITER.as_mut().unwrap(), "Main thread end loop!"); };
        }
    }
}

#[repr(C)]
pub struct DummyProcess {

    value : usize
}

impl DummyProcess {
    fn process_message1(&mut self, message : Message) -> () {

        let x = 10;

        if message.is::<IncreaseCtr>() {

            let msg= message.downcast::<IncreaseCtr>().unwrap();
            let vl = msg.some;
            let x = vl;

           unsafe { writeln!(VGA_WRITER.as_mut().unwrap(), "I am dummy! inside process 1 {}", x); }
        }
    }
}

impl Process for DummyProcess {
    fn process_message(&mut self, message: Message) -> () {
        unsafe {
            self.process_message1(message);
            loop {
                // unsafe { writeln!(VGA_WRITER.as_mut().unwrap(), "Inside dummy end"); }
            }
        }
    }
}

pub struct IncreaseCtr {
    pub some : usize
}

pub struct SenderProcess {

    pub root : process::ProcessRef,

    pub child : process::ProcessRef,
}

impl Process for SenderProcess {
    fn process_message(&mut self, message: Message) -> () {
        unsafe {
            writeln!(VGA_WRITER.as_mut().unwrap(), "Sending inc to Id = {}!", self.child.id());

            self.child.post_message(Box::new(IncreaseCtr { some : 1488 }));
        }
    }
}

fn memory_allocator_should_properly_allocate_and_free_memory() {
    // everything inside inner block will get deleted after block exit
    {
        let mut test_array : [u64;6] = [0 ;6];

        {
            let boxin = Box::new(test_array);

            let b = boxin;
        }
    }

    let result = unsafe { HEAP_ALLOCATOR.is_fully_free() };

    assert_eq!(result, true, "Allocator wasn't fully free after allocating memory in isolated block");
}

fn preallocate_memory_for_allocator_aux_data_structures(memory_start : usize, memory_end : usize) -> usize {
    let aux_data_structures_size = SlabAllocator::total_aux_data_structures_size(memory_start, memory_end);

    let premade_bump_end_address  = Frame::address_align_up(memory_start + aux_data_structures_size);
    let mut premade_bump                = ConstSizeBumpAllocator::from_address(memory_start, premade_bump_end_address, FRAME_SIZE);

    // |aux structures page tables|aux structures working memory|allocator working memory|
    // premap memory for memory allocator inner data structures
    let aux_structures_start_address = premade_bump_end_address + FRAME_SIZE; // next frame
    let aux_structures_end_address = Frame::address_align_up(aux_structures_start_address + aux_data_structures_size);

    for frame in Frame::range_inclusive(aux_structures_start_address, aux_structures_end_address) {
        let p4_table = paging::p4_table();
        p4_table.map_page_1_to_1(frame, page_table::PRESENT | page_table::WRITABLE, &mut premade_bump);
    }

    test_allocator_aux_data_structures_memory(aux_structures_start_address, aux_structures_end_address);

    aux_structures_start_address
}

fn test_allocator_aux_data_structures_memory(aux_structures_start_address : usize, aux_structures_end_address : usize) {
    for frame in Frame::range_inclusive(aux_structures_start_address, aux_structures_end_address) {
        let p4_table = paging::p4_table();
        let present = p4_table.is_present(frame);

        unsafe { writeln!(VGA_WRITER.as_mut().unwrap(), "Is present {}, val {}", frame, present); }

        Frame::zero_frame(&frame);
    }
}

use core::panic::PanicInfo;

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}
#[lang = "panic_impl"]
#[no_mangle]
pub extern "C" fn panic_impl(pi: &PanicInfo) -> ! {

    //unsafe { writeln!(VGA_WRITER.as_mut().unwrap(), "Rust code panicked with {}", pi.message()); }

    loop {}
}
#[lang = "oom"]
#[no_mangle]
pub extern "C" fn oom(l: Layout) -> ! {
    loop {}
}

fn print_multiboot_data(multiboot_header : &MultibootHeader, vga_writer : &mut Writer) {
    writeln!(vga_writer, "---Basic memory info---");

    let mem_info = multiboot_header
            .read_tag::<memory_map::MemoryMap>()
            .unwrap();

    let mut mem_sections =
            mem_info
                .entries()
                .filter(|e| e.entry_type() == memory_map::MemoryMapEntryType::Available as u32);

    writeln!(vga_writer, "---Available memory {}", mem_info.available_memory());
    writeln!(vga_writer, "---Memory sections---");
    while let Some(e) = mem_sections.next() {
        writeln!(vga_writer, "{}", e);
    }

    let elf_sections = multiboot_header
            .read_tag::<elf::ElfSections>()
            .unwrap();
    let mut elf_sections_it = elf_sections.entries();

    writeln!(vga_writer, "---Elf sections---");
    writeln!(vga_writer, "Elf sections start: {}", elf_sections.entries_start_address().unwrap());
    writeln!(vga_writer, "Elf sections end: {}", elf_sections.entries_end_address().unwrap());

    while let Some(e) = elf_sections_it.next() {
        writeln!(vga_writer, "{}", e);
    }
}

unsafe fn paging_map_should_properly_map_pages(page_table : &mut page_table::P4Table, frame_alloc : &mut BuddyAllocator, vga_writer : &mut Writer) {

    let virtual_frame = Frame::from_address(0x400000000000);
    let physical_frame = Frame::from_address(frame_alloc.allocate(FRAME_SIZE).expect("No frames for paging test"));

    page_table.map_page(virtual_frame, physical_frame, page_table::PRESENT | page_table::WRITABLE, frame_alloc);

    // try read whole frame from virtual address, if it succeeds without a segfault then
    // map function worked correctly
    let virtual_frame_address = virtual_frame.address();

    for i in 0..FRAME_SIZE {
        unsafe {
            // reading into var is important to prevent compiler optimizing the read away
            let _result = *((virtual_frame_address + i as usize) as *const u8);
        }
    }

    frame_alloc.free(physical_frame.address());
    page_table.unmap_page(virtual_frame);
}

unsafe fn paging_translate_page_should_properly_translate_pages(page_table : &mut page_table::P4Table, frame_alloc : &mut BuddyAllocator) {
    let virtual_frame = Frame::from_address(42 * 512 * 512 * 4096);
    let physical_frame = Frame::from_address(frame_alloc.allocate(FRAME_SIZE).expect("No frames for paging test"));

    page_table.map_page(virtual_frame, physical_frame, page_table::PRESENT, frame_alloc);

    let result = page_table.translate_page(virtual_frame);

    sanity_assert_translate_page_result(virtual_frame, physical_frame, result);

    frame_alloc.free(physical_frame.address());
    page_table.unmap_page(virtual_frame);
}

unsafe fn paging_translate_address_should_properly_translate_virtual_address(page_table : &mut page_table::P4Table, frame_alloc : &mut BuddyAllocator) {
    let virtual_frame = Frame::from_address(42 * 512 * 512 * 4096);
    let physical_frame = Frame::from_address(frame_alloc.allocate(FRAME_SIZE).expect("No frames for paging test"));

    page_table.map_page(virtual_frame, physical_frame, page_table::PRESENT, frame_alloc);

    let virtual_frame_address = virtual_frame.address();
    let physical_frame_address = physical_frame.address();

    for frame_offset in 0..FRAME_SIZE {
        let virtual_address  = virtual_frame_address + frame_offset as usize;
        let physical_address = physical_frame_address + frame_offset as usize;
        let result = page_table.translate(virtual_address);

        sanity_assert_translate_address_result(virtual_address, physical_address, result);
    }

    frame_alloc.free(physical_frame.address());
    page_table.unmap_page(virtual_frame);
}

unsafe fn paging_unmap_should_properly_unmap_elements(page_table : &mut page_table::P4Table, frame_alloc : &mut BuddyAllocator) {
    let virtual_frame = Frame::from_address(42 * 512 * 512 * 4096);
    let physical_frame = Frame::from_address(frame_alloc.allocate(FRAME_SIZE).expect("No frames for paging test"));

    page_table.map_page(virtual_frame, physical_frame, page_table::PRESENT, frame_alloc);
    page_table.unmap_page(virtual_frame);

    let result = page_table.translate_page(virtual_frame);

    assert!(result.is_none(),
        "Translation of virtual page {} returned physical frame {} after unmap, but should return empty result",
        virtual_frame,
        result.unwrap());

    frame_alloc.free(physical_frame.address());
}

fn sanity_assert_translate_page_result(virtual_frame : Frame, physical_frame : Frame, result : Option<Frame>) {
    assert!(result.is_some(),
        "Returned empty result for translation of virtual frame {}",
        virtual_frame);

    let result_frame = result.unwrap();

    assert!(physical_frame == result_frame,
        "Returned invalid translation result for virtual frame {}. Should be frame {} but was {}",
        virtual_frame,
        physical_frame,
        result_frame);
}

fn sanity_assert_translate_address_result(virtual_address : usize, physical_address : usize, result : Option<usize>){
    assert!(result.is_some(),
        "Returned empty result for translation of virtual frame {}",
        virtual_address);

    let result_address = result.unwrap();

    assert_eq!(physical_address, result_address, "Returned invalid translation result for virtual frame {}. Should be frame {} but was {}", virtual_address, physical_address, result_address);
}

