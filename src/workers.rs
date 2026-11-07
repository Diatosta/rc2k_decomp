//! The drawing's threads (the optimisation phase's multi-core drawing, `notes/conversion.md`):
//! a view's entries can be drawn by several threads at once, each from its own copy of the
//! drawing's working state, the copies' changes put back in the entries' order afterwards
//! (`race::draw_chunks` showed that this draws exactly what one thread does).
//!
//! A store of that working state is a `PerWorker<T>`: one copy per drawing thread, and code
//! reaches the copy of the thread it runs on through `Deref`, so it reads as any other store
//! (`frame.matrices.turned_matrix`). Copy 0 is the game's own thread's (and every thread's
//! that is not a drawing one): with one thread drawing, it is the only copy used and the
//! program runs as before. The stores in this state are those the write watch found two runs
//! of a list reaching (`notes/conversion.md`, "the write watch and the private set").

use std::cell::Cell;

/// The most drawing threads, the game's own one with them.
pub const COUNT: usize = 4;

thread_local! {
    /// The running thread's copy: 0 unless it is one of the drawing threads.
    static INDEX: Cell<usize> = const { Cell::new(0) };
}

/// The running thread's copy number.
#[inline(always)]
pub fn index() -> usize {
    INDEX.get()
}

/// A store with one copy per drawing thread (see the module's header).
pub struct PerWorker<T>([T; COUNT]);

impl<T> PerWorker<T> {
    pub const fn new(copies: [T; COUNT]) -> Self {
        Self(copies)
    }

    /// The game's own thread's copy (the one every other thread's starts from).
    pub const fn first(&self) -> &T {
        &self.0[0]
    }

    /// Every copy (what is set up for all threads at once: a stage's records zeroed).
    pub fn all(&self) -> &[T; COUNT] {
        &self.0
    }
}

impl<T> core::ops::Deref for PerWorker<T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        // `index` is below `COUNT` (only `COUNT` drawing threads are given one).
        &self.0[index()]
    }
}

/// The running thread's copy of a store kept as the game's own (`first`, in its place in a
/// block with the image's layout) and the other drawing threads' (`others`).
#[inline(always)]
pub fn pick<'a, T>(first: &'a T, others: &'a [T; COUNT - 1]) -> &'a T {
    match index() {
        0 => first,
        n => &others[n - 1],
    }
}
