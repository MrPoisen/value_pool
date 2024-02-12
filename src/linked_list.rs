//! This file includes a DoubleLinkedList implementation made with a ValuePool
//!

use std::collections::BTreeMap;

use crate::{ValuePool, ValueRef};

#[cfg(feature = "nightly")]
fn closest_entry<V>(tree: &BTreeMap<usize, V>, key: usize) -> Option<(&usize, &V)> {
    use std::ops::Bound::*;
    use std::collections::btree_map::Cursor;
    let lower_bound_cursor:Cursor<usize, V> = tree.lower_bound(Included(&key));
    let lower_bound = lower_bound_cursor.peek_prev(); //lower_bound_cursor.key();
    let upper_bound = lower_bound_cursor.peek_next();

    if let Some((lower_index, lower_value)) = lower_bound {
        if upper_bound.is_none() {
            return Some((lower_index, lower_value));
        }

        let (upper_index, upper_value);
        #[cfg(feature = "unsafe")]
        unsafe {
            (upper_index, upper_value) = upper_bound.unwrap_unchecked();
        }
        #[cfg(not(feature = "unsafe"))]
        {
            (upper_index, upper_value) = upper_bound?;
        }

        if (key - *lower_index) >= (*upper_index - key) {
            return Some((upper_index, upper_value));
        } else {
            return Some((lower_index, lower_value));
            
        }
    } else {
        return upper_bound;
    }
}

#[cfg(not(feature="nightly"))]
//TODO: optimise this function, currently nightly is needed for faster version
fn closest_entry<V>(tree: &BTreeMap<usize, V>, key: usize) -> Option<(&usize, &V)> {
    use std::ops::Bound::*;
    let mut before = tree.range((Unbounded, Included(key)));
    let mut after = tree.range((Included(key), Unbounded));

    // note: if some, after_next.0 is >= before_last.0
    let before_last = before.next_back();
    let after_next = after.next();

    if let Some((index_before, value_before)) = before_last {
        if after_next.is_none() {
            return before_last;
        }
        // both before_last and after_next are some
        //let (index_after, value_after) = after_next.unwrap();
        let (index_after, value_after);
        #[cfg(feature = "unsafe")]
        unsafe {
            (index_after, value_after) = after_next.unwrap_unchecked();
        }
        #[cfg(not(feature = "unsafe"))]
        {
            (index_after, value_after) = after_next?;
        }

        if (key - *index_before) >= (*index_after - key) {
            return Some((index_after, value_after));
        } else {
            return Some((index_before, value_before));
        }
    } else {
        return after_next;
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct DoubleLinkedView<T> {
    store_index: ValueRef<DoubleLinkedNode<T>>,
}

impl<T> DoubleLinkedView<T> {
    fn new(v: ValueRef<DoubleLinkedNode<T>>) -> DoubleLinkedView<T> {
        DoubleLinkedView { store_index: (v) }
    }
}

#[derive(Clone, Debug)]
struct DoubleLinkedNode<T> {
    value: T,
    prev: Option<ValueRef<DoubleLinkedNode<T>>>,
    next: Option<ValueRef<DoubleLinkedNode<T>>>,
}

pub struct DoubleLinkedListIterator<'a, T> {
    dl_list: &'a DoubleLinkedList<T>,
    current_ref: Option<ValueRef<DoubleLinkedNode<T>>>,
    remaining_size: usize,
}

pub struct DoubleLinkedListReverseIterator<'a, T> {
    dl_list: &'a DoubleLinkedList<T>,
    current_ref: Option<ValueRef<DoubleLinkedNode<T>>>,
    remaining_size: usize,
}

pub struct DoubleLinkedListIntoIterator<T> {
    dl_list: DoubleLinkedList<T>,
    current_ref: Option<ValueRef<DoubleLinkedNode<T>>>,
}

impl<'a, T> Iterator for DoubleLinkedListIterator<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        let node = self.dl_list.store.get(self.current_ref?)?;
        self.remaining_size -= 1;
        self.current_ref = node.next;
        Some(&node.value)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining_size, Some(self.remaining_size))
    }
}
impl<'a, T> Iterator for DoubleLinkedListReverseIterator<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        let node = self.dl_list.store.get(self.current_ref?)?;
        self.remaining_size -= 1;
        self.current_ref = node.prev;
        Some(&node.value)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining_size, Some(self.remaining_size))
    }
}

impl<T> Iterator for DoubleLinkedListIntoIterator<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        let node = self.dl_list.store.take(self.current_ref?)?;
        self.current_ref = node.next;
        Some(node.value)
    }
}
#[derive(Clone, Debug)]
pub struct DoubleLinkedList<T> {
    store: ValuePool<DoubleLinkedNode<T>>,
    start: ValueRef<DoubleLinkedNode<T>>,
    end: ValueRef<DoubleLinkedNode<T>>,
}

