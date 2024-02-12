#![cfg_attr(feature = "nightly", feature(btree_cursors))]
//! This libraries allows easy use of self-referencial structs by storing them in one place, the `ValuePool<T>`
//! and referencing the stored values with `UntypedValueRef` or `ValueRef<T>`.
//!
//! # Showcase
//! ```
//! use value_pool::{ValueRef, ValuePool,UntypedValueRef};
//! let mut pool: ValuePool<u32> = ValuePool::new();
//! let ref_to_first: ValueRef<u32> = pool.push(12);
//!
//! // you can convert ValueRef<T> to UntypedValueRef and the other way round. UntypedValueRef is useful if the type information of ValueRef<T> gets in your way
//! let untyped_ref_to_first: UntypedValueRef = ref_to_first.into();
//! let wrongly_typed_ref_to_first: ValueRef<u8> = untyped_ref_to_first.into();  // original type information is lost
//! // notice the wrong type of `wrongly_typed_ref_to_first`
//! //pool.get(wrongly_typed_ref_to_first); // compile time error: Trait From<ValueRef<u8>> is not implemented for ValueRef<u32>
//!
//! assert_eq!(pool.get(ref_to_first), Some(&12));
//! assert_eq!(pool.element_count(), 1);
//! // you can take this value
//! assert_eq!(pool.take(ref_to_first), Some(12));
//! assert_eq!(pool.element_count(), 0);
//!
//! let mut ref_to_13 = pool.push(13);
//! let mut ref_to_14 = pool.push(14);
//! let copy_ref_to_14 = ref_to_14;
//!
//! // pool.swap is marked unsafe cause it causes all references to 14 (and 13) to point to the wrong value, except for the returned refs
//! unsafe{(ref_to_13,ref_to_14) = pool.swap(ref_to_13, ref_to_14).unwrap();}
//! assert_eq!(ref_to_13, copy_ref_to_14);
//!
//! // unsafe cause now ref_to_13 will be invalid (all refs >= ref_to_14)
//! unsafe{pool.remove_full(ref_to_14);}
//! assert!(ref_to_13 > ref_to_14);
//! assert_eq!(pool.find(&13).unwrap(), ref_to_14);
//! assert_eq!(pool.find(&13).unwrap(), ValueRef::new(0));
//! ```
use std::{borrow::Borrow, hash::Hash, marker::PhantomData, ops::Deref};

#[cfg(feature = "double_linked_list")]
pub mod linked_list;
#[allow(unused_imports)]
#[cfg(feature = "double_linked_list")]
use linked_list::{DoubleLinkedList, DoubleLinkedView};

/// Struct that stores a location of an item in ValuePool. It implements Copy.
///
/// Usually, you get this struct by with `from` or `into`:
/// ```
/// use value_pool::{UntypedValueRef, ValueRef};
///
/// let value_ref: ValueRef<usize> = ValueRef::new(2);
///
/// let untyped_value_ref: UntypedValueRef = value_ref.into();
/// assert_eq!(*untyped_value_ref, *value_ref); // * returns the stored location (usize)
///
/// //or
/// let untyped_value_ref = UntypedValueRef::new(2); // usually not needed or recommended
/// assert_eq!(*untyped_value_ref, 2usize);
/// ```
#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UntypedValueRef {
    index: usize,
}

impl UntypedValueRef {
    pub fn new(index: usize) -> UntypedValueRef {
        UntypedValueRef { index }
    }
}
impl<T> From<ValueRef<T>> for UntypedValueRef {
    fn from(value: ValueRef<T>) -> Self {
        UntypedValueRef {
            index: (value.index),
        }
    }
}
impl<T> From<UntypedValueRef> for ValueRef<T> {
    fn from(value: UntypedValueRef) -> Self {
        ValueRef {
            index: (value.index),
            type_info: (PhantomData),
        }
    }
}

impl Deref for UntypedValueRef {
    type Target = usize;
    fn deref(&self) -> &usize {
        &self.index
    }
}

impl<T> Deref for ValueRef<T> {
    type Target = usize;
    fn deref(&self) -> &usize {
        &self.index
    }
}
/// Struct that stores a location of an item in ValuePool as well as the Type.
///
/// Usually, you get this struct rough methods from ValuePool. It implements Copy.
/// ```
/// use value_pool::{UntypedValueRef, ValueRef, ValuePool};
///
/// let mut pool: ValuePool<usize> = ValuePool::new();
///
/// let value_ref_2: ValueRef<usize> = pool.push(2);
///  
/// //or
/// let value_ref: ValueRef<usize> = ValueRef::new(4);
/// assert_eq!(*value_ref, 4usize);
/// assert_eq!(pool.get(value_ref), None);
/// ```
///
/// Trough the type information, you gain safety.
/// ```compile_fail
/// use value_pool::{UntypedValueRef, ValueRef, ValuePool};
///
/// let mut pool: ValuePool<usize> = ValuePool::new();
///
/// pool.push(2);
///
/// let value_ref: ValueRef<u32> = ValueRef::new(4); // usually not needed or recommended
/// assert_eq!(pool.get(value_ref), None); // Compiler error her, pool stores `usize`, but valze_ref is `ValueRef<u32>`
/// ```
#[derive(Debug)]
pub struct ValueRef<T> {
    index: usize,
    type_info: PhantomData<T>,
}

