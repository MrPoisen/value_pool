use std::marker::PhantomData;

use crate::{ValuePool, ValueRef};

#[derive(Debug, Clone)]
pub struct AliveIndex<'a, T> {
    idx: ValueRef<T>,
    _phantom: PhantomData<&'a ()>,
}

pub struct AliveValuePool<T>   {
    pool: ValuePool<T>,
}

impl<'a, T> From<AliveIndex<'a, T>> for ValueRef<T> {
    fn from(value: AliveIndex<T>) -> Self {
        value.idx
    }
}

impl<T> AliveValuePool<T> {
    pub fn new() -> Self {
        Self {
            pool: ValuePool::new(),
        }
    }

    pub fn push<'a>(&mut self, value: T) -> AliveIndex<'a, T> {
        let idx = self.pool.push(value);
        AliveIndex {
            idx,
            _phantom: PhantomData,
        }
    
    }

    pub fn get<'a>(&self, index: impl Into<AliveIndex<'a, T>>) -> &T {
        let index = index.into();
        self.pool.get(index).unwrap() // unwrap_unchecked should be possible
    }

    pub fn get_mut<'a>(&mut self, index: impl Into<AliveIndex<'a, T>>) -> &mut T {
        let index = index.into();
        self.pool.get_mut(index).unwrap() // unwrap_unchecked should be possible
    }

    pub fn swap<'a>(&mut self, index1: impl Into<AliveIndex<'a, T>>, index2: impl Into<AliveIndex<'a, T>>) {
        let index1 = index1.into();
        let index2 = index2.into();
        self.pool.swap(index1, index2);
    }
    pub fn replace<'a>(&mut self, index: impl Into<AliveIndex<'a, T>>, value: T) -> T {
        todo!()
    }
        
}

#[cfg(test)]
mod tests {
    use super::{AliveIndex, AliveValuePool};

    #[test]
    fn test_general(){
        let mut pool = AliveValuePool::new();
        let zero_idx = pool.push(0);

        let two_idx = pool.push(2);

        assert_eq!(pool.get(zero_idx.clone()), &0);
        *pool.get_mut(two_idx.clone()) = 22;

        assert_eq!(pool.get(two_idx.clone()), &22);

        pool.swap(zero_idx.clone(), two_idx.clone());

        assert_eq!(pool.get(zero_idx), &22);
        assert_eq!(pool.get(two_idx), &0);
    }
}