impl<T> DoubleLinkedList<T> {
    #[inline]
    fn index_to_valueref(&self, index: usize) -> Option<ValueRef<DoubleLinkedNode<T>>> {
        if index >= self.len() {
            return None;
        } else if index == self.len() - 1 {
            return Some(self.end);
        } else if index == 0 {
            return Some(self.start);
        }
        if index > self.len() / 2 {
            let mut node_idx = self.end;
            let mut iteration_index = index;
            while iteration_index < self.len() - 1 {
                // cause self.length-1 is the last index
                #[cfg(feature = "unsafe")]
                {
                    node_idx = unsafe {
                        self.store
                            .get_unchecked(node_idx)
                            .unwrap_unchecked()
                            .prev
                            .unwrap_unchecked()
                    };
                }
                #[cfg(not(feature = "unsafe"))]
                {
                    node_idx = self.store.get(node_idx)?.prev?;
                }

                iteration_index += 1;
            }
            return Some(node_idx);
        }
        let mut node_idx = self.start;
        let mut iteration_index = 0usize;
        while iteration_index < index {
            #[cfg(feature = "unsafe")]
            {
                node_idx = unsafe {
                    self.store
                        .get_unchecked(node_idx)
                        .unwrap_unchecked()
                        .next
                        .unwrap_unchecked()
                };
            }
            #[cfg(not(feature = "unsafe"))]
            {
                node_idx = self.store.get(node_idx)?.next?
            };
            iteration_index += 1;
        }
        Some(node_idx)
    }

    pub fn get_left_neighbour(
        &self,
        view: &DoubleLinkedView<T>,
        n: usize,
    ) -> Option<DoubleLinkedView<T>> {
        if n == 0 {
            return Some(DoubleLinkedView {
                store_index: view.store_index,
            });
        }
        let mut value_ref = view.store_index;

        // could i use get_unchecked?
        for _ in 0..(n) {
            value_ref = self.store.get(value_ref)?.prev?;
        }
        Some(DoubleLinkedView {
            store_index: (value_ref),
        })
    }

    pub unsafe fn get_unchecked_left_neighbour(
        &self,
        view: &DoubleLinkedView<T>,
        n: usize,
    ) -> DoubleLinkedView<T> {
        if n == 0 {
            return DoubleLinkedView {
                store_index: view.store_index,
            };
        }
        let mut value_ref = view.store_index;

        // could i use get_unchecked?
        for _ in 0..(n) {
            value_ref = self
                .store
                .get_unchecked(value_ref)
                .unwrap_unchecked()
                .prev.unwrap_unchecked();
        }
        DoubleLinkedView {
            store_index: (value_ref),
        }
    }

    pub fn get_right_neighbour(
        &self,
        view: &DoubleLinkedView<T>,
        n: usize,
    ) -> Option<DoubleLinkedView<T>> {
        if n == 0 {
            return Some(DoubleLinkedView {
                store_index: view.store_index,
            });
        }

        let mut value_ref = view.store_index;
        for _ in 0..n {
            value_ref = self.store.get(value_ref)?.next?;
        }
        Some(DoubleLinkedView {
            store_index: (value_ref),
        })
    }

    pub unsafe fn get_unchecked_right_neighbour(
        &self,
        view: &DoubleLinkedView<T>,
        n: usize,
    ) -> DoubleLinkedView<T> {
        if n == 0 {
            return DoubleLinkedView {
                store_index: view.store_index,
            };
        }

        let mut value_ref = view.store_index;
        for _ in 0..n {
            value_ref = self
                .store
                .get_unchecked(value_ref)
                .unwrap_unchecked()
                .next.unwrap_unchecked();
        }
        DoubleLinkedView {
            store_index: (value_ref),
        }
    }

    pub fn new() -> DoubleLinkedList<T> {
        let store: ValuePool<DoubleLinkedNode<T>> = ValuePool::new();
        DoubleLinkedList {
            store: (store),
            start: (ValueRef::new(0)),
            end: (ValueRef::new(0)),
        }
    }

    pub fn with_capacity(capacity: usize) -> DoubleLinkedList<T> {
        let store: ValuePool<DoubleLinkedNode<T>> = ValuePool::with_capacity(capacity);
        DoubleLinkedList {
            store: (store),
            start: (ValueRef::new(0)),
            end: (ValueRef::new(0)),
        }
    }

    pub fn push(&mut self, value: T) -> DoubleLinkedView<T> {
        if self.store.element_count() == 0 {
            self.start = self.store.push(DoubleLinkedNode {
                value: (value),
                prev: (None),
                next: (None),
            });
            self.end = self.start;
            return DoubleLinkedView {
                store_index: (self.start),
            };
        }
        let cur_last_ref: ValueRef<DoubleLinkedNode<T>> = self.end;
        let node = DoubleLinkedNode {
            value: (value),
            prev: Some(cur_last_ref),
            next: (None),
        };
        let new_node_ref = self.store.push(node);

        #[cfg(feature = "unsafe")]
        unsafe {
            self.store
                .get_unchecked_mut(cur_last_ref)
                .unwrap_unchecked()
                .next = Some(new_node_ref);
        }
        #[cfg(not(feature = "unsafe"))]
        {
            self.store.get_mut(cur_last_ref).unwrap().next = Some(new_node_ref);
        }

        self.end = new_node_ref;
        DoubleLinkedView {
            store_index: (new_node_ref),
        }
    }

