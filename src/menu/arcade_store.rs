//! The arcade game's variables, out of RAL.EXE's image (IDA 0x7587E8 .. 0x758806; phase 7, typed): the
//! scene switch, the level, stage and retry being played and the levels won. A `#[repr(C)]` struct
//! with the image's layout, one static; `moved::MOVED` points the range at it. (Split 2026-10-25 from
//! the front end's block `menu::store`.)

use crate::session::{Bytes, Dword};

/// The arcade game's variables (IDA 0x7587E8 .. 0x758806).
#[repr(C)]
pub struct Arcade {
    /// `dword_7587E8`: 1 when the front end's scene was let go for a race (`menu::reload_front_end` sets it up again).
    pub scene_let_go: Dword,
    /// `g_arcadeLevel`: The arcade level being played.
    pub level: Dword,
    /// `g_arcadeStage`: The arcade stage being played.
    pub stage: Dword,
    /// `g_arcadeRetry`: The retry count.
    pub retry: Dword,
    /// `dword_7587F8`: The places the arcade game ranks (`menu::arcade_steps`).
    pub places: Dword,
    /// Unreferenced bytes (IDA 0x7587FC .. 0x758800).
    pub _gap_7587fc: Bytes<4>,
    /// `byte_758800`: One byte a level: won before (the options file's last six bytes; `screens::locked_notes`).
    pub levels_won: Bytes<6>,
}

/// First values the image's (`tests::arcade_game_starts_as_the_games`).
pub static ARCADE_GAME: Arcade = Arcade {
    scene_let_go: Dword::new(0x0),
    level: Dword::new(0x0),
    stage: Dword::new(0x1),
    retry: Dword::new(0x0),
    places: Dword::new(0x64),
    _gap_7587fc: Bytes::new([0; 4]),
    levels_won: Bytes::new([0; 6]),
};

/// IDA 0x7587E8, the first byte of the piece.
pub const START: usize = 0x7587E8;
/// IDA 0x758806, the end of the piece.
pub const END: usize = 0x758806;

/// Where IDA 0x7587E8 is now (`moved::MOVED`'s home).
pub fn arcade_addr() -> usize {
    crate::pointers::host_number(&ARCADE_GAME as *const Arcade)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moved::image_bytes;
    use core::mem::{offset_of, size_of};

    #[test]
    fn arcade_game_starts_as_the_games() {
        assert!(
            END - START <= size_of::<Arcade>()
                && size_of::<Arcade>() <= (END - START).next_multiple_of(4)
        );
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(arcade_addr()) as *const u8,
                END - START,
            )
        };
        assert_eq!(mine, &image_bytes(START, END - START)[..]);
        let at = |field: usize, ida: usize| assert_eq!(field, ida - START, "{ida:#X}");
        at(offset_of!(Arcade, scene_let_go), 0x7587E8);
        at(offset_of!(Arcade, level), 0x7587EC);
        at(offset_of!(Arcade, stage), 0x7587F0);
        at(offset_of!(Arcade, retry), 0x7587F4);
        at(offset_of!(Arcade, places), 0x7587F8);
        at(offset_of!(Arcade, levels_won), 0x758800);
    }
}
