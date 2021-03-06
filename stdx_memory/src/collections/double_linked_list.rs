use MemoryAllocator;
use collections::array::Array;
use heap;
use stdx::iterator;
use stdx::Iterable;
use stdx::Sequence;
use core::iter;
use core::mem;
use core::fmt;
use core::marker;

pub struct DoubleLinkedList<T> {
    p : marker::PhantomData<T>,
    head : Option<heap::SharedBox<DoubleLinkedListCell<T>>>,
    tail : Option<heap::SharedBox<DoubleLinkedListCell<T>>>,    
}

impl<T> DoubleLinkedList<T> {

    /// Creates new Empty DoubleLinkedList
    /// # Arguments    
    /// * `memory_allocator` - memory allocator
    pub fn new() -> Self
    {        
        DoubleLinkedList {
            p : marker::PhantomData,
            head : None,
            tail : None
        }
    }
    
    pub fn add_to_tail<A>(&mut self, value : T, memory_allocator : &mut A) where A : MemoryAllocator{
        self.add_to_tail_inner(value, memory_allocator);
    }


    /// Adds new DoubleLinkedListCell::Cell to the back of `self.tail`
    /// # Arguments
    /// * `value` - value to add
    /// * `memory_allocator` - memory allocator    
    fn add_to_tail_inner<A>(&mut self, value : T, memory_allocator : &mut A) -> heap::SharedBox<DoubleLinkedListCell<T>> where A : MemoryAllocator{
        if self.tail.is_none() {
            self.tail = Some(DoubleLinkedListCell::new(value, memory_allocator));            
        }
        else {
            self.tail = self.tail.map(|mut e| e.add(value, memory_allocator));
        }                        

        if self.head.is_none() {
            self.head = self.tail;
        }

        self.tail.unwrap()
    }
        
    pub fn head(&self) -> Option<&T> {
        match self.head {
            Some(ref cell) => Some(cell.value_ref()),
            _ => None    
        }
    }

    pub fn tail(&self) -> Option<&T> {
        match self.tail {
            Some(ref cell) => Some(cell.value_ref()),
            _ => None    
        }
    }

    fn head_cell(&self) -> Option<heap::SharedBox<DoubleLinkedListCell<T>>> {
        self.head
    }

    fn tail_cell(&self) -> Option<heap::SharedBox<DoubleLinkedListCell<T>>> {
        self.tail
    }

    /// Determines if this linked list consists only of DoubleLinkedListCell::Nil    
    pub fn is_nil(&self) -> bool {
        self.head.is_none() && self.tail.is_none()
    }

    /// Determines if this linked list contains any DoubleLinkedListCell::Cell
    pub fn is_cell(&self) -> bool {
        !self.is_nil()
    }

    /// Determines if this linked list consists only of one DoubleLinkedListCell::Cell
    pub fn is_one_cell(&self) -> bool {
        self.head.map(|e| e.single_cell()).unwrap_or(false) && 
        self.tail.map(|e| e.single_cell()).unwrap_or(false)
    }

    pub fn head_equals_tail(&self) -> bool {
        // head is equal to tail in two cases:
        // 1: they are both pointing to DoubleLinkedList::Nil
        // 2: DoubleLinkedList::is_end() is true for `self.head` (start cell is also a end cell) and
        //    DoubleLinkedList:is_start() is true for `self.tail` (end cell is also a start cell)
        self.is_nil() || self.is_one_cell()
    }

    /// # Arguments    
    /// * `memory_allocator` - memory allocator    
    pub fn remove_head<A>(&mut self, memory_allocator : &mut A) where A : MemoryAllocator {
        // calling this before self.head.take_next is important to
        // prevent reading freed memory!
        if !self.is_nil() {
            let head_equals_tail = self.is_one_cell();
            let result = self.head.unwrap().remove_next(memory_allocator);
            
            if head_equals_tail {
                self.head = result;
                self.tail = result;
            }
            else {
                self.head = result;
            }          
        }
    }
    
    /// # Arguments
    /// * `memory_allocator` - memory allocator
    pub fn remove_tail<A>(&mut self, memory_allocator : &mut A) where A : MemoryAllocator{
        if !self.is_nil() {
            // calling this before self.head.take_next is important to
            // prevent reading freed memory!
            let head_equals_tail = self.is_one_cell();
            let result = self.tail.unwrap().remove_prev(memory_allocator);
            
            if head_equals_tail {
                self.head = result;
                self.tail = result;
            }
            else {
                self.tail = result;
            }            
        }        
    }    
}

