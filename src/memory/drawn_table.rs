//! A read table the drawing parts and the game's thread both read, as one reader finds it.

use crate::memory::session::{ArcRead, DrawArc};

/// A table (`session::DrawArc`) as one reader finds it: the game's thread's own, read through the
/// store when used, or a drawing part's copy of it (`ArcRead`). The tables are written only on
/// the game's thread outside a drawing scope.
pub enum DrawnTable<'a, T> {
    Store(&'a DrawArc<T>),
    Copy(&'a T),
}

impl<T> Clone for DrawnTable<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for DrawnTable<'_, T> {}

impl<T> DrawnTable<'_, T> {
    /// `f` on the table as it stands (a short read: nothing in `f` may write the store).
    pub fn read<R>(self, f: impl FnOnce(&T) -> R) -> R {
        match self {
            Self::Store(store) => f(&store.borrow()),
            Self::Copy(table) => f(table),
        }
    }
}

impl<'a, T> From<&'a ArcRead<T>> for DrawnTable<'a, T> {
    fn from(copy: &'a ArcRead<T>) -> Self {
        Self::Copy(copy)
    }
}
