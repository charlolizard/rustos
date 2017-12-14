use core;
use core::marker;
use core::mem;
use core::ops;
use core::iter;
use util::free_list_allocator::FreeListAllocator;
use allocator::MemoryAllocator;
use stdx::ptr;
use stdx::iterator;


pub struct Array<T> {
    length : usize,
    start_address : usize,    
    phantom : marker::PhantomData<T>
}

impl <T> Array<T> {    
     pub unsafe fn new(length : usize, memory_allocator : &mut MemoryAllocator) -> Array<T> {

        let size = mem::size_of::<T>() * length;
        let start_address = memory_allocator.allocate(size).expect("No memory for Array");
                
        // zero vector memory
        for i in 0..size {
            let address = start_address + i;
            core::ptr::write_unaligned(address as *mut u8, 0);
        }

        Array { 
            length : length,
            start_address : start_address,            
            phantom : marker::PhantomData
        }
    }

    pub fn size(&self) -> usize {
        mem::size_of::<T>() * self.length 
    }

    pub fn length(&self) -> usize {
        self.length
    }
    
    pub fn update(&mut self, index : usize, value : T) {
        assert!(index < self.length);

        let start_address = self.start_address;
        let entry_address = start_address + (mem::size_of::<T>() * index); 
        
        unsafe { core::ptr::write_unaligned(entry_address as *mut T, value);         }
    }    

    pub fn free(self, memory_allocator : &mut MemoryAllocator) {
        memory_allocator.free(self.start_address)
    }

    pub fn iterator(&self) -> ArrayIterator<T> {
        ArrayIterator::new(self)
    }

    pub fn elem_ref(&self, index : usize) -> &T {
        let start_address = &self as *const _ as usize;
        let entry_address = start_address + (mem::size_of::<T>() * index); 
        
        unsafe { &*(entry_address as *const T) }
    }

    pub fn elem_ref_mut(&self, index : usize) -> &mut T {
        let start_address = &self as *const _ as usize;
        let entry_address = start_address + (mem::size_of::<T>() * index);
        
        unsafe { &mut *(entry_address as *mut T) }
    }    
}

impl <T> Array<T> where T : Copy {
    pub fn elem_val(&self, index : usize) -> T {
        let start_address = &self as *const _ as usize;
        let entry_address = start_address + (mem::size_of::<T>() * index); 
        
        unsafe { *(entry_address as *mut T) }
    }
}

impl <T> Array<T> where T : Default {
    pub unsafe fn new_fill_default(length : usize, memory_allocator : &mut MemoryAllocator) -> Array<T> {

        let elem_size = mem::size_of::<T>();
        let total_size = elem_size * length;
        let start_address = memory_allocator.allocate(total_size).expect("No memory for Array");
                
        // fill default values
        for i in (0..total_size).step_by(elem_size) {
            let address = start_address + i;
            core::ptr::write_unaligned(address as *mut T, T::default());
        }

        Array { 
            length : length,
            start_address : start_address,            
            phantom : marker::PhantomData
        }
    }
}

impl<T> ops::Index<usize> for Array<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        let start_address = &self as *const _ as usize;
        let entry_address = start_address + (mem::size_of::<T>() * index); 
        
        unsafe { &*(entry_address as *const T) }
    }
}

impl<T> ops::IndexMut<usize> for Array<T> {

    fn index_mut(&mut self, index: usize) -> &mut T {
        let start_address = &self as *const _ as usize;
        let entry_address = start_address + (mem::size_of::<T>() * index); 
        
        unsafe { &mut *(entry_address as *mut T) }
    }
}

pub struct ArrayIterator<'a, T> where T : 'a {
    i : usize,
    array : &'a Array<T>,    
}

impl<'a, T> ArrayIterator <'a, T> {

    fn new(array : &'a Array<T>) -> Self {
        ArrayIterator {
            i  : 0,
            array : array,            
        }
    }
}

impl<'a, T> iter::Iterator for ArrayIterator<'a, T> {
    type Item = ptr::Unique<T>;

    fn next(&mut self) -> Option<ptr::Unique<T>> {
        if self.i >= self.array.length() {
            None
        }
        else {
            let result = self.array.elem_ref(self.i);
            self.i += 1;

            Some(ptr::Unique::new(result))
        }
    }
}