    pub fn multi_push(&mut self, mut values: impl Iterator<Item = T>) -> Option<()> {
        let size_hint = values.size_hint();
        self.store.reserve(size_hint.1.unwrap_or(size_hint.0));
        let mut last_node_view;
        if self.len() == 0 {
            last_node_view = self.store.push(DoubleLinkedNode {
                value: (values.next()?),
                prev: (None),
                next: (None),
            });
            self.start = last_node_view;
        } else {
            last_node_view = self.store.push(DoubleLinkedNode {
                value: (values.next()?),
                prev: Some(self.end),
                next: (None),
            });
            // should never panic
            #[cfg(feature = "unsafe")]
            unsafe {
                self.store
                    .get_unchecked_mut(self.end)
                    .unwrap_unchecked()
                    .next = Some(last_node_view)
            }
            #[cfg(not(feature = "unsafe"))]
            {
                self.store.get_mut(self.end)?.next = Some(last_node_view);
            }
        }

        for value in values {
            let node = DoubleLinkedNode {
                value,
                prev: Some(last_node_view),
                next: (None),
            };
            let about_to_be_second_last_view = last_node_view;
            last_node_view = self.store.push(node);
            #[cfg(feature = "unsafe")]
            unsafe {
                self.store
                    .get_unchecked_mut(about_to_be_second_last_view)
                    .unwrap_unchecked()
                    .next = Some(last_node_view)
            }
            #[cfg(not(feature = "unsafe"))]
            {
                self.store
                    .get_mut(about_to_be_second_last_view)
                    .unwrap()
                    .next = Some(last_node_view);
            }
            //self.store.get_mut(about_to_be_second_last_view).unwrap().next = Some(last_node_view);
        }
        self.end = last_node_view;

        Some(())
    }
    pub fn push_front(&mut self, value: T) -> DoubleLinkedView<T> {
        //
        if self.len() == 0 {
            self.start = self.store.push(DoubleLinkedNode {
                value: (value),
                prev: (None),
                next: (None),
            });
            self.end = self.start;
            return DoubleLinkedView {
                store_index: (self.start),
            };
        }
        // self.start should always be valid
        self.insert_left(
            &DoubleLinkedView {
                store_index: (self.start),
            },
            value,
        )
        .unwrap()
    }

    pub fn multi_push_front(&mut self, mut values: impl Iterator<Item = T>) -> Option<()> {
        let size_hint = values.size_hint();
        self.store.reserve(size_hint.1.unwrap_or(size_hint.0));
        let mut first_node_view;
        if self.len() == 0 {
            first_node_view = self.store.push(DoubleLinkedNode {
                value: (values.next()?),
                prev: (None),
                next: (None),
            });
            self.start = first_node_view;
        } else {
            first_node_view = self.store.push(DoubleLinkedNode {
                value: (values.next()?),
                prev: (None),
                next: Some(self.start),
            });
            // should never panic
            #[cfg(feature = "unsafe")]
            unsafe {
                self.store
                    .get_unchecked_mut(self.end)
                    .unwrap_unchecked()
                    .prev = Some(first_node_view)
            }
            #[cfg(not(feature = "unsafe"))]
            {
                self.store.get_mut(self.end).unwrap().prev = Some(first_node_view);
            }
            //self.store.get_mut(self.start).unwrap().prev = Some(first_node_view);
        }

        for value in values {
            let node = DoubleLinkedNode {
                value,
                prev: (None),
                next: (Some(first_node_view)),
            };
            let about_to_be_second_first_view = first_node_view;
            first_node_view = self.store.push(node);

            // should never panic
            #[cfg(feature = "unsafe")]
            unsafe {
                self.store
                    .get_unchecked_mut(about_to_be_second_first_view)
                    .unwrap_unchecked()
                    .prev = Some(first_node_view)
            }
            #[cfg(not(feature = "unsafe"))]
            {
                self.store
                    .get_mut(about_to_be_second_first_view)
                    .unwrap()
                    .prev = Some(first_node_view);
            }
            //self.store.get_mut(about_to_be_second_first_view).unwrap().prev = Some(first_node_view);
        }
        self.start = first_node_view;

        Some(())
    }

    pub fn pop(&mut self) -> Option<T> {
        let last_node = self.store.get_mut(self.end)?;
        let before_last_ref = last_node.prev.unwrap_or(ValueRef::new(0)); // in case this is the first value

        self.store.get_mut(before_last_ref)?.next = None;
        let value_taken = self.store.take(self.end);
        self.end = before_last_ref;
        value_taken.map(|x| x.value)
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        #[cfg(feature = "unsafe")]
        unsafe {
            if index >= self.len() {
                None
            } else {
                self.store
                    .get_unchecked(self.index_to_valueref(index)?)
                    .map(|x| &x.value)
            }
        }

        #[cfg(not(feature = "unsafe"))]
        {
            self.store
                .get(self.index_to_valueref(index)?)
                .map(|x| &x.value)
        }
    }

