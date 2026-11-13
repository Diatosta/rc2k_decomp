//! A view's own working state (frame plan F4, `notes/frame_plan.md`): what a view's drawing
//! writes and reads within itself, one `ViewScratch` per view slot (each player's view and
//! each one's mirror), set back to its first values as each race view's scene begins
//! (`ViewScratch::begin`; the front end's slot keeps its values between the menus' draws),
//! passed down the drawing in a `DrawCtx` rather than reached through statics. Never read by
//! another step.
//!
//! The race's four slots (views 0 and 1, mirrors 0 and 1) are `PartCell`s: the parts of a
//! drawing scope get theirs from `workers::draw_parts` (`workers::PartRun::view`), the game's
//! thread reaches them by `main_view`, `mirror` and `slot` outside the parts. The front end's
//! slot belongs to the game's thread alone (`front_end`, a `GameThread`).
//!
//! Every cell in a `ViewScratch` is a plain one (the sort lists, the view's car, the matrices
//! the effects and the listener read included): the scratch is reached by one thread at a time
//! and handed from one to the next, so it is `Send` (asserted below) and not `Sync`. What
//! stays atomic beside it is statics (`face_work::LAST_FACE_VIEW`) and the mutex of a part's
//! noted drawing (`workers::PartScratch::noted`).
//!
//! Holds so far the two sort lists the view's drawing fills, sorts and draws (the original's
//! `g_depthEntries` / `g_sortEntries` at 0x9D3730 in the buffers block, one for every view)
//! and their counts (the original's `RACE_VIEW.depth_entry_count` / `sort_entry_count`), and
//! the view's fog (`scene::fog`), its object's placing (`scene::object_place`) and its
//! object lighting (`scene::object_light`), its face work (`race::face_work`), its cars'
//! drawing state (`race::car_draw_state`), its matrices (`scene::view_matrices`) and its
//! polygon pipeline's work (`scene::polygon_work`).

use crate::errors::BadInput;
use crate::part_cell::{PartCell, PartTurn};
use crate::session::{GameThread, PlainDword};

/// The most entries in either list.
pub const ENTRIES: usize = 0x1000;

/// The lists' cells: the depth entries, the sort entries, then the overrun.
const CELLS: usize = 4 * ENTRIES + 0x10;

/// A view's sort lists, 8-byte entries (+0 the key, +4 the second word): the depth entries
/// (lamps, flares, cars, faces drawn by depth) and the sort entries (the course faces); then
/// 0x10 dwords that a flare's entry is read on into as a description of 0x3C bytes
/// (`scene::draw_described_object`), past the last sort entry as past the original's.
#[repr(C)]
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
    pub fn depth_entry(&self, n: u32) -> Result<&[PlainDword], BadInput> {
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
    pub fn depth_from(&self, n: usize) -> &[PlainDword] {
        self.cells.get(2 * n..).unwrap_or(&[])
    }

    /// The same from sort entry `n` on.
    pub fn sort_from(&self, n: usize) -> &[PlainDword] {
        self.cells.get(2 * ENTRIES + 2 * n..).unwrap_or(&[])
    }
}

/// A view slot's working state.
#[repr(C)]
pub struct ViewScratch {
    /// The slot's number (`number`).
    slot: u8,
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
    /// The view's car: a copy of its block that the view's camera, matrices, course and lights
    /// read (`race::frame::load_view_car`). The original loaded it into the work car `g_car`,
    /// the physics' own, and set `g_currentCar`; neither is touched now. (The drawing's
    /// headers name its fields as the original's `g_car` +offsets.)
    pub car: crate::physics::car_store::ViewCar,
    /// What the view's scene left its overlays (`race::scene_hand_off`).
    pub hand_off: crate::race::scene_hand_off::SceneHandOff,
    /// The view variables as the view's drawing has them (`race().frame.view_and_records.
    /// view_vars`, the original's, shared by every view in turn): taken from the shared ones
    /// as the view's drawing begins (`take_vars`), what it changed handed back on the game's
    /// thread in the views' order (`hand_on_vars`; frame plan F7).
    pub vars: crate::race::view_vars_store::ViewVarsCopy,
    /// `vars` as taken.
    vars_taken: crate::race::view_vars_store::ViewVarsCopy,
}

