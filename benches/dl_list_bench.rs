//!
//! Following Python function is used to create values:
//! ````python
//! def get_random_values(amount: int):
//!	    from random import randint
//!	    l = []
//!	    for i in range(amount):
//!		    l.append((randint(0,1000),randint(0, i)))
//!	    return l
//! ```
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

mod constant_values;
use constant_values::{VALUES_100, VALUES_1000, VALUES_10000, VALUES_500, VALUES_5000};

mod dl_list {
    use criterion::black_box;
    use value_pool::linked_list::{reuse_insert_left, DoubleLinkedList, DoubleLinkedView};
    pub fn push(values: &[(i32, usize)]) -> DoubleLinkedList<i32> {
        let mut dl = DoubleLinkedList::new();
        for (value, _) in values.iter() {
            dl.push(*value);
        }
        dl
    }

    pub fn multi_push(values: &[(i32, usize)]) {
        let mut dl = DoubleLinkedList::new();
        assert!(dl.multi_push(values.iter().map(|(value, _)| *value)).is_some());
    }

    pub fn insert(values: &[(i32, usize)]) {
        let mut dl = DoubleLinkedList::new();
        dl.push(0);
        for (value, index) in values.iter() {
            dl.insert(*index, *value);
        }
    }

    pub fn improved_insert(values: &[(i32, usize)]) {
        //let mut tmp_insert_view;
        let mut dl = DoubleLinkedList::new();
        let mut last_insert = (0, dl.push(0));
        for (value, index) in values.iter() {
            unsafe {
                last_insert = (
                    *index,
                    reuse_insert_left(&mut dl, (last_insert.0, &last_insert.1), (*index, *value))
                        .expect("All indexes should be valid"),
                );
            }
            //letdl.insert(*index, *value);
        }
    }

    pub fn multi_insert(values: &[(i32, usize)]) {
        let mut dl = DoubleLinkedList::new();
        dl.multi_insert(values.iter().map(|(value, index)| (*index, *value)));
    }

    pub fn push_front(values: &[(i32, usize)]) {
        let mut dl = DoubleLinkedList::new();

        for (value, _) in values.iter() {
            dl.push_front(*value);
        }
    }

    pub fn multi_push_front(values: &[(i32, usize)]) {
        let mut dl = DoubleLinkedList::new();
        dl.multi_push_front(values.iter().map(|(value, _)| *value));
    }

    pub fn from_vec(values: Vec<i32>) {
        let _ = DoubleLinkedList::from(values);
    }

    pub fn get(l: &DoubleLinkedList<i32>, values: &[(i32, usize)]) {
        for (_, index) in values.iter() {
            let _ = black_box(l.get(*index));
        }
    }

    pub fn get_with_iter(l: &DoubleLinkedList<i32>, values: &[(i32, usize)]) {
        for (_, index) in values.iter() {
            let _ = black_box(l.iter().nth(*index));
        }
    }

    pub fn multi_get_view(l: &DoubleLinkedList<i32>, values: &[(i32, usize)]) -> Vec<DoubleLinkedView<i32>>{
        black_box(l.multi_get_view(values.iter().map(|x| x.1))).unwrap()
    }
}

mod vector {
    //use rand::{rngs::ThreadRng, Rng};
    pub fn push(values: &[(i32, usize)]) {
        let mut dl = Vec::new();
        for (value, _) in values.iter() {
            dl.push(*value);
        }
    }

    pub fn insert(values: &[(i32, usize)]) {
        let mut dl = Vec::new();
        dl.push(0);
        for (value, index) in values.iter() {
            dl.insert(*index, *value);
        }
    }

    pub fn push_front(values: &[(i32, usize)]) {
        let mut dl = Vec::new();

        for (value, _) in values.iter() {
            dl.insert(0, *value);
        }
    }
}

mod vector_deq {
    use std::collections::VecDeque;

    //use rand::{rngs::ThreadRng, Rng};
    pub fn push(values: &[(i32, usize)]) {
        let mut dl = VecDeque::new();
        for (value, _) in values.iter() {
            dl.push_back(*value);
        }
    }

