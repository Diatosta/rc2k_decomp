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

use crate::errors::{BadInput, Refused};
use std::cell::Cell;
use std::ops::Range;
use std::sync::{Condvar, Mutex, OnceLock};

/// The most drawing threads, the game's own one with them.
pub const COUNT: usize = 4;

thread_local! {
    /// The running thread's copy: 0 unless it is one of the drawing threads.
    static INDEX: Cell<usize> = const { Cell::new(0) };
    /// Whether the thread is drawing a run of a list drawn by several threads.
    static IN_RUN: Cell<bool> = const { Cell::new(false) };
}

/// Whether the running thread is drawing a run of a list drawn by several threads (the
/// game's thread drawing the first run included): what is each run's own then is the
/// thread's (`race::scene_points`' points past the count).
#[inline]
pub fn in_run() -> bool {
    IN_RUN.get()
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

// ---- the copies ----

/// A store with one copy per drawing thread, reached by copy number (0 the game's own).
pub trait Copies: Sync {
    fn copy(&self, n: usize) -> &dyn crate::pointers::Store;
}

impl<T: crate::pointers::Store> Copies for PerWorker<T> {
    fn copy(&self, n: usize) -> &dyn crate::pointers::Store {
        &self.0[n]
    }
}

/// The stores of the drawing's working state a run reads as the list began (the write
/// watch's list, `notes/conversion.md`): copied into each run's copy, the changes merged
/// back.
fn carried_stores() -> [&'static dyn Copies; 18] {
    let race = crate::race::owner::race();
    let scene = crate::scene::owner::scene();
    [
        &race.frame.matrices,
        &race.frame.fade_alpha,
        &race.frame.scratch,
        &race.frame.sound_spots,
        &race.physics.camera,
        &race.physics.ground,
        &race.course.course,
        &race.course.level_grid,
        &race.shared.vars,
        &race.shared.view,
        &race.shared.switches,
        &scene.drawing.scene,
        &scene.drawing.object_work,
        &scene.lights.state,
        &scene.lights.work,
        &scene.objects.slots,
        &scene.objects.tables,
        &crate::d3d::owner::d3d().cards,
    ]
}

/// The stores each run makes for itself before reading (neither copied in nor merged; the
/// replays' every flush is the same as on one thread): the point records (a cache of the
/// points' projections, each record named by its point, the frame's clear reaching every
/// copy: `point_records::first_records_everywhere`), the vertex buffer and the polygon work.
fn scratch_stores() -> [&'static dyn Copies; 3] {
    [
        crate::scene::point_records::copies(),
        &crate::d3d::vertices_store::VERTICES,
        &crate::scene::polygon_store::POLYGON_WORK,
    ]
}

fn load(store: &dyn crate::pointers::Store) -> Vec<u8> {
    let mut bytes = vec![0; store.size()];
    // The whole store: inside it.
    let _ = store.load(0, &mut bytes);
    bytes
}

/// The changed bytes of copy `from` (against `start`) put into copy 0, run by run.
fn merge(store: &dyn Copies, from: usize, start: &[u8]) {
    let now = load(store.copy(from));
    let first = store.copy(0);
    let mut k = 0;
    while k < now.len() {
        if now[k] == start[k] {
            k += 1;
            continue;
        }
        let s = k;
        while k < now.len() && now[k] != start[k] {
            k += 1;
        }
        let _ = first.store(s, &now[s..k]);
    }
}

// ---- the runs ----

/// The drawing threads asked for: `RC2K_DRAW_THREADS` (the game's with them; 1, the
/// default, draws on the game's thread alone), at most `COUNT`.
pub fn threads() -> usize {
    static N: OnceLock<usize> = OnceLock::new();
    *N.get_or_init(|| {
        std::env::var("RC2K_DRAW_THREADS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1)
            .clamp(1, COUNT)
    })
}

/// With `RC2K_DRAW_SERIAL` (a check): the runs drawn one after another on the game's thread,
/// each on its own copy as on its own thread, so that what the copies and their merging do is
/// told apart from what threads drawing at once do.
fn serial() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("RC2K_DRAW_SERIAL").is_some())
}

/// The other drawing threads.
fn pool() -> Option<&'static rayon::ThreadPool> {
    static POOL: OnceLock<Option<rayon::ThreadPool>> = OnceLock::new();
    POOL.get_or_init(|| {
        if threads() > 1 {
            // The copies are regions as copy 0 (the store's home) is: a place in them named by
            // its game address is read through their cells.
            for s in carried_stores().into_iter().chain(scratch_stores()) {
                for n in 1..COUNT {
                    crate::pointers::register_store(s.copy(n));
                }
            }
        }
        (threads() > 1)
            .then(|| {
                rayon::ThreadPoolBuilder::new()
                    .num_threads(threads() - 1)
                    .thread_name(|n| format!("draw {}", n + 1))
                    .build()
                    .ok()
            })
            .flatten()
    })
    .as_ref()
}

/// The runs of the list being drawn that have ended (by run number).
static ENDED: Mutex<[bool; COUNT]> = Mutex::new([false; COUNT]);
static ENDED_CHANGED: Condvar = Condvar::new();

