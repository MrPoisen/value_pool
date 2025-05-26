//! This module implements [`CountedValuePool<T>`] which automatically keeps track of the references to the items stored within
#[cfg(feature = "unsafe")]
use std::mem::MaybeUninit;
use std::{
    cell::{Cell, RefCell},
    marker::PhantomData,
    rc::Rc,
};

use nonmax::NonMaxUsize;

#[derive(Debug, Clone, PartialEq, Eq)]
struct CountedIndex(Rc<Cell<NonMaxUsize>>);

impl CountedIndex {
    fn get_count(&self) -> usize {
        Rc::strong_count(&self.0)
    }
    fn get_index(&self) -> NonMaxUsize {
        self.0.get()
    }
    fn get_index_as_usize(&self) -> usize {
        self.0.get().get()
    }
    fn set(&self, value: NonMaxUsize) {
        self.0.set(value);
    }
    fn new(index: NonMaxUsize) -> CountedIndex {
        CountedIndex(Rc::new(Cell::new(index)))
    }
}

/// Struct that stores a location of an item in [`CountedValuePool<T>`] as well as the type. It implements [`Clone`].
///
/// That item is kept alive by the existing [CountedValueRef].
#[derive(Debug)]
pub struct CountedValueRef<T> {
    open_indices: Rc<RefCell<Vec<NonMaxUsize>>>,
    index: CountedIndex,
    type_: PhantomData<T>,
}

impl<T> PartialEq for CountedValueRef<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<T> Eq for CountedValueRef<T> {}

impl<T> Drop for CountedValueRef<T> {
    fn drop(&mut self) {
        if self.index.get_count() == 2 {
            // only this and the one stored in CountedValuePool
            let mut open_indices = self.open_indices.borrow_mut();
            open_indices.push(self.index.get_index());
        }
    }
}

impl<T> Clone for CountedValueRef<T> {
    fn clone(&self) -> Self {
        CountedValueRef {
            open_indices: self.open_indices.clone(),
            index: self.index.clone(),
            type_: PhantomData,
        }
    }
}

impl<T> CountedValueRef<T> {
    /// Doesn't increase the ref count
    fn new_nonmax(
        open_indices: Rc<RefCell<Vec<NonMaxUsize>>>,
        index: CountedIndex,
    ) -> CountedValueRef<T> {
        CountedValueRef {
            open_indices,
            index,
            type_: PhantomData,
        }
    }

    /// Returns the amount of [CountedValueRef]
    pub fn get_count(&self) -> usize {
        self.index.get_count() - 1
    }

    /// Only this [CountedValueRef] points to the value this [CountedValueRef] points to
    pub fn is_unique(&self) -> bool {
        self.get_count() == 1
    }
}

/// Struct that stores a location of an item in [`CountedValuePool<T>`] as well as the type. It is guaranteed to be the only reference to that item.
#[derive(Debug)]
pub struct UniqueValueRef<T> {
    inner: CountedValueRef<T>,
}
impl<T> UniqueValueRef<T> {
    /// Returns a Ok([UniqueValueRef]) if `reference` is unique ([CountedValueRef::is_unique()] is True).
    ///
    /// Returns the given [CountedValueRef] if it isn't unique as an Err.
    pub fn new(reference: CountedValueRef<T>) -> Result<UniqueValueRef<T>, CountedValueRef<T>> {
        if reference.is_unique() {
            Ok(UniqueValueRef { inner: reference })
        } else {
            Err(reference)
        }
    }

    /// Returns the inner [CountedValueRef]
    pub fn into_inner(self) -> CountedValueRef<T> {
        self.inner
    }
}

impl<T> AsRef<CountedValueRef<T>> for UniqueValueRef<T> {
    fn as_ref(&self) -> &CountedValueRef<T> {
        &self.inner
    }
}

impl<T> From<UniqueValueRef<T>> for CountedValueRef<T> {
    fn from(value: UniqueValueRef<T>) -> Self {
        value.into_inner()
    }
}

impl<T> AsRef<CountedValueRef<T>> for CountedValueRef<T> {
    fn as_ref(&self) -> &CountedValueRef<T> {
        self
    }
}

/// A [`CountedValuePool<T>`] allows referencing data stored within without a lifetime bound.
/// You can access that data with the [CountedValueRef]s returned by its methods.
#[derive(Debug)]
pub struct CountedValuePool<T> {
    store: Vec<Option<(T, CountedIndex)>>,
    open_indices: Rc<RefCell<Vec<NonMaxUsize>>>,
}

