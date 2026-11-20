//! A view's own working state: what a view's drawing writes and reads within itself, one
//! `ViewScratch` per view slot (each player's view and each one's mirror), set back to its first
//! values as each race view's scene begins (`ViewScratch::begin`; the front end's slot keeps its
//! values between the menus' draws), passed down the drawing in a `DrawCtx`. Never read by
//! another step.
//!
//! The slots are held by `scene::workers::DrawingScratch`. The race's four (views 0 and 1, mirrors 0
//! and 1) are `PartCell`s: the parts of a drawing scope get theirs from `scene::workers::draw_parts`
//! (`scene::workers::PartRun::view`), the main thread reaches them by `main_view`, `mirror` and `slot`
//! outside the parts. The front end's slot belongs to the main thread alone (`front_end`).
//!
//! Every cell in a `ViewScratch` is a plain one (the sort lists, the view's car, the matrices
//! the effects and the listener read included): the scratch is reached by one thread at a time
//! and handed from one to the next, so it is `Send` (asserted below) and not `Sync`. What
//! stays atomic beside it is `DrawingScratch`'s last-drawn-view slot and the mutex of a part's
//! noted drawing (`scene::workers::PartScratch::noted`).
//!
//! Holds the two sort lists the view's drawing fills, sorts and draws and their counts, the
//! view's fog (`scene::fog`), its object's placing (`scene::object_place`) and its object lighting
//! (`scene::object_light`), its face work (`race::face_work`), its cars' drawing state
//! (`race::car_draw_state`), its matrices (`scene::view_matrices`) and its polygon pipeline's work
//! (`scene::polygon_work`).

use crate::errors::BadInput;
use crate::memory::session::PlainDword;

/// The most entries in either list.
pub const ENTRIES: usize = 0x1000;

/// The lists' cells: the depth entries, the sort entries, then the overrun.
const CELLS: usize = 4 * ENTRIES + 0x10;

/// A view's sort lists, 8-byte entries (+0 the key, +4 the second word): the depth entries (lamps,
/// flares, cars, faces drawn by depth) and the sort entries (the course faces); then 0x10 dwords
/// that a flare's entry is read on into as a description of 0x3C bytes
/// (`scene::draw_described_object`), past the last sort entry.
#[derive(rc2k_derive::Cells)]
pub struct SortLists {
    cells: [PlainDword; CELLS],
    pub depth_entry_count: PlainDword,
    pub sort_entry_count: PlainDword,
}

impl SortLists {
    const fn new() -> Self {
        Self {
            cells: [const { PlainDword::new(0) }; CELLS],
            depth_entry_count: PlainDword::new(0),
            sort_entry_count: PlainDword::new(0),
        }
    }

    /// The depth entries' cells.
    pub fn depth_entries(&self) -> &[PlainDword] {
        &self.cells[..2 * ENTRIES]
    }

    /// The sort entries' cells.
    pub fn sort_entries(&self) -> &[PlainDword] {
        &self.cells[2 * ENTRIES..4 * ENTRIES]
    }

    /// Depth entry `n` (the key, the second word). One past the 0x1000 refused.
    pub(super) fn depth_entry(&self, n: u32) -> Result<&[PlainDword], BadInput> {
        let n = n as usize;
        self.depth_entries()
            .get(2 * n..2 * n + 2)
            .ok_or(BadInput("a depth entry past the table"))
    }

    /// Sort entry `n`. One past the 0x1000 refused.
    pub fn sort_entry(&self, n: u32) -> Result<&[PlainDword], BadInput> {
        let n = n as usize;
        self.sort_entries()
            .get(2 * n..2 * n + 2)
            .ok_or(BadInput("a sort entry past the table"))
    }

    /// The cells from depth entry `n` on to the end (the sort entries and the overrun after
    /// them): what a flare drawn the described way reads as its description.
    pub(super) fn depth_from(&self, n: usize) -> &[PlainDword] {
        self.cells.get(2 * n..).unwrap_or(&[])
    }

