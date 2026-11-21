//! The drawing threads' own coordination: which drawing thread is running, whether a drawing
//! scope is open, which runs of the list being drawn have ended, and the turn of a piece drawn
//! aside. They are statics because code that holds no drawing context reads them (the random
//! numbers' wait for the earlier runs, the draw-shared stores' check); they hold no drawing
//! data. The drawing itself (`scene::workers`) sets them.

use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Condvar, Mutex};

/// The most drawing threads, the game's own one with them.
pub const COUNT: usize = 4;

thread_local! {
    /// The running thread's copy: 0 unless it is one of the drawing threads.
    #[cfg_attr(
        target_os = "android",
        allow(
            clippy::missing_const_for_thread_local,
            reason = "clippy reports an initializer that is already const when building for Android"
        )
    )]
    static INDEX: Cell<usize> = const { Cell::new(0) };
}

/// The running thread's copy number.
#[inline(always)]
pub fn index() -> usize {
    INDEX.get()
}

/// The running thread is drawing run `n` (0 again when it stops drawing).
pub fn set_index(n: usize) {
    INDEX.set(n);
}

/// Whether a drawing scope is open (`DrawScope`): the parts are being drawn.
static IN_SCOPE: AtomicBool = AtomicBool::new(false);

/// Whether the parts of a drawing are being drawn: between `draw_parts`' start of the parts
/// and the end of the last (`session::DrawShared::get` is not taken then).
pub fn in_scope() -> bool {
    IN_SCOPE.load(Ordering::Relaxed)
}

/// What a drawing scope frees when it ends: the parts' and slots' claims.
pub trait ScopeClaims: Sync {
    /// The scope `ending`: the claimed elements are the game's thread's again.
    fn end_claims(&self, ending: &DrawScope<'_>);
}

/// The parts of one drawing being drawn (`scene::workers::draw_parts`): opened on the game's
/// thread before the first part starts, ended once every part has ended. The stores lent to
/// the parts (`session::PhaseRead`) borrow it, so none outlives it. Only `draw_parts` opens one.
pub struct DrawScope<'o> {
    /// What the runs claimed, freed when the scope ends.
    claims: &'o dyn ScopeClaims,
}

impl<'o> DrawScope<'o> {
    /// Opens the scope; called by `scene::workers::draw_parts` alone.
    pub(crate) fn open(claims: &'o dyn ScopeClaims) -> Self {
        crate::memory::session::assert_game_thread();
        let was = IN_SCOPE.swap(true, Ordering::Relaxed);
        debug_assert!(!was, "a drawing scope opened inside another");
        Self { claims }
    }
}

impl Drop for DrawScope<'_> {
    fn drop(&mut self) {
        crate::memory::session::end_lending();
        self.claims.end_claims(self);
        IN_SCOPE.store(false, Ordering::Relaxed);
    }
}

/// The runs of the list being drawn that have ended (by run number).
static ENDED: Mutex<[bool; COUNT]> = Mutex::new([false; COUNT]);
static ENDED_CHANGED: Condvar = Condvar::new();

/// A list is about to be drawn: no run has ended.
pub fn begin_list() {
    *ENDED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = [false; COUNT];
}

/// Run `n` has ended.
pub fn ended(n: usize) {
    let mut e = ENDED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    e[n] = true;
    ENDED_CHANGED.notify_all();
}

/// Waits until the runs before this thread's have ended: what must be done in the list's
/// order (the random numbers, `math::next_random`) is then done after them, as by one
/// thread. Nothing to wait for on the game's thread (run 0) or outside a list.
pub fn wait_for_earlier_runs() {
    let n = index();
    if n == 0 {
        return;
    }
    if n == ASIDE_INDEX {
        // A piece drawn aside (`draw_aside`): after what its part draws before it.
        wait_for_aside_turn();
        return;
    }
    if n > ASIDE_INDEX {
        // A run of a list cut into runs (`draw_list_in_runs`): after the list's earlier runs.
        wait_for_list_runs_before(n - ASIDE_INDEX);
        return;
    }
    let mut e = ENDED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    while e[..n].iter().any(|&done| !done) {
        e = ENDED_CHANGED
            .wait(e)
            .unwrap_or_else(std::sync::PoisonError::into_inner);
    }
}

/// The run number of a piece drawn aside (`index` on its thread): past the parts'.
pub const ASIDE_INDEX: usize = COUNT;

/// Whether the part drawing a piece aside has come to the piece's place in its drawing: what
/// the piece must do in the drawing's order (`wait_for_earlier_runs`) waits for it.
static ASIDE_TURN: Mutex<bool> = Mutex::new(false);
static ASIDE_TURN_CHANGED: Condvar = Condvar::new();

/// The part has (or has not) come to the aside piece's place.
pub fn set_aside_turn(reached: bool) {
    *ASIDE_TURN
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = reached;
    ASIDE_TURN_CHANGED.notify_all();
}

fn wait_for_aside_turn() {
    let mut reached = ASIDE_TURN
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    while !*reached {
        reached = ASIDE_TURN_CHANGED
            .wait(reached)
            .unwrap_or_else(std::sync::PoisonError::into_inner);
    }
}

/// The most runs a list inside a part is cut into besides the part's own first run
/// (`scene::workers::draw_list_in_runs`): one scratch each.
pub const LIST_RUNS: usize = 3;

/// The run number (`index`) of the thread drawing run `run` (1 ..= `LIST_RUNS`) of a list cut
/// into runs: past the aside piece's. The list's first run is its part's own, on the part's
/// thread with the part's number.
pub const fn list_run_index(run: usize) -> usize {
    ASIDE_INDEX + run
}

/// The runs of the list cut into runs that have ended (the part's own first run is 0).
static LIST_RUNS_ENDED: Mutex<[bool; LIST_RUNS + 1]> = Mutex::new([false; LIST_RUNS + 1]);
static LIST_RUNS_CHANGED: Condvar = Condvar::new();

/// A list is about to be drawn in runs: none has ended.
pub fn begin_list_runs() {
    *LIST_RUNS_ENDED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = [false; LIST_RUNS + 1];
}

/// Run `run` of the list cut into runs has ended (or will draw nothing more).
pub fn list_run_ended(run: usize) {
    let mut ended = LIST_RUNS_ENDED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(flag) = ended.get_mut(run) {
        *flag = true;
    }
    LIST_RUNS_CHANGED.notify_all();
}

/// Waits until every run before `run` of the list cut into runs has ended.
fn wait_for_list_runs_before(run: usize) {
    let mut ended = LIST_RUNS_ENDED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    while ended.iter().take(run).any(|&done| !done) {
        ended = LIST_RUNS_CHANGED
            .wait(ended)
            .unwrap_or_else(std::sync::PoisonError::into_inner);
    }
}
