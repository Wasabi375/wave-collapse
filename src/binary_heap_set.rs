use std::{
    cmp::Ord,
    collections::{BinaryHeap, HashSet},
    hash::Hash,
};

/// A [BinaryHeap] that does not allow for duplicate entries.
#[derive(Default)]
pub struct BinaryHeapSet<T: Clone + Ord + Hash> {
    heap: BinaryHeap<T>,
    set: HashSet<T>,
}

impl<T: Clone + Ord + Hash> BinaryHeapSet<T> {
    pub fn new() -> Self {
        BinaryHeapSet {
            heap: BinaryHeap::new(),
            set: HashSet::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    pub fn push(&mut self, value: T) -> bool {
        if self.set.insert(value.clone()) {
            self.heap.push(value);
            true
        } else {
            false
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        let result = self.heap.pop();

        if let Some(value) = &result {
            self.set.remove(value);
        }

        result
    }
}
