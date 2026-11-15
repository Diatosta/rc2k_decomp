//! The race's grid and scene work: four 256-byte face tables that `object_bytes` fills, the 16
//! lamps, the course grid's sizes and view cell, the 32 x 32 course grid (`course_grid`), the far
//! cell limit, the turned view position and depth shift, and the flares' place-x table. Used by
//! `race::frame_draw`, `scene::add_point_lights`, `scene::draw_polygon`, `race::course_faces`,
//! `hud::overlay_pass`. Plain cells: a drawing store (`session::DrawShared`), written on the game's
//! thread between frames and read by the drawing parts through their `scene::workers::DrawReads`.

use crate::memory::cells::ZeroCells;
use crate::memory::session::{PlainBytes, PlainDword, PlainFloat};

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct LevelGrid {
    /// The first of four 256-byte face tables (`object_bytes` fills them). Unused: each view keeps
    /// its own (`race::view_course::ViewCourse`).
    pub face_table_a: PlainBytes<256>,
    /// The second face table. Unused: each view keeps its own (`race::view_course::ViewCourse`).
    pub face_table_b: PlainBytes<256>,
    /// The third face table. Unused: each view keeps its own (`race::view_course::ViewCourse`).
    pub face_table_c: PlainBytes<256>,
    /// The fourth face table. Unused: each view keeps its own (`race::view_course::ViewCourse`).
    pub face_table_d: PlainBytes<256>,
    /// 16 lamps of 0x10 bytes. Unused: each view keeps its own (`race::lamp_work::LampWork`).
    pub lamps: [PlainDword; 64],
    /// The course grid's half size. Unused: each view keeps its own (`race::view_grid`).
    pub grid_half: PlainDword,
    /// The course grid's span. Unused: each view keeps its own (`race::view_grid`).
    pub grid_span: PlainDword,
    /// The polygon smoothing switch. Unused: each view keeps its own (`scene::polygon_work`).
    pub smooth: PlainDword,
    /// "Plain" flag of the first triangle's texture set. Unused: each view keeps its own
    /// (`scene::polygon_work`).
    pub plain_first: PlainDword,
    /// "Plain" flag of the second triangle's set. Unused: each view keeps its own
    /// (`scene::polygon_work`).
    pub plain_second: PlainDword,
    /// The view value kept by `race::draw_mirror_view` around its drawing. Unused: each view keeps
    /// its own (`race::view_course::ViewCourse`).
    pub view_value_kept: PlainDword,
    /// Grid cell x of the view, a refresh value. Unused: each view keeps its own
    /// (`race::view_grid`).
    pub grid_x: PlainDword,
    /// Grid cell z of the view, a refresh value. Unused: each view keeps its own
    /// (`race::view_grid`).
    pub grid_z: PlainDword,
    /// 32 x 32 course records. Unused: each view keeps its own (`race::view_grid`);
    /// `race::find_sound_spots` reads past a view's grid on into the fields after this one.
    pub course_grid: [PlainDword; 1024],
    /// How many cells out are drawn. Unused: now a value in
    /// `race::frame_draw::prepare_course_view`.
    pub far_cell_limit: PlainDword,
    /// The view-turned x; never read.
    pub turned_x: PlainFloat,
    /// The view-turned z; never read.
    pub turned_z: PlainFloat,
    /// The depth shift used by `race::frame_draw`.
    pub depth_shift: PlainFloat,
    /// Per-flare x place (`race::frame_draw`'s flare list, indexed by flare number). Unused: each
    /// view keeps its own (`race::lamp_work::LampWork`).
    pub flare_place_x: [PlainDword; 256],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece (a field of `race::owner::Race::course`), with its first values.
impl LevelGrid {
    pub const fn new() -> Self {
        Self {
            depth_shift: PlainFloat::new(3000000.0),
            ..ZeroCells::ZERO
        }
    }
}
