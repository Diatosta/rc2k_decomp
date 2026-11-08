//! The picture and level file names the game writes numbers into (IDA 0x60AF0C .. 0x60B0BD;
//! phase 7, typed): the twelve `txxxxx.pcx` names of the texture pictures (a name and a
//! second name for each of six folders), then the course's `editpref`, `cdat00c` and
//! `localdat` paths. `#[repr(C)]` with the image's layout, over the race block's bytes
//! (`LEVEL_PATHS`). Names are readings unless IDA's.

use crate::session::Bytes;

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct LevelPaths {
    /// +0x0 (0x60AF0C): `files\a\txxxxx.pcx` (`scene::load_texture_picture`; the number written in).
    pub picture_a: Bytes<0x13>,
    /// +0x13 (0x60AF1F): `files\m\txxxxx.pcx`.
    pub picture_m: Bytes<0x13>,
    /// +0x26 (0x60AF32): `files\b\txxxxx.pcx`.
    pub picture_b: Bytes<0x13>,
    /// +0x39 (0x60AF45): `files\b\gravel\txxxxx.pcx`.
    pub picture_gravel: Bytes<0x1a>,
    /// +0x53 (0x60AF5F): `files\g\frnt\txxxxx.pcx`.
    pub picture_front: Bytes<0x18>,
    /// +0x6b (0x60AF77): `files\s\extr\txxxxx.pcx`.
    pub picture_extra: Bytes<0x18>,
    /// +0x83 (0x60AF8F): the second name of the same, `files\a\txxxxx.pcx`.
    pub second_a: Bytes<0x13>,
    /// +0x96 (0x60AFA2): second name, `files\m\txxxxx.pcx`.
    pub second_m: Bytes<0x13>,
    /// +0xa9 (0x60AFB5): second name, `files\b\txxxxx.pcx`.
    pub second_b: Bytes<0x13>,
    /// +0xbc (0x60AFC8): second name, `files\b\gravel\txxxxx.pcx`.
    pub second_gravel: Bytes<0x1a>,
    /// +0xd6 (0x60AFE2): second name, `files\g\frnt\txxxxx.pcx`.
    pub second_front: Bytes<0x18>,
    /// +0xee (0x60AFFA): second name, `files\s\extr\txxxxx.pcx`.
    pub second_extra: Bytes<0x18>,
    /// +0x106 (0x60B012): `srclvls\level01\editpref` (`race::set_course_number` writes the number at +0xD).
    pub edit_path: Bytes<0x19>,
    /// +0x11f (0x60B02B): `srclvls\level01\cdat00c` (`race::course_data`, `race::find_present_courses`).
    pub cdat_src: Bytes<0x18>,
    /// +0x137 (0x60B043): `srclvls\levelxx\localdat` (`race::stage_prefs`).
    pub localdat_src: Bytes<0x19>,
    /// +0x150 (0x60B05C): `files\levels\levelxx\localdat`.
    pub localdat_files: Bytes<0x1e>,
    /// +0x16e (0x60B07A): `files\levels\level01\cdat00c.sq`.
    pub cdat_files: Bytes<0x20>,
    /// +0x18e (0x60B09A): `srclvls\level01\cdat00c`, a second copy.
    pub cdat_src_2: Bytes<0x18>,
    /// +0x1a6 (0x60B0B2): `files\s\!!`.
    pub files_s: Bytes<0xb>,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece: a field of `race::owner::Race::course`, formerly a static of its own since 2026-10-29 (it left the race block, phase 13:
/// `race::store::LEFT`), with the image's first values.
impl LevelPaths {
    pub const fn new() -> Self {
        crate::pointers::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START))
    }
}

/// IDA 0x60AF0C, the first byte of the piece.
pub const START: usize = 0x60AF0C;
/// IDA 0x60B0BD, the end of the piece.
pub const END: usize = 0x60B0BD;

const _: () = assert!(END - START == core::mem::size_of::<LevelPaths>());

/// Where IDA 0x60AF0C is now.
#[cfg(test)]
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(core::ptr::from_ref(
        &crate::race::owner::RACE.course.level_paths,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(
            &crate::race::owner::RACE.course.level_paths,
            START,
            END,
        );
    }

    #[test]
    fn piece_has_its_home() {
        assert_eq!(
            crate::moved::place(END - 1),
            (piece_addr() + (END - START) as u32 - 1) as usize
        );
        assert_eq!(offset_of!(LevelPaths, picture_gravel), 0x60AF45 - START);
        assert_eq!(offset_of!(LevelPaths, second_a), 0x60AF8F - START);
        assert_eq!(offset_of!(LevelPaths, edit_path), 0x60B012 - START);
        assert_eq!(offset_of!(LevelPaths, cdat_src), 0x60B02B - START);
        assert_eq!(offset_of!(LevelPaths, localdat_files), 0x60B05C - START);
        assert_eq!(offset_of!(LevelPaths, cdat_files), 0x60B07A - START);
        assert_eq!(offset_of!(LevelPaths, files_s), 0x60B0B2 - START);
    }
}
