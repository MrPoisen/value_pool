//! This libraries allows easy use of self-referential structs by storing them in one place, the [`ValuePool<T>`]
//! and referencing the stored values with [`UntypedValueRef`] or [`ValueRef<T>`].
//!
//! # Showcase
//! ```
//! use value_pool::{ValueRef, ValuePool,UntypedValueRef};
//! let mut pool: ValuePool<u32> = ValuePool::new();
//! let ref_to_first: ValueRef<u32> = pool.push(12);
//!
//! // You can convert ValueRef<T> to UntypedValueRef and the other way round.
//! // UntypedValueRef is useful if the type information of ValueRef<T> gets in your way
//! let untyped_ref_to_first: UntypedValueRef = ref_to_first.into();
//!
//! // original type information gets lost
//! let wrongly_typed_ref_to_first: ValueRef<u8> = untyped_ref_to_first.into();  
//! // Notice the wrong type of `wrongly_typed_ref_to_first`
//! // Following line would result in compile time error:
//! //  `Trait From<ValueRef<u8>> is not implemented for ValueRef<u32>`
//! //pool.get(wrongly_typed_ref_to_first); // Error here
//!
//! assert_eq!(pool.get(ref_to_first), Some(&12));
//! assert_eq!(pool.element_count(), 1);
//!
//! // You can take a value
//! assert_eq!(pool.take(ref_to_first), Some(12));
//! assert_eq!(pool.element_count(), 0);
//!
//! let mut ref_to_13 = pool.push(13);
//! let mut ref_to_14 = pool.push(14);
//! let copy_ref_to_14 = ref_to_14; // ValueRef implements Copy
//!
//! // pool.swap is marked unsafe cause it causes all references to 14 (and 13) to point to the
//! // wrong value, except for the returned refs
//! unsafe{(ref_to_13,ref_to_14) = pool.swap(ref_to_13, ref_to_14).unwrap();}
//! assert_eq!(ref_to_13, copy_ref_to_14);
//!
//! // unsafe cause now ref_to_13 will be invalid (Actually: all refs >= ref_to_14)
//! unsafe{pool.remove_full(ref_to_14);}
//! assert!(ref_to_13 > ref_to_14);
//! assert_eq!(pool.find(&13).unwrap(), ref_to_14);
//! assert_eq!(pool.find(&13).unwrap(), ValueRef::new(0));
//! ```
//! # Features
//! - *unsafe* - Library will use unsafe code to (potentially) improve speed. This could result in UB if implemented faulty even though it shouldn't and the behavior of your code should be unchanged.
#![warn(missing_docs)]

use nonmax::NonMaxUsize;
use std::{borrow::Borrow, hash::Hash, marker::PhantomData};
pub mod smart_value_pool;

/// Struct that stores a location of an item in [`ValuePool<T>`]. It implements [`Copy`].
///
/// Usually, you get this struct with `from` or `into`:
/// ```
/// use value_pool::{UntypedValueRef, ValueRef};
///
/// let value_ref: ValueRef<usize> = ValueRef::new(2);
///
/// let untyped_value_ref: UntypedValueRef = value_ref.into();
/// assert_eq!(untyped_value_ref, value_ref);
///
/// //or
/// let untyped_value_ref = UntypedValueRef::new(2); // usually not needed or recommended
/// assert_eq!(untyped_value_ref, value_ref);
/// ```

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UntypedValueRef {
    index: NonMaxUsize,
}

impl UntypedValueRef {
    /// Creates a new [`UntypedValueRef`] for a given index. This is usually not needed.
    ///
    /// # Panic
    /// This will panic if [`index == usize::MAX`](usize::MAX).
    #[inline]
    pub fn new(index: usize) -> UntypedValueRef {
        UntypedValueRef {
            index: NonMaxUsize::new(index).expect("Given index to not be the maximum value"),
        }
    }

    /// Creates a new [`ValueRef`] for a given index. This is usually not needed.
    #[inline]
    pub fn new_non_max(index: NonMaxUsize) -> UntypedValueRef {
        UntypedValueRef { index: (index) }
    }
}

impl Default for UntypedValueRef {
    #[inline]
    fn default() -> Self {
        UntypedValueRef {
            index: NonMaxUsize::ZERO,
        }
    }
}

