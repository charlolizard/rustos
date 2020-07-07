use ::x86_64::interrupts::handler::{InterruptHandler, InterruptHandlerWithErrorCode};
use ::x86_64::interrupts::pic::PIC_1_OFFSET;
use core::marker::PhantomData;
use core::ops::{Index, IndexMut};

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum HardwareInterrupts {
    Timer = 0,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct InterruptTableEntry<HandlerFunc> {
    lower_pointer_bits : u16,
    gdt_selector : GDTSelector,
    options : InterruptOptions,
    middle_pointer_bits : u16,
    remaining_pointer_bits : u32,
    reserved : u32,
    ph : PhantomData<HandlerFunc>
}

impl<HandlerFunc> InterruptTableEntry<HandlerFunc> {
    fn new(handler_address : u64) -> Self {

        let lower_pointer_bits              = handler_address as u16;
        let middle_pointer_bits            = (handler_address >> 16) as u16;
        let remaining_pointer_bits      = (handler_address >> 32) as u32;
        let options           = InterruptOptions::new();
        let gdt_selector = GDTSelector::minimal();

        InterruptTableEntry {
            lower_pointer_bits,
            gdt_selector,
            options,
            middle_pointer_bits,
            remaining_pointer_bits,
            reserved : 0,
            ph : PhantomData
        }
    }

    pub fn create_present_entry(handler : InterruptHandler) -> Self {
        let mut result = InterruptTableEntry::<HandlerFunc>::new(handler as u64);
        result.options.set_present();

        result
    }

    pub fn create_present_entry1(handler : InterruptHandlerWithErrorCode) -> Self {
        let mut result = InterruptTableEntry::<HandlerFunc>::new( handler as u64);
        result.options.set_present();

        result
    }

    const fn empty() -> Self {

        let options = InterruptOptions::minimal();
        let gdt_selector = GDTSelector::empty();

        InterruptTableEntry {
            lower_pointer_bits : 0,
            gdt_selector,
            options,
            middle_pointer_bits : 0,
            remaining_pointer_bits : 0,
            reserved : 0,
            ph : PhantomData
        }
    }
}

#[repr(C)]
#[repr(align(16))]
pub struct InterruptTable {

    // handlers for cpu exceptions
    pub divide_by_zero : InterruptTableEntry<InterruptHandler>,

    pub debug : InterruptTableEntry<InterruptHandler>,

    pub non_maskable_interrupt : InterruptTableEntry<InterruptHandler>,

    pub breakpoint : InterruptTableEntry<InterruptHandler>,

    pub overflow : InterruptTableEntry<InterruptHandler>,

    pub bound_range_exceed : InterruptTableEntry<InterruptHandler>,

    pub invalid_opcode : InterruptTableEntry<InterruptHandler>,

    pub device_not_available : InterruptTableEntry<InterruptHandler>,

    pub double_fault : InterruptTableEntry<InterruptHandler>,

    coprocessor_segment_overrun : InterruptTableEntry<InterruptHandler>,

    pub invalid_tss : InterruptTableEntry<InterruptHandler>,

    pub segment_not_present : InterruptTableEntry<InterruptHandler>,

    pub stack_segment_fault : InterruptTableEntry<InterruptHandler>,

    pub general_protection_fault : InterruptTableEntry<InterruptHandler>,

    pub page_fault : InterruptTableEntry<InterruptHandler>,

    reserved_0 : InterruptTableEntry<InterruptHandler>,

    pub x87_floating_point_exception : InterruptTableEntry<InterruptHandler>,

    pub aligment_check : InterruptTableEntry<InterruptHandler>,

    pub machine_check : InterruptTableEntry<InterruptHandler>,

    pub simd_floating_point_exception : InterruptTableEntry<InterruptHandler>,

    pub virtualization_exception : InterruptTableEntry<InterruptHandler>,

    reserved_1 : [InterruptTableEntry<InterruptHandler>; 9],

    pub security_exception : InterruptTableEntry<InterruptHandler>,

    reserved_10 : InterruptTableEntry<InterruptHandler>,

    // handlers for user defined and hardware interrupts
    interrupts : [InterruptTableEntry<InterruptHandler>; 256 - 32]
}

impl InterruptTable {
    pub const fn new() -> Self {
        InterruptTable {
            divide_by_zero: InterruptTableEntry::empty(),
            debug: InterruptTableEntry::empty(),
            non_maskable_interrupt: InterruptTableEntry::empty(),
            breakpoint: InterruptTableEntry::empty(),
            overflow: InterruptTableEntry::empty(),
            bound_range_exceed: InterruptTableEntry::empty(),
            invalid_opcode: InterruptTableEntry::empty(),
            device_not_available: InterruptTableEntry::empty(),
            double_fault: InterruptTableEntry::empty(),
            coprocessor_segment_overrun: InterruptTableEntry::empty(),
            invalid_tss: InterruptTableEntry::empty(),
            segment_not_present: InterruptTableEntry::empty(),
            stack_segment_fault: InterruptTableEntry::empty(),
            general_protection_fault: InterruptTableEntry::empty(),
            page_fault: InterruptTableEntry::empty(),
            reserved_0: InterruptTableEntry::empty(),
            x87_floating_point_exception: InterruptTableEntry::empty(),
            aligment_check: InterruptTableEntry::empty(),
            machine_check: InterruptTableEntry::empty(),
            simd_floating_point_exception: InterruptTableEntry::empty(),
            virtualization_exception: InterruptTableEntry::empty(),
            reserved_1: [InterruptTableEntry::empty(); 9],
            security_exception: InterruptTableEntry::empty(),
            reserved_10: InterruptTableEntry::empty(),
            interrupts :  [InterruptTableEntry::empty(); 256 - 32]
        }
    }

    pub fn set_interrupt_handler(&mut self, idx : usize, handler : InterruptHandler) {
        let entry = InterruptTableEntry::create_present_entry(handler);

        self[idx] = entry
    }

    pub(crate) fn pointer(&self) -> InterruptTablePointer {
        use core::mem;

        let base = self as *const _ as u64;
        let limit = (mem::size_of::<Self>() - 1) as u16; // -1 because address must be inclusive according to spec

        InterruptTablePointer {
            limit,
            base
        }
    }
}

impl Index<usize> for InterruptTable {
    type Output = InterruptTableEntry<InterruptHandler>;

    fn index(&self, index: usize) -> &InterruptTableEntry<InterruptHandler> {
        match index {
            i @ 32 ..=255 => &self.interrupts[i - 32],
            _ => panic!("Interrupt table index out of range")
        }
    }
}

impl IndexMut<usize> for InterruptTable {
    fn index_mut(&mut self, index: usize) -> &mut InterruptTableEntry<InterruptHandler> {
        match index {
            i @ 32 ..=255 => &mut self.interrupts[i - 32],
            _ => panic!("Interrupt table index out of range")
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct InterruptOptions {
    value : u16
}

impl InterruptOptions {

    pub const fn minimal() -> Self {
        let validValue = 0b1110_0000_0000; // bits 9-11 must be set to 1 according to spec

        InterruptOptions {
            value : validValue
        }
    }

    pub fn new() -> Self {
        let mut minimal = InterruptOptions::minimal();

        minimal.set_present();

        minimal
    }

    pub fn value(&self) -> u16 {
        self.value
    }

    pub fn flags(&self) -> InterruptOptionsFlags {
        InterruptOptionsFlags::from_bits_truncate(self.value)
    }

    pub fn set_flags(&mut self, new_flags : InterruptOptionsFlags) {
        self.value = new_flags.bits();
    }

    pub fn disable_interrupt(&mut self) {
        let mut flags = self.flags();

        flags.remove(DISABLE_INTERRUPT);

        self.value = flags.bits();
    }

    pub fn set_present(&mut self) {
        let mut flags = self.flags();

        flags.insert(IS_PRESENT);

        self.value = flags.bits();
    }

    pub fn set_unused(&mut self) {
        let mut flags = self.flags();
        flags.remove(IS_PRESENT);

        self.value = flags.bits();
    }
}

bitflags! {
    pub struct InterruptOptionsFlags : u16 {
        const DISABLE_INTERRUPT = 1 << 8;
        const ALWAYS_PRESENT = 1 << 9;
        const ALWAYS_PRESENT1 = 1 << 10;
        const ALWAYS_PRESENT2 = 1 << 11;
        const IS_PRESENT =      1 << 15;
    }
}


#[repr(C, packed)]
pub(crate) struct InterruptTablePointer {
    limit : u16,
    base : u64
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct GDTSelector {
    value : u16
}

impl GDTSelector {
    pub fn minimal() -> Self {
        use x86_64::registers;

        let cs_value = registers::cs();

        GDTSelector {
            value : cs_value
        }
    }

    fn new(index : u16, privilege_level : u16) -> Self {
        use x86_64::registers;

        let cs_value = registers::cs();

        //let new_value = index << 3 | privilege_level;

        GDTSelector {
            value : cs_value
        }
    }

    const fn empty() -> Self {
        GDTSelector {
            value : 0
        }
    }
}