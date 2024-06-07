//! Guarantees the the AliveIndex points to the same Value like at creation time

use std::{marker::PhantomData, sync::atomic::{AtomicUsize, Ordering}};

use crate::{ValuePool, ValueRef};

pub struct AliveIndex<'a, T> {
    idx: ValueRef<ValuePoolEntry<T>>,
    counter: *mut usize,
    _phantom: PhantomData<&'a ()>,
}

impl<'a, T> AliveIndex<'a, T> {
    fn get_counter(&self) -> &AtomicUsize {
        unsafe {&AtomicUsize::from_ptr(self.counter)} // Safe?
    }
}
impl<'a, T> Drop for AliveIndex<'a, T>{
    fn drop(&mut self) {
        self.get_counter().fetch_sub(1, Ordering::SeqCst);
    }
}
impl<'a, T> Clone for AliveIndex<'a, T>{
    fn clone(&self) -> Self {
        self.get_counter().fetch_add(1, Ordering::SeqCst);
        AliveIndex {
            idx: self.idx,
            counter: self.counter,
            _phantom: PhantomData,
        }
    }
}

pub struct ValuePoolEntry<T> {
    value: T,
    active_references: AtomicUsize
}

impl<T> ValuePoolEntry<T>{
    fn new(value: T) -> Self {
        ValuePoolEntry {
            value,
            active_references: AtomicUsize::new(1),
        }
    }
}

pub struct AliveValuePool<T>   {
    pool: ValuePool<ValuePoolEntry<T>>,
}

impl<T> AliveValuePool<T> {
    pub fn new() -> Self {
        AliveValuePool {
            pool: ValuePool::new(),
        }
    }

    fn access_counter(&self, idx: ValueRef<ValuePoolEntry<T>>) -> *mut usize{
        let counter = &self.pool.get(idx).unwrap().active_references; // looks fucking dangerous
        //counter.fetch_add(1, Ordering::SeqCst);
        counter.as_ptr()
    }

    pub fn push<'a>(&mut self, value: T) -> AliveIndex<'a, T> {
        let idx = self.pool.push(ValuePoolEntry::new(value));
        AliveIndex {
            idx,
            counter: self.pool.get(idx).unwrap().active_references.as_ptr(),
            _phantom: PhantomData,
        }
    }

    pub fn get<'a>(&self, index: impl Into<AliveIndex<'a, T>>) -> &T {
        let index: AliveIndex<'a, T> = index.into();
        &self.pool.get(index.idx).unwrap().value
    }

    pub fn get_mut<'a>(&mut self, index: impl Into<AliveIndex<'a, T>>) -> &mut T {
        let index: AliveIndex<'a, T> = index.into();
        &mut self.pool.get_mut(index.idx).unwrap().value
    }

    pub fn take<'a>(&mut self, index: impl Into<AliveIndex<'a, T>>) -> Option<T> {
        let index: AliveIndex<'a, T> = index.into();
        if index.get_counter().load(Ordering::SeqCst) == 1 {
            self.pool.take(index.idx).and_then(|x| Some(x.value))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;

    use super::{AliveIndex, AliveValuePool};

    #[test]
    fn test_general(){
        let mut pool = AliveValuePool::new();
        let zero_idx = pool.push(0);

        let two_idx = pool.push(2);

        assert_eq!(pool.get(zero_idx.clone()), &0);
        *pool.get_mut(two_idx.clone()) = 22;

        assert_eq!(pool.get(two_idx.clone()), &22);
    }

    #[test]
    fn test_counting(){
        let mut pool = AliveValuePool::new();
        let zero_idx = pool.push(0);
        {
            let two_idx = pool.push(2);
            let second_two_idx = two_idx.clone();

            assert_eq!(pool.pool.get(two_idx.idx).unwrap().active_references.load(Ordering::Acquire), 2);
            assert_eq!(pool.take(second_two_idx), None);
            assert_eq!(pool.pool.get(two_idx.idx).unwrap().active_references.load(Ordering::Acquire), 1);

            assert_eq!(pool.take(two_idx), Some(2));
        }
        
        assert_eq!(pool.take(zero_idx), Some(0));
    }
}