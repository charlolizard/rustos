use MemoryAllocator;
use core::ptr;
use core::fmt;

const BITMAP_ENTRY_SIZE: usize = 8; //number of bits in byte

pub struct FrameBitMap {
    start_address : usize,
    size : usize,
    frames_count : usize,
}

impl FrameBitMap {

    pub fn new_from_available_memory<A>(available_memory: usize, frame_size: usize, memory_allocator : &mut A) -> Self where A : MemoryAllocator {
        let frames_count = available_memory / frame_size;
        FrameBitMap::new(frames_count, memory_allocator)
    }

    pub fn mem_size_for(frames_count : usize) -> usize {
        let bitmap_size_help = frames_count % BITMAP_ENTRY_SIZE;

        if bitmap_size_help > 0 {
            (frames_count / BITMAP_ENTRY_SIZE) + 1
        } else {
            frames_count / BITMAP_ENTRY_SIZE
        }
    }

    pub fn new<A>(frames_count : usize, memory_allocator : &mut A) -> Self where A : MemoryAllocator {

        let bitmap_size_help = frames_count % BITMAP_ENTRY_SIZE;
        let bitmap_size = if bitmap_size_help > 0 {
            (frames_count / BITMAP_ENTRY_SIZE) + 1
        } else {
            frames_count / BITMAP_ENTRY_SIZE
        };

        let address = memory_allocator
            .allocate(bitmap_size)
            .expect("Failed to allocate memory for frame bitmap");
            
        for i in address..(address + bitmap_size) {
            unsafe { ptr::write(i as *mut FrameBitMapEntry, FrameBitMapEntry::new()) }
        }

        FrameBitMap {
            start_address : address,
            size          : bitmap_size,
            frames_count  : frames_count,            
        }
    }

    //todo check for out of bounds
    fn index(&self, frame_number: usize) -> &mut FrameBitMapEntry {        
        let index = frame_number / BITMAP_ENTRY_SIZE;

        unsafe { &mut (*((self.start_address + index) as *mut FrameBitMapEntry)) }
    }

    pub fn is_in_use(&self, frame_number: usize) -> bool {
        self.index(frame_number).is_in_use(frame_number)
    }

    pub fn is_free(&self, frame_number : usize) -> bool {
        !self.is_in_use(frame_number)
    }

    pub fn set_in_use(&mut self, frame_number: usize) {
        self.index(frame_number).set_in_use(frame_number)
    }

    pub fn set_free(&mut self, frame_number: usize) {
        self.index(frame_number).set_free(frame_number)
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn frames_count(&self) -> usize {
        self.frames_count
    }
}

impl fmt::Display for FrameBitMap {    
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"")
    }    
}

#[repr(C)]
struct FrameBitMapEntry {
    value: u8,
}

impl FrameBitMapEntry {
    fn new() -> FrameBitMapEntry {
        FrameBitMapEntry { value: 0 }
    }

    fn index_in_byte_field(frame_number: usize) -> usize {
        frame_number % BITMAP_ENTRY_SIZE
    }

    fn offset_count(frame_number: usize) -> usize {
        let index_in_byte_field = FrameBitMapEntry::index_in_byte_field(frame_number);
        BITMAP_ENTRY_SIZE - 1 - index_in_byte_field
    }

    fn is_in_use(&self, frame_number: usize) -> bool {
        let offset_count = FrameBitMapEntry::offset_count(frame_number);
        let bit_mask = (1 as u8) << offset_count;

        self.value & bit_mask > 0
    }

    fn set_in_use(&mut self, frame_number: usize) {
        let offset_count = FrameBitMapEntry::offset_count(frame_number);
        let bit_mask = (1 as u8) << offset_count;

        self.value = self.value | bit_mask
    }

    fn set_free(&mut self, frame_number: usize) {
        let offset_count = FrameBitMapEntry::offset_count(frame_number);
        let bit_mask = !((1 as u8) << offset_count);

        self.value = self.value & bit_mask
    }
}

impl fmt::Display for FrameBitMapEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:b}", self.value)
    }
}