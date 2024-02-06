use std::{
    hash::Hash,
    marker::PhantomData,
    ops::Deref,
};

#[cfg(feature = "double_linked_list")]
pub mod linked_list;

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UntypedValueRef {
    index: usize,
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
    pub fn with_capacity(capacity: usize) -> ValuePool<T> {
        ValuePool {
            store: (Vec::with_capacity(capacity)),
            open_indicis: (Vec::with_capacity(capacity / 4)),
        }
    }

    pub fn new() -> ValuePool<T> {
        ValuePool {
            store: (Vec::new()),
            open_indicis: (Vec::new()),
        }
    }

    pub fn element_count(&self) -> usize {
        self.store.len() - self.open_indicis.len()
    }

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

    pub fn _remove_get_value(&mut self, reference: ValueRef<T>) -> Option<T> {
        if self.store.get(reference.index).is_none() {
            // check if index exists
            return None;
        }
        let mut tmp = None;
        std::mem::swap(&mut tmp, self.store.get_mut(reference.index).unwrap());

        self.open_indicis.push(reference.index);
        return tmp;
    }
    pub fn remove(&mut self, reference: ValueRef<T>) {
        self._remove_get_value(reference);
    }

    /// ATTENTION: INVALIDATES all ValueRefs greater then reference
    pub fn remove_full(&mut self, reference: ValueRef<T>) -> Option<T> {
        self.store.remove(reference.index)
    }

    pub fn get(&self, reference: ValueRef<T>) -> Option<&T> {
        self.store
            .get(reference.index)
            .map(|x| x.as_ref())
            .flatten()
    }

    pub fn get_mut(&mut self, reference: ValueRef<T>) -> Option<&mut T> {
        self.store
            .get_mut(reference.index)
            .map(|x| x.as_mut())
            .flatten()
    }

    pub fn get_last(&self) -> Option<&T> {
        self.store.last().map(|x| x.as_ref()).flatten()
    }

    pub fn get_last_mut(&mut self) -> Option<&mut T> {
        self.store.last_mut().map(|x| x.as_mut()).flatten()
    }

    pub fn pop(&mut self) -> Option<T> {
        self.store.pop().flatten()
    }

    pub fn swap(
        &mut self,
        ref_1: ValueRef<T>,
        ref_2: ValueRef<T>,
    ) -> Option<(ValueRef<T>, ValueRef<T>)> {
        if ref_1.index >= self.store.len() || ref_2.index >= self.store.len() {
            return None;
        }
        self.store.swap(ref_1.index, ref_2.index);
        Some((ref_2, ref_1))
    }

    pub fn next_push_ref(&self) -> ValueRef<T> {
        if self.open_indicis.len() == 0 {
            return ValueRef::new(self.store.len());
        }
        ValueRef::new(*self.open_indicis.last().unwrap())
    }

    /// replaces value at reference with None and returns value
    pub fn take(&mut self, reference: ValueRef<T>) -> Option<T> {
        let mut tmp = None;
        std::mem::swap(&mut tmp, self.store.get_mut(reference.index)?);
        tmp
    }

    pub fn reserve(&mut self, additional: usize) {
        self.store.reserve(additional);
    }

    pub fn has_item(&self, reference: ValueRef<T>) -> bool {
        self.get(reference).is_some()
    }
}
