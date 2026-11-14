//! The scratch point, work vector and kept frame: the tangent scratch, the scratch point's three
//! floats, the work vector and its squares (`math::normalise_vector`), and the locked surface's
//! kept pitch and pixels. The fields keep a fixed layout.

use crate::cells::ZeroCells;

use crate::session::{PlainDword, PlainFloat};

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct FrameScratch {
    /// The tangent's doubled number (`scene::fill_engine_tables`).
    pub tangent_scratch: PlainDword,
    /// The scratch point's z (`math::rotate_scratch_about_y`).
    pub scratch_z: PlainFloat,
    /// The scratch point's x (`math::rotate_scratch_about_y`).
    pub scratch_x: PlainFloat,
    /// The scratch point's y (`math::rotate_point_about_y`).
    pub scratch_y: PlainFloat,
    /// Unused; it only keeps the layout.
    pub unused_10: [PlainDword; 2],
    /// Three floats and a fourth (`math::normalise_vector`, `physics::wall_contacts`,
    /// `race::norms`); taken in place.
    pub work_vector: [PlainDword; 4],
    /// The four floats squared (`math::normalise_vector`).
    pub work_squares: [PlainDword; 4],
    /// Unused; it only keeps the layout.
    pub unused_38: [PlainDword; 6],
    /// Where the debug font's last print ended
    /// (`debug::draw::print`, a print at (0, 0) goes on from here).
    pub debug_cursor_x: PlainDword,
    /// Its row.
    pub debug_cursor_y: PlainDword,
    /// Unused; it only keeps the layout.
    pub unused_58: [PlainDword; 4],
    /// The locked surface's pitch (`scene::lock_frame`, `scene::unlock_frame`).
    pub frame_pitch: PlainDword,
    /// The locked surface's pixels (`scene::lock_frame`, `scene::unlock_frame`).
    pub frame_pixels: PlainDword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece, with its first values.
impl FrameScratch {
    pub const fn new() -> Self {
        Self {
            unused_58: [
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0x320),
                PlainDword::new(0x258),
            ],
            frame_pitch: PlainDword::new(0x640),
            ..ZeroCells::ZERO
        }
    }
}
