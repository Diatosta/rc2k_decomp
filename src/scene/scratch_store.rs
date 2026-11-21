//! The scratch the drawing works on: each drawing part's own (`PartScratch`), the view slots',
//! and the copies a piece drawn aside and a list cut into runs work on, all held in one
//! `DrawingScratch` by the scene. The drawing threads that claim them are in `scene::workers`.

use crate::memory::draw_order::{COUNT, DrawScope, ScopeClaims, in_scope, index};
use crate::memory::part_cell::{PartCell, PartTurn};
use crate::memory::session::GameThread;
use crate::race::view_scratch::{PART_SLOTS, ViewScratch};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};

/// What a drawing part makes for itself before reading (see the module's header): neither
/// copied in nor merged, each part's work on its own (the replays' every flush the same as on
/// one thread).
pub struct PartScratch {
    /// The polygon vertex buffer (`d3d::vertices_store`).
    pub vertices: crate::d3d::vertices_store::PolygonVertices,
    /// The polygon work: the entries, their lists, the clipper's pool
    /// (`scene::polygon_store`).
    pub polygon: crate::scene::polygon_store::PolygonWork,
    /// The point records, a cache of the points' projections (`scene::point_records`).
    pub records: crate::scene::point_records::Records,
    /// Which records have been written since they were last forgotten
    /// (`scene::point_records::Written`).
    pub records_written: crate::scene::point_records::Written,
    /// Whether the point records are taken (`scene::point_records::take`), set for every part
    /// on the game's thread between scopes.
    pub records_taken: crate::memory::session::PlainFlag,
    /// The held matrix columns while a part other than the first is drawn
    /// (`scene::view_matrices::Held`), from the list's start.
    pub columns: crate::scene::view_matrices::OwnColumns,
    /// The scene points past the count while the part is drawn in a run
    /// (`race::scene_points::OwnPoints`).
    pub points: crate::race::scene_points::OwnPoints,
    /// The render states' and the current texture's cache (`d3d::render_cache::OwnCache`):
    /// part 0's is always in use, the others' while their run is drawn.
    pub cache: crate::d3d::render_cache::OwnCache,
    /// Where such a part first set each state (`d3d::run_states::OwnRun`).
    pub run: crate::d3d::run_states::OwnRun,
    /// The drawing it has noted (`d3d::device::Noted`): the first part's what the game's
    /// thread flushes, the others' put after it at their runs' ends. Behind a mutex: it is
    /// reached by the game's thread (`d3d::device::game_part`) as well as by the part's own.
    pub noted: crate::d3d::device::Noted,
    /// Whether a race is on (`race::numbers_store::Numbers::in_race`) as the parts of a
    /// drawing scope read it, set from the store before the scope opens (`draw_parts`): the
    /// GPU draws only then. Read through `in_race`.
    pub in_race: crate::memory::session::PlainFlag,
    /// The screen's width, height and bits per pixel (`session::Screen`) as the parts of a
    /// drawing scope read them for the noted drawing, set from the store before the scope
    /// opens (`draw_parts`), as `in_race` is. Read by `d3d::device`.
    pub screen_size: [crate::memory::session::PlainDword; 3],
}

impl PartScratch {
    /// The scratch of part 0 (`first`: its render cache in use from the start) or another part,
    /// zeroed.
    const fn new(first: bool) -> Self {
        Self {
            vertices: crate::d3d::vertices_store::PolygonVertices::new(),
            polygon: crate::scene::polygon_store::PolygonWork::new(),
            records: crate::scene::point_records::new_records(),
            records_written: crate::scene::point_records::new_written(),
            records_taken: crate::memory::session::PlainFlag::new(false),
            columns: crate::scene::view_matrices::OwnColumns::new(),
            points: crate::race::scene_points::OwnPoints::new(),
            cache: if first {
                crate::d3d::render_cache::OwnCache::first()
            } else {
                crate::d3d::render_cache::OwnCache::new()
            },
            run: crate::d3d::run_states::OwnRun::new(),
            noted: crate::d3d::device::Noted::new(),
            in_race: crate::memory::session::PlainFlag::new(false),
            screen_size: [const { crate::memory::session::PlainDword::new(0) }; 3],
        }
    }
}