    pub fn insert(values: &[(i32, usize)]) {
        let mut dl = VecDeque::new();
        dl.push_back(0);
        for (value, index) in values.iter() {
            dl.insert(*index, *value);
        }
    }

    pub fn push_front(values: &[(i32, usize)]) {
        let mut dl = VecDeque::new();

        for (value, _) in values.iter() {
            dl.push_front(*value);
        }
    }
}

mod linked_list {
    use criterion::black_box;
    use std::collections::LinkedList;
    pub fn push(values: &[(i32, usize)]) -> LinkedList<i32> {
        let mut ll = LinkedList::new();
        for (value, _) in values.iter() {
            ll.push_back(*value);
        }
        ll
    }

    pub fn push_front(values: &[(i32, usize)]) {
        let mut dl = LinkedList::new();

        for (value, _) in values.iter() {
            dl.push_front(*value);
        }
    }

    pub fn insert(values: &[(i32, usize)]) {
        let mut ll = LinkedList::new();
        for (value, index) in values.iter() {
            // includes index
            let mut higher_part = ll.split_off(*index);
            ll.push_back(*value);
            ll.append(&mut higher_part);
        }
    }

    pub fn get(l: &LinkedList<i32>, values: &[(i32, usize)]) {
        for (_, index) in values.iter() {
            let _ = black_box(l.iter().nth(*index));
        }
    }
}

fn dl_list_solo_benchmark(c: &mut Criterion) {
    c.bench_function("from_vec value_pool::DoubeLinkedList 500", |b| {
        b.iter(|| dl_list::from_vec(black_box(&VALUES_500).iter().map(|x| x.0).collect()))
    });
    c.bench_function("from_vec value_pool::DoubeLinkedList 10000", |b| {
        b.iter(|| dl_list::from_vec(black_box(&VALUES_10000).iter().map(|x| x.0).collect()))
    });
}

fn compare_pushes(c: &mut Criterion) {
    let mut group = c.benchmark_group("Container Pushes");

    for data in [
        &VALUES_100[0..10],
        &VALUES_100[0..50],
        VALUES_100.as_slice(),
        VALUES_500.as_slice(),
        VALUES_1000.as_slice(),
        VALUES_5000.as_slice(),
        VALUES_10000.as_slice(),
    ]
    .iter()
    {
        group.bench_with_input(
            BenchmarkId::new("std::vec::Vec::push", data.len()),
            *data,
            |b, i| b.iter(|| vector::push(i)),
        );

        group.bench_with_input(
            BenchmarkId::new("std::collections::VecDeque::push_back", data.len()),
            *data,
            |b, i| b.iter(|| vector_deq::push(i)),
        );

        group.bench_with_input(
            BenchmarkId::new("value_pool::DoubleLinkedList::push", data.len()),
            *data,
            |b, i| b.iter(|| dl_list::push(i)),
        );
        group.bench_with_input(
            BenchmarkId::new("value_pool::DoubleLinkedList::multi_push", data.len()),
            *data,
            |b, i| b.iter(|| dl_list::multi_push(i)),
        );

        group.bench_with_input(
            BenchmarkId::new("std::collections::LinkedList::push_back", data.len()),
            *data,
            |b, i| b.iter(|| linked_list::push(i)),
        );
    }

    group.finish();
}