impl<T> PartialEq<ValueRef<T>> for UntypedValueRef {
    #[inline]
    fn eq(&self, other: &ValueRef<T>) -> bool {
        self.index == other.index
    }
}

impl<T> PartialOrd<ValueRef<T>> for UntypedValueRef {
    #[inline]
    fn partial_cmp(&self, other: &ValueRef<T>) -> Option<std::cmp::Ordering> {
        Some(self.index.cmp(&other.index))
    }
}

impl<T> From<ValueRef<T>> for UntypedValueRef {
    #[inline]
    fn from(value: ValueRef<T>) -> Self {
        UntypedValueRef {
            index: (value.index),
        }
    }
}
impl<T> From<UntypedValueRef> for ValueRef<T> {
    #[inline]
    fn from(value: UntypedValueRef) -> Self {
        ValueRef {
            index: (value.index),
            type_info: (PhantomData),
        }
    }
}

/// Struct that stores a location of an item in [`ValuePool<T>`] as well as the type. It implements [`Copy`].
///
/// Usually, you get this struct trough methods from [`ValuePool<T>`]. 
/// ```
/// use value_pool::{UntypedValueRef, ValueRef, ValuePool};
///
/// let mut pool: ValuePool<usize> = ValuePool::new();
///
/// let value_ref_2: ValueRef<usize> = pool.push(2);
///  
/// //or
/// let value_ref: ValueRef<usize> = ValueRef::new(4);
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
/// assert_eq!(pool.get(value_ref), None); // Compiler error her, pool stores `usize`, but `value_ref` is `ValueRef<u32>`
/// ```
#[derive(Debug)]
pub struct ValueRef<T> {
    index: NonMaxUsize,
    type_info: PhantomData<T>,
}

impl<T> PartialEq<UntypedValueRef> for ValueRef<T> {
    #[inline]
    fn eq(&self, other: &UntypedValueRef) -> bool {
        self.index == other.index
    }
}
impl<T> PartialOrd<UntypedValueRef> for ValueRef<T> {
    #[inline]
    fn partial_cmp(&self, other: &UntypedValueRef) -> Option<std::cmp::Ordering> {
        Some(self.index.cmp(&other.index))
    }
}

impl<T> Default for ValueRef<T> {
    #[inline]
    fn default() -> Self {
        ValueRef {
            index: (NonMaxUsize::ZERO),
            type_info: (PhantomData),
        }
    }
}

impl<T> Hash for ValueRef<T> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(self.index.get());
    }
}

impl<T> Clone for ValueRef<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
        // ValueRef {
        //     index: (self.index),
        //     type_info: self.type_info,
        // }
    }
}
impl<T> Copy for ValueRef<T> {}

impl<T> PartialOrd for ValueRef<T> {
    #[inline]
    fn ge(&self, other: &Self) -> bool {
        self.index >= other.index
    }
    #[inline]
    fn lt(&self, other: &Self) -> bool {
        self.index < other.index
    }
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl<T> Ord for ValueRef<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.index == other.index {
            return std::cmp::Ordering::Equal;
        }
        if self.index > other.index {
            return std::cmp::Ordering::Greater;
        }
        // self.index < other.index
        std::cmp::Ordering::Less
    }
}

impl<T> ValueRef<T> {
    /// Creates a new [`ValueRef`] for a given index. This is usually not needed.
    ///
    /// # Panic
    /// Will panic if [`index == usize::MAX`](usize::MAX).
    #[inline]
    pub fn new(index: usize) -> ValueRef<T> {
        ValueRef {
            index: (NonMaxUsize::new(index).expect("Given index to not be the maximum value")),
            type_info: (PhantomData),
        }
    }

    /// Creates a new [`ValueRef<T>`] for a given index. This is usually not needed.
    #[inline]
    pub fn new_nonmax(index: NonMaxUsize) -> ValueRef<T> {
        ValueRef {
            index: (index),
            type_info: (PhantomData),
        }
    }
}

impl<T> PartialEq for ValueRef<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}
impl<T> Eq for ValueRef<T> {}

// TODO: use SmallVec (as a feature) when it hits v2 (https://github.com/servo/rust-smallvec/tree/v2)

