use core::marker::PhantomData;
use core::ptr::read;
use core::iter;

pub struct TagEntryIterator<T> {
    phantom: PhantomData<T>,
    entry_address: usize,
    tag_end_address: usize,
    entry_size: usize,
}

impl<T> TagEntryIterator<T> {
    pub fn new(entry_address: usize,
               tag_end_address: usize,
               entry_size: usize)
               -> TagEntryIterator<T> {
        TagEntryIterator {
            phantom: PhantomData,
            entry_address: entry_address,
            tag_end_address: tag_end_address,
            entry_size: entry_size,
        }
    }
}

impl<T> iter::Iterator for TagEntryIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.entry_address >= self.tag_end_address {
            None
        } else {
            let result = unsafe { Some(read(self.entry_address as *const T)) };
            self.entry_address += self.entry_size;
            result
        }
    }
}