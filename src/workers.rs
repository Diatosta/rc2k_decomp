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

/// A store's copies: one per drawing thread, and the state the runs of a list start from
/// (copy `COUNT`), against which their changes are found.
pub const COPIES: usize = COUNT + 1;

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

/// A store with one copy per drawing thread (see the module's header), and one of the state
/// the runs start from.
pub struct PerWorker<T>([T; COPIES]);

impl<T> PerWorker<T> {
    pub const fn new(copies: [T; COPIES]) -> Self {
        Self(copies)
    }

    /// The game's own thread's copy (the one every other thread's starts from).
    pub const fn first(&self) -> &T {
        &self.0[0]
    }

    /// Every copy (what is set up for all threads at once: a stage's records zeroed).
    pub fn all(&self) -> &[T; COPIES] {
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

/// A store whose copies are set from the game's own and merged back (`carried_stores`), cell
/// by cell with no run searched.
pub trait Carried: Copies {
    /// Copy 0 kept as the runs' start, and into copies 1 .. `runs`.
    fn begin_runs(&self, runs: usize);
    /// What copy `n` changed since the start put into copy 0.
    fn merge_run(&self, n: usize);
}

impl<T: crate::pointers::Cells + crate::pointers::Store> Carried for PerWorker<T> {
    fn begin_runs(&self, runs: usize) {
        let [first, rest @ ..] = &self.0;
        rest[COUNT - 1].copy_from(first);
        for copy in &rest[..runs - 1] {
            copy.copy_from(first);
        }
    }

    fn merge_run(&self, n: usize) {
        self.0[0].merge_changes(&self.0[n], &self.0[COUNT]);
    }
}

/// The stores of the drawing's working state a run reads as the list began (the write
/// watch's list, `notes/conversion.md`): copied into each run's copy, the changes merged
/// back.
fn carried_stores() -> [&'static dyn Carried; 20] {
    let race = crate::race::owner::race();
    let scene = crate::scene::owner::scene();
    [
        &race.frame.matrices,
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
        &crate::d3d::owner::d3d().cards,
        // The stores the mirror shares with its view (`draw_beside`, the write watch's list).
        &race.frame.frame_work,
        &race.frame.view_and_records,
        &race.shared.work,
        &crate::race::blocks_store::CAR_BLOCKS,
        &crate::audio::store::ENVIRONMENT,
    ]
}

/// The stores each run makes for itself before reading (neither copied in nor merged; the
/// replays' every flush is the same as on one thread): the point records (a cache of the
/// points' projections, each record named by its point, the frame's clear reaching every
/// copy: `point_records::first_records_everywhere`), the vertex buffer and the polygon work.
/// (The sort lists are each view's own: `race::view_scratch`.)
fn scratch_stores() -> [&'static dyn Copies; 3] {
    [
        crate::scene::point_records::copies(),
        &crate::d3d::vertices_store::VERTICES,
        &crate::scene::polygon_store::POLYGON_WORK,
    ]
}

// ---- the runs ----

/// The drawing threads a list is split over (`draw_in_runs`): `RC2K_DRAW_THREADS` (the
/// game's with them; 1, the default, draws a list on the game's thread alone), at most
/// `COUNT`.
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

/// The rear-view mirror drawn beside its view (`draw_beside`, `race::draw_race_scene`): on a
/// machine of two cores or more, drawing and sounding the same as one after the other.
/// `RC2K_MIRROR_BESIDE=0` turns it off, any other value on.
pub fn mirror_beside() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| match std::env::var_os("RC2K_MIRROR_BESIDE") {
        Some(v) => v != "0",
        None => std::thread::available_parallelism().map_or(1, usize::from) >= 2,
    })
}

/// Whether parts are being drawn beside each other (`draw_beside`): a list inside one of them
/// is not split into runs, the copies being the parts' own.
static BESIDE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Whether parts are being drawn beside each other (`draw_beside`).
pub fn beside() -> bool {
    BESIDE.load(std::sync::atomic::Ordering::Relaxed)
}

/// With `RC2K_DRAW_SERIAL` (a check): the runs drawn one after another on the game's thread,
/// each on its own copy as on its own thread, so that what the copies and their merging do is
/// told apart from what threads drawing at once do.
fn serial() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("RC2K_DRAW_SERIAL").is_some())
}