impl<T> Hash for ValueRef<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(self.index);
    }
}

impl<T> Clone for ValueRef<T> {
    fn clone(&self) -> Self {
        ValueRef {
            index: (self.index),
            type_info: (self.type_info.clone()),
        }
    }
}
impl<T> Copy for ValueRef<T> {}

impl<T> PartialOrd for ValueRef<T> {
    fn ge(&self, other: &Self) -> bool {
        self.index >= other.index
    }
    fn lt(&self, other: &Self) -> bool {
        self.index < other.index
    }
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.index == other.index {
            return Some(std::cmp::Ordering::Equal);
        }
        if self.index > other.index {
            return Some(std::cmp::Ordering::Greater);
        }
        // self.index < other.index
        Some(std::cmp::Ordering::Less)
    }
}
impl<T> Ord for ValueRef<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl<T> ValueRef<T> {
    pub fn new(index: usize) -> ValueRef<T> {
        ValueRef {
            index: (index),
            type_info: (PhantomData),
        }
    }
}

impl<T> PartialEq for ValueRef<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}
impl<T> Eq for ValueRef<T> {}

#[derive(Debug, Clone)]
pub struct ValuePool<T> {
    store: Vec<Option<T>>,
    open_indicis: Vec<usize>,
}

impl<T> ValuePool<T> {
    /// Creates a new ValuePool that can store `capacity` many items
    pub fn with_capacity(capacity: usize) -> ValuePool<T> {
        ValuePool {
            store: (Vec::with_capacity(capacity)),
            open_indicis: (Vec::with_capacity(capacity / 4)),
        }
    }
    /// Creates a new, empty ValuePool
    pub fn new() -> ValuePool<T> {
        ValuePool {
            store: (Vec::new()),
            open_indicis: (Vec::new()),
        }
    }

    /// Returns the number of elements stored in this ValuePool
    pub fn element_count(&self) -> usize {
        self.store.len() - self.open_indicis.len()
    }

    /// Returns the number of items that can be stored before reallocation happens
    pub fn capacity(&self) -> usize {
        self.store.capacity()
    }

    /// Returns the number of positions that are currently empty. These positions are prioritized when pushing new values.
    pub fn waiting_positions(&self) -> usize {
        self.open_indicis.len()
    }

    /// Pushes a new value into the ValuePool and returns a `ValueRef<T>` (that stores its position)
    /// You can access this value with `get`.
    pub fn push(&mut self, value: T) -> ValueRef<T> {
        if self.open_indicis.len() != 0 {
            let index = self.open_indicis.pop().unwrap();
            self.store[index] = Some(value);
            return ValueRef::new(index);
        } else {
            self.store.push(Some(value));
            return ValueRef::new(self.store.len() - 1);
        }
    }

    /// Removes an item from ValuePool
    pub fn remove(&mut self, reference: impl Into<ValueRef<T>>) {
        let reference: ValueRef<T> = reference.into();
        if !self.has_item(reference) {
            return;
        }
        // if reference is the last index and has a value; this can prevent reallocation of self.open_indicis
        if reference.index + 1 == self.store.len() {
            self.store.pop();
            return;
        }

        if let Some(value) = self.store.get_mut(reference.index) {
            // if value is none, reference.index should already be in self.open_indicis
            if value.is_some() {
                self.open_indicis.push(reference.index);
                *value = None;
            }
        }
    }

    /// # Important
    /// INVALIDATES all ValueRefs greater equal then reference
    pub unsafe fn remove_full(&mut self, reference: impl Into<ValueRef<T>>) -> Option<T> {
        let reference: ValueRef<T> = reference.into();
        self.store.swap_remove(reference.index)
    }

    /// gets a borrow of the item pointed to by `reference` if it exists
    #[inline]
    pub fn get(&self, reference: impl Into<ValueRef<T>>) -> Option<&T> {
        self.store
            .get(reference.into().index)
            .map(|x| x.as_ref())
            .flatten()
    }

    #[inline]
    pub unsafe fn get_unchecked(&self, reference: impl Into<ValueRef<T>>) -> Option<&T> {
        let reference: ValueRef<T> = reference.into();
        self.store.get_unchecked(reference.index).as_ref()
    }