    //TODO: improve performence
    pub fn multi_get_view(
        &self,
        indexes: impl Iterator<Item = usize>,
    ) -> Option<Vec<DoubleLinkedView<T>>> {
        let size_hint = indexes.size_hint();
        let mut views = Vec::with_capacity(size_hint.1.unwrap_or(size_hint.0));
        let mut store_index_views: BTreeMap<usize, DoubleLinkedView<T>> = BTreeMap::new();
        
        store_index_views.insert(
            0,
            DoubleLinkedView {
                store_index: self.start,
            },
        );
        store_index_views.insert(
            self.len() - 1,
            DoubleLinkedView {
                store_index: self.end,
            },
        );

        for index in indexes {
            if index >= self.len() {
                continue;
            }
            let (&closest_found_index, closest_found_view) =
                closest_entry(&store_index_views, index)?;
            
            let true_view;
            #[cfg(feature = "unsafe")]
            unsafe {
                if index <= closest_found_index {
                    true_view = self.get_unchecked_left_neighbour(
                        closest_found_view,
                        closest_found_index - index,
                    );
                } else {
                    true_view = self.get_unchecked_right_neighbour(
                        closest_found_view,
                        index - closest_found_index,
                    );
                }
            }
            #[cfg(not(feature = "unsafe"))]
            {
                if index <= closest_found_index {
                    true_view =
                        self.get_left_neighbour(closest_found_view, closest_found_index - index)?;
                } else {
                    true_view =
                        self.get_right_neighbour(closest_found_view, index - closest_found_index)?;
                }
            }
            views.push(DoubleLinkedView {
                store_index: (true_view.store_index),
            });

            store_index_views.insert(index, true_view);
        }
        Some(views)
    }

    pub fn multi_get(
        &self,
        indexes: impl Iterator<Item = usize>,
    ) -> Option<Vec<&T>> {
        let size_hint = indexes.size_hint();
        let mut borrows = Vec::with_capacity(size_hint.1.unwrap_or(size_hint.0));
        let mut store_index_views: BTreeMap<usize, DoubleLinkedView<T>> = BTreeMap::new();
        
        store_index_views.insert(
            0,
            DoubleLinkedView {
                store_index: self.start,
            },
        );
        store_index_views.insert(
            self.len() - 1,
            DoubleLinkedView {
                store_index: self.end,
            },
        );

        for index in indexes {
            if index >= self.len() {
                continue;
            }
            let (&closest_found_index, closest_found_view) =
                closest_entry(&store_index_views, index)?;
            let true_view;
            
            #[cfg(feature = "unsafe")]
            unsafe {
                if index <= closest_found_index {
                    true_view = self.get_unchecked_left_neighbour(
                        closest_found_view,
                        closest_found_index - index,
                    );
                } else {
                    true_view = self.get_unchecked_right_neighbour(
                        closest_found_view,
                        index - closest_found_index,
                    );
                }
            }
            #[cfg(not(feature = "unsafe"))]
            {
                if index <= closest_found_index {
                    true_view =
                        self.get_left_neighbour(closest_found_view, closest_found_index - index)?;
                } else {
                    true_view =
                        self.get_right_neighbour(closest_found_view, index - closest_found_index)?;
                }
            }
            #[cfg(feature = "unsafe")]
            unsafe{borrows.push(&self.store.get_unchecked(true_view.store_index).unwrap_unchecked().value);}
            #[cfg(not(feature = "unsafe"))]
            {borrows.push(&self.store.get(true_view.store_index)?.value);}
            

            store_index_views.insert(index, true_view);
        }
        Some(borrows)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        #[cfg(feature = "unsafe")]
        unsafe {
            if index >= self.len() {
                None
            } else {
                self.store
                    .get_unchecked_mut(self.index_to_valueref(index)?)
                    .map(|x| &mut x.value)
            }
        }

        #[cfg(not(feature = "unsafe"))]
        {
            self.store
                .get_mut(self.index_to_valueref(index)?)
                .map(|x| &mut x.value)
        }
    }

    /// `inner_swap` swaps to values as pointed to by the views. It returns new views as the given ones have been moved.
    ///  
    /// This function can invalidate `DoubleLinkedView<T>`s. If this happens, your programm might panic if it doesn't account for this.
    /// Using `inner_swap` can result in better cache-locality then `swap`.
    /// // runs long, why?
    /// ```
    /// use value_pool::linked_list::DoubleLinkedList;
    /// let mut l = DoubleLinkedList::from(vec![12,13,14,2,1,2,4,5]);
    /// let mut view_to_13 = l.get_view(1).unwrap(); // view points to 13
    /// let mut view_to_1 = l.get_view(4).unwrap(); // view points to 1
    /// let another_view_to_1 = l.get_view(4).unwrap(); // view points to 1
    ///
    /// // returned views are in the same order as the input
    /// let (view_to_13,view_to_1) = unsafe{l.inner_swap(view_to_13, view_to_1).unwrap()}; // returns Some((view_to_13, view_to_1)) if the values have been swapped (includes swapping view_to_1 with another_view_to_1)
    /// assert_eq!(another_view_to_1, view_to_13);
    /// ```
    pub unsafe fn inner_swap(
        &mut self,
        view1: DoubleLinkedView<T>,
        view2: DoubleLinkedView<T>,
    ) -> Option<(DoubleLinkedView<T>, DoubleLinkedView<T>)> {
        let node1_prev;
        let node1_next;
        let node2_prev;
        let node2_next;
        {
            let node1 = self.store.get(view1.store_index)?;
            //let node2 = self.store.get_mut(view2.store_index)?;

            node1_prev = node1.prev;
            node1_next = node1.next;
        }
        {
            let node2 = self.store.get(view2.store_index)?;
            //let node2 = self.store.get_mut(view2.store_index)?;

            node2_prev = node2.prev;
            node2_next = node2.next;
        }
        // reassign node1
        {
            let node1 = self.store.get_mut(view1.store_index)?;
            // if ... <-> N_1 <-> N_2 <-> ... => N_1.prev must point to view1.store_index (current index of N_1, will be index of N_2)
            node1.prev = match node2_prev {
                Some(x) if x == view1.store_index => Some(view1.store_index),
                x => x,
            };
            node1.next = match node2_next {
                Some(x) if x == view1.store_index => Some(view1.store_index),
                x => x,
            };
        }
        // reassign node2
        {
            let node2 = self.store.get_mut(view2.store_index)?;
            node2.prev = match node1_prev {
                Some(x) if x == view2.store_index => Some(view2.store_index),
                x => x,
            };
            node2.next = match node1_next {
                Some(x) if x == view2.store_index => Some(view2.store_index),
                x => x,
            };
        }
        self.store.swap(view1.store_index, view2.store_index);
        Some((view2, view1))
    }