impl ViewScratch {
    const fn new(slot: u8) -> Self {
        Self {
            slot,
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
    pub fn take_vars(&self, reads: crate::workers::DrawReads<'_>) {
        let shared = &reads.view_and_records.view_vars;
        crate::cells::copy_whole(shared, &self.vars);
        crate::cells::copy_whole(shared, &self.vars_taken);
    }

    /// What the view's drawing changed in its view variables into the shared ones (the game's
    /// thread, once it is drawn).
    pub fn hand_on_vars(&self, race: &crate::race::owner::Race) {
        crate::cells::merge_cell_changes(
            &race.frame.view_and_records.get().view_vars,
            &self.vars,
            &self.vars_taken,
        );
    }

    /// Every part but the lamps and the sort lists set back to its first values: a race view's
    /// drawing starts from nothing a frame or another view before it left (frame plan F7; with
    /// the parts reset at every race view, held races draw and sound the same as when each kept
    /// its last). The lamps are the view's own and keep what its last drawing left (its light and
    /// sky read them before `race::reset_lamps`); the sort lists' counts are set at each view's
    /// start (`race::reset_lamps`).
    pub fn begin(&self) {
        use crate::cells::Cells;
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
        // What the view's set-up gave before its scene, kept: the mirror's flag
        // (`race::mirror_view_window`), the view's value (`race::fetch_view_value`) and its
        // window (`scene::view_window`). Made beside and copied in whole, so the window is
        // never seen reset: the mirror's part reads its view's depth while the view begins.
        let fresh = crate::scene::polygon_work::PolygonWork::new();
        fresh.window.copy_from(&self.polygon.window);
        fresh.view_mirrored.set(self.polygon.view_mirrored.get());
        fresh.sun.view_out.set(self.polygon.sun.view_out.get());
        self.polygon.copy_from(&fresh);
        self.grid
            .copy_from(&crate::race::view_grid::ViewGrid::new());
        self.course
            .copy_from(&crate::race::view_course::ViewCourse::new());
        self.quad
            .copy_from(&crate::race::quad_work::QuadWork::new());
        self.state
            .copy_from(&crate::race::view_state::ViewState::new());
    }

    /// The slot's number (`slot`).
    pub fn number(&self) -> usize {
        usize::from(self.slot)
    }

    /// The view drawn (`RACE_VIEW.drawn_view`, the original's `g_drawnView`): the player
    /// whose view or mirror the slot is; the front end's slot reads the store, which only the
    /// game thread writes (the race's views set it before each is drawn, frame plan F7).
    pub fn drawn_view(&self, race: &crate::race::owner::Race) -> u32 {
        match self.number() {
            4 => race.shared.view.get().drawn_view.get(),
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

/// The race's view slots: the two players' views, then their mirrors. Each is claimed by the
/// run that draws it (`claim`), or reached by the game's thread (`slot`).
static VIEWS: [PartCell<ViewScratch>; PART_SLOTS] = [
    PartCell::new(ViewScratch::new(0)),
    PartCell::new(ViewScratch::new(1)),
    PartCell::new(ViewScratch::new(2)),
    PartCell::new(ViewScratch::new(3)),
]; // m6-root

/// The front end's 3D (the menus' objects and carousel; the HUD, the loading bar and the
/// effects draw on its polygon work): the game's thread's alone, never drawn by a part.
static FRONT_END: GameThread<ViewScratch> = GameThread::new(ViewScratch::new(4)); // m6-root

/// The slots the parts of a drawing draw: the views and the mirrors.
const PART_SLOTS: usize = 4;

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

/// Slot `slot` (below `PART_SLOTS`) for the run whose `turn` it is, until the scope ends
/// (`workers::draw_parts`).
pub(crate) fn claim<'s>(slot: usize, turn: &PartTurn<'s>) -> &'s ViewScratch {
    VIEWS[slot].claim(turn)
}

/// The scope ending: the slots are the game's thread's again.
pub(crate) fn end_claims(ending: &crate::workers::DrawScope) {
    for view in &VIEWS {
        view.end_claim(ending);
    }
}

/// Slot `n` (below `SLOT_COUNT`), on the game's thread.
pub fn slot(n: usize) -> &'static ViewScratch {
    match VIEWS.get(n) {
        Some(view) => view.on_game_thread(),
        None => front_end(),
    }
}

/// Player `car`'s view's working state, on the game's thread (the parts are handed theirs).
pub fn main_view(car: usize) -> &'static ViewScratch {
    VIEWS[car].on_game_thread()
}

/// Player `car`'s mirror's working state, on the game's thread.
pub fn mirror(car: usize) -> &'static ViewScratch {
    VIEWS[2 + car].on_game_thread()
}

