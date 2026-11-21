//! A view's own working values as a plain record behind a `RefCell`: what a view's drawing
//! writes and reads within itself through the view's shared reference. A drawing copies a view's
//! slots for a piece of it on another thread (`ViewScratch::copy_drawing_from`) and puts what the
//! piece changed back over the view's own (`ViewScratch::merge_drawing`), field by field.

use std::cell::{Ref, RefCell, RefMut};

/// A record that can be merged: the fields of `now` that differ from `start` are put into
/// `self`, the others left as they are.
pub trait Merge: Copy {
    fn merge_changes(&mut self, now: &Self, start: &Self);
}

/// A merge of one field.
pub fn merge_value<T: Copy + PartialEq>(mine: &mut T, now: &T, start: &T) {
    if now != start {
        *mine = *now;
    }
}

/// A merge of a run of fields, each by itself.
pub fn merge_run<T: Copy + PartialEq>(mine: &mut [T], now: &[T], start: &[T]) {
    for ((m, n), s) in mine.iter_mut().zip(now).zip(start) {
        merge_value(m, n, s);
    }
}

/// One view's record. Borrows are held for one statement, never across a call that borrows it.
pub struct PlainSlot<T: Merge>(RefCell<T>);

impl<T: Merge> PlainSlot<T> {
    pub const fn new(value: T) -> Self {
        Self(RefCell::new(value))
    }

    /// The record, read.
    pub fn borrow(&self) -> Ref<'_, T> {
        self.0.borrow()
    }

    /// The record, to write: no other borrow may be held.
    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        self.0.borrow_mut()
    }

    /// Every field set from `other`'s.
    pub fn copy_from(&self, other: &PlainSlot<T>) {
        let value = *other.0.borrow();
        *self.0.borrow_mut() = value;
    }

    /// What `now` changed since it was `start` put into this record.
    pub fn merge_changes(&self, now: &PlainSlot<T>, start: &PlainSlot<T>) {
        let (now, start) = (*now.0.borrow(), *start.0.borrow());
        self.0.borrow_mut().merge_changes(&now, &start);
    }

    /// The record set back to `first`.
    pub fn reset(&self, first: T) {
        *self.0.borrow_mut() = first;
    }
}