/// A [`ValuePool<T>`] allows referencing data stored within without a lifetime bound.  
/// It works by returning an [`Option<T>`]. It's your responsibility to handel [`None`]s.
/// ```
/// use value_pool::ValuePool;
/// let mut pool: ValuePool<i32>= ValuePool::with_capacity(10);
/// let ten_ref = pool.push(10);
/// pool.push(20);
/// let minus_ten_ref = pool.push(-10);
/// 
/// assert_eq!(pool.get(ten_ref), Some(&10i32));
/// let minus_ten = pool.take(minus_ten_ref);
/// assert_eq!(minus_ten, Some(-10i32));
/// assert_eq!(pool.get(minus_ten_ref), None);
/// ```
#[derive(Debug, Clone)]
pub struct ValuePool<T> {
    store: Vec<Option<T>>,
    open_indices: Vec<NonMaxUsize>,
}

impl<T> Default for ValuePool<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ValuePool<T> {
    /// Creates a new [`ValuePool`] that can store `capacity` many items.
    #[inline]
    pub fn with_capacity(capacity: usize) -> ValuePool<T> {
        ValuePool {
            store: (Vec::with_capacity(capacity)),
            open_indices: (Vec::with_capacity(capacity / 4)),
        }
    }
    /// Creates a new, empty [`ValuePool`].
    #[inline]
    pub fn new() -> ValuePool<T> {
        ValuePool {
            store: (Vec::new()),
            open_indices: (Vec::new()),
        }
    }

    /// Returns the number of elements stored in this [`ValuePool`].
    #[inline]
    pub fn element_count(&self) -> usize {
        self.store.len() - self.open_indices.len()
    }