fn compare_inserts(c: &mut Criterion) {
    let mut group = c.benchmark_group("Container Inserts");

    for data in [
        &VALUES_100[0..10],
        &VALUES_100[0..50],
        VALUES_100.as_slice(),
        VALUES_500.as_slice(),
        VALUES_1000.as_slice(),
        VALUES_5000.as_slice(),
        VALUES_10000.as_slice(),
    ]
    .iter()
    {
        group.bench_with_input(
            BenchmarkId::new("std::vec::Vec::insert", data.len()),
            *data,
            |b, i| b.iter(|| vector::insert(i)),
        );

        group.bench_with_input(
            BenchmarkId::new("std::collections::VecDeque::insert", data.len()),
            *data,
            |b, i| b.iter(|| vector_deq::insert(i)),
        );

        group.bench_with_input(
            BenchmarkId::new("value_pool::DoubleLinkedList::improved_insert", data.len()),
            *data,
            |b, i| b.iter(|| dl_list::improved_insert(i)),
        );
        group.bench_with_input(
            BenchmarkId::new("value_pool::DoubleLinkedList::multi_insert", data.len()),
            *data,
            |b, i| b.iter(|| dl_list::multi_insert(i)),
        );
        group.bench_with_input(
            BenchmarkId::new("value_pool::DoubleLinkedList::insert", data.len()),
            *data,
            |b, i| b.iter(|| dl_list::insert(i)),
        );

        group.bench_with_input(
            BenchmarkId::new("std::collections::LinkedList hacked insert", data.len()),
            *data,
            |b, i| b.iter(|| linked_list::insert(i)),
        );
    }

    group.finish();
}

fn compare_pushfront(c: &mut Criterion) {
    let mut group = c.benchmark_group("Container Push to Front");

    for data in [
        &VALUES_100[0..10],
        &VALUES_100[0..50],
        VALUES_100.as_slice(),
        VALUES_500.as_slice(),
        VALUES_1000.as_slice(),
        VALUES_5000.as_slice(),
        VALUES_10000.as_slice(),
    ]
    .iter()
    {
        group.bench_with_input(
            BenchmarkId::new("std::vec::Vec::insert(0, ...)", data.len()),
            *data,
            |b, i| b.iter(|| vector::push_front(i)),
        );

        group.bench_with_input(
            BenchmarkId::new("std::collections::VecDeque::push_front", data.len()),
            *data,
            |b, i| b.iter(|| vector_deq::push_front(i)),
        );

        group.bench_with_input(
            BenchmarkId::new("value_pool::DoubleLinkedList::push_front", data.len()),
            *data,
            |b, i| b.iter(|| dl_list::push_front(i)),
        );
        group.bench_with_input(
            BenchmarkId::new("value_pool::DoubleLinkedList::multi_push_front", data.len()),
            *data,
            |b, i| b.iter(|| dl_list::multi_push_front(i)),
        );

        group.bench_with_input(
            BenchmarkId::new("std::collections::LinkedList::push_front", data.len()),
            *data,
            |b, i| b.iter(|| linked_list::push_front(i)),
        );
    }

    group.finish();
}

fn compare_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("Container Get");

    let ll = black_box(linked_list::push(VALUES_10000.as_slice()));
    let double_ll = black_box(dl_list::push(VALUES_10000.as_slice()));

    assert!(ll.iter().nth(100).is_some());

    for data in black_box([
        &VALUES_100[0..10],
        &VALUES_100[0..50],
        VALUES_100.as_slice(),
        VALUES_500.as_slice(),
        VALUES_1000.as_slice(),
        VALUES_5000.as_slice(),
        VALUES_10000.as_slice(),
    ])
    .iter()
    {
        group.bench_with_input(
            BenchmarkId::new("value_pool::DoubleLinkedList::get", data.len()),
            *data,
            |b, i| b.iter(|| dl_list::get(&double_ll, i)),
        );
        group.bench_with_input(
            BenchmarkId::new("value_pool::DoubleLinkedList::multi_get_view", data.len()),
            *data,
            |b, i| b.iter_with_large_drop(|| dl_list::multi_get_view(&double_ll, i)),
        );
        group.bench_with_input(
            BenchmarkId::new("value_pool::DoubleLinkedList::iter .nth", data.len()),
            *data,
            |b, i| b.iter(|| dl_list::get_with_iter(&double_ll, i)),
        );

        group.bench_with_input(
            BenchmarkId::new("std::collections::LinkedList::iter .nth", data.len()),
            *data,
            |b, i| b.iter(|| linked_list::get(&ll, i)),
        );
    }

    group.finish();
}

criterion_group!(name=compare_datastructures; config = Criterion::default().measurement_time(std::time::Duration::from_secs(5));targets=dl_list_solo_benchmark, compare_inserts, compare_pushfront, compare_pushes, compare_get);
criterion_main!(compare_datastructures);