    /// The same from sort entry `n` on.
    pub(super) fn sort_from(&self, n: usize) -> &[PlainDword] {
        self.cells.get(2 * ENTRIES + 2 * n..).unwrap_or(&[])
    }
}

/// `$f` (`Cells::copy_from` or `merge_changes`) on each part of a view's state but the sort
/// lists (`copy_drawing_from`, `merge_drawing`).
macro_rules! for_each_part {
    ($f:ident, $to:expr, $from:expr $(, $start:expr)?) => {{
        use crate::memory::cells::Cells;
        let (to, from) = ($to, $from);
        to.fog.$f(&from.fog $(, &$start.fog)?);
        to.object.$f(&from.object $(, &$start.object)?);
        to.light.$f(&from.light $(, &$start.light)?);
        to.face.$f(&from.face $(, &$start.face)?);
        to.cars.$f(&from.cars $(, &$start.cars)?);
        to.matrices.$f(&from.matrices $(, &$start.matrices)?);
        to.polygon.$f(&from.polygon $(, &$start.polygon)?);
        to.grid.$f(&from.grid $(, &$start.grid)?);
        to.course.$f(&from.course $(, &$start.course)?);
        to.quad.$f(&from.quad $(, &$start.quad)?);
        to.lamps.$f(&from.lamps $(, &$start.lamps)?);
        to.state.$f(&from.state $(, &$start.state)?);
        to.car.$f(&from.car $(, &$start.car)?);
        to.hand_off.$f(&from.hand_off $(, &$start.hand_off)?);
        to.vars.$f(&from.vars $(, &$start.vars)?);
        to.vars_taken.$f(&from.vars_taken $(, &$start.vars_taken)?);
    }};
}

/// A view slot's working state.
pub struct ViewScratch {
    /// The slot's number (`number`); a copy keeps the slot it was copied from
    /// (`copy_drawing_from`).
    slot: std::cell::Cell<u8>,
    pub lists: SortLists,
    /// The view's fog, set up at the start of its drawing (`race::set_up_fog`).
    pub fog: crate::scene::fog::Fog,
    /// The placing, fading and reflecting of the object being drawn.
    pub object: crate::scene::object_place::ObjectPlace,
    /// The view's object lighting (`scene::object_light`).
    pub light: crate::scene::object_light::ObjectLight,
    /// The course face being drawn (`race::face_work`).
    pub face: crate::race::face_work::FaceWork,
    /// The cars' drawing state (`race::car_draw_state`).
    pub cars: crate::race::car_draw_state::CarDrawState,
    /// The view's matrices and turning work (`scene::view_matrices`).
    pub matrices: crate::scene::view_matrices::ViewMatrices,
    /// The polygon pipeline's working values (`scene::polygon_work`).
    pub polygon: crate::scene::polygon_work::PolygonWork,
    /// The view's course grid (`race::view_grid`).
    pub grid: crate::race::view_grid::ViewGrid,
    /// The course working values (`race::view_course`).
    pub course: crate::race::view_course::ViewCourse,
    /// The screen quad's texture coordinates and corners (`race::quad_work`).
    pub quad: crate::race::quad_work::QuadWork,
    /// The lamps and flares (`race::lamp_work`).
    pub lamps: crate::race::lamp_work::LampWork,
    /// The camera values and the drawn car (`race::view_state`).
    pub state: crate::race::view_state::ViewState,
    /// The view's car: a copy of its block that the view's camera, matrices, course and lights read
    /// (`race::frame::load_view_car`); the physics' work car and `current_car` are not touched.
    pub car: crate::physics::car_store::ViewCar,
    /// What the view's scene left its overlays (`race::scene_hand_off`).
    pub hand_off: crate::race::scene_hand_off::SceneHandOff,
    /// The view variables as the view's drawing has them: taken from the shared ones
    /// (`race().frame.view_and_records.view_vars`) as the view's drawing begins (`take_vars`),
    /// what it changed handed back on the main thread in the views' order (`hand_on_vars`).
    pub vars: crate::race::view_vars_store::ViewVarsCopy,
    /// `vars` as taken.
    vars_taken: crate::race::view_vars_store::ViewVarsCopy,
}