    pub fn swap(&mut self, view1: &DoubleLinkedView<T>, view2: &DoubleLinkedView<T>) -> Option<()> {
        let node1_prev;
        let node1_next;
        let node2_prev;
        let node2_next;
        {
            let node1 = self.store.get(view1.store_index)?;
            //let node2 = self.store.get_mut(view2.store_index)?;

            node1_prev = node1.prev;
            node1_next = node1.next;
        }
        {
            let node2 = self.store.get(view2.store_index)?;
            //let node2 = self.store.get_mut(view2.store_index)?;

            node2_prev = node2.prev;
            node2_next = node2.next;
        }

        // reassign node1
        {
            let node1 = self.store.get_mut(view1.store_index)?;
            node1.prev = match node2_prev {
                Some(x) if x == view1.store_index => node1_next,
                x => x,
            };
            node1.next = match node2_next {
                Some(x) if x == view1.store_index => node1_prev,
                x => x,
            };
        }
        // reassign neighbours of old node1
        {
            if let Some(left) = node1_prev {
                if left != view2.store_index {
                    self.store.get_mut(left)?.next = Some(view2.store_index);
                }
            }
            if let Some(right) = node1_next {
                if right != view2.store_index {
                    self.store.get_mut(right)?.prev = Some(view2.store_index);
                }
            }
        }
        // reassign node2
        {
            let node2 = self.store.get_mut(view2.store_index)?;
            node2.prev = match node1_prev {
                Some(x) if x == view2.store_index => node2_next,
                x => x,
            };
            node2.next = match node1_next {
                Some(x) if x == view2.store_index => node2_prev,
                x => x,
            };
        }
        // reassign neighbours of old node2
        {
            if let Some(left) = node2_prev {
                if left != view1.store_index {
                    self.store.get_mut(left)?.next = Some(view1.store_index);
                }
            }
            if let Some(right) = node2_next {
                if right != view1.store_index {
                    self.store.get_mut(right)?.prev = Some(view1.store_index);
                }
            }
        }
        if view1.store_index == self.start {
            self.start = view2.store_index;
        } else if view2.store_index == self.start {
            self.start = view1.store_index;
        }
        if view1.store_index == self.end {
            self.end = view2.store_index;
        } else if view2.store_index == self.end {
            self.end = view1.store_index;
        }
        Some(())
    }

    pub fn get_view(&self, index: usize) -> Option<DoubleLinkedView<T>> {
        Some(DoubleLinkedView::new(self.index_to_valueref(index)?))
    }

    pub fn peek_view(&self, view: DoubleLinkedView<T>) -> Option<&T> {
        self.store.get(view.store_index).map(|x| &x.value)
    }
    pub fn peek_view_mut(&mut self, view: DoubleLinkedView<T>) -> Option<&mut T> {
        self.store.get_mut(view.store_index).map(|x| &mut x.value)
    }

    pub fn insert_left(
        &mut self,
        view: &DoubleLinkedView<T>,
        value: T,
    ) -> Option<DoubleLinkedView<T>> {
        let view_node_prev = self.store.get(view.store_index)?.prev;
        let new_node = DoubleLinkedNode {
            value,
            prev: view_node_prev,
            next: Some(view.store_index),
        };

        let new_node_ref = self.store.push(new_node);

        // modify old left node
        if let Some(left) = view_node_prev {
            #[cfg(feature = "unsafe")]
            unsafe {
                self.store.get_unchecked_mut(left).unwrap_unchecked().next = Some(new_node_ref);
            }
            #[cfg(not(feature = "unsafe"))]
            {
                self.store.get_mut(left).unwrap().next = Some(new_node_ref);
            }
        } else {
            self.start = new_node_ref;
        }

        // modify self
        #[cfg(feature = "unsafe")]
        unsafe {
            self.store
                .get_unchecked_mut(view.store_index)
                .unwrap_unchecked()
                .prev = Some(new_node_ref);
        }
        #[cfg(not(feature = "unsafe"))]
        {
            self.store.get_mut(view.store_index).unwrap().prev = Some(new_node_ref);
        }

        Some(DoubleLinkedView {
            store_index: (new_node_ref),
        })
    }

