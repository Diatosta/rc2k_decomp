//! A view's own working state (frame plan F4, `notes/frame_plan.md`): what a view's drawing
//! writes and reads within itself, one `ViewScratch` per view slot (each player's view and
//! each one's mirror), kept from frame to frame, passed down the drawing in a `DrawCtx`
//! rather than reached through statics. Never read by another step.
//!
//! Holds so far the two sort lists the view's drawing fills, sorts and draws (the original's
//! `g_depthEntries` / `g_sortEntries` at 0x9D3730 in the buffers block, one for every view)
//! and their counts (the original's `RACE_VIEW.depth_entry_count` / `sort_entry_count`), and
//! the view's fog (`scene::fog`) and its object's placing (`scene::object_place`).

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
pub struct ViewScratch {
    pub lists: SortLists,
    /// The view's fog, set up at the start of its drawing (`race::set_up_fog`).
    pub fog: crate::scene::fog::Fog,
    /// The placing, fading and reflecting of the object being drawn.
    pub object: crate::scene::object_place::ObjectPlace,
}

impl ViewScratch {
    const fn new() -> Self {
        Self {
            lists: SortLists::new(),
            fog: crate::scene::fog::Fog::new(),
            object: crate::scene::object_place::ObjectPlace::new(),
        }
    }
}

/// The view slots: the two players' views, then their mirrors, then the front end's 3D (the
/// menus' objects and carousel).
static SLOTS: [ViewScratch; 5] = [const { ViewScratch::new() }; 5];

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
