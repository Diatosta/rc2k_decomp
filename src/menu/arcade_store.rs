//! The arcade game's variables, out of RAL.EXE's image (IDA 0x7587E8 .. 0x758806): the
//! scene switch, the level, stage and retry being played and the levels won. One static.

use crate::session::{PlainBytes, PlainDword};

/// The arcade game's variables (IDA 0x7587E8 .. 0x758806).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Arcade {
    /// `dword_7587E8`: 1 when the front end's scene was let go for a race (`menu::reload_front_end` sets it up again).
    pub scene_let_go: PlainDword,
    /// `g_arcadeLevel`: The arcade level being played.
    pub level: PlainDword,
    /// `g_arcadeStage`: The arcade stage being played.
    pub stage: PlainDword,
    /// `g_arcadeRetry`: The retry count.
    pub retry: PlainDword,
    /// `dword_7587F8`: The places the arcade game ranks (`menu::arcade_steps`).
    pub places: PlainDword,
    /// `byte_758800`: One byte a level: won before (the options file's last six bytes; `screens::locked_notes`).
    pub levels_won: PlainBytes<8>,
}

/// First values the image's (`tests::arcade_game_starts_as_the_games`).
impl Arcade {
    pub const fn new() -> Self {
        Arcade {
            scene_let_go: PlainDword::new(0x0),
            level: PlainDword::new(0x0),
            stage: PlainDword::new(0x1),
            retry: PlainDword::new(0x0),
            places: PlainDword::new(0x64),
            levels_won: PlainBytes::new([0; 8]),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn arcade_game_starts_as_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            &crate::menu::owner::menus().play.arcade,
        );
    }
}