impl ViewScratch {
    pub(crate) const fn new(slot: u8) -> Self {
        Self {
            slot: std::cell::Cell::new(slot),
            lists: SortLists::new(),
            fog: crate::scene::fog::Fog::new(),
            object: crate::scene::object_place::ObjectPlace::new(),
            light: crate::scene::object_light::ObjectLight::new(),
            face: crate::race::face_work::FaceWork::new(),
            cars: crate::race::car_draw_state::CarDrawState::new(),
            matrices: crate::scene::view_matrices::ViewMatrices::new(),
            polygon: crate::scene::polygon_work::PolygonWork::new(),
            grid: crate::race::view_grid::ViewGrid::new(),
            course: crate::race::view_course::ViewCourse::new(),
            quad: crate::race::quad_work::QuadWork::new(),
            lamps: crate::race::lamp_work::LampWork::new(),
            state: crate::race::view_state::ViewState::new(),
            car: crate::physics::car_store::ViewCar::new(),
            hand_off: crate::race::scene_hand_off::SceneHandOff::new(),
            vars: crate::race::view_vars_store::ViewVarsCopy::new(),
            vars_taken: crate::race::view_vars_store::ViewVarsCopy::new(),
        }
    }

    /// The shared view variables (`reads.view_and_records`) into the view's (`vars`), as its
    /// drawing begins.
    pub(super) fn take_vars(&self, reads: crate::scene::workers::DrawReads<'_>) {
        let shared = &reads.view_and_records.view_vars;
        use crate::memory::cells::Cells;
        self.vars.copy_from(shared);
        self.vars_taken.copy_from(shared);
    }

    /// Hands what the view's drawing changed in its view variables into the shared ones (on the
    /// main thread, once it is drawn).
    pub(super) fn hand_on_vars(&self, race: &crate::race::owner::Race) {
        use crate::memory::cells::Cells;
        race.frame
            .view_and_records
            .get()
            .view_vars
            .merge_changes(&self.vars, &self.vars_taken);
    }

    /// Every part but the lamps and the sort lists set back to its first values: a race view's
    /// drawing starts from nothing a frame or another view before it left (held races draw and
    /// sound the same as when each part kept its last values). The lamps are the view's own and
    /// keep what its last drawing left (its light and sky read them before `race::reset_lamps`);
    /// the sort lists' counts are set at each view's start (`race::reset_lamps`).
    pub fn begin(&self) {
        use crate::memory::cells::Cells;
        self.fog.copy_from(&crate::scene::fog::Fog::new());
        self.object
            .copy_from(&crate::scene::object_place::ObjectPlace::new());
        self.light
            .copy_from(&crate::scene::object_light::ObjectLight::new());
        self.face
            .copy_from(&crate::race::face_work::FaceWork::new());
        self.cars
            .copy_from(&crate::race::car_draw_state::CarDrawState::new());
        self.matrices
            .copy_from(&crate::scene::view_matrices::ViewMatrices::new());
        // What the view's set-up gave before its scene is kept: the mirror's flag
        // (`race::mirror_view_window`), the view's value (`race::fetch_view_value`) and its
        // window (`scene::view_window`). The fresh copy is made beside and copied in whole, so the
        // window is never seen reset: the mirror's part reads its view's depth while the view
        // begins.
        let fresh = crate::scene::polygon_work::PolygonWork::new();
        fresh.window.copy_from(&self.polygon.window);
        fresh.view_mirrored.set(self.polygon.view_mirrored.get());
        fresh.sun.view_out.set(self.polygon.sun.view_out.get());
        self.polygon.copy_from(&fresh);
        // The course grid is kept: it is made again only when the view's corner moves, and a
        // stage's start marks every corner unknown (`view_grid::reset_corners`).
        self.course
            .copy_from(&crate::race::view_course::ViewCourse::new());
        self.quad
            .copy_from(&crate::race::quad_work::QuadWork::new());
        self.state
            .copy_from(&crate::race::view_state::ViewState::new());
    }