// A part's scratch is handed from one thread to the next (`PartCell`'s proof needs `Send`).
const _: () = {
    const fn assert_send<T: Send>() {}
    assert_send::<PartScratch>();
};

/// Every scratch the drawing works on, held by the scene (`Scene::scratch`) and reached through
/// it: the parts' (`PartScratch`), the view slots' (`ViewScratch`), the front end's, and the copies
/// the car drawn aside works on.
///
/// Run `n` of a drawing scope claims part `n` and its view slot (`draw_parts`); the game's thread
/// reaches part 0, the front end's slot and (with no scope open) the others without a claim.
pub struct DrawingScratch {
    parts: [PartCell<PartScratch>; COUNT],
    /// The scratch a piece drawn aside draws on: one piece at a time, its thread holding the lock
    /// for its run (`draw_aside`), so it is reached by one thread at a time with no `unsafe`.
    aside: Mutex<PartScratch>,
    /// The race's view slots: the two players' views, then their mirrors. Each is claimed by the
    /// run that draws it, or reached by the game's thread.
    views: [PartCell<ViewScratch>; PART_SLOTS],
    /// The front end's 3D (the menus' objects and carousel; the HUD, the loading bar and the
    /// effects draw on its polygon work): the game's thread's alone, never drawn by a part.
    front_end: GameThread<ViewScratch>,
    /// The copy of the view the car drawn aside works on (`race::car_part`): the thread drawing the
    /// car holds the lock for its run, the view's thread to copy into it and merge from it.
    car_view: Mutex<ViewScratch>,
    /// The view as it was when the car's copy was made: what the copy changed since is merged.
    taken: GameThread<ViewScratch>,
    /// What the runs of a list cut into runs draw on (`draw_list_in_runs`): each run but the
    /// part's own first one a part's scratch and a copy of the view, held by the thread drawing
    /// the run for its length.
    pub(crate) list_runs: [RunScratch; crate::memory::draw_order::LIST_RUNS],
    /// The view as it was when the list's runs began: what each run's copy starts from and what
    /// its changes are told by (the game's thread's).
    pub(crate) list_start: GameThread<ViewScratch>,
    /// The slot of the view that last drew a course face (`race::face_work`), `usize::MAX` for
    /// none yet. Written by every drawing thread.
    last_face_view: AtomicUsize,
}

impl DrawingScratch {
    /// Every scratch in its first state.
    pub const fn new() -> Self {
        Self {
            parts: [
                PartCell::new(PartScratch::new(true)),
                PartCell::new(PartScratch::new(false)),
                PartCell::new(PartScratch::new(false)),
                PartCell::new(PartScratch::new(false)),
            ],
            aside: Mutex::new(PartScratch::new(false)),
            views: [
                PartCell::new(ViewScratch::new(0)),
                PartCell::new(ViewScratch::new(1)),
                PartCell::new(ViewScratch::new(2)),
                PartCell::new(ViewScratch::new(3)),
            ],
            front_end: GameThread::new(ViewScratch::new(4)),
            car_view: Mutex::new(ViewScratch::new(0)),
            taken: GameThread::new(ViewScratch::new(0)),
            list_runs: [const { RunScratch::new() }; crate::memory::draw_order::LIST_RUNS],
            list_start: GameThread::new(ViewScratch::new(0)),
            last_face_view: AtomicUsize::new(usize::MAX),
        }
    }

    /// Every part's scratch, on the game's thread with no drawing scope open (so no run holds
    /// one): what a clear of the point records reaches.
    pub fn all_parts_on_game_thread(&self) -> impl Iterator<Item = &PartScratch> {
        debug_assert!(
            !in_scope(),
            "every part's scratch reached inside a drawing scope"
        );
        self.parts.iter().map(PartCell::on_game_thread)
    }