fn ended(n: usize) {
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
    let mut e = ENDED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    while e[..n].iter().any(|&done| !done) {
        e = ENDED_CHANGED
            .wait(e)
            .unwrap_or_else(std::sync::PoisonError::into_inner);
    }
}

/// The `count` entries drawn by `draw` (a range of them at a time), split into runs over the
/// drawing threads: the game's thread draws the first, each other thread one of the others
/// on its own copy of the working state, set from the game's own at the start. The noted
/// drawing is put together in the runs' order, and each copy's changes put into the game's
/// own in that order (`race::draw_chunks` showed this draws what one thread does).
pub fn draw_in_runs(
    count: usize,
    draw: impl Fn(Range<usize>) -> Result<(), Refused> + Sync,
) -> Result<(), Refused> {
    let k = threads().min(count);
    let Some(pool) = pool().filter(|_| k > 1) else {
        return draw(0..count);
    };
    let ranges: Vec<Range<usize>> = (0..k).map(|i| i * count / k..(i + 1) * count / k).collect();
    let stores = carried_stores();
    let start: Vec<Vec<u8>> = stores.iter().map(|s| load(s.copy(0))).collect();
    for n in 1..k {
        for (s, bytes) in stores.iter().zip(&start) {
            let _ = s.copy(n).store(0, bytes);
        }
    }
    *ENDED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = [false; COUNT];
    let control = crate::math::x87::control_word();
    let mark = crate::platform_3d::noted_len();
    let flushes = crate::platform_3d::flushes();
    // Run `n` of the others, on the thread it is called on: its noted drawing from `from` on.
    let other = |n: usize, from: usize| {
        INDEX.set(n);
        IN_RUN.set(true);
        crate::math::x87::set_control_word(control);
        crate::race::scene_points::forget_own();
        #[cfg(feature = "write_watch")]
        crate::write_watch::begin();
        crate::d3d::run_states::begin_run();
        let result = draw(ranges[n].clone());
        let states = crate::d3d::run_states::end_run();
        #[cfg(feature = "write_watch")]
        WATCHED
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(crate::write_watch::end());
        ended(n);
        IN_RUN.set(false);
        let noted = crate::platform_3d::take_noted_from(from);
        INDEX.set(0);
        OtherRun {
            result,
            noted,
            states,
        }
    };
    #[cfg(feature = "write_watch")]
    crate::write_watch::begin();
    // Run 0, on the game's thread.
    let first_run = || {
        IN_RUN.set(true);
        crate::race::scene_points::forget_own();
        let result = draw(ranges[0].clone());
        IN_RUN.set(false);
        ended(0);
        result
    };
    let (first, mine, others) = if serial() {
        let first = first_run();
        let mine = crate::platform_3d::take_noted_from(mark);
        let others: Vec<OtherRun> = (1..k).map(|n| other(n, mark)).collect();
        (first, mine, others)
    } else {
        let slots: Vec<Mutex<Option<OtherRun>>> = (1..k).map(|_| Mutex::new(None)).collect();
        let mut first = Ok(());
        // In place: the closure (run 0) on the game's thread, the spawned runs on the pool.
        pool.in_place_scope(|s| {
            for (n, slot) in slots.iter().enumerate() {
                let other = &other;
                s.spawn(move |_| {
                    let run = other(n + 1, 0);
                    *slot
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(run);
                });
            }
            first = first_run();
        });
        let mine = crate::platform_3d::take_noted_from(mark);
        let others = slots
            .into_iter()
            .map(|slot| {
                slot.into_inner()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .ok_or(BadInput("a drawing run that did not end"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        (first, mine, others)
    };
    if crate::platform_3d::flushes() != flushes {
        crate::framework::log("workers: a flush while the runs were drawn");
    }
    #[cfg(feature = "write_watch")]
    {
        // Run 0's watch ends here (in the serial check the others' notes are in theirs).
        let mut watched = WATCHED
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        watched.push(crate::write_watch::end());
        crate::race::draw_chunks::shared::add(&watched);
        watched.clear();
    }
    crate::platform_3d::put_noted(mine);
    let mut result = first;
    for (n, mut run) in others.into_iter().enumerate() {
        result = result.and(run.result);
        // Its fans before it set a state draw with what the earlier runs left.
        let carried = crate::d3d::run_states::carried();
        crate::d3d::run_states::patch(&mut run.noted, &run.states, &carried);
        crate::platform_3d::put_noted(run.noted);
        for (s, bytes) in stores.iter().zip(&start) {
            merge(*s, n + 1, bytes);
        }
        crate::d3d::run_states::carry(&run.states, n + 1);
    }
    // The game's thread drew run 0 with its own states: the next fan takes the merged ones.
    crate::platform_3d::forget_states();
    result
}

/// What a drawing thread's run left.
struct OtherRun {
    result: Result<(), Refused>,
    noted: Vec<crate::render::Command>,
    /// Where it set the render states.
    states: crate::d3d::run_states::RunStates,
}

/// With the feature `write_watch`: what each run of the list being drawn reached.
#[cfg(feature = "write_watch")]
static WATCHED: Mutex<Vec<crate::write_watch::Seen>> = Mutex::new(Vec::new());