impl<T> DoubleLinkedList<T> where T : Copy {
    
    /// Deletes current `self.head` from memory and returns copy of its data if it was DoubleLinkedList::Cell.
    /// Returns None otherwise.
    /// # Arguments    
    /// * `memory_allocator` - memory allocator
    pub fn take_head<A>(&mut self, memory_allocator : &mut A) -> Option<T> where A : MemoryAllocator {
        if !self.is_nil() {
            // calling this before self.head.take_next is important to
            // prevent reading freed memory!
            let head_equals_tail = self.is_one_cell();
            let (result, new_head) = self.head.unwrap().take_next(memory_allocator);

            if head_equals_tail {
                    self.head = new_head;
                    self.tail = new_head;
                }
                else {
                    self.head = new_head;
                };

            Some(result)
        }
        else {
            None
        }        
    }
}

impl<T> fmt::Display for DoubleLinkedList<T> where T : fmt::Display + Copy {    
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.iterator().fold(write!(f, ""), |_base, e| write!(f, "entry: {} ", e))        
    }    
}

impl<T> Iterable for DoubleLinkedList<T> where T : Copy {
    
    type Item = T;

    type IntoIter = DoubleLinkedListIterator<T>;

    fn iterator(&self) -> DoubleLinkedListIterator<T> {
        DoubleLinkedListIterator::new(self.head_cell())
    }
}

impl<T> Sequence for DoubleLinkedList<T> where T : Copy {
    
    fn length(&self) -> usize {
        self.iterator().count()
    }

    fn cell_size() -> usize {
        mem::size_of::<DoubleLinkedListCell<T>>()
    }

    fn mem_size_for(length : usize) -> usize {
        let cell_size = Self::cell_size();
        cell_size * length + 2 * cell_size // 2 cells are for DoubleLinkedList::Nil
    }
}

#[repr(C, packed)]
struct DoubleLinkedListCell<T> {
    value : T,
    prev : Option<heap::SharedBox<DoubleLinkedListCell<T>>>,
    next : Option<heap::SharedBox<DoubleLinkedListCell<T>>>
}

impl<T> DoubleLinkedListCell<T> {

    pub fn new<A>(value: T, memory_allocator : &mut A) -> heap::SharedBox<Self> where A : MemoryAllocator {
        let new_cell = DoubleLinkedListCell {
                value : value,
                next  : None,
                prev  : None
        };

        heap::SharedBox::new(new_cell, memory_allocator)
    }

    pub fn add<A>(&mut self, value: T, memory_allocator : &mut A) -> heap::SharedBox<Self> where A : MemoryAllocator {        
        let new_cell = DoubleLinkedListCell {
                value : value,
                next  : None,
                prev  : Some(heap::SharedBox::from_pointer(self))
        };

        let result = heap::SharedBox::new(new_cell, memory_allocator);

        self.next = Some(result);
        result
    }

    /// Deletes this DoubleLinkedList from memory. Returns `prev` and `next` pointers if this was a
    /// DoubleLinkedList::Cell, returns None otherwise.    
    /// # Arguments
    /// * `memory_allocator` - memory allocator
    /// # Warning : modifies cells pointed by `self.next` and `self.prev`
    pub fn remove<A>(&mut self, memory_allocator : &mut A) -> (Option<heap::SharedBox<Self>>, Option<heap::SharedBox<Self>>)
    where A : MemoryAllocator {
        if let Some(mut next) = self.next {
            next.prev = self.prev;
        }

        if let Some(mut prev) = self.prev {
            prev.next = self.next;
        }                 
                
        let result = (self.prev, self.next);

        memory_allocator.free(self as *const _ as usize);
        result
    }

    /// Deletes this DoubleLinkedList from memory. Returns `prev` pointer if this was a
    /// DoubleLinkedList::Cell, returns None otherwise.
    /// # Arguments
    /// * `memory_allocator` - memory allocator
    /// # Warning : modifies cells pointed by `self.next` and `self.prev`
    pub fn remove_prev<A>(&mut self, memory_allocator : &mut A) -> Option<heap::SharedBox<Self>>
    where A : MemoryAllocator {
        self.remove(memory_allocator).0
    }