    /// The first part's scratch, on the game's thread (inside a drawing scope, run 0 is that
    /// thread): what everything drawn outside the parts (the overlays, the HUD, the front end)
    /// works on, and what the platform's flush and the render states' cache reach. The other
    /// parts' are handed to their runs (`PartRun`).
    pub fn game_scratch(&self) -> &PartScratch {
        debug_assert_eq!(index(), 0, "the game's scratch on a drawing part");
        self.parts[0].on_game_thread()
    }

    /// Part `n`'s scratch for the run whose `turn` it is, until the scope ends.
    pub(crate) fn claim_part<'s>(&'s self, n: usize, turn: &PartTurn<'s>) -> &'s PartScratch {
        self.parts[n].claim(turn)
    }

    /// View slot `slot` (below `PART_SLOTS`) for the run whose `turn` it is, until the scope
    /// ends.
    pub(crate) fn claim_view<'s>(&'s self, slot: usize, turn: &PartTurn<'s>) -> &'s ViewScratch {
        self.views[slot].claim(turn)
    }

    /// Slot `n` (below `SLOT_COUNT`), on the game's thread.
    pub fn slot(&self, n: usize) -> &ViewScratch {
        match self.views.get(n) {
            Some(view) => view.on_game_thread(),
            None => self.front_end(),
        }
    }

    /// Player `car`'s view's working state, on the game's thread (the parts are handed theirs).
    pub fn main_view(&self, car: usize) -> &ViewScratch {
        self.views[car].on_game_thread()
    }

    /// Player `car`'s mirror's working state, on the game's thread.
    pub fn mirror(&self, car: usize) -> &ViewScratch {
        self.views[2 + car].on_game_thread()
    }

    /// The front end's 3D's working state, on the game's thread.
    pub fn front_end(&self) -> &ViewScratch {
        self.front_end.get()
    }

    /// The front end's polygon work: the HUD's, the loading bar's, the frame's set-up and the
    /// effects', which draw outside any view's pipeline.
    pub fn front_end_polygon(&self) -> &crate::scene::polygon_work_store::PolygonWork {
        &self.front_end().polygon
    }

    /// The copy of the view the car drawn aside works on, locked.
    pub fn car_view(&self) -> std::sync::MutexGuard<'_, ViewScratch> {
        self.car_view
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// The view as it was when the car's copy was made (the game's thread's).
    pub fn taken(&self) -> &ViewScratch {
        self.taken.get()
    }

    /// The scratch a piece drawn aside draws on, locked.
    pub(crate) fn aside(&self) -> std::sync::MutexGuard<'_, PartScratch> {
        self.aside
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// The view in slot `slot` is drawing a face.
    pub fn note_face_drawn(&self, slot: usize) {
        self.last_face_view.store(slot, Ordering::Relaxed);
    }

    /// The slot of the view that last drew a course face, if any has.
    pub fn last_face_view(&self) -> Option<usize> {
        match self.last_face_view.load(Ordering::Relaxed) {
            usize::MAX => None,
            slot => Some(slot),
        }
    }
}

impl ScopeClaims for DrawingScratch {
    /// The scope ending: the parts and the slots are the game's thread's again.
    fn end_claims(&self, ending: &DrawScope<'_>) {
        for part in &self.parts {
            part.end_claim(ending);
        }
        for view in &self.views {
            view.end_claim(ending);
        }
    }
}

impl Default for DrawingScratch {
    fn default() -> Self {
        Self::new()
    }
}

/// What a run other than its part's own draws on: a part's scratch and a copy of the view,
/// each behind a mutex its run's thread holds while it draws (and the part's thread while it
/// copies the view in and puts the run's work back).
pub struct RunScratch {
    part: Mutex<PartScratch>,
    view: Mutex<ViewScratch>,
}

impl RunScratch {
    pub const fn new() -> Self {
        Self {
            part: Mutex::new(PartScratch::new(false)),
            view: Mutex::new(ViewScratch::new(0)),
        }
    }

    pub fn part(&self) -> MutexGuard<'_, PartScratch> {
        self.part.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn view(&self) -> MutexGuard<'_, ViewScratch> {
        self.view.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
