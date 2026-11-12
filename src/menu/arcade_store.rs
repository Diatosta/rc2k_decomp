//! The arcade game's variables, out of RAL.EXE's image (IDA 0x7587E8 .. 0x758806): the
//! scene switch, the level, stage and retry being played and the levels won. A `#[repr(C)]` struct
//! with the image's layout, one static.

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
    /// Unreferenced bytes (IDA 0x7587FC .. 0x758800).
    pub _gap_7587fc: PlainBytes<4>,
    /// `byte_758800`: One byte a level: won before (the options file's last six bytes; `screens::locked_notes`).
    pub levels_won: PlainBytes<8>,
}

/// The variables as one run of bytes (a read off `levels_won`'s start runs into the dwords
/// before it, as the original's does).
impl crate::place::Run for Arcade {
    fn len(&self) -> usize {
        size_of::<Arcade>()
    }
    fn get(&self, n: usize) -> u8 {
        let dword = |d: &PlainDword| d.get().to_le_bytes()[n % 4];
        match n / 4 {
            0 => dword(&self.scene_let_go),
            1 => dword(&self.level),
            2 => dword(&self.stage),
            3 => dword(&self.retry),
            4 => dword(&self.places),
            5 => self._gap_7587fc.get(n - 20),
            _ => self.levels_won.get(n - 24),
        }
    }
    fn set(&self, n: usize, v: u8) {
        let dword = |d: &PlainDword| {
            let mut bytes = d.get().to_le_bytes();
            bytes[n % 4] = v;
            d.set(u32::from_le_bytes(bytes));
        };
        match n / 4 {
            0 => dword(&self.scene_let_go),
            1 => dword(&self.level),
            2 => dword(&self.stage),
            3 => dword(&self.retry),
            4 => dword(&self.places),
            5 => self._gap_7587fc.set(n - 20, v),
            _ => self.levels_won.set(n - 24, v),
        }
    }
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
            _gap_7587fc: PlainBytes::new([0; 4]),
            levels_won: PlainBytes::new([0; 8]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn arcade_game_starts_as_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            &crate::menu::owner::menus().play.arcade,
        );
        assert_eq!(offset_of!(Arcade, scene_let_go), 0x0);
        assert_eq!(offset_of!(Arcade, level), 0x4);
        assert_eq!(offset_of!(Arcade, stage), 0x8);
        assert_eq!(offset_of!(Arcade, retry), 0xc);
        assert_eq!(offset_of!(Arcade, places), 0x10);
        assert_eq!(offset_of!(Arcade, levels_won), 0x18);
    }
}
