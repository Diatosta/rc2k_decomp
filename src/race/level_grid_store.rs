//! The race's grid and scene work, 0x63DC5C .. 0x63F58C: four 256-byte face tables that `object_bytes` fills,
//! the 16 lamps, the course grid's sizes and view cell, the 32 x 32 course grid (`g_courseGrid`), the far cell
//! limit, the turned view position and depth shift, and the flares' place-x table.
//! Used by `race::frame_draw`, `scene::add_point_lights`, `scene::draw_polygon`, `race::course_faces`, `hud::overlay_pass`.
//! Names are readings unless IDA's.

use crate::session::{Bytes, Dword, Float};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct LevelGrid {
    /// +0x0 (0x63DC5C): the first of four 256-byte tables `object_bytes` fills (`race::frame_draw` `FACE_TABLES`); used in place (`byte_63DC5C`).
    pub face_table_a: Bytes<256>,
    /// +0x100 (0x63DD5C): the second face table (`byte_63DD5C`).
    pub face_table_b: Bytes<256>,
    /// +0x200 (0x63DE5C): the third face table (`byte_63DE5C`).
    pub face_table_c: Bytes<256>,
    /// +0x300 (0x63DF5C): the fourth face table (`byte_63DF5C`).
    pub face_table_d: Bytes<256>,
    /// +0x400 (0x63E05C): `g_lamps`: 16 lamps of 0x10 bytes (`race::frame_draw`, `scene::add_point_lights`); IDA splits `dword_63E060` / `dword_63E064` out of it.
    pub lamps: [Dword; 64],
    /// +0x500 (0x63E15C): the course grid's half size (`race::frame_draw`, `dword_63E15C`) (a reading).
    pub grid_half: Dword,
    /// +0x504 (0x63E160): the course grid's span (`race::frame_draw`, `dword_63E160`) (a reading).
    pub grid_span: Dword,
    /// +0x508 (0x63E164): the polygon smoothing switch (`scene::draw_polygon`, `dword_63E164`) (a reading).
    pub smooth: Dword,
    /// +0x50c (0x63E168): "plain" flag of the first triangle's texture set (`race::course_faces` `FIRST_SET`, `hud::overlay_pass`, `scene::draw_polygon`).
    pub plain_first: Dword,
    /// +0x510 (0x63E16C): "plain" flag of the second triangle's set (`race::course_faces` `SECOND_SET`, `hud::overlay_pass`, `scene::draw_polygon`).
    pub plain_second: Dword,
    /// +0x514 (0x63E170): the view value kept by `race::draw_mirror_view` (pushed and popped around it; `dword_63E170`).
    pub view_value_kept: Dword,
    /// +0x518 (0x63E174): grid cell x of the view, a refresh value (`race::level_init`, `race::frame_draw`).
    pub grid_x: Dword,
    /// +0x51c (0x63E178): grid cell z of the view, a refresh value (`race::level_init`, `race::frame_draw`).
    pub grid_z: Dword,
    /// +0x520 (0x63E17C): `g_courseGrid`: 32 x 32 course records, used in place (`race::frame_draw`, `race::sound_spots`).
    pub course_grid: [Dword; 1024],
    /// +0x1520 (0x63F17C): `g_farCellLimit`: how many cells out are drawn; now a value in `race::frame_draw::prepare_course_view`; unused.
    pub far_cell_limit: Dword,
    /// +0x1524 (0x63F180): the view-turned x (`flt_63F180`), never read; unused.
    pub turned_x: Float,
    /// +0x1528 (0x63F184): the view-turned z (`flt_63F184`), never read; unused.
    pub turned_z: Float,
    /// +0x152c (0x63F188): the depth shift (`race::frame_draw`, `flt_63F188`).
    pub depth_shift: Float,
    /// +0x1530 (0x63F18C): `dword_63F18C`: per-flare x place (`race::frame_draw` flare list, indexed by flare number; 0x400 bytes, a table of its own).
    pub flare_place_x: [Dword; 256],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece: a field of `race::owner::Race::course`, formerly a static of its own since 2026-10-29 (it left the race block, phase 13:
/// `race::store::LEFT`), with the image's first values.
impl LevelGrid {
    pub const fn new() -> Self {
        crate::pointers::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START))
    }
}

/// IDA 0x63DC5C, the first byte of the piece.
pub const START: usize = 0x63DC5C;
/// IDA 0x63F58C, the end of the piece.
pub const END: usize = 0x63F58C;

const _: () = assert!(END - START == core::mem::size_of::<LevelGrid>());

/// Where IDA 0x63DC5C is now (its own static).
#[cfg(test)]
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(core::ptr::from_ref(
        crate::race::owner::RACE.course.level_grid.first(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(
            crate::race::owner::RACE.course.level_grid.first(),
            START,
            END,
        );
    }

    #[test]
    fn piece_has_its_home() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            piece_addr(),
            crate::framework::game_addr(crate::framework::rva(START))
        );
        assert_eq!(offset_of!(LevelGrid, face_table_a), 0x63DC5C - START);
        assert_eq!(offset_of!(LevelGrid, grid_span), 0x63E160 - START);
        assert_eq!(offset_of!(LevelGrid, grid_z), 0x63E178 - START);
        assert_eq!(offset_of!(LevelGrid, flare_place_x), 0x63F18C - START);
    }
}
