//! The scratch point, work vector and kept frame (IDA 0x5C5F48 .. 0x5C5FB8; phase 7, typed): the tangent scratch, the scratch point's three floats, the work vector and its squares (`math::normalise_vector`), and the locked surface's kept pitch and pixels. The fields are in the image's order. Names are readings unless IDA's.

use crate::cells::ZeroCells;

use crate::session::{PlainDword, PlainFloat};

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct FrameScratch {
    /// +0x0 (0x5C5F48): `dword_5C5F48`: the tangent's doubled number (`scene::fill_engine_tables`).
    pub tangent_scratch: PlainDword,
    /// +0x4 (0x5C5F4C): `g_scratchZ`: the scratch point's z (`math::rotate_scratch_about_y`).
    pub scratch_z: PlainFloat,
    /// +0x8 (0x5C5F50): `g_scratchX`: the scratch point's x (`math::rotate_scratch_about_y`).
    pub scratch_x: PlainFloat,
    /// +0xc (0x5C5F54): `g_scratchY`: the scratch point's y (`math::rotate_point_about_y`).
    pub scratch_y: PlainFloat,
    /// +0x10 (0x5C5F58): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_10: [PlainDword; 2],
    /// +0x18 (0x5C5F60): `g_workVector`: three floats and a fourth (`math::normalise_vector`, `physics::wall_contacts`, `race::norms`); taken in place.
    pub work_vector: [PlainDword; 4],
    /// +0x28 (0x5C5F70): `xmmword_5C5F70`: the four floats squared (`math::normalise_vector`).
    pub work_squares: [PlainDword; 4],
    /// +0x38 (0x5C5F80): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_38: [PlainDword; 6],
    /// +0x50 (0x5C5F98): `g_debugCursorX`: where the debug font's last print ended
    /// (`debug::draw::print`, a print at (0, 0) goes on from here).
    pub debug_cursor_x: PlainDword,
    /// +0x54 (0x5C5F9C): `g_debugCursorY`: its row.
    pub debug_cursor_y: PlainDword,
    /// +0x58 (0x5C5FA0): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_58: [PlainDword; 4],
    /// +0x68 (0x5C5FB0): `dword_5C5FB0`: the locked surface's pitch (`scene::lock_frame`, `scene::unlock_frame`).
    pub frame_pitch: PlainDword,
    /// +0x6c (0x5C5FB4): `dword_5C5FB4`: the locked surface's pixels (`scene::lock_frame`, `scene::unlock_frame`).
    pub frame_pixels: PlainDword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece, with the image's first values.
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