    pub fn insert_right(
        &mut self,
        view: &DoubleLinkedView<T>,
        value: T,
    ) -> Option<DoubleLinkedView<T>> {
        let view_node_next = self.store.get(view.store_index)?.next;
        let new_node = DoubleLinkedNode {
            value,
            prev: Some(view.store_index),
            next: (view_node_next),
        };

        let new_node_ref = self.store.push(new_node);

        // modify old left node
        if let Some(right) = view_node_next {
            #[cfg(feature = "unsafe")]
            unsafe {
                self.store.get_unchecked_mut(right).unwrap_unchecked().prev = Some(new_node_ref);
            }
            #[cfg(not(feature = "unsafe"))]
            {
                self.store.get_mut(right).unwrap().prev = Some(new_node_ref);
            }
        } else {
            self.end = new_node_ref;
        }

        // modify self
        #[cfg(feature = "unsafe")]
        unsafe {
            self.store
                .get_unchecked_mut(view.store_index)
                .unwrap_unchecked()
                .next = Some(new_node_ref);
        }
        #[cfg(not(feature = "unsafe"))]
        {
            self.store.get_mut(view.store_index).unwrap().next = Some(new_node_ref);
        }

        Some(DoubleLinkedView {
            store_index: (new_node_ref),
        })
    }

    #[inline]
    pub fn insert(&mut self, index: usize, value: T) -> Option<DoubleLinkedView<T>> {
        let node_ref = self.index_to_valueref(index)?;
        // insert left should increase self.length
        self.insert_left(
            &DoubleLinkedView {
                store_index: (node_ref),
            },
            value,
        )
        //self.store.get_mut(node_ref)?.insert_left(value, self);
    }

    pub fn multi_insert(&mut self, iter: impl Iterator<Item = (usize, T)>) -> Option<()> {
        let size_hint = iter.size_hint();
        self.store.reserve(size_hint.1.unwrap_or(size_hint.0));
        let mut store_index_views: BTreeMap<usize, DoubleLinkedView<T>> = BTreeMap::new();
        store_index_views.insert(
            0,
            DoubleLinkedView {
                store_index: self.start,
            },
        );
        store_index_views.insert(
            self.len() - 1,
            DoubleLinkedView {
                store_index: self.end,
            },
        );

        for (index, value) in iter {
            if index >= self.len() {
                continue;
            }
            let (&closest_found_index, closest_found_view) =
                closest_entry(&store_index_views, index)?;
            let true_view;

            // should be safe if index out of bounds because closest_entry would then be None
            #[cfg(feature = "unsafe")]
            unsafe {
                if index <= closest_found_index {
                    true_view = self.get_unchecked_left_neighbour(
                        closest_found_view,
                        closest_found_index - index,
                    );
                } else {
                    true_view = self.get_unchecked_right_neighbour(
                        closest_found_view,
                        index - closest_found_index,
                    );
                }
            }
            #[cfg(not(feature = "unsafe"))]
            {
                if index <= closest_found_index {
                    true_view =
                        self.get_left_neighbour(closest_found_view, closest_found_index - index)?;
                } else {
                    true_view =
                        self.get_right_neighbour(closest_found_view, index - closest_found_index)?;
                }
            }

            let new_view = self.insert_left(&true_view, value)?;
            store_index_views.insert(index, new_view);
            // all indexs after index now point to the view at index+1 instead index
            for (_, view) in store_index_views.range_mut((index + 1)..self.len()) {
                #[cfg(feature = "unsafe")]
                unsafe {
                    *view = DoubleLinkedView {
                        store_index: self
                            .store
                            .get_unchecked(view.store_index)
                            .unwrap_unchecked()
                            .prev
                            .unwrap_unchecked(),
                    };
                }
                #[cfg(not(feature = "unsafe"))]
                {
                    *view = DoubleLinkedView {
                        store_index: self.store.get(view.store_index)?.prev?,
                    };
                }
            }
        }
        Some(())
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.store.element_count()
    }

    #[inline]
    pub fn iter(&self) -> DoubleLinkedListIterator<T> {
        DoubleLinkedListIterator {
            dl_list: (self),
            current_ref: Some(self.start),
            remaining_size: (self.len()),
        }
    }
    #[inline]
    pub fn iter_reverse(&self) -> DoubleLinkedListReverseIterator<T> {
        DoubleLinkedListReverseIterator {
            dl_list: (self),
            current_ref: Some(self.end),
            remaining_size: (self.len()),
        }
    }
}

impl<T> IntoIterator for DoubleLinkedList<T> {
    type IntoIter = DoubleLinkedListIntoIterator<T>;
    type Item = T;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        DoubleLinkedListIntoIterator {
            current_ref: Some(self.start),
            dl_list: self,
        }
    }
}
impl<'a, T> IntoIterator for &'a DoubleLinkedList<T> {
    type IntoIter = DoubleLinkedListIterator<'a, T>;
    type Item = &'a T;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        DoubleLinkedListIterator {
            dl_list: self,
            current_ref: Some(self.start),
            remaining_size: (self.len()),
        }
    }
}
// would multi_push be faster?
impl<T> From<Vec<T>> for DoubleLinkedList<T> {
    #[inline]
    fn from(value: Vec<T>) -> Self {
        let mut dl_list = DoubleLinkedList::with_capacity(value.len() * 2);
        dl_list.multi_push(value.into_iter());
        dl_list
    }
}

impl<T> From<DoubleLinkedList<T>> for Vec<T> {
    #[inline]
    fn from(value: DoubleLinkedList<T>) -> Self {
        value.into_iter().collect()
    }
}