/// The front end's 3D's working state, on the game's thread.
pub fn front_end() -> &'static ViewScratch {
    FRONT_END.get()
}

/// What a view's drawing works with: the race, the scene, the network session and the keys (read only, by the
/// lamps' choice), the view's own state, and whether polygons are cut at the far depth.
#[derive(Clone, Copy)]
pub struct DrawCtx<'a> {
    pub race: &'a crate::race::owner::Race,
    pub scene: &'a crate::scene::owner::Scene,
    /// Read only: the session and the network player (`scene::add_car_lights`).
    pub net: &'a crate::net::owner::Net,
    /// Read only: the keys held (`scene::set_object_light`).
    pub input: &'a crate::input::owner::Input,
    pub view: &'a ViewScratch,
    /// The scene's clock as the frame took it (`restart_clock` in `race::draw_race_scene`,
    /// before any view is drawn), what the animated shapes step by.
    pub clock: u32,
    /// Off for the own car drawn over the view (`race::draw_view_overlays`; the original
    /// cleared `far_on` around it): `scene::clip_polygon_by_depth`'s far pass.
    pub far_clip: bool,
    /// The marks the faces' kinds pick: the car's while one is drawn (`race::draw_car`), none
    /// between cars, the menus' outside a race (`scene::car_marks`).
    pub marks: &'a crate::scene::car_marks::CarMarks,
    /// The drawing part's scratch (`workers::PartScratch`): the vertex buffer, the polygon
    /// work and the point records it draws with.
    pub part: &'a crate::workers::PartScratch,
    /// The stores every drawing part reads (`workers::DrawReads`): the part's own inside a
    /// drawing scope, the owners' on the game's thread outside one.
    pub reads: crate::workers::DrawReads<'a>,
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
        net: &'a crate::net::owner::Net,
        input: &'a crate::input::owner::Input,
        view: &'a ViewScratch,
        part: &'a crate::workers::PartScratch,
        reads: crate::workers::DrawReads<'a>,
    ) -> Self {
        Self {
            race,
            scene,
            net,
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
/// where the HUD's overlay left them the frame before (1, 1) or the menus' passes; for the
/// others 0. In the original one set of variables went from view to view: the mirror and the
/// second view read what the first view's last list entry left (0, measured), the first view the
/// HUD's (its sky drawn smooth, the others not). On the game's thread, before the parts
/// (the front end's slot is not a part's); the view's drawing is handed the values.
pub fn filter_input(slot: usize) -> [u32; 3] {
    if slot == 0 {
        let from = &front_end().polygon;
        [
            from.smooth.get(),
            from.plain_first.get(),
            from.plain_second.get(),
        ]
    } else {
        [0; 3]
    }
}
