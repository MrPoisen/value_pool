use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

// IDEA: same as value_pool, but it simplifies self-referential structs (especially when the first value is pushed)
use crate::{ValuePool, ValueRef};

// TODO: more/better documentation
#[derive(Debug)]
pub struct SmartValuePool<T, O> {
    pool: ValuePool<T>,
    on_empty: fn(&mut ValuePool<T>, &mut O),
    on_empty_push: fn(&mut ValuePool<T>, ValueRef<T>, &mut O),
    object_type: PhantomData<O>,
}

impl<T, O> Deref for SmartValuePool<T, O> {
    type Target = ValuePool<T>;
    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}

impl<T, O> DerefMut for SmartValuePool<T, O> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.pool
    }
}

impl<T, O> SmartValuePool<T, O> {
    /// Creates a `SmartValuePool` with the given functions.
    #[inline]
    pub fn make_smart(
        pool: ValuePool<T>,
        on_empty: fn(&mut ValuePool<T>, &mut O),
        on_empty_push: fn(&mut ValuePool<T>, ValueRef<T>, &mut O),
    ) -> SmartValuePool<T, O> {
        SmartValuePool {
            pool,
            on_empty,
            on_empty_push,
            object_type: (PhantomData),
        }
    }
    /// Same as [`ValuePool<T>::push`] but it will call the previously given `on_empty_push` if needed
    #[inline]
    pub fn smart_push(&mut self, value: T, object: &mut O) -> ValueRef<T> {
        let tmp = self.pool.push(value);
        if self.pool.element_count() == 1 {
            (self.on_empty_push)(&mut self.pool, tmp, object);
        }
        tmp
    }
    
    /// Same as [`ValuePool<T>::take`] but it will call the previously given `on_empty` if needed
    #[inline]
    pub fn smart_take(&mut self, reference: ValueRef<T>, object: &mut O) -> Option<T> {
        let tmp = self.pool.take(reference);
        if self.is_empty() {
            (self.on_empty)(&mut self.pool, object);
        }
        tmp
    }

    /// Same as [`ValuePool<T>::take_unchecked`] but it will call the previously given `on_empty` if needed
    /// 
    /// # Safety
    /// Calling this method with an reference that is out of bounds, is UB. You can check beforehand with [`ValuePool::is_ref_in_bounce`].
    #[inline]
    pub unsafe fn smart_take_unchecked(
        &mut self,
        reference: impl Into<ValueRef<T>>,
        object: &mut O,
    ) -> Option<T> {
        let tmp = self.take_unchecked(reference);
        if self.is_empty() {
            (self.on_empty)(&mut self.pool, object);
        }
        tmp
    }

    /// Same as [`ValuePool<T>::remove`] but it will call the previously given `on_empty` if needed
    #[inline]
    pub fn smart_remove(
        &mut self,
        reference: impl Into<ValueRef<T>>,
        object: &mut O,
    )  {
        self.remove(reference);
        if self.is_empty() {
            (self.on_empty)(&mut self.pool, object);
        }
        
    }

}
