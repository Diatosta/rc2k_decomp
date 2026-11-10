//! A view's own working state (frame plan F4, `notes/frame_plan.md`): what a view's drawing
//! writes and reads within itself, one `ViewScratch` per view slot (each player's view and
//! each one's mirror), set back to its first values as each race view's scene begins
//! (`ViewScratch::begin`; the front end's slot keeps its values between the menus' draws),
//! passed down the drawing in a `DrawCtx` rather than reached through statics. Never read by
//! another step.
//!
//! Holds so far the two sort lists the view's drawing fills, sorts and draws (the original's
//! `g_depthEntries` / `g_sortEntries` at 0x9D3730 in the buffers block, one for every view)
//! and their counts (the original's `RACE_VIEW.depth_entry_count` / `sort_entry_count`), and
//! the view's fog (`scene::fog`), its object's placing (`scene::object_place`) and its
//! object lighting (`scene::object_light`), its face work (`race::face_work`), its cars'
//! drawing state (`race::car_draw_state`), its matrices (`scene::view_matrices`) and its
//! polygon pipeline's work (`scene::polygon_work`).

use crate::errors::BadInput;
use crate::session::Dword;

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
    cells: [Dword; CELLS],
    pub depth_entry_count: Dword,
    pub sort_entry_count: Dword,
}

impl SortLists {
    const fn new() -> Self {
        Self {
            cells: [const { Dword::new(0) }; CELLS],
            depth_entry_count: Dword::new(0),
            sort_entry_count: Dword::new(0),
        }
    }

    /// The depth entries' cells.
    pub fn depth_entries(&self) -> &[Dword] {
        &self.cells[..2 * ENTRIES]
    }

    /// The sort entries' cells.
    pub fn sort_entries(&self) -> &[Dword] {
        &self.cells[2 * ENTRIES..4 * ENTRIES]
    }

    /// Depth entry `n` (the key, the second word). One past the 0x1000 refused.
    pub fn depth_entry(&self, n: u32) -> Result<&[Dword], BadInput> {
        let n = n as usize;
        self.depth_entries()
            .get(2 * n..2 * n + 2)
            .ok_or(BadInput("a depth entry past the table"))
    }

    /// Sort entry `n`. One past the 0x1000 refused.
    pub fn sort_entry(&self, n: u32) -> Result<&[Dword], BadInput> {
        let n = n as usize;
        self.sort_entries()
            .get(2 * n..2 * n + 2)
            .ok_or(BadInput("a sort entry past the table"))
    }

    /// The cells from depth entry `n` on to the end (the sort entries and the overrun after
    /// them): what a flare drawn the described way reads as its description.
    pub fn depth_from(&self, n: usize) -> &[Dword] {
        self.cells.get(2 * n..).unwrap_or(&[])
    }

    /// The same from sort entry `n` on.
    pub fn sort_from(&self, n: usize) -> &[Dword] {
        self.cells.get(2 * ENTRIES + 2 * n..).unwrap_or(&[])
    }
}

/// A view slot's working state.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ViewScratch {
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
    pub car: crate::physics::car_store::Car,
}

impl ViewScratch {
    const fn new() -> Self {
        Self {
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
            car: crate::physics::car_store::Car::new(),
        }
    }

    /// Every part but the sort lists (their counts are set at each view's start,
    /// `race::reset_lamps`) set back to its first values: a race view's drawing starts from
    /// nothing a frame or another view before it left (frame plan F7; with the parts reset at
    /// every race view, held races draw and sound the same as when each kept its last).
    pub fn begin(&self) {
        use crate::pointers::Cells;
        self.fog.copy_from(&FRESH.fog);
        self.object.copy_from(&FRESH.object);
        self.light.copy_from(&FRESH.light);
        self.face.copy_from(&FRESH.face);
        self.cars.copy_from(&FRESH.cars);
        self.matrices.copy_from(&FRESH.matrices);
        self.polygon.copy_from(&FRESH.polygon);
        self.grid.copy_from(&FRESH.grid);
        self.course.copy_from(&FRESH.course);
        self.quad.copy_from(&FRESH.quad);
        self.state.copy_from(&FRESH.state);
        self.lamps.copy_from(&crate::race::lamp_work::PUBLISHED);
    }

    /// The slot's number (`slot`).
    pub fn number(&self) -> usize {
        SLOTS
            .iter()
            .position(|s| std::ptr::eq(s, self))
            .unwrap_or_else(|| {
                crate::fatal::raise::end("a view's slot", BadInput("not one of the slots"))
            })
    }
}

/// The view slots: the two players' views, then their mirrors, then the front end's 3D (the
/// menus' objects and carousel).
static SLOTS: [ViewScratch; SLOT_COUNT] = [const { ViewScratch::new() }; SLOT_COUNT];

/// The slots' parts as they start (`ViewScratch::begin`).
static FRESH: ViewScratch = ViewScratch::new();

/// How many slots there are.
pub const SLOT_COUNT: usize = 5;

/// Slot `n` (below `SLOT_COUNT`).
pub fn slot(n: usize) -> &'static ViewScratch {
    &SLOTS[n]
}

/// Player `car`'s view's working state.
pub fn main_view(car: usize) -> &'static ViewScratch {
    &SLOTS[car]
}

/// Player `car`'s mirror's working state.
pub fn mirror(car: usize) -> &'static ViewScratch {
    &SLOTS[2 + car]
}

/// The front end's 3D's working state.
pub fn front_end() -> &'static ViewScratch {
    &SLOTS[4]
}

/// What a view's drawing works with: the race, the scene, the view's own state, and whether
/// polygons are cut at the far depth.
#[derive(Clone, Copy)]
pub struct DrawCtx<'a> {
    pub race: &'a crate::race::owner::Race,
    pub scene: &'a crate::scene::owner::Scene,
    pub view: &'a ViewScratch,
    /// Off for the own car drawn over the view (`race::draw_view_overlays`; the original
    /// cleared `dword_621900` around it): `scene::clip_polygon_by_depth`'s far pass.
    pub far_clip: bool,
    /// The marks the faces' kinds pick: the car's while one is drawn (`race::draw_car`), none
    /// between cars, the menus' outside a race (`scene::car_marks`).
    pub marks: &'a crate::scene::car_marks::CarMarks,
}

impl<'a> DrawCtx<'a> {
    /// A view's drawing, cut at the far depth, with no car's marks.
    pub fn new(
        race: &'a crate::race::owner::Race,
        scene: &'a crate::scene::owner::Scene,
        view: &'a ViewScratch,
    ) -> Self {
        Self {
            race,
            scene,
            view,
            far_clip: true,
            marks: &crate::scene::car_marks::NONE,
        }
    }
}

/// The texture filter's switches a view starts from (`scene::polygon_work`'s `smooth`,
/// `plain_first`, `plain_second`): for the first view (slot 0) the front end's slot's, where
/// the HUD's overlay left them the frame before (1, 1) or the menus' passes; for the others
/// 0. In the original one set of variables went from view to view: the mirror and the second
/// view read what the first view's last list entry left (0, measured), the first view the
/// HUD's (its sky drawn smooth, the others not).
pub fn filter_input(view: &ViewScratch) -> [u32; 3] {
    if view.number() == 0 {
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

/// The view slot whose polygon work `poly` is (the bad polygon's log names the view's object
/// and face).
pub fn of_polygon(poly: &crate::scene::polygon_work::PolygonWork) -> Option<&'static ViewScratch> {
    SLOTS
        .iter()
        .find(|s| std::ptr::eq(&raw const s.polygon, poly))
}