    /// gets a mut borrow of the item pointed to by `reference` if it exists
    #[inline]
    pub fn get_mut(&mut self, reference: impl Into<ValueRef<T>>) -> Option<&mut T> {
        self.store
            .get_mut(reference.into().index)
            .map(|x| x.as_mut())
            .flatten()
    }

    #[inline]
    pub unsafe fn get_unchecked_mut(
        &mut self,
        reference: impl Into<ValueRef<T>>,
    ) -> Option<&mut T> {
        let reference: ValueRef<T> = reference.into();
        self.store.get_unchecked_mut(reference.index).as_mut()
    }

    /// swaps ref_1 with ref_2, all other refs equal two the both will point to the wrong element
    pub unsafe fn swap(
        &mut self,
        ref_1: impl Into<ValueRef<T>>,
        ref_2: impl Into<ValueRef<T>>,
    ) -> Option<(ValueRef<T>, ValueRef<T>)> {
        let (ref_1, ref_2) = (ref_1.into(), ref_2.into());
        if ref_1.index >= self.store.len() || ref_2.index >= self.store.len() {
            return None;
        }
        self.store.swap(ref_1.index, ref_2.index);
        Some((ref_2, ref_1))
    }

    /// returns the value_ref value the next call to ValuePool::push would return
    pub fn next_push_ref(&self) -> ValueRef<T> {
        if self.open_indicis.len() == 0 {
            return ValueRef::new(self.store.len());
        }
        #[cfg(feature="unsafe")]
        unsafe{return ValueRef::new(*self.open_indicis.last().unwrap_unchecked());}
        #[cfg(not(feature="unsafe"))]
        {return ValueRef::new(*self.open_indicis.last().unwrap());}
    }

    /// Takes value at reference and returns it. if the returned value is Some, then calling it again with the input will return None
    /// ```
    /// use value_pool::ValuePool;
    /// let mut pool: ValuePool<usize> = ValuePool::new();
    ///
    /// pool.push(2);
    /// let ref_to_3 = pool.push(3);
    /// pool.push(4);
    /// let taken_three = pool.take(ref_to_3);
    /// assert_eq!(taken_three, Some(3usize));
    /// assert_eq!(pool.take(ref_to_3), None);
    /// ```
    pub fn take(&mut self, reference: impl Into<ValueRef<T>>) -> Option<T> {
        let mut tmp = None;
        let reference: ValueRef<T> = reference.into();
        std::mem::swap(&mut tmp, self.store.get_mut(reference.index)?);
        if tmp.is_some() {
            // if tmp is none, reference.index should already be in self.open_indicis
            self.open_indicis.push(reference.index);
        }
        tmp
    }

    /// Ensures at least `additional` elements can be stored without reallocation
    pub fn reserve(&mut self, additional: usize) {
        self.store.reserve(additional);
    }

    /// Returns true, if an item is stored at `reference`
    /// Equilivant to `ValuePool::get(&mut pool, reference).is_some()`
    pub fn has_item(&self, reference: impl Into<ValueRef<T>>) -> bool {
        self.get(reference).is_some()
    }

    pub fn find<Q: Eq>(&self, value: &Q) -> Option<ValueRef<T>>
    where
        T: Borrow<Q>,
    {
        Some(ValueRef {
            index: (self
                .store
                .iter()
                .position(|v| v.as_ref().is_some_and(|x| *x.borrow() == *value))?),
            type_info: (PhantomData),
        })
    }

    pub fn clear(&mut self) {
        self.open_indicis.clear();
        self.store.clear();
    }
}

#[cfg(test)]
mod tests {
    use crate::{ValuePool, ValueRef};

    fn get_store() -> ValuePool<u32> {
        let mut store: ValuePool<u32> = ValuePool::with_capacity(10);

        for i in [12, 3, 123, 5, 1, 5, 8, 3, 0, 74, 52] {
            store.push(i);
        }
        store
    }

    #[test]
    fn test_simple_stats() {
        let mut store: ValuePool<u32> = ValuePool::with_capacity(8);

        for i in [12, 3, 123, 5, 1, 5, 8, 3] {
            store.push(i);
        }
        let ref_to_123 = ValueRef::new(2);
        store.remove(ref_to_123);

        assert_eq!(store.element_count(), 7);
        assert_eq!(store.waiting_positions(), 1);
        assert_eq!(store.capacity(), 8);
    }

    #[test]
    fn test_next_push_ref() {
        let mut store = get_store(); // 12,3,123,5,1,5,8,3,0,74,52
        assert_eq!(store.next_push_ref(), ValueRef::new(11));

        store.remove(ValueRef::new(2));
        assert_eq!(store.next_push_ref(), ValueRef::new(2));
        store.push(9);

        // 12,3,9,5,1,5,8,3,0,74,52 => 12,3,9,1,5,8,3,0,74,52
        unsafe { store.remove_full(ValueRef::new(3)) };
        assert_eq!(store.next_push_ref(), ValueRef::new(10));
    }
}
