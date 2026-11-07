use std::collections::VecDeque;

use crate::change::{Change, ChangeSet};

enum Operation {
    Retain(usize),
    Delete(usize),
    Insert(String),
}

impl Operation {
    fn len(&self) -> usize {
        match self {
            Self::Retain(count) | Self::Delete(count) => *count,
            Self::Insert(text) => text.len(),
        }
    }

    fn split(self, at: usize) -> (Self, Self) {
        match self {
            Self::Retain(count) => (Self::Retain(at), Self::Retain(count - at)),
            Self::Delete(count) => (Self::Delete(at), Self::Delete(count - at)),
            Self::Insert(mut head) => {
                let tail = head.split_off(at);
                (Self::Insert(head), Self::Insert(tail))
            }
        }
    }
}

pub(crate) fn compose(first: &ChangeSet, later: &ChangeSet) -> ChangeSet {
    let mut first = operations(first);
    let mut later = operations(later);
    let mut composed = Vec::new();
    for _ in 0..first.len() + later.len() {
        if let Some(Operation::Delete(_)) = first.front() {
            composed.extend(first.pop_front());
            continue;
        }
        if let Some(Operation::Insert(_)) = later.front() {
            composed.extend(later.pop_front());
            continue;
        }
        let limit = match (first.front(), later.front()) {
            (None, None) => break,
            (a, b) => length_or_unbounded(a).min(length_or_unbounded(b)),
        };
        match (take(&mut first, limit), take(&mut later, limit)) {
            (Operation::Retain(_), Operation::Retain(_)) => {
                composed.push(Operation::Retain(limit));
            }
            (Operation::Retain(_), Operation::Delete(_)) => {
                composed.push(Operation::Delete(limit));
            }
            (inserted @ Operation::Insert(_), Operation::Retain(_)) => composed.push(inserted),
            _ => {}
        }
    }
    change_set(composed)
}

fn length_or_unbounded(operation: Option<&Operation>) -> usize {
    operation.map_or(usize::MAX, Operation::len)
}

fn take(operations: &mut VecDeque<Operation>, limit: usize) -> Operation {
    let Some(front) = operations.pop_front() else {
        return Operation::Retain(limit);
    };
    if front.len() <= limit {
        return front;
    }
    let (head, tail) = front.split(limit);
    operations.push_front(tail);
    head
}

fn operations(set: &ChangeSet) -> VecDeque<Operation> {
    let mut operations = VecDeque::new();
    let mut cursor = 0;
    for change in set.changes() {
        let retained = change.range.start - cursor;
        if retained > 0 {
            operations.push_back(Operation::Retain(retained));
        }
        if change.removed_len() > 0 {
            operations.push_back(Operation::Delete(change.removed_len()));
        }
        if !change.text.is_empty() {
            operations.push_back(Operation::Insert(change.text.clone()));
        }
        cursor = change.range.end;
    }
    operations
}

fn change_set(operations: Vec<Operation>) -> ChangeSet {
    let mut changes = Vec::new();
    let mut pending: Option<Change> = None;
    let mut position = 0;
    for operation in operations {
        match operation {
            Operation::Retain(count) => {
                changes.extend(pending.take());
                position += count;
            }
            Operation::Delete(count) => {
                let change = pending.get_or_insert_with(|| Change::insert(position, ""));
                change.range.end += count;
                position += count;
            }
            Operation::Insert(text) => {
                let change = pending.get_or_insert_with(|| Change::insert(position, ""));
                change.text.push_str(&text);
            }
        }
    }
    changes.extend(pending);
    ChangeSet::new(changes)
}