    /// Deletes this DoubleLinkedList from memory. Returns `next` pointer if this was a
    /// DoubleLinkedList::Cell, returns None otherwise.
    /// # Arguments
    /// * `memory_allocator` - memory allocator
    /// # Warning : modifies cells pointed by `self.next` and `self.prev`
    pub fn remove_next<A>(&mut self, memory_allocator : &mut A) -> Option<heap::SharedBox<Self>>
    where A : MemoryAllocator {
        self.remove(memory_allocator).1
    }    

    pub fn single_cell(&self) -> bool {
        self.prev.is_none() && self.next.is_none()
    }

    /// Determines if this type is DoubleLinkedList::Cell which has `prev` pointing to DoubleLinkedList::Nil
    pub fn is_start(&self) -> bool {
        self.prev.is_none()
    }

    /// Determines if this type is DoubleLinkedList::Cell which has `next` pointing to DoubleLinkedList::Nil
    pub fn is_end(&self) -> bool {
        self.next.is_none()
    }

    pub fn value_ref(&self) -> &T {
        &self.value
    }
}

impl<T> DoubleLinkedListCell<T> where T : Copy {

    pub fn value(&self) -> T {
        self.value
    }

    /// Returns copy of the cell data if `self` is DoubleLinkedList::Cell then removes this from linked list,
    /// returns None if `self` is DoubleLinkedList::Cell.
    /// # Arguments
    /// * `memory_allocator` - memory allocator
    /// # Warning : modifies cells pointed by `self.next` and `self.prev`
    pub fn take<A>(&self, memory_allocator : &mut A) -> (T, Option<heap::SharedBox<Self>>, Option<heap::SharedBox<Self>>) where A : MemoryAllocator {

        if let Some(mut next) = self.next {
            next.prev = self.prev;
        }

        if let Some(mut prev) = self.prev {
            prev.next = self.next;
        }                 
                
        let result = (self.value, self.prev, self.next);

        memory_allocator.free(self as *const _ as usize);
        result
    }

    /// Returns copy of the cell data and pointer to previous DoubleLinkedList
    /// if `self` is DoubleLinkedList::Cell then removes this from linked list,
    /// returns None if `self` is DoubleLinkedList::Cell.
    /// # Arguments
    /// * `memory_allocator` - memory allocator
    /// # Warning : modifies cells pointed by `self.next` and `self.prev`
    pub fn take_prev<A>(&self, memory_allocator : &mut A) -> (T, Option<heap::SharedBox<Self>>) where A : MemoryAllocator {
        let (value, prev, _) = self.take(memory_allocator);
        (value, prev)
    }

    /// Returns copy of the cell data and pointer to next DoubleLinkedList
    /// if `self` is DoubleLinkedList::Cell then removes this from linked list,
    /// returns None if `self` is DoubleLinkedList::Cell.
    /// # Arguments
    /// * `memory_allocator` - memory allocator
    /// # Warning : modifies cells pointed by `self.next` and `self.prev`
    pub fn take_next<A>(&self, memory_allocator : &mut A) -> (T, Option<heap::SharedBox<Self>>) where A : MemoryAllocator {
        let (value, _, next) = self.take(memory_allocator);
        (value, next)    
    }
}

pub struct DoubleLinkedListIterator<T> {
    current : Option<heap::SharedBox<DoubleLinkedListCell<T>>>,
    p : marker::PhantomData<T>
}

impl<T> DoubleLinkedListIterator<T> {
    fn new(head: Option<heap::SharedBox<DoubleLinkedListCell<T>>>) -> DoubleLinkedListIterator<T> {
        DoubleLinkedListIterator { 
            current : head,
            p : marker::PhantomData
        }
    }
}

impl<T> iter::Iterator for DoubleLinkedListIterator<T> where T : Copy {
    
    type Item = T;

    fn next(&mut self) -> Option<T> {
        match self.current {
            Some(cell) => {
                self.current = cell.next;
                Some(cell.value)
            },
            _ => None
        }
    }
}

impl<T> iterator::IteratorExt for DoubleLinkedListIterator<T> where T : Copy { }

pub struct UsizeLinkedMap<T> {
    frame_to_free_buddy : Array<Option<heap::SharedBox<DoubleLinkedListCell<T>>>>,
    free_blocks         : DoubleLinkedList<T>,
}

impl<T> UsizeLinkedMap<T> where T : Copy {
    pub fn new<A>(length : usize, memory_allocator : &mut A) -> Self 
    where A : MemoryAllocator {
        let mut array = Array::new(length, memory_allocator);

        // set list as fully occupied
        array.fill_value(None);

        UsizeLinkedMap {
            frame_to_free_buddy : array,
            free_blocks         : DoubleLinkedList::new(),            
        }
    }