impl<T> Default for CountedValuePool<T> {
    fn default() -> Self {
        CountedValuePool::new()
    }
}

impl<T> CountedValuePool<T> {
    /// Creates a new, empty [`CountedValuePool`].
    pub fn new() -> CountedValuePool<T> {
        CountedValuePool {
            store: Vec::new(),
            open_indices: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// Creates a new [`CountedValuePool`] that can store `capacity` many items.
    pub fn with_capacity(capacity: usize) -> CountedValuePool<T> {
        CountedValuePool {
            store: Vec::with_capacity(capacity),
            open_indices: Rc::new(RefCell::new(Vec::with_capacity(capacity))),
        }
    }

    /// Returns the number of elements stored in this [`CountedValuePool`].
    #[inline]
    pub fn element_count(&self) -> usize {
        self.store.len() - self.open_indices.borrow().len()
    }

    /// Returns true if any `T`s are stored. Equivalent to: [`CountedValuePool::element_count() == 0`](CountedValuePool::element_count()).
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
        self.open_indices.borrow().len()
    }

    /// Pushes a new value into the [`CountedValuePool<T>`] and returns a [`CountedValueRef<T>`].
    /// You can access this value with [`CountedValuePool::get`].
    pub fn push(&mut self, value: T) -> CountedValueRef<T> {
        let mut open_indices = self.open_indices.borrow_mut();
        if !open_indices.is_empty() {
            let index = open_indices.pop().unwrap();
            let index = CountedIndex::new(index);
            self.store[index.get_index_as_usize()] = Some((value, index.clone()));
            CountedValueRef::new_nonmax(self.open_indices.clone(), index)
        } else {
            let index = CountedIndex::new(NonMaxUsize::new(self.store.len()).unwrap());
            self.store.push(Some((value, index.clone())));
            CountedValueRef::new_nonmax(self.open_indices.clone(), index)
        }
    }

    /// Pushes a new value into the [`CountedValuePool<T>`] and returns a [`UniqueValueRef<T>`].
    /// You can access this value with [`CountedValuePool::get`].
    pub fn push_unique(&mut self, value: T) -> UniqueValueRef<T> {
        let mut open_indices = self.open_indices.borrow_mut();
        if !open_indices.is_empty() {
            let index = open_indices.pop().unwrap();
            let index = CountedIndex::new(index);
            self.store[index.get_index_as_usize()] = Some((value, index.clone()));
            UniqueValueRef {
                inner: CountedValueRef::new_nonmax(self.open_indices.clone(), index),
            }
        } else {
            let index = CountedIndex::new(NonMaxUsize::new(self.store.len()).unwrap());
            self.store.push(Some((value, index.clone())));
            UniqueValueRef {
                inner: CountedValueRef::new_nonmax(self.open_indices.clone(), index),
            }
        }
    }

    /// Takes the value pointed to by `reference` from this [CountedValuePool] if `reference` is unique (`reference.is_unique() == true`)
    ///
    /// # Panics
    /// Might panic in the future if `reference` doesn't belong to this [CountedValuePool]
    pub fn take(&mut self, reference: impl Into<CountedValueRef<T>>) -> Option<T> {
        let reference: CountedValueRef<T> = reference.into();
        if !reference.is_unique() || !self.belongs(&reference) {
            // Do i need to check with self.belongs?
            return None;
        }
        let index = reference.index.get_index_as_usize();

        #[cfg(feature = "unsafe")]
        let (value, counted_index) = unsafe {
            self.store.get_mut(index).unwrap_unchecked().take().unwrap_unchecked()
        };
        #[cfg(not(feature = "unsafe"))]
        let (value, counted_index) =
           self.store.get_mut(index).unwrap().take().unwrap();

        drop(counted_index);
        drop(reference); // won't add index to self.open_indices cause only one CountedIndex exists
        Some(value)
    }

    /// Takes the value pointed to by `reference` from this [CountedValuePool] if `reference` is unique (`reference.is_unique() == true`)
    ///
    /// # Panics
    /// Will panic if `reference` doesn't belong to this [CountedValuePool]
    pub fn take_unique(&mut self, reference: UniqueValueRef<T>) -> T {
        let reference: CountedValueRef<T> = reference.into();
        if !self.belongs(&reference) {
            panic!("given reference doesn't belong to this CountedValuePool");
        }
        let index = reference.index.get_index_as_usize();

        #[cfg(feature = "unsafe")]
        let (value, counted_index) = unsafe {
            self.store.get_mut(index).unwrap_unchecked().take().unwrap_unchecked()
        };
        #[cfg(not(feature = "unsafe"))]
        let (value, counted_index) =
            self.store.get_mut(index).unwrap().take().unwrap();

        drop(counted_index);
        drop(reference); // won't add index to self.open_indices cause only one CountedIndex exists
        value
    }

    /// Takes the value pointed to by `reference` from this
    ///
    /// # Safety
    /// `reference` must be unique (`reference.is_unique() == true`) and belong to this [CountedValuePool]
    pub unsafe fn take_unchecked(&mut self, reference: impl Into<CountedValueRef<T>>) -> T {
        let reference: CountedValueRef<T> = reference.into();
        let index = reference.index.get_index_as_usize();
        let (value, counted_index) = unsafe { self.store.swap_remove(index).unwrap_unchecked() };
        drop(counted_index);
        drop(reference); // won't add index to self.open_indices cause only one CountedIndex exists
        value
    }

    /// Takes the value pointed to by `reference` from this
    ///
    /// # Safety
    /// `reference` must belong to this [CountedValuePool]
    pub unsafe fn take_unchecked_unique(&mut self, reference: UniqueValueRef<T>) -> T {
        let reference: CountedValueRef<T> = reference.into();
        let index = reference.index.get_index_as_usize();
        let (value, counted_index) = unsafe { self.store.swap_remove(index).unwrap_unchecked() };
        drop(counted_index);
        drop(reference); // won't add index to self.open_indices cause only one CountedIndex exists
        value
    }

    /// Gets a borrow of the item pointed to by `reference`.
    /// It is guaranteed to exist.
    ///
    /// # Panic
    /// Will panic if `reference` doesn't belong `self`
    pub fn get(&self, reference: impl AsRef<CountedValueRef<T>>) -> &T {
        let reference = reference.as_ref();

        if reference.open_indices.as_ptr() != self.open_indices.as_ptr() {
            // Without check: might panic anyway later (at first or second `.unwrap()`)
            panic!("given reference doesn't belong to this CountedValuePool");
        }
        debug_assert!(reference.open_indices.as_ptr() == self.open_indices.as_ptr());
        #[cfg(feature = "unsafe")]
        unsafe {
            &self
                .store
                .get(reference.index.get_index_as_usize())
                .unwrap_unchecked()
                .as_ref()
                .unwrap_unchecked()
                .0
        }

        #[cfg(not(feature = "unsafe"))]
        &self
            .store
            .get(reference.index.get_index_as_usize())
            .unwrap()
            .as_ref()
            .unwrap()
            .0
    }

    /// Gets a mutable borrow of the item pointed to by `reference`.
    /// It is guaranteed to exist.
    ///
    /// # Panic
    /// Will panic if `reference` doesn't belong `self`
    pub fn get_mut(&mut self, reference: impl AsRef<CountedValueRef<T>>) -> &mut T {
        let reference: &CountedValueRef<T> = reference.as_ref();
        if !self.belongs(reference) {
            // Without check: might panic/UB later (at first or second `.unwrap()`)
            panic!("given reference doesn't belong to this CountedValuePool");
        }
        #[cfg(feature = "unsafe")]
        unsafe {
            &mut self
                .store
                .get_unchecked_mut(reference.index.get_index_as_usize())
                .as_mut()
                .unwrap_unchecked()
                .0
        }

        #[cfg(not(feature = "unsafe"))]
        &mut self
            .store
            .get_mut(reference.index.get_index_as_usize())
            .unwrap()
            .as_mut()
            .unwrap()
            .0
    }

    /// Gets the mutable borrows of the items pointed to by `reference_1` and `reference_2.
    ///
    /// None will be returned if both [CountedValueRef] point to the same value.
    ///
    /// # Panics
    /// Will panic if either [CountedValueRef] doesn't belong to this [CountedValuePool]
    pub fn get_mut_2(
        &mut self,
        reference_1: impl AsRef<CountedValueRef<T>>,
        reference_2: impl AsRef<CountedValueRef<T>>,
    ) -> Option<(&mut T, &mut T)> {
        let reference_1: &CountedValueRef<T> = reference_1.as_ref();
        let reference_2: &CountedValueRef<T> = reference_2.as_ref();
        if !self.belongs(reference_1) || !self.belongs(reference_2) {
            panic!("a given reference doesn't belong to this CountedValuePool");
        }
        if reference_1.index.get_index() == reference_2.index.get_index() {
            return None;
        }

        #[cfg(feature = "unsafe")]
        if reference_1.index.get_index() < reference_2.index.get_index() {
            let t = unsafe {
                self.store
                    .split_at_mut_unchecked(reference_1.index.get_index_as_usize() + 1)
            };
            unsafe {
                Some((
                    &mut t
                        .0
                        .last_mut()
                        .unwrap_unchecked()
                        .as_mut()
                        .unwrap_unchecked()
                        .0,
                    &mut t
                        .1
                        .get_unchecked_mut(
                            reference_2.index.get_index_as_usize()
                                - reference_1.index.get_index_as_usize()
                                - 1,
                        )
                        .as_mut()
                        .unwrap_unchecked()
                        .0,
                ))
            }
        } else {
            // reference_1.index.get_index() > reference_2.index.get_index()
            let t = unsafe {
                self.store
                    .split_at_mut_unchecked(reference_2.index.get_index_as_usize() + 1)
            };
            unsafe {
                Some((
                    &mut t
                        .1
                        .get_unchecked_mut(
                            reference_1.index.get_index_as_usize()
                                - reference_2.index.get_index_as_usize()
                                - 1,
                        )
                        .as_mut()
                        .unwrap_unchecked()
                        .0,
                    &mut t
                        .0
                        .last_mut()
                        .unwrap_unchecked()
                        .as_mut()
                        .unwrap_unchecked()
                        .0,
                ))
            }
        }

        #[cfg(not(feature = "unsafe"))]
        if reference_1.index.get_index() < reference_2.index.get_index() {
            let t = self
                .store
                .split_at_mut_checked(reference_1.index.get_index_as_usize() + 1)
                .unwrap();
            Some((
                &mut t.0.last_mut().unwrap().as_mut().unwrap().0,
                &mut t
                    .1
                    .get_mut(
                        reference_2.index.get_index_as_usize()
                            - reference_1.index.get_index_as_usize()
                            - 1,
                    )
                    .unwrap()
                    .as_mut()
                    .unwrap()
                    .0,
            ))
        } else {
            // reference_1.index.get_index() > reference_2.index.get_index()
            let t = self
                .store
                .split_at_mut_checked(reference_2.index.get_index_as_usize() + 1)
                .unwrap();
            Some((
                &mut t
                    .1
                    .get_mut(
                        reference_1.index.get_index_as_usize()
                            - reference_2.index.get_index_as_usize()
                            - 1,
                    )
                    .unwrap()
                    .as_mut()
                    .unwrap()
                    .0,
                &mut t.0.last_mut().unwrap().as_mut().unwrap().0,
            ))
        }
    }

    #[cfg(feature = "unsafe")]
    /// Gets a multiple mutable borrows of the items pointed to by the `references`.
    /// None will be returned if any of the items in `references` point to the same index (exists i,j such that `references[i] == references[j]`)
    ///
    /// # Panic
    /// Will panic if `reference` doesn't belong `self`
    pub fn get_mut_n<const N: usize>(
        &mut self,
        references: [&CountedValueRef<T>; N],
    ) -> Option<[&mut T; N]> {
        for i in 0..N {
            if references[i].open_indices.as_ptr() != self.open_indices.as_ptr() {
                panic!(
                    "given reference (at index {}) doesn't belong to this CountedValuePool",
                    i
                );
            }
            for j in (i + 1)..N {
                if references[i].index == references[j].index {
                    return None;
                }
            }
        }
        let store_mut_ptr = self.store.as_mut_ptr();
        let mut ar: [MaybeUninit<&mut T>; N] = [const { MaybeUninit::uninit() }; N];
        for (i, uninit) in ar.iter_mut().enumerate() {
            uninit.write(unsafe {
                &mut (*store_mut_ptr.add(references[i].index.get_index_as_usize()))
                    .as_mut()
                    .unwrap()
                    .0
            });
        }

        Some(unsafe { std::mem::transmute_copy::<_, [&mut T; N]>(&ar) })
    }

    /// Ensures that for all open slots (element_count() - capacity()) only [Option::None] is saved
    ///
    /// A slot may still have a value saved if the last [CountedValueRef] pointing to it was dropped but no new item has taken its place.
    /// In that case the slot has already been marked as free to use
    pub fn drop_unused(&mut self) {
        let open_indices = self.open_indices.borrow();
        for x in open_indices.iter() {
            self.store[x.get()] = None;
        }
    }

    /// Compacts the internally used values such that they live closer together.
    ///
    /// All [CountedValueRef] will remain valid.
    pub fn compact(&mut self) {
        let mut write_index = 0; // <=> Amount of Option::Some correctly positioned <=> first None (after first None encounter)

        let max_write_index = self.element_count();
        for i in 0..self.store.len() {
            if self.store[i].as_ref().is_some_and(|x| x.1.get_count() > 1) {
                // if `x.1.get_count() == 1` => no CountedValueRef exists => slot must be marked free in `self.open_indices`
                self.store[i]
                    .as_ref()
                    .unwrap()
                    .1
                    .set(NonMaxUsize::new(write_index).unwrap());
                self.store.swap(write_index, i);

                write_index += 1;
            }
            if write_index >= max_write_index {
                break;
            }
        }
        // correct open_indices
        let mut open_indices = self.open_indices.borrow_mut();
        open_indices.clear();
        for i in (write_index..self.store.len()).rev() {
            // rev => open_indices pops smallest first
            open_indices.push(NonMaxUsize::new(i).unwrap());
        }
    }

    /// Checks if `other` belongs to this [CountedValuePool] (<=> `other` points to a value in this [CountedValuePool])
    pub fn belongs(&self, other: impl AsRef<CountedValueRef<T>>) -> bool {
        let other: &CountedValueRef<T> = other.as_ref();
        self.open_indices.as_ptr() == other.open_indices.as_ptr()
    }

    /// swaps the values `reference_1` and `reference_2` point to
    ///
    /// # Panics
    /// Will panic if the [CountedValueRef] don't belong to this [CountedValuePool]
    pub fn swap(
        &mut self,
        reference_1: impl AsRef<CountedValueRef<T>>,
        reference_2: impl AsRef<CountedValueRef<T>>,
    ) {
        let reference_1: &CountedValueRef<T> = reference_1.as_ref();
        let reference_2: &CountedValueRef<T> = reference_2.as_ref();
        if !self.belongs(reference_1) || !self.belongs(reference_2) {
            panic!("a given reference doesn't belong to this CountedValuePool");
        }
        let index_1 = reference_1.index.get_index();
        let index_2 = reference_2.index.get_index();
        self.store.swap(index_1.get(), index_2.get());

        reference_1.index.set(index_2);
        reference_2.index.set(index_1);
    }

    /// Replaces the value at `reference` with `value`.
    pub fn replace(&mut self, reference: &CountedValueRef<T>, value: T) -> T {
        std::mem::replace(self.get_mut(reference), value)
    }

    /// Ensures at least `additional` elements can be stored without additional reallocations.
    pub fn reserve(&mut self, additional: usize) {
        self.store.reserve(additional);
        self.open_indices.borrow_mut().reserve(additional);
    }
}
#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use nonmax::NonMaxUsize;
    use std::{cell::Cell, rc::Rc};

