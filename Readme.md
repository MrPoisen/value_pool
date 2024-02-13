This crate implements a ValuePool struct that makes the creation of self-referential datastructures easier and safer. 

- [Docs](https://docs.rs/value_pool/0.1.0/value_pool/)

# Features
- `unsafe` - uses unsafe code for (potential) speed improvements. This should not create UB or change the behavior off your code.  


# Example
```rust
use value_pool::{ValuePool, ValueRef};

#[derive(Debug, Clone)]
struct Node<T> { // Our self-referential struct
    value: T,
    next: Option<ValueRef<Node<T>>>, // Note the `ValueRef<Node<T>>`
}

impl<T> Node<T> {
    fn new(value: T, next: Option<ValueRef<Node<T>>>) -> Node<T> {
        Node { value, next }
    }
}
#[derive(Debug, Clone)]
struct LinkedList<T> {
    start: ValueRef<Node<T>>,
    end: ValueRef<Node<T>>,
    store: ValuePool<Node<T>>,
}

impl<T> LinkedList<T> {
    pub fn new() -> LinkedList<T> {
        // We can use a ValueRef equal to 0 and then check in all methods if `store.is_empty`
        LinkedList {
            start: (ValueRef::new(0)),
            end: (ValueRef::new(0)),
            store: (ValuePool::new()),
        }
    }

    pub fn push_back(&mut self, value: T) -> Option<()> {
        // will return Option<()> here for ease of use
        // think of reference as a weird "pointer"
        let reference = self.store.push(Node::new(value, None));

        // Note: There is no guarantee that calling `self.store.push` on an empty store will
        // return ValueRef::new(0). We have to make sure self.start points to the right element
        if self.store.element_count() == 1 { // We just pushed the first value
            self.start = reference;
        }
        self.store.get_mut(self.end)?.next = Some(reference); 
        self.end = reference;

        Some(())
    }

    pub fn pop_front(&mut self) -> Option<T> {
        // will return Option<T> here in case self.store is empty
        if self.store.is_empty() {
            return None;
        }
        // We expect that `self.store.take(self.start)` returns Some(...) if our LinkedList is in a valid state.
        // We could use `unwrap_unchecked` if we can guarantee that our LinkedList is in such a state or if we don't care about our safety.
        let first_node = self.store.take(self.start)?;
        // We can't use `.unwrap` or `?` here, because `first_node` could have no `first_node.next` node.
        // We don't want a panic or early return. We want to return `first_node.value`.
        // Note: `ValueRef::default() == ValueRef::new(0)`
        self.start = first_node.next.unwrap_or_default();

        Some(first_node.value)
    }
}
```