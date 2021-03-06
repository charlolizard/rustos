pub mod idt;
pub mod handler;
pub mod pic;

use ::x86_64::interrupts::idt::InterruptTable;

/// Tells the processor to stop handling interrupts
#[inline(always)]
pub fn disable_interrupts() {
    unsafe {
        asm!("cli" :::: "volatile");
    }
}

/// Enables interrupt handling in processor
#[inline(always)]
pub fn enable_interrupts() {
    unsafe {
        asm!("sti" :::: "volatile");
    }
}

/// Loads interrupt table address into interrupt descriptor table address register (IDTR).
/// This should be done before calling `enable_interrupts`, otherwise no interrupts will get handled and processor will restart.
#[inline(always)]
pub fn load_interrupt_table(table : &InterruptTable){
    let ptr = &table.pointer();

    unsafe { asm!("lidt ($0)" :: "r" (ptr) : "memory") };
}

use core::ptr;

pub struct InterruptTableHelp {
    pub value : Option<ptr::NonNull<InterruptTable>>
}