    pub fn mem_size_for_array(length : usize) -> usize {
        Array::<heap::SharedBox<DoubleLinkedListCell<T>>>::mem_size_for(length)
    }

    pub fn mem_size_for_linked_list(length : usize) -> usize {
        DoubleLinkedList::<T>::mem_size_for(length)
    }    

    pub fn has_key(&self, index : usize) -> bool {
        self.frame_to_free_buddy[index].is_some()
    }

    pub fn has_value(&self) -> bool {
        self.free_blocks.is_cell()
    }

    /// Sets the block as occupied
    /// # Arguments
    /// * `block_start_address` - start address of memory block
    /// * `memory_allocator` - memory allocator
    pub fn remove<A>(&mut self, index : usize, memory_allocator : &mut A)
    where A : MemoryAllocator {        
        if let Some(cell) = self.frame_to_free_buddy.value(index) {            
            self.remove_free_block(cell, memory_allocator);
            self.frame_to_free_buddy.update(index, None);        
        }
    }

    /// Sets the block as free to use
    /// # Arguments
    /// * `block_start_address` - start address of memory block
    /// * `memory_allocator` - memory allocator
    pub fn add_if_no_key<A>(&mut self, index : usize, value : T, memory_allocator : &mut A) 
    where A : MemoryAllocator {
        if !self.has_key(index) {
            let cell = self.free_blocks.add_to_tail_inner(value, memory_allocator);            
            self.frame_to_free_buddy.update(index, Some(cell));
        }
    }

    fn remove_free_block<A>(&mut self, mut cell : heap::SharedBox<DoubleLinkedListCell<T>>, memory_allocator : &mut A)
    where A : MemoryAllocator {
        if self.free_blocks.head_equals_tail() && cell.is_start() {
            self.free_blocks.remove_head(memory_allocator);            
        }
        else if cell.is_start() {
            self.free_blocks.remove_head(memory_allocator);
        }
        else if cell.is_end() {
            self.free_blocks.remove_tail(memory_allocator);            
        }
        else {
            let c : &mut DoubleLinkedListCell<T> = cell.pointer_mut();
            c.remove(memory_allocator);
        }
    }
}

impl<T> Iterable for UsizeLinkedMap<T> where T : Copy {
    
    type Item = T;

    type IntoIter = DoubleLinkedListIterator<T>;

    fn iterator(&self) -> DoubleLinkedListIterator<T> {
         unimplemented!();
    }
}

impl<T> Sequence for UsizeLinkedMap<T> where T : Copy {
    
    fn length(&self) -> usize {
        self.frame_to_free_buddy.length()
    }

    fn cell_size() -> usize {
        DoubleLinkedList::<T>::cell_size()
    }
}

pub struct BuddyMap(pub UsizeLinkedMap<usize>);

impl BuddyMap {
    
    /// Returns first unused memory block if any.
    /// # Arguments
    /// * `memory_allocator` - memory allocator
    pub fn first_free_block<A>(&mut self, memory_allocator : &mut A) -> Option<usize> 
    where A : MemoryAllocator{
        let result = self.0.free_blocks.take_head(memory_allocator);

        if let Some(index) = result {            
            self.0.frame_to_free_buddy.update(index, None);
        }

        result
    }
    
    pub fn add_if_no_key<A>(&mut self, index : usize, memory_allocator : &mut A) 
    where A : MemoryAllocator {
        self.0.add_if_no_key(index, index, memory_allocator);        
    }

    pub fn mem_size_for_array(length : usize) -> usize {
        Array::<heap::SharedBox<DoubleLinkedListCell<usize>>>::mem_size_for(length)
    }

    pub fn mem_size_for_linked_list(length : usize) -> usize {
        DoubleLinkedList::<usize>::mem_size_for(length)
    }
}

impl Iterable for BuddyMap {
    
    type Item = usize;

    type IntoIter = DoubleLinkedListIterator<usize>;

    fn iterator(&self) -> DoubleLinkedListIterator<usize> {
         unimplemented!();
    }
}

impl Sequence for BuddyMap {
    
    fn length(&self) -> usize {
        self.0.length()
    }

    fn cell_size() -> usize {
        DoubleLinkedList::<usize>::cell_size()
    }
}