pub unsafe fn reuse_insert_left<T>(
    dll: &mut DoubleLinkedList<T>,
    last_insert: (usize, &DoubleLinkedView<T>),
    new_insert: (usize, T),
) -> Option<DoubleLinkedView<T>> {
    let distance_to_start = new_insert.0;
    let distance_to_end = dll.len() - new_insert.0;

    let normal_minimal_distance = distance_to_end.min(distance_to_start);

    if new_insert.0 >= last_insert.0 {
        // compare index, new index after last insert index?
        let distance_to_last = new_insert.0 - last_insert.0;
        if distance_to_last < normal_minimal_distance {
            let target_view = dll.get_right_neighbour(last_insert.1, distance_to_last)?;
            return dll.insert_left(&target_view, new_insert.1);
        }
    } else {
        let distance_to_last = last_insert.0 - new_insert.0;
        if distance_to_last < normal_minimal_distance {
            let target_view = dll.get_left_neighbour(last_insert.1, distance_to_last)?;
            return dll.insert_left(&target_view, new_insert.1);
        }
    }
    dll.insert_left(&dll.get_view(new_insert.0)?, new_insert.1)
}

pub unsafe fn reuse_insert_right<T>(
    dll: &mut DoubleLinkedList<T>,
    last_insert: (usize, &DoubleLinkedView<T>),
    new_insert: (usize, T),
) -> Option<DoubleLinkedView<T>> {
    if new_insert.0 >= dll.len() {
        return None;
    }
    let distance_to_start = new_insert.0;
    let distance_to_end = dll.len() - new_insert.0;

    let normal_minimal_distance = distance_to_end.min(distance_to_start);

    if new_insert.0 >= last_insert.0 {
        // compare index, new index after last insert index?
        let distance_to_last = new_insert.0 - last_insert.0;
        if distance_to_last < normal_minimal_distance {
            let target_view = dll.get_unchecked_right_neighbour(last_insert.1, distance_to_last);
            return dll.insert_right(&target_view, new_insert.1);
        }
    } else {
        let distance_to_last = last_insert.0 - new_insert.0;
        if distance_to_last < normal_minimal_distance {
            let target_view = dll.get_unchecked_left_neighbour(last_insert.1, distance_to_last);
            return dll.insert_right(&target_view, new_insert.1);
        }
    }
    dll.insert_right(&dll.get_view(new_insert.0).unwrap_unchecked(), new_insert.1)
}

#[cfg(test)]
mod test {

    use crate::linked_list::DoubleLinkedView;

    use super::{reuse_insert_left, DoubleLinkedList};

    fn get_ll() -> DoubleLinkedList<u32> {
        let mut l = DoubleLinkedList::new();
        l.push(32);
        l.push(12);
        l.push(55);
        l.push(12);
        l // 32,12,55,12
    }

    fn get_large_data() -> Vec<(u32, usize)> {
        vec![
            (860, 0),
            (587, 0),
            (151, 0),
            (246, 0),
            (77, 3),
            (334, 4),
            (49, 5),
            (448, 4),
            (571, 4),
            (35, 5),
            (45, 1),
            (384, 1),
            (63, 0),
            (959, 10),
            (148, 13),
            (787, 0),
            (610, 16),
            (384, 6),
            (468, 4),
            (399, 16),
            (884, 3),
            (690, 13),
            (275, 18),
            (100, 23),
            (810, 3),
            (828, 16),
            (273, 23),
            (771, 14),
            (70, 24),
            (940, 23),
            (880, 7),
            (522, 15),
            (481, 2),
            (286, 4),
            (911, 5),
            (257, 34),
            (771, 36),
            (565, 15),
            (224, 19),
            (721, 32),
            (194, 31),
            (74, 26),
            (338, 20),
            (197, 16),
            (90, 27),
            (624, 28),
            (416, 24),
            (463, 12),
            (562, 47),
            (454, 19),
            (352, 46),
            (753, 42),
            (195, 10),
            (442, 38),
            (258, 43),
            (998, 24),
            (437, 36),
            (673, 33),
            (525, 47),
            (410, 24),
            (188, 57),
            (788, 39),
        ]
    }
    #[test]
    fn test_get() {
        let l = get_ll();
        assert_eq!(l.len(), 4);
        assert_eq!(l.get(0), Some(&32));
        assert_eq!(l.get(1), Some(&12));
        assert_eq!(l.len(), 4);
        assert_eq!(l.get(2), Some(&55));
        assert_eq!(l.get(3), Some(&12));
        assert_eq!(l.get(4), None);
    }

    #[test]
    fn test_pop() {
        let mut l = get_ll();
        assert_eq!(l.len(), 4);
        assert_eq!(l.pop(), Some(12));
        assert_eq!(l.len(), 3);
        assert_eq!(l.pop(), Some(55));
        assert_eq!(l.pop(), Some(12));
        assert_eq!(l.len(), 1);
        assert_eq!(l.pop(), Some(32));
        assert_eq!(l.len(), 0);
        assert_eq!(l.pop(), None);
    }
    #[test]
    fn test_view() {
        let l = get_ll();
        let view1 = l.get_view(0).unwrap();
        let view2 = l.get_view(l.len() - 1).unwrap();

        assert!(l.get_view(l.len()).is_none());

        assert_eq!(l.peek_view(view1), Some(&32));
        assert_eq!(l.peek_view(view2), Some(&12));
    }

