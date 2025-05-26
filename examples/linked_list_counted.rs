use value_pool::counted_value_pool::{CountedValuePool, CountedValueRef};

#[derive(Debug)]
struct Node<T> {
    value: T,
    next: Option<CountedValueRef<Node<T>>>,
}

impl<T> Node<T> {
    fn new(value: T, next: Option<CountedValueRef<Node<T>>>) -> Node<T> {
        Node { value, next }
    }
}
#[derive(Debug)]
struct LinkedList<T> {
    start: Option<CountedValueRef<Node<T>>>, // Need to use Option as there is no sensible default value for CountedValueRef
    end: Option<CountedValueRef<Node<T>>>,
    store: CountedValuePool<Node<T>>,
}

impl<T> LinkedList<T> {
    pub fn new() -> LinkedList<T> {
        // We can use a ValueRef equal to 0 and then check in all methods if `store.is_empty`
        LinkedList {
            start: (None),
            end: (None),
            store: (CountedValuePool::new()),
        }
    }

    pub fn push_back(&mut self, value: T) {
        // will return Option<()> here for ease of use
        // think of reference as a weird "pointer"
        let reference = self.store.push(Node::new(value, None));

        // Note: There is no guarantee that calling `self.store.push` on an empty store will
        // return ValueRef::new(0). We have to make sure self.start points to the right element
        if self.store.element_count() == 1 {
            // We just pushed the first value
            self.start = Some(reference.clone());
            self.end = Some(reference);
            return;
        }
        self.store.get_mut(self.end.as_ref().unwrap()).next = Some(reference.clone());
        self.end = Some(reference);
    }

    pub fn pop_front(&mut self) -> Option<T> {
        // will return Option<T> here in case self.store is empty
        if self.store.is_empty() {
            return None;
        }
        if self.store.element_count() == 1 {
            // self.start == self.end
            self.end = None;
            return self.store.take(self.start.take()?).map(|x| x.value);
        }

        // self.start != self.end
        let first_node = self.store.take(self.start.take()?)?;
        self.start = first_node.next;

        Some(first_node.value)
    }
}

fn main() {
    let example_data = [1, 2, 3, 12, 2, 1, 4, 22, 8, 19];
    let mut ll: LinkedList<i32> = LinkedList::new();
    println!("Creating a populated Linked List");
    for i in example_data.iter() {
        ll.push_back(*i);
    }

    println!("Our Linked List as debug print: {:#?}", &ll);
    println!("first element: {}", ll.pop_front().unwrap());
}