    /// Returns true if any `T`s are stored. Equivalent to: [`ValuePool::element_count() == 0`](ValuePool::element_count()).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.element_count() == 0
    }

    /// Returns the number of items that can be stored before reallocation happens.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.store.capacity()
    }

    /// Returns the number of positions that are currently empty. These positions are prioritized when pushing new values.
    #[inline]
    pub fn waiting_positions(&self) -> usize {
        self.open_indices.len()
    }

    /// Checks if the given reference is in bounce. If true, this means [`ValuePool::get_unchecked`] and the likes can be called without UB.
    /// These methods can *still* return [`None`].
    ///
    /// # Complexity
    /// `O(1)`
    #[inline]
    pub fn is_ref_in_bounce(&self, reference: impl Into<ValueRef<T>>) -> bool {
        let reference: ValueRef<T> = reference.into();
        reference.index.get() < self.store.len()
    }

    /// Pushes a new value into the [`ValuePool`] and returns a [`ValueRef<T>`] (that stores its position).
    /// You can access this value with `get`.
    ///
    /// # Complexity
    /// `O(1)`
    #[inline]
    pub fn push(&mut self, value: T) -> ValueRef<T> {
        if !self.open_indices.is_empty() {
            let index = self.open_indices.pop().unwrap();
            self.store[index.get()] = Some(value);
            ValueRef::new_nonmax(index)
        } else {
            self.store.push(Some(value));
            ValueRef::new(self.store.len() - 1)
        }
    }

    /// Removes an item from [`ValuePool`].  
    /// If this item is stored last its position won't be marked empty but instead thee underlying  
    /// data structure will be reduced in length.  
    /// Note: This will **not** reduce the used memory of this [`ValuePool<T>`].
    ///
    /// # Complexity
    /// `O(1)`
    #[inline]
    pub fn remove(&mut self, reference: impl Into<ValueRef<T>>) {
        let reference: ValueRef<T> = reference.into();
        if !self.has_item(reference) {
            return;
        }
        // => there is an item at reference

        // if `reference` is the last index and has a value; this can prevent reallocation of `self.open_indices`
        if reference.index.get() + 1 == self.store.len() {
            self.store.pop();
            return;
        }

        #[cfg(feature="unsafe")]
        unsafe{
            // value must exist cause `self.has_item` is true
            let value = self.store.get_unchecked_mut(reference.index.get());
            self.open_indices.push(reference.index);
            *value = None
        }
        #[cfg(not(feature="unsafe"))]
        {   
            // value must exist cause `self.has_item` is true
            let value = self.store.get_mut(reference.index.get()).unwrap();
            self.open_indices.push(reference.index);
            *value = None;
        }
 
    }

    /// # Safety
    /// Makes the greatest [`ValueRef<T>`] point to the wrong (actually now [`None`]) element.
    /// This function will not panic or create UB.
    ///
    /// # Complexity
    /// `O(1)`
    #[inline]
    pub unsafe fn remove_full(&mut self, reference: impl Into<ValueRef<T>>) -> Option<T> {
        let reference: ValueRef<T> = reference.into();
        self.store.swap_remove(reference.index.get())
    }

    /// Gets a borrow of the item pointed to by `reference` if it exists.
    ///
    /// # Complexity
    /// `O(1)`
    #[inline]
    pub fn get(&self, reference: impl Into<ValueRef<T>>) -> Option<&T> {
        let reference: ValueRef<T> = reference.into();
        self.store
            .get(reference.index.get())
            .and_then(|x| x.as_ref())
    }

    /// Gets a borrow of the item pointed to by `reference` if an item is stored there.
    ///
    /// # Safety
    /// Calling this method with an `reference` that is out of bounds, is UB. You can check beforehand with [`ValuePool::is_ref_in_bounce`].
    ///
    /// # Complexity
    /// `O(1)`
    #[inline]
    pub unsafe fn get_unchecked(&self, reference: impl Into<ValueRef<T>>) -> Option<&T> {
        let reference: ValueRef<T> = reference.into();
        self.store.get_unchecked(reference.index.get()).as_ref()
    }

    /// Gets a mut borrow of the item pointed to by `reference` if it exists.
    /// 
    /// # Complexity
    /// `O(1)`
    #[inline]
    pub fn get_mut(&mut self, reference: impl Into<ValueRef<T>>) -> Option<&mut T> {
        let reference: ValueRef<T> = reference.into();
        self.store
            .get_mut(reference.index.get())
            .and_then(|x| x.as_mut())
    }

    /// Gets a mut borrow of the item pointed to by `reference` if an item is stored there.
    ///
    /// # Safety
    /// Calling this method with an reference that is out of bounds, is UB. You can check beforehand with [`ValuePool::is_ref_in_bounce`].
    ///
    /// # Complexity
    /// `O(1)`
    #[inline]
    pub unsafe fn get_unchecked_mut(
        &mut self,
        reference: impl Into<ValueRef<T>>,
    ) -> Option<&mut T> {
        let reference: ValueRef<T> = reference.into();
        self.store.get_unchecked_mut(reference.index.get()).as_mut()
    }

    /// Swaps `ref_1` with `ref_2`, all other refs equal two the both will point to the wrong element.
    ///
    /// # Note
    /// All other references equal to `ref_1` or `ref_2` now point to the wrong element.
    ///
    /// # Complexity
    /// `O(1)`
    #[inline]
    pub fn swap(
        &mut self,
        ref_1: impl Into<ValueRef<T>>,
        ref_2: impl Into<ValueRef<T>>,
    ) -> Option<(ValueRef<T>, ValueRef<T>)> {
        let (ref_1, ref_2) = (ref_1.into(), ref_2.into());
        if ref_1.index.get() >= self.store.len() || ref_2.index.get() >= self.store.len() {
            return None;
        }
        self.store.swap(ref_1.index.get(), ref_2.index.get());
        Some((ref_2, ref_1))
    }

    /// Returns the value_ref value the next call to [`ValuePool::push`] would return.
    ///
    /// # Complexity
    /// `O(1)`
    #[inline]
    pub fn next_push_ref(&self) -> ValueRef<T> {
        if self.open_indices.is_empty() {
            return ValueRef::new(self.store.len());
        }
        #[cfg(feature = "unsafe")]
        unsafe {
            return ValueRef::new_nonmax(*self.open_indices.last().unwrap_unchecked());
        }
        #[cfg(not(feature = "unsafe"))]
        {
            return ValueRef::new_nonmax(*self.open_indices.last().unwrap());
        }
    }

    /// Takes value at `reference` and returns it. Calling it again with the same `reference` _(without modifying this [`ValuePool<T>`])_ will always return [`None`].  
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
    ///
    /// # Complexity
    /// `O(1)`
    #[inline]
    pub fn take(&mut self, reference: impl Into<ValueRef<T>>) -> Option<T> {
        let mut tmp = None;
        let reference: ValueRef<T> = reference.into();
        std::mem::swap(&mut tmp, self.store.get_mut(reference.index.get())?);
        if tmp.is_some() {
            // if tmp is none, reference.index should already be in self.open_indices
            self.open_indices.push(reference.index);
        }
        tmp
    }

    /// Takes value at `reference` and returns it. Calling it again with the same `reference` _(without modifying this [`ValuePool<T>`])_ will always return [`None`].  
    ///
    /// # Safety
    /// Calling this method with an reference that is out of bounds, is UB. You can check beforehand with [`ValuePool::is_ref_in_bounce`].
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
    ///
    /// # Complexity
    /// `O(1)`
    #[inline]
    pub unsafe fn take_unchecked(&mut self, reference: impl Into<ValueRef<T>>) -> Option<T> {
        let mut tmp = None;
        let reference: ValueRef<T> = reference.into();
        std::mem::swap(
            &mut tmp,
            self.store.get_unchecked_mut(reference.index.get()),
        );
        if tmp.is_some() {
            // if tmp is none, reference.index should already be in self.open_indices
            self.open_indices.push(reference.index);
        }
        tmp
    }

    /// Ensures at least `additional` elements can be stored without additional reallocations.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.store.reserve(additional);
    }

    /// Returns true, if an item is stored at `reference`.
    /// Equivalent to [`ValuePool::get`]`.is_some()`.
    ///
    /// # Complexity
    /// `O(1)`
    #[inline]
    pub fn has_item(&self, reference: impl Into<ValueRef<T>>) -> bool {
        self.get(reference).is_some()
    }

    /// If `value` exists, than the corresponding [`ValueRef<T>`] will be returned.
    /// ```
    /// use value_pool::{ValuePool, ValueRef};
    /// let mut pool = ValuePool::new();
    /// pool.push(1);
    /// pool.push(2);
    /// pool.push(3);
    /// pool.push(4);
    /// assert_eq!(pool.find(&3), Some(ValueRef::new(2)));
    ///
    /// pool.remove(ValueRef::new(2));
    /// pool.push(5); // returns ValueRef::new(2)
    /// pool.push(3);
    /// assert_eq!(pool.find(&3), Some(ValueRef::new(4)));
    /// ```
    ///
    /// # Complexity
    /// Be n = [`ValuePool::element_count()`] + [`ValuePool::waiting_positions()`].   
    /// Worst-Case: `O(n)`  
    /// Average-Case: `O(n/2)`   
    /// Best-Case: `O(1)`   
    #[inline]
    pub fn find<Q: Eq>(&self, value: &Q) -> Option<ValueRef<T>>
    where
        T: Borrow<Q>,
    {
        Some(ValueRef::new(self.store.iter().position(|v| {
            v.as_ref().is_some_and(|x| *x.borrow() == *value)
        })?))
    }

    /// Clears this [`ValuePool<T>`].
    /// ```
    /// use value_pool::ValuePool;
    ///
    /// let mut pool = ValuePool::new();
    /// pool.push(1);
    /// pool.push(2);
    /// pool.push(3);
    /// assert_eq!(pool.element_count(),3);
    /// pool.clear();
    /// assert_eq!(pool.element_count(),0);
    /// ```
    ///
    /// # Complexity
    /// O(1)
    #[inline]
    pub fn clear(&mut self) {
        self.open_indices.clear();
        self.store.clear();
    }
}

#[cfg(test)]
mod tests {
    use crate::{UntypedValueRef, ValuePool, ValueRef};

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

    #[test]
    fn test_correct_sizes() {
        struct Dummy;
        assert_eq!(
            std::mem::size_of::<ValueRef<Dummy>>(),
            std::mem::size_of::<Option<ValueRef<Dummy>>>()
        );
        assert_eq!(std::mem::size_of::<UntypedValueRef>(), std::mem::size_of::<Option<UntypedValueRef>>());
    }
}