    /// The slot's number.
    pub fn number(&self) -> usize {
        usize::from(self.slot.get())
    }

    /// Every part but the sort lists set from `other`'s, and its slot: a copy of the view as
    /// its drawing has it now, for a piece of its drawing on another thread
    /// (`race::car_part`), which reads no list.
    pub fn copy_drawing_from(&self, other: &ViewScratch) {
        self.slot.set(other.slot.get());
        for_each_part!(copy_from, self, other);
    }

    /// What `now` (a copy, `copy_drawing_from`) changed since it was `start` put into this
    /// view, part by part but the sort lists: the copy's drawing put after this view's own.
    pub(super) fn merge_drawing(&self, now: &ViewScratch, start: &ViewScratch) {
        for_each_part!(merge_changes, self, now, start);
    }

    /// The view drawn (`RaceView::drawn_view`): the player whose view or mirror the slot is; the
    /// front end's slot reads the store, which only the main thread writes (the race's views set
    /// it before each is drawn).
    pub fn drawn_view(&self, race: &crate::race::owner::Race) -> u32 {
        match self.number() {
            4 => race.shared.view.get().drawn_view.get(),
            // With more than two players the four slots are the players' views (no mirrors).
            n if many_views() => n as u32,
            n => (n % 2) as u32,
        }
    }
}

// A view's scratch is handed from one thread to the next (`PartCell`'s proof needs `Send`):
// no atomic is left in it, and no cell of it may be shared by reference between threads.
const _: () = {
    const fn assert_send<T: Send>() {}
    assert_send::<ViewScratch>();
};

static MANY_VIEWS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Whether the race is a split screen of three or four players: then the view slots are the
/// players' views (slot `n` player `n`) and there are no mirrors, where with one or two players
/// slots 2 and 3 are the first two players' mirrors. Set by the frame's drawing, before its
/// parts run.
pub(super) fn set_many_views(many: bool) {
    MANY_VIEWS.store(many, std::sync::atomic::Ordering::Relaxed);
}

/// Whether the view slots are the players' (`set_many_views`).
pub(super) fn many_views() -> bool {
    MANY_VIEWS.load(std::sync::atomic::Ordering::Relaxed)
}

/// The slots the parts of a drawing draw: the views and the mirrors.
pub(crate) const PART_SLOTS: usize = 4;

/// How many slots there are.
pub const SLOT_COUNT: usize = 5;

/// Whether `slots` can be drawn by one part each: at least one, each a view's or a mirror's,
/// no two the same.
pub fn drawable_slots(slots: &[usize]) -> bool {
    !slots.is_empty()
        && slots.iter().all(|&s| s < PART_SLOTS)
        && slots
            .iter()
            .enumerate()
            .all(|(n, s)| !slots[..n].contains(s))
}

/// What a view's drawing works with: the race, the scene, the network session and the keys (read
/// only, by the lamps' choice), the view's own state, and whether polygons are cut at the far
/// depth.
#[derive(Clone, Copy)]
pub struct DrawCtx<'a> {
    pub race: &'a crate::race::owner::Race,
    pub scene: &'a crate::scene::owner::Scene,
    /// Read only: the keys held (`scene::set_object_light`).
    pub input: &'a crate::input::owner::Input,
    pub view: &'a ViewScratch,
    /// The scene's clock as the frame took it (`restart_clock` in `race::draw_race_scene`,
    /// before any view is drawn), what the animated shapes step by.
    pub clock: u32,
    /// Off for the own car drawn over the view (`race::draw_view_overlays`):
    /// `scene::clip_polygon_by_depth`'s far pass.
    pub far_clip: bool,
    /// The marks the faces' kinds pick: the car's while one is drawn (`race::draw_car`), none
    /// between cars, the menus' outside a race (`scene::car_marks`).
    pub marks: &'a crate::scene::car_marks::CarMarks,
    /// The drawing part's scratch (`scene::workers::PartScratch`): the vertex buffer, the polygon
    /// work and the point records it draws with.
    pub part: &'a crate::scene::workers::PartScratch,
    /// The stores every drawing part reads (`scene::workers::DrawReads`): the part's own inside a
    /// drawing scope, the owners' on the game's thread outside one.
    pub reads: crate::scene::workers::DrawReads<'a>,
}

