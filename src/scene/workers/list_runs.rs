//! A list inside a part drawn in runs: the list's entries cut into runs of entries next to each
//! other, the first drawn by the part's own thread on the part's own state, each other on a
//! drawing thread with a scratch of its own (`RunScratch`: a part's scratch and a copy of the
//! view as the list began). When every run has ended the others' drawing is put after the
//! first's in the runs' order, as `draw_parts` puts the parts' together: the noted fans (those
//! before a run's own setting of a render state given the value the earlier runs left), the
//! render states and the texture each run set, the point records it wrote, the held matrix
//! columns it changed, and what it changed in its copy of the view (`put_view`, the caller's).
//! So the noted drawing is what one thread notes for the whole list.
//!
//! That holds for a list whose entries each draw from the view as the list began and from
//! what the entry itself makes: no entry may read what the entry before it left (in the view,
//! the part's scratch or the point records). The course's sorted faces are such a list
//! (`race::face_runs`, where the proof is told).
//!
//! What must happen in the list's order whatever the threads do (the random numbers) waits in
//! a later run until every earlier run has ended (`memory::draw_order::list_run_index`).

use super::{AsideRun, AsideStart, DrawReads, DrawingScratch, Helper, PartScratch, run_aside};
use crate::errors::{BadInput, Refused};
use crate::memory::draw_order::{
    LIST_RUNS, begin_list_runs, index, list_run_ended, list_run_index, set_index,
};
use crate::race::view_scratch::ViewScratch;
use std::ops::Range;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock, PoisonError};

/// What a run other than its part's own draws on: a part's scratch and a copy of the view,
/// each behind a mutex its run's thread holds while it draws (and the part's thread while it
/// copies the view in and puts the run's work back).
pub(super) struct RunScratch {
    part: Mutex<PartScratch>,
    view: Mutex<ViewScratch>,
}

impl RunScratch {
    pub(super) const fn new() -> Self {
        Self {
            part: Mutex::new(PartScratch::new(false)),
            view: Mutex::new(ViewScratch::new(0)),
        }
    }

    fn part(&self) -> MutexGuard<'_, PartScratch> {
        self.part.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn view(&self) -> MutexGuard<'_, ViewScratch> {
        self.view.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// What draws a run's entries (the range of the list's) on the run's part scratch and view,
/// reading the scope's stores: called on whichever thread draws the run.
pub type DrawRun<'a> = dyn Fn(&PartScratch, &ViewScratch, DrawReads<'_>, Range<usize>) -> Result<(), Refused>
    + Sync
    + 'a;

/// The fewest entries a run is worth its copy of the view and its thread's waking for.
const FEWEST_IN_A_RUN: usize = 48;

/// `RC2K_FACE_RUNS` (a switch for checks and comparisons of times): `0` draws every list on
/// its part's thread alone, as before; a number `n` cuts every list into up to `n` runs
/// whatever its length (with `RC2K_DRAW_SERIAL` any number of them, one after another: 4096
/// draws every entry as a run of its own). Unset: runs as the idle drawing threads allow.
fn asked_runs() -> Option<usize> {
    static ASKED: OnceLock<Option<usize>> = OnceLock::new();
    *ASKED.get_or_init(|| std::env::var("RC2K_FACE_RUNS").ok()?.trim().parse().ok())
}

/// How many runs a list of `entries` entries is drawn in by a part with `helper`, of whose
/// spare drawing threads `busy` are taken (a piece drawn aside): 1 for the part's thread
/// alone.
pub fn list_runs_wanted(helper: Option<Helper<'_>>, busy: usize, entries: usize) -> usize {
    let Some(helper) = helper else {
        return 1;
    };
    match asked_runs() {
        Some(0) => 1,
        // One after another any number can be drawn: each run's scratch is free again once
        // its work is put back.
        Some(n) if helper.pool.is_none() => n.min(entries).max(1),
        Some(n) => n.min(1 + LIST_RUNS).min(entries).max(1),
        None => {
            let threads = 1 + helper.spare.saturating_sub(busy).min(LIST_RUNS);
            threads.min(entries / FEWEST_IN_A_RUN).max(1)
        }
    }
}

