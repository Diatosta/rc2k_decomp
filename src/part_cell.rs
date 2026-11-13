//! The cells of the drawing's per-part state: `PartCell<T>`, an element of `workers::PARTS`
//! (one `PartScratch` per drawing thread) or of `race::view_scratch::VIEWS` (one `ViewScratch`
//! per drawn piece), reached by exactly one thread at a time.
//!
//! Two doors lead in. `claim` is the parts' own: `workers::draw_parts` makes a `PartTurn` for
//! each run it starts (on the thread that runs it) and the run claims its part's scratch and its
//! piece's view with it; the reference lives as long as the drawing scope. `on_game_thread` is
//! the game's thread's, for everything drawn outside the parts (the overlays, the HUD, the menus)
//! and for what is put together after them. Behind both doors a debug build checks that no
//! element is claimed twice in a scope and that the game's thread does not reach an element a run
//! on another thread holds.

#![allow(unsafe_code)]

use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

/// An element of a per-part array: `T` is reached by one thread at a time (see the module's
/// header and the `Sync` impl's proof).
pub struct PartCell<T> {
    value: UnsafeCell<T>,
    /// 0 free; else the run that claimed it in this scope, plus 1.
    claimed: AtomicUsize,
}

// SAFETY: sharing a `&PartCell<T>` between threads lets each thread take `&T` to the element,
// through two doors only; the thread that holds it is the only thread that may hold it at that
// time, so a `T` that is not `Sync` (plain cells) is never shared, only handed from one thread to
// the next, which `T: Send` allows.
//
// 1. `claim` needs a `PartTurn`, which only `workers::draw_parts` can make (the constructor is
//    `pub(crate)` and takes a `DrawScope`, which only `draw_parts` opens). It makes one per run, on the
//    thread that runs it, and each run is started once per scope; it claims `PARTS[n]` for run
//    `n` and `VIEWS[slots[n]]` with `slots` checked pairwise distinct before any run starts (a
//    refusal, in every build), so no two runs hold the same element. The reference is `&'s T`
//    with `'s` the turn's, which borrows the scope: it cannot be kept past the scope's end, and
//    `PartTurn` is `!Send`, so a run cannot pass the right to claim to another thread.
// 2. `on_game_thread` is called on the game's thread only (`assert_game_thread`, as
//    `GameThread::get` does, in debug builds). Outside a scope no run holds an element, so
//    nothing else reaches it. Inside a scope the game's thread is run 0, so it reaches the
//    elements run 0 claimed (`PARTS[0]` and run 0's view) as itself; an element a run other
//    than 0 holds is on another thread, and `on_game_thread` stops at it in debug builds
//    (`claimed` above 1). The callers of the accessors outside the parts
//    (`workers::part_scratch`, `game_scratch`, `race::view_scratch::{main_view, mirror, slot,
//    front_end}`) are the game's thread's own code, and the parts are handed what they draw
//    on (`PartRun`).
// 3. Order between scopes: a scope's runs are spawned after everything the game's thread did
//    before it and joined (rayon's `in_place_scope`, or the same thread) before anything it
//    does after, so every access of one scope happens before every access of the next.
//
// What would break it: code that makes a `PartTurn` outside `draw_parts`; two runs given the
// same view slot (refused before they start); the game's thread reaching, inside a scope, an
// element another run holds (the debug check); a part sending an element's reference to another
// thread (not possible for `T: !Sync`, as `&T` is then `!Send`). The debug-build two-player race
// with the mirror beside its view, with `RC2K_DRAW_SERIAL` and without, is the check of the
// last.
unsafe impl<T: Send> Sync for PartCell<T> {}

/// The right of one run of a drawing scope to claim elements (`PartCell::claim`): made by
/// `workers::draw_parts` on the thread that runs it, not `Send`.
pub struct PartTurn<'s> {
    run: usize,
    /// The scope it lasts (`'s`); the raw pointer keeps it on its thread.
    _scope: PhantomData<(&'s (), *const ())>,
}

impl<'s> PartTurn<'s> {
    /// Run `run`'s turn in `scope` (only `workers::draw_parts` opens one).
    pub(crate) fn new(_scope: &'s crate::workers::DrawScope, run: usize) -> Self {
        Self {
            run,
            _scope: PhantomData,
        }
    }

    /// A turn without a scope, for the tests of the cells.
    #[cfg(test)]
    fn for_test(run: usize) -> PartTurn<'static> {
        PartTurn {
            run,
            _scope: PhantomData,
        }
    }
}

impl<T> PartCell<T> {
    pub const fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
            claimed: AtomicUsize::new(0),
        }
    }

    /// The element, for the run whose `turn` it is, until the scope ends. Twice in one scope
    /// stops a debug build.
    pub fn claim<'s>(&'s self, turn: &PartTurn<'s>) -> &'s T {
        let was = self.claimed.swap(turn.run + 1, Relaxed);
        debug_assert_eq!(
            was, 0,
            "a per-part element claimed twice in one drawing scope"
        );
        // SAFETY: by the `Sync` impl's proof, point 1: this run is the only one holding the
        // element until the scope ends.
        unsafe { &*self.value.get() }
    }

    /// The element, on the game's thread: outside a drawing scope, or inside one an element
    /// run 0 (the game's thread) holds or none does. Debug builds check the thread and that no
    /// other run holds it.
    pub fn on_game_thread(&self) -> &T {
        crate::session::assert_game_thread();
        debug_assert!(
            self.claimed.load(Relaxed) <= 1,
            "the game's thread reached a per-part element another run holds"
        );
        // SAFETY: by the `Sync` impl's proof, point 2.
        unsafe { &*self.value.get() }
    }

    /// The scope ending (`workers::DrawScope`'s drop): the element is free again.
    pub(crate) fn end_claim(&self, _ending: &crate::workers::DrawScope) {
        self.claimed.store(0, Relaxed);
    }

    #[cfg(test)]
    fn end_claim_for_test(&self) {
        self.claimed.store(0, Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn a_claim_reaches_the_element_and_ends() {
        let cell = PartCell::new(Cell::new(5));
        let turn = PartTurn::for_test(2);
        assert_eq!(cell.claim(&turn).get(), 5);
        cell.end_claim_for_test();
        let again = PartTurn::for_test(1);
        cell.claim(&again).set(6);
        cell.end_claim_for_test();
        assert_eq!(cell.on_game_thread().get(), 6);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "claimed twice")]
    fn a_double_claim_stops_a_debug_build() {
        let cell = PartCell::new(Cell::new(0));
        let (first, second) = (PartTurn::for_test(0), PartTurn::for_test(1));
        let _ = cell.claim(&first);
        let _ = cell.claim(&second);
    }

    /// The game's thread reaches an element it holds as run 0, or none; one a run on another
    /// thread holds stops a debug build.
    #[test]
    #[cfg(debug_assertions)]
    fn out_of_scope_access_stops_a_debug_build() {
        let cell = PartCell::new(Cell::new(0));
        let held_by_run_0 = PartCell::new(Cell::new(0));
        let _ = held_by_run_0.claim(&PartTurn::for_test(0));
        let _ = cell.claim(&PartTurn::for_test(3));
        assert_eq!(held_by_run_0.on_game_thread().get(), 0);
        let stopped =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| cell.on_game_thread().get()));
        assert!(
            stopped.is_err(),
            "run 3's element reached from the game's thread"
        );
    }
}