/// The copies made regions as copy 0 (the store's home) is, once: a place in them named by
/// its game address is read through their cells.
fn register_copies() {
    static DONE: OnceLock<()> = OnceLock::new();
    DONE.get_or_init(|| {
        let carried = carried_stores().map(|s| -> &'static dyn Copies { s });
        for s in carried.into_iter().chain(scratch_stores()) {
            for n in 1..COUNT {
                crate::pointers::register_store(s.copy(n));
            }
        }
    });
}

/// The other drawing threads: `threads()`, and with `mirror_beside` at least one besides the
/// game's on a machine of two cores or more (the handheld: 25 to 30 frames a second).
fn pool() -> Option<&'static rayon::ThreadPool> {
    static POOL: OnceLock<Option<rayon::ThreadPool>> = OnceLock::new();
    POOL.get_or_init(|| {
        let cores = std::thread::available_parallelism().map_or(1, usize::from);
        let all = threads().max(if mirror_beside() { cores.min(2) } else { 1 });
        (all > 1)
            .then(|| {
                rayon::ThreadPoolBuilder::new()
                    .num_threads(all - 1)
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
///
/// `view` is the view's state: run 0 draws on it, each other run on a copy of it
/// (`view_scratch::run_copy`), whose changes are put into it in the runs' order.
pub fn draw_in_runs(
    view: &crate::race::view_scratch::ViewScratch,
    count: usize,
    draw: impl Fn(Range<usize>, &crate::race::view_scratch::ViewScratch) -> Result<(), Refused> + Sync,
) -> Result<(), Refused> {
    carried_census("a list", || draw_in_runs_now(view, count, &draw))
}

fn draw_in_runs_now(
    view: &crate::race::view_scratch::ViewScratch,
    count: usize,
    draw: &(
         impl Fn(Range<usize>, &crate::race::view_scratch::ViewScratch) -> Result<(), Refused> + Sync
     ),
) -> Result<(), Refused> {
    use crate::race::view_scratch::{begin_runs, merge_run, run_copy};
    let k = threads().min(count);
    let beside = BESIDE.load(std::sync::atomic::Ordering::Relaxed);
    let Some(pool) = pool().filter(|_| k > 1 && !beside) else {
        return draw(0..count, view);
    };
    let ranges: Vec<Range<usize>> = (0..k).map(|i| i * count / k..(i + 1) * count / k).collect();
    begin_runs(view, k);
    let drawn = flushes_before(|| {
        draw_parts(Some(pool), k, Merged::ChangedInLists, |n| {
            draw(ranges[n].clone(), if n == 0 { view } else { run_copy(n) })
        })
    });
    for n in 1..k {
        merge_run(view, n);
    }
    if crate::platform_3d::flushes() != drawn? {
        crate::framework::log("workers: a flush while the runs were drawn");
    }
    Ok(())
}

/// `parts` drawn and the flushes counted before it.
fn flushes_before(parts: impl FnOnce() -> Result<(), Refused>) -> Result<u64, Refused> {
    let flushes = crate::platform_3d::flushes();
    parts()?;
    Ok(flushes)
}

/// `first` drawn on the game's thread and `second` beside it on a drawing thread, each from
/// the working state as it is now (the mirror beside its view, `race::draw_race_scene`):
/// `second`'s drawing goes after `first`'s, which may flush, and its changes to the working
/// state over `first`'s. With fewer than two drawing threads both on the game's thread, one
/// after the other, each on its own copy as on its own thread, so that what is drawn is the
/// same however many threads draw.
pub fn draw_beside(
    first: impl Fn() -> Result<(), Refused> + Sync,
    second: impl Fn() -> Result<(), Refused> + Sync,
) -> Result<(), Refused> {
    carried_census("parts beside", || {
        let pool = pool();
        BESIDE.store(true, std::sync::atomic::Ordering::Relaxed);
        let drawn = draw_parts(pool, 2, Merged::All, |n| {
            if n == 0 { first() } else { second() }
        });
        BESIDE.store(false, std::sync::atomic::Ordering::Relaxed);
        drawn
    })
}

/// Whether `RC2K_CARRIED_CENSUS` is set (`carried_census`).
fn census() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("RC2K_CARRIED_CENSUS").is_some())
}

/// With `RC2K_CARRIED_CENSUS` (frame plan F4): which carried stores the drawing changes. The
/// game's copy of each before and after `drawing`, the changed bytes unioned per store over
/// the run and logged every 200 calls: a store never changed needs no copy per run. Other
/// threads' writes in the same time show too (the audio thread's environment).
fn carried_census(
    label: &str,
    drawing: impl FnOnce() -> Result<(), Refused>,
) -> Result<(), Refused> {
    if !census() {
        return drawing();
    }
    static CHANGED: Mutex<(u32, Vec<Vec<bool>>)> = Mutex::new((0, Vec::new()));
    let bytes = || -> Vec<Vec<u8>> {
        carried_stores()
            .iter()
            .map(|s| {
                let store = s.copy(0);
                let mut v = vec![0u8; store.size()];
                let _ = store.load(0, &mut v);
                v
            })
            .collect()
    };
    let before = bytes();
    let drawn = drawing();
    let after = bytes();
    let mut c = CHANGED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if c.1.is_empty() {
        c.1 = before.iter().map(|b| vec![false; b.len()]).collect();
    }
    for ((b, a), m) in before.iter().zip(&after).zip(&mut c.1) {
        for (k, flag) in m.iter_mut().enumerate() {
            *flag |= b[k] != a[k];
        }
    }
    c.0 += 1;
    if c.0.is_multiple_of(200) {
        let lines: Vec<String> =
            c.1.iter()
                .enumerate()
                .map(|(n, m)| {
                    let changed = m.iter().filter(|&&f| f).count();
                    let first = m.iter().position(|&f| f);
                    let last = m.iter().rposition(|&f| f);
                    format!(
                        "{n}:{changed}/{}{}",
                        m.len(),
                        match (first, last) {
                            (Some(f), Some(l)) => format!("@{f:#x}..{l:#x}"),
                            _ => String::new(),
                        }
                    )
                })
                .collect();
        crate::framework::log(&format!(
            "workers: carried census after {} calls ({label} last): {}",
            c.0,
            lines.join(" ")
        ));
        // The changed bytes as runs, for naming the fields (`first..end` each).
        let runs: Vec<String> =
            c.1.iter()
                .enumerate()
                .filter(|(_, m)| m.contains(&true))
                .map(|(n, m)| {
                    let mut out = Vec::new();
                    let mut k = 0;
                    while k < m.len() {
                        if m[k] {
                            let start = k;
                            while k < m.len() && m[k] {
                                k += 1;
                            }
                            out.push(format!("{start:#x}..{k:#x}"));
                        }
                        k += 1;
                    }
                    format!("{n}:{}", out.join(","))
                })
                .collect();
        crate::framework::log(&format!(
            "workers: carried runs ({label} last): {}",
            runs.join(" ")
        ));
    }
    drawn
}

/// Which carried stores' copies are merged back after the runs.
#[derive(Clone, Copy, PartialEq)]
enum Merged {
    /// Every one (parts beside each other: the mirror changes much of what its view does).
    All,
    /// Those a list's drawing changes (`changed_in_lists`); with `RC2K_CARRIED_CENSUS` every
    /// one, so that a change in another shows.
    ChangedInLists,
}

/// Of the carried stores, whether a list's drawing changes `store` (the carried census, frame
/// plan F7: only the render states set last, `Cards3d::kept_states` and `current_texture`).
/// The others' copies are only read by the runs of a list.
fn changed_in_lists(store: &dyn Carried) -> bool {
    let cards: &dyn Carried = &crate::d3d::owner::d3d().cards;
    std::ptr::addr_eq(store, cards)
}

/// Parts 0 .. `k` of a drawing (`part`), part 0 on the game's thread and each other on a
/// drawing thread with its own copy of the working state, all set from the game's own at the
/// start; the other parts' noted drawing put after part 0's in their order, and each copy's
/// changes put into the game's own in that order.
fn draw_parts(
    pool: Option<&rayon::ThreadPool>,
    k: usize,
    merged: Merged,
    part: impl Fn(usize) -> Result<(), Refused> + Sync,
) -> Result<(), Refused> {
    register_copies();
    // One after another on the game's thread without a pool or with `RC2K_DRAW_SERIAL`.
    let pool = pool.filter(|_| !serial());
    let stores = carried_stores();
    for s in stores {
        s.begin_runs(k);
    }
    *ENDED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = [false; COUNT];
    let control = crate::math::x87::control_word();
    // Run `n` of the others, on the thread it is called on: its noted drawing from `from` on.
    let other = |n: usize, from: usize| {
        INDEX.set(n);
        IN_RUN.set(true);
        crate::math::x87::set_control_word(control);
        crate::race::scene_points::forget_own();
        #[cfg(feature = "write_watch")]
        crate::write_watch::begin();
        crate::d3d::run_states::begin_run();
        let result = part(n);
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
        let result = part(0);
        IN_RUN.set(false);
        ended(0);
        result
    };
    let (first, others) = if let Some(pool) = pool {
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
        let others = slots
            .into_iter()
            .map(|slot| {
                slot.into_inner()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .ok_or(BadInput("a drawing run that did not end"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        (first, others)
    } else {
        let first = first_run();
        // Run 0's watch ends before the others' begin on this thread.
        #[cfg(feature = "write_watch")]
        WATCHED
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(crate::write_watch::end());
        let others: Vec<OtherRun> = (1..k)
            .map(|n| other(n, crate::platform_3d::noted_len()))
            .collect();
        (first, others)
    };
    #[cfg(feature = "write_watch")]
    {
        // Run 0's watch ends here (one after another it ended before the others').
        let mut watched = WATCHED
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if pool.is_some() {
            watched.push(crate::write_watch::end());
        }
        crate::race::draw_chunks::shared::add(&watched);
        watched.clear();
    }
    let mut result = first;
    for (n, mut run) in others.into_iter().enumerate() {
        result = result.and(run.result);
        // Its fans before it set a state draw with what the earlier runs left.
        let carried = crate::d3d::run_states::carried();
        crate::d3d::run_states::patch(&mut run.noted, &run.states, &carried);
        crate::platform_3d::put_noted(run.noted);
        for s in stores {
            if merged == Merged::All || census() || changed_in_lists(s) {
                s.merge_run(n + 1);
            }
        }
        crate::d3d::run_states::carry(&run.states, n + 1);
        crate::race::view_grid::publish_part(n + 1);
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