/// The entries of run `run` of `runs` over `entries` entries: even shares, in order.
fn run_entries(entries: usize, runs: usize, run: usize) -> Range<usize> {
    entries * run / runs..entries * (run + 1) / runs
}

/// A run's end told when dropped, however the run ends: no later run waits for a turn that
/// will not come.
struct RunEnds(usize);

impl Drop for RunEnds {
    fn drop(&mut self) {
        list_run_ended(self.0);
    }
}

/// A run other than the first drawn on `scratch` from `start`, on the calling thread. `turn`:
/// its number among runs drawn at once, which what it does in the list's order waits by; none
/// for runs drawn one after another.
fn draw_run(
    scratch: &RunScratch,
    start: &AsideStart,
    turn: Option<usize>,
    entries: Range<usize>,
    reads: DrawReads<'_>,
    draw: &DrawRun<'_>,
) -> AsideRun {
    let was = index();
    let _ends = turn.map(|run| {
        set_index(list_run_index(run));
        RunEnds(run)
    });
    let part = scratch.part();
    let view = scratch.view();
    // No scene point's record is taken for made: each the run reads it makes.
    crate::scene::point_records::mark_scene_records_unmade(&part);
    let ran = run_aside(&part, start, reads, |part, reads| {
        draw(part, &view, reads, entries)
    });
    set_index(was);
    ran
}

/// Run `run`'s work put after the part's so far: its fans (those before its own setting of a
/// state given the part's value of it now), the states and the texture it set, the point
/// records it wrote, the columns it changed, and by `put_view` what it changed in its copy
/// of the view since `start_view`.
fn put_run(
    owner: &DrawingScratch,
    scratch: &RunScratch,
    part: &PartScratch,
    start: &AsideStart,
    mut run: AsideRun,
    put_view: &mut dyn FnMut(&ViewScratch, &ViewScratch),
) -> Result<(), Refused> {
    let carried = crate::d3d::run_states::carried(&part.cache);
    crate::d3d::run_states::patch(&mut run.noted.commands, &run.states, &carried);
    let now = crate::d3d::render_cache::current(&part.cache);
    if crate::d3d::run_states::drew_before_texture(&run.noted.commands, &run.states)
        && !crate::d3d::render_cache::same_texture(&now, &start.cache)
    {
        crate::program_log::log(
            "workers: a list's run drew before setting its texture (not as one thread)",
        );
    }
    crate::d3d::device::put_noted(part, run.noted);
    crate::d3d::run_states::carry_set(&part.cache, &run.states);
    {
        let run_part = scratch.part();
        crate::scene::point_records::put_written(&run_part, part);
        crate::scene::point_records::clear_written(&run_part);
    }
    if run.columns.is_some() && run.columns != start.columns {
        crate::scene::view_matrices::set_part_columns(part, run.columns);
    }
    put_view(&scratch.view(), owner.list_start.get());
    crate::d3d::device::forget_states(part);
    run.result
}

