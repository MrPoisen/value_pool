//! This module implements [`SmartValuePool<T>`] which can automatically call a function if a method call changes it state from empty to one element or vice versa.
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::{ValuePool, ValueRef};

/// [`SmartValuePool<T, O>`] can automatically call a function if a method call changes it state from empty to one element or vice versa.
/// ```
/// use value_pool::{ValuePool, smart_value_pool::SmartValuePool, ValueRef};
/// 
/// fn on_empty<T>(pool: &mut ValuePool<T>, text: &mut String) {
///     println!("Waiting positions in now empty pool: {}", pool.waiting_positions());
///     text.push_str("|Called on_empty|");
/// }
/// 
/// fn on_empty_push<T>(pool: &mut ValuePool<T>, reference: ValueRef<T>, text: &mut String){
///     println!("Waiting positions in now 1 element pool: {}", pool.waiting_positions());
///     text.push_str("|Called on_empty_push|");
/// }
/// 
/// let mut pool: SmartValuePool<usize, String> = SmartValuePool::make_smart(ValuePool::new(), on_empty, on_empty_push);
/// let mut text = "Start: ".to_string();
/// let three_ref = pool.smart_push(3usize, &mut text); // prints "Waiting positions in now 1 element pool: 0"
/// assert_eq!(&text, "Start: |Called on_empty_push|");
/// assert_eq!(pool.waiting_positions(), 0);
/// let four_ref = pool.smart_push(4usize, &mut text);
/// let five_ref = pool.push(5); // No check will happen
/// let six_ref = pool.push(6); // No check will happen
/// 
/// pool.remove(three_ref); // No check
/// pool.smart_remove(four_ref, &mut text);
/// pool.smart_remove(five_ref, &mut text);
/// pool.smart_remove(six_ref, &mut text); // prints "Waiting positions in now empty pool: 3"
/// // Why 3 waiting positions?: `six_ref` is stored last, so instead of marking its position as empty,
/// // we remove its position.
/// // Note: This does **not** reduce the used memory of this `SmartValuePool<T>`
/// assert_eq!(&text, "Start: |Called on_empty_push||Called on_empty|");
/// assert_eq!(pool.waiting_positions(), 3);
/// ```
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