impl<'a> DrawCtx<'a> {
    /// A view's drawing on its drawing part's scratch (`part`) and reads (`reads`), cut at the
    /// far depth, with no car's marks.
    #[allow(
        clippy::too_many_arguments,
        reason = "the drawing's owners, its view, scratch and reads: what the context holds"
    )]
    pub fn new(
        race: &'a crate::race::owner::Race,
        scene: &'a crate::scene::owner::Scene,
        input: &'a crate::input::owner::Input,
        view: &'a ViewScratch,
        part: &'a crate::scene::workers::PartScratch,
        reads: crate::scene::workers::DrawReads<'a>,
    ) -> Self {
        Self {
            race,
            scene,
            input,
            view,
            clock: reads.physics_tables.scene_clock.get(),
            far_clip: true,
            marks: &crate::scene::car_marks::NONE,
            part,
            reads,
        }
    }
}

/// The texture filter's switches the view in slot `slot` starts from (`scene::polygon_work`'s
/// `smooth`, `plain_first`, `plain_second`): for the first view (slot 0) the front end's slot's,
/// where the HUD's overlay left them the frame before (1, 1) or the menus' passes; for the others
/// 0 (measured: the mirror and the second view always started from what the first view's last list
/// entry left). The first view's sky is drawn smooth, the others' not. Taken on the main thread,
/// before the parts (the front end's slot is not a part's); the view's drawing is handed the
/// values.
pub fn filter_input(scratch: &crate::scene::workers::DrawingScratch, slot: usize) -> [u32; 3] {
    if slot == 0 {
        let from = &scratch.front_end().polygon;
        [
            from.smooth.get(),
            from.plain_first.get(),
            from.plain_second.get(),
        ]
    } else {
        [0; 3]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A view's course grid outlives its frame, and a fresh one is not taken for made: at a
    /// corner of (0, 0) a grid of record 0 everywhere would be drawn (the course vanished
    /// where a stage's camera sat at that corner).
    #[test]
    fn the_course_grid_is_kept_between_frames_and_starts_unmade() {
        let view = ViewScratch::new(0);
        assert_eq!(view.grid.x.get(), crate::race::view_grid::UNKNOWN_CORNER);
        assert_eq!(view.grid.z.get(), crate::race::view_grid::UNKNOWN_CORNER);
        view.grid.x.set(0);
        view.grid.z.set(0);
        view.grid.cells[3].set(42);
        view.begin();
        assert_eq!((view.grid.x.get(), view.grid.z.get()), (0, 0));
        assert_eq!(view.grid.cells[3].get(), 42);
    }

    /// A copy answers with its source's slot; merging it back puts what it changed since it
    /// was taken over what the view changed meanwhile, and leaves the rest as the view has it.
    #[test]
    fn a_copy_is_merged_over_the_view() {
        let (view, copy, taken) = (
            ViewScratch::new(1),
            ViewScratch::new(0),
            ViewScratch::new(0),
        );
        view.cars.car_depth.set(5);
        view.light.view_kind.set(7);
        copy.copy_drawing_from(&view);
        taken.copy_drawing_from(&view);
        assert_eq!(copy.number(), 1);
        // The view draws on; the copy changes the depth and the object number.
        view.light.view_kind.set(2);
        view.cars.object_number.set(3);
        copy.cars.car_depth.set(9);
        copy.cars.object_number.set(4);
        view.merge_drawing(&copy, &taken);
        assert_eq!(view.cars.car_depth.get(), 9);
        assert_eq!(view.cars.object_number.get(), 4);
        assert_eq!(view.light.view_kind.get(), 2);
        assert_eq!(view.number(), 1);
    }
}