/// The `entries` entries of a list of part `part` (the first, on the game's thread) drawn in
/// `runs` runs (`list_runs_wanted`; see the module's header): `draw(part, view, reads,
/// entries)` draws a run's entries, the first run's on `part` and `view` themselves, each
/// other's on a scratch and a copy of `view` as it is now; `put_view(copy, start)` puts what
/// a run changed in its copy since the list began (`start`) into `view`, called in the runs'
/// order once all have ended. `point_count`: the scene points' count. Without drawing threads
/// (`RC2K_DRAW_SERIAL`) the runs are drawn one after another on the part's thread, each on
/// its scratch and copy all the same.
#[allow(
    clippy::too_many_arguments,
    reason = "the list's part, view and length, and what draws and merges a run"
)]
pub fn draw_list_in_runs<'a>(
    helper: Helper<'a>,
    part: &'a PartScratch,
    view: &ViewScratch,
    point_count: u32,
    entries: usize,
    runs: usize,
    draw: &DrawRun<'_>,
    put_view: &mut dyn FnMut(&ViewScratch, &ViewScratch),
) -> Result<(), Refused> {
    let owner = helper.lent.scratch;
    let lent = helper.lent;
    if runs < 2 {
        return draw(part, view, lent.reads(), 0..entries);
    }
    if helper.pool.is_some() && runs > 1 + LIST_RUNS {
        return Err(BadInput("more runs of a list than scratches for them").into());
    }
    let start = AsideStart {
        point_count,
        cache: crate::d3d::render_cache::current(&part.cache),
        columns: crate::scene::view_matrices::part_columns(part),
        control: crate::math::x87::control_word(),
        in_race: part.in_race.is_set(),
        records_taken: part.records_taken.is_set(),
        screen_size: part
            .screen_size
            .each_ref()
            .map(crate::memory::session::PlainDword::get),
    };
    owner.list_start.get().copy_drawing_from(view);
    let Some(pool) = helper.pool else {
        // One after another: each run on the first scratch, from the view as the list began,
        // its work put back before the next.
        let mut result = draw(part, view, lent.reads(), run_entries(entries, runs, 0));
        let scratch = &owner.list_runs[0];
        for run in 1..runs {
            scratch.view().copy_drawing_from(owner.list_start.get());
            let ran = draw_run(
                scratch,
                &start,
                None,
                run_entries(entries, runs, run),
                lent.reads(),
                draw,
            );
            result = result.and(put_run(owner, scratch, part, &start, ran, put_view));
        }
        return result;
    };
    for scratch in &owner.list_runs[..runs - 1] {
        scratch.view().copy_drawing_from(owner.list_start.get());
    }
    begin_list_runs();
    // The runs not yet begun, from the second: whichever thread is free takes the next.
    let next = AtomicUsize::new(1);
    let ran: [Mutex<Option<AsideRun>>; LIST_RUNS] = [const { Mutex::new(None) }; LIST_RUNS];
    let take_runs = || {
        loop {
            let run = next.fetch_add(1, Ordering::Relaxed);
            if run >= runs {
                break;
            }
            let done = draw_run(
                &owner.list_runs[run - 1],
                &start,
                Some(run),
                run_entries(entries, runs, run),
                lent.reads(),
                draw,
            );
            *ran[run - 1].lock().unwrap_or_else(PoisonError::into_inner) = Some(done);
        }
    };
    let first = pool.in_place_scope(|s| {
        for _ in 1..runs {
            s.spawn(|_| take_runs());
        }
        let first = {
            let _ends = RunEnds(0);
            draw(part, view, lent.reads(), run_entries(entries, runs, 0))
        };
        // The part's thread draws on with the runs no drawing thread has begun.
        take_runs();
        first
    });
    let mut result = first;
    for (scratch, ran) in owner.list_runs.iter().zip(ran).take(runs - 1) {
        let ran = ran
            .into_inner()
            .unwrap_or_else(PoisonError::into_inner)
            .ok_or(BadInput("a run of a list that did not end"))?;
        result = result.and(put_run(owner, scratch, part, &start, ran, put_view));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The runs' entries are the list's, each once, in order, whatever the counts.
    #[test]
    fn the_runs_cover_the_list_in_order() {
        for entries in [0, 1, 2, 23, 24, 100, 4096] {
            for runs in 1..=4 {
                let mut next = 0;
                for run in 0..runs {
                    let range = run_entries(entries, runs, run);
                    assert_eq!(range.start, next);
                    next = range.end;
                }
                assert_eq!(next, entries);
            }
        }
    }

    /// Without a helper a list is drawn by its part's thread alone.
    #[test]
    fn no_helper_no_runs() {
        assert_eq!(list_runs_wanted(None, 0, 4096), 1);
    }
}