    #[test]
    fn test_insert_left() {
        let mut l = get_ll();
        let view1 = l.get_view(0).unwrap();
        let view2 = l.get_view(l.len() - 1).unwrap();

        assert!(l.insert_left(&view1, 0).is_some());
        assert!(l.insert_left(&view2, 1).is_some());

        assert_eq!(vec![0, 32, 12, 55, 1, 12], Vec::from(l));
    }

    #[test]
    fn test_reuse_insert_left() {
        let mut l = get_ll(); // 32,12,55,12
        let mut old_view = l.push(76); // 32,12,55,12,76
        let old_insert = (l.len() - 1, old_view);

        // => 32,12,10,55,12,76
        unsafe {
            old_view = reuse_insert_left(&mut l, (old_insert.0, &old_insert.1), (2, 10))
                .expect("valid view");
        }

        // => 32,12,10,55,80, 12,76
        unsafe {
            reuse_insert_left(&mut l, (2, &old_view), (4, 80));
        }
        assert_eq!(vec![32, 12, 10, 55, 80, 12, 76], Vec::from(l));
    }

    #[test]
    fn test_insert_right() {
        let mut l = get_ll();
        let view1 = l.get_view(0).unwrap();
        let view2 = l.get_view(l.len() - 1).unwrap();

        assert!(l.insert_right(&view1, 0).is_some());
        assert!(l.insert_right(&view2, 1).is_some());

        assert_eq!(vec![32, 0, 12, 55, 12, 1], Vec::from(l));
    }

    #[test]
    fn test_swap() {
        let mut l = get_ll(); // [32,12,55,12]
        let view1 = l.get_view(0).unwrap(); // 32
        let view2 = l.get_view(l.len() - 1).unwrap(); // last 12

        assert!(l.swap(&view1, &view2).is_some());

        assert_eq!(vec![12, 12, 55, 32], Vec::from(l.clone()));

        let view3 = l.get_view(2).unwrap(); // view should point to 55

        assert!(l.swap(&view1, &view3).is_some());

        assert_eq!(vec![12, 12, 32, 55], Vec::from(l));
    }

    #[test]
    fn test_inner_swap() {
        let mut l = get_ll(); // [32,12,55,12]
        let mut view1 = l.get_view(0).unwrap(); // 32
        let view2 = l.get_view(l.len() - 1).unwrap(); // last 12

        unsafe {
            (view1, _) = l.inner_swap(view1, view2).unwrap();
        }

        assert_eq!(vec![12, 12, 55, 32], Vec::from(l.clone()));

        let view3 = l.get_view(2).unwrap(); // view should point to 55

        unsafe {
            assert!(l.inner_swap(view1, view3).is_some());
        }

        assert_eq!(vec![12, 12, 32, 55], Vec::from(l));
    }

    #[test]
    fn test_insert() {
        let mut l = get_ll();
        let inserts = [(6, 0), (3, 0), (4, 3), (2, 5)];
        for (value, index) in inserts.iter() {
            l.insert(*index, *value);
        }
        assert_eq!(vec![3, 6, 32, 4, 12, 2, 55, 12], Vec::from(l));
    }

    #[test]
    fn test_multi_insert() {
        let mut l = get_ll();
        let mut compare_l = get_ll();
        let data = get_large_data();

        for (value, index) in data.iter() {
            compare_l.insert(*index, *value);
        }
        l.multi_insert(data.into_iter().map(|(v, i)| (i, v)));
        assert_eq!(Vec::from(compare_l), Vec::from(l));
    }

    #[test]
    fn test_multi_insert_2() {
        let mut l = get_ll();
        let mut compare_l = get_ll();
        let data = vec![(3, 10), (1, 20), (0, 30), (2, 40), (4, 50)];

        for (index, value) in data.iter() {
            compare_l.insert(*index, *value);
        }
        l.multi_insert(data.into_iter());
        assert_eq!(Vec::from(compare_l), Vec::from(l));
    }

    #[test]
    fn test_multi_push_get() {
        let mut l = get_ll();
        let mut compare_l = get_ll();
        let data = get_large_data();

        for (value, _) in data.iter() {
            compare_l.push(*value);
        }
        l.multi_push(data.into_iter().map(|(value, _)| value));
        assert_eq!(Vec::from(compare_l.clone()), Vec::from(l.clone()));

        let indexes = vec![12, 2, 14, 3, 5, 20];
        let iterative_gotten_views: Vec<DoubleLinkedView<u32>> = indexes
            .iter()
            .map(|index| l.get_view(*index).unwrap())
            .collect();
        assert_eq!(
            l.multi_get_view(indexes.iter().map(|x| *x)).unwrap(),
            iterative_gotten_views
        );
        assert_eq!(
            compare_l.multi_get_view(indexes.into_iter()).unwrap(),
            iterative_gotten_views
        );
    }

    #[test]
    fn test_multi_push_front() {
        let mut l = get_ll();
        let mut compare_l = get_ll();
        let data = get_large_data();

        for (value, _) in data.iter() {
            compare_l.push_front(*value);
        }
        l.multi_push_front(data.into_iter().map(|(value, _)| value));
        assert_eq!(Vec::from(compare_l), Vec::from(l));
    }
}