    use super::CountedValuePool;

    #[test]
    fn basic() {
        let mut s: CountedValuePool<i64> = CountedValuePool::new();
        let ref1 = s.push(1);
        let ref2 = s.push(2);

        assert_eq!(s.get(&ref1), &1);
        assert_eq!(s.get(&ref2), &2);
        assert_eq!(s.get_mut(&ref1), &mut 1);

        assert_eq!(s.get_mut_2(&ref1, &ref2), Some((&mut 1, &mut 2)));
        assert_eq!(s.get_mut_2(&ref2, &ref1), Some((&mut 2, &mut 1)));
    }

    #[test]
    fn test_compacting() {
        let mut s: CountedValuePool<i64> = CountedValuePool::new();

        let mut container = Vec::with_capacity(10);
        for i in 0..10 {
            container.push(s.push(i));
        }
        container.remove(2);
        container.remove(2);
        container.remove(6); // Index 8 before

        s.compact();

        assert_eq!(s.element_count(), 7);
        for (i, c) in container.iter().enumerate() {
            if i < 2 {
                assert_eq!(*s.get(c), i as i64);
            } else if i < 6 {
                assert_eq!(*s.get(c), i as i64 + 2);
            } else {
                assert_eq!(*s.get(c), i as i64 + 3);
            }
        }
    }

    #[cfg(feature = "unsafe")]
    #[test]
    fn test_get_mut_n() {
        let mut s: CountedValuePool<i64> = CountedValuePool::new();

        let mut container = Vec::with_capacity(10);
        for i in 0..10 {
            container.push(s.push(i));
        }

        assert_eq!(
            None,
            s.get_mut_n([&container[0], &container[1], &container[0]])
        );

        let x = s.get_mut_n([&container[0], &container[1], &container[9]]);
        assert!(x.is_some());
        let x = x.unwrap();

        assert_eq!(*x[0], 0);
        assert_eq!(*x[1], 1);
        assert_eq!(*x[2], 9);
    }
}
