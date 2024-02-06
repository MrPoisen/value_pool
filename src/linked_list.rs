use crate::{ValuePool, ValueRef};

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
}

pub struct DoubleLinkedListReverseIterator<'a, T> {
    dl_list: &'a DoubleLinkedList<T>,
    current_ref: Option<ValueRef<DoubleLinkedNode<T>>>,
}

pub struct DoubleLinkedListIntoIterator<T> {
    dl_list: DoubleLinkedList<T>,
    current_ref: Option<ValueRef<DoubleLinkedNode<T>>>,
}

impl<'a, T> Iterator for DoubleLinkedListIterator<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        let node = self.dl_list.store.get(self.current_ref?)?;
        self.current_ref = node.next;
        Some(&node.value)
    }
}
impl<'a, T> Iterator for DoubleLinkedListReverseIterator<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        let node = self.dl_list.store.get(self.current_ref?)?;
        self.current_ref = node.prev;
        Some(&node.value)
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
    fn index_to_valueref(&self, index: usize) -> Option<ValueRef<DoubleLinkedNode<T>>> {
        if index >= self.len() {
            return None;
        }
        if index > self.len() / 2 {
            let mut node_idx = self.end;
            let mut iteration_index = index;
            while iteration_index < self.len() - 1 {
                // cause self.length-1 is the last index
                node_idx = self.store.get(node_idx)?.prev?;
                iteration_index += 1;
            }
            return Some(node_idx);
        }
        let mut node_idx = self.start;
        let mut iteration_index = 0usize;
        while iteration_index < index {
            node_idx = self.store.get(node_idx)?.next?;
            iteration_index += 1;
        }
        Some(node_idx)
    }

    pub fn get_left_neighbour(
        &self,
        view: &DoubleLinkedView<T>,
        n: usize,
    ) -> Option<DoubleLinkedView<T>> {
        let mut value_ref = view.store_index;
        for _ in 0..n {
            value_ref = self.store.get(value_ref)?.prev?;
        }
        Some(DoubleLinkedView {
            store_index: (value_ref),
        })
    }
    pub fn get_right_neighbour(
        &self,
        view: &DoubleLinkedView<T>,
        n: usize,
    ) -> Option<DoubleLinkedView<T>> {
        let mut value_ref = view.store_index;
        for _ in 0..n {
            value_ref = self.store.get(value_ref)?.next?;
        }
        Some(DoubleLinkedView {
            store_index: (value_ref),
        })
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

        self.store.get_mut(cur_last_ref).unwrap().next = Some(new_node_ref);

        self.end = new_node_ref;
        DoubleLinkedView {
            store_index: (new_node_ref),
        }
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

    pub fn pop(&mut self) -> Option<T> {
        let last_node = self.store.get_mut(self.end)?;
        let before_last_ref = last_node.prev.unwrap_or(ValueRef::new(0)); // in case this is the first value

        self.store.get_mut(before_last_ref)?.next = None;

        self.end = before_last_ref;
        self.store.pop().map(|x| x.value)
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.store
            .get(self.index_to_valueref(index)?)
            .map(|x| &x.value)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.store
            .get_mut(self.index_to_valueref(index)?)
            .map(|x| &mut x.value)
    }

    /// `inner_swap` swaps to values as pointed to by the views. It returns new views as the given ones have been moved.
    ///  
    /// This function can invalidate ´DoubleLinkedView<T>´s. If this happens, your programm might panic if it doesn't account for this.
    /// Using `inner_swap` can result in better cache-locality then `swap`.
    ///
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
            self.store.get_mut(left).unwrap().next = Some(new_node_ref);
        } else {
            self.start = new_node_ref;
        }

        // modify self
        self.store.get_mut(view.store_index).unwrap().prev = Some(new_node_ref);
        //self.prev = Some(new_node_ref);
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
            self.store.get_mut(right).unwrap().prev = Some(new_node_ref);
        } else {
            self.end = new_node_ref;
        }

        // modify self
        self.store.get_mut(view.store_index).unwrap().next = Some(new_node_ref);
        //self.prev = Some(new_node_ref);
        Some(DoubleLinkedView {
            store_index: (new_node_ref),
        })
    }

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

    pub fn len(&self) -> usize {
        self.store.element_count()
    }

    pub fn iter(&self) -> DoubleLinkedListIterator<T> {
        DoubleLinkedListIterator {
            dl_list: (self),
            current_ref: Some(self.start),
        }
    }
    pub fn iter_reverse(&self) -> DoubleLinkedListReverseIterator<T> {
        DoubleLinkedListReverseIterator {
            dl_list: (self),
            current_ref: Some(self.end),
        }
    }
}

impl<T> IntoIterator for DoubleLinkedList<T> {
    type IntoIter = DoubleLinkedListIntoIterator<T>;
    type Item = T;

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

    fn into_iter(self) -> Self::IntoIter {
        DoubleLinkedListIterator {
            dl_list: self,
            current_ref: Some(self.start),
        }
    }
}

impl<T> From<Vec<T>> for DoubleLinkedList<T> {
    fn from(value: Vec<T>) -> Self {
        let mut dl_list = DoubleLinkedList::with_capacity(value.len() * 2);
        for v in value.into_iter() {
            dl_list.push(v);
        }
        dl_list
    }
}

impl<T> From<DoubleLinkedList<T>> for Vec<T> {
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
    let distance_to_start = new_insert.0;
    let distance_to_end = dll.len() - new_insert.0;

    let normal_minimal_distance = distance_to_end.min(distance_to_start);

    if new_insert.0 >= last_insert.0 {
        // compare index, new index after last insert index?
        let distance_to_last = new_insert.0 - last_insert.0;
        if distance_to_last < normal_minimal_distance {
            let target_view = dll.get_right_neighbour(last_insert.1, distance_to_last)?;
            return dll.insert_right(&target_view, new_insert.1);
        }
    } else {
        let distance_to_last = last_insert.0 - new_insert.0;
        if distance_to_last < normal_minimal_distance {
            let target_view = dll.get_left_neighbour(last_insert.1, distance_to_last)?;
            return dll.insert_right(&target_view, new_insert.1);
        }
    }
    dll.insert_right(&dll.get_view(new_insert.0)?, new_insert.1)
}

#[cfg(test)]
mod test {

    use super::{reuse_insert_left, DoubleLinkedList};

    fn get_ll() -> DoubleLinkedList<u32> {
        let mut l = DoubleLinkedList::new();
        l.push(32);
        l.push(12);
        l.push(55);
        l.push(12);
        l // 32,12,55,12
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
}
