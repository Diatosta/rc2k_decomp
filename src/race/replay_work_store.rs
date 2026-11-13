//! The replay's work (IDA 0x719AFC .. 0x719FD8; phase 7, typed): the car header (stage, mode, players, car kinds, start place, seed, game-over
//! flags, co-driver sexes), the sealed header part (0x100 bytes, some fields unaligned), the patch
//! switches and the co-driver's door kept in the header, and the replay cameras' wishes and last
//! draw frames. The fields are in the image's order. Names are readings unless IDA's.

use crate::cells::ZeroCells;

#[cfg(test)]
use crate::race::owner::race;
use crate::session::PlainDword;

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct ReplayWork {
    /// +0x0 (0x719AFC): only pointed to by data offsets in the table at 0x5ECBC6 .. (a pointer table nothing ported reads).
    pub unknown_640: [PlainDword; 6],
    /// +0x18 (0x719B14): `dword_719B14`: the car header's stage setting, header offset 0 (`race::replay`, `race::race_end`, `race::viewer`).
    pub header_stage: PlainDword,
    /// +0x1c (0x719B18): `dword_719B18`: the header's mode, offset 4; the viewer copies it to the run kind (`race::viewer_load_stage`).
    pub header_mode: PlainDword,
    /// +0x20 (0x719B1C): `dword_719B1C`: the header's players, offset 8 (`race::race_end`).
    pub header_players: PlainDword,
    /// +0x24 (0x719B20): `dword_719B20`: the header's cars setting, offset 0xC (`race::race_end`).
    pub header_cars: PlainDword,
    /// +0x28 (0x719B24): `dword_719B24`: the header's car kind 0 (the first car's, limits 0x16 .. 0x21 checked), offset 0x10 (`race::viewer`, `race::race_end`).
    pub header_kind_0: PlainDword,
    /// +0x2c (0x719B28): `dword_719B28`: the header's car kind 1, offset 0x14 (`race::viewer_load_stage`).
    pub header_kind_1: PlainDword,
    /// +0x30 (0x719B2C): `dword_719B2C`: the header's car kind 2, offset 0x18 (`race::race_end`).
    pub header_kind_2: PlainDword,
    /// +0x34 (0x719B30): `dword_719B30`: the header's car kind 3, offset 0x1C (`race::race_end`).
    pub header_kind_3: PlainDword,
    /// +0x38 (0x719B34): `dword_719B34`: one player (1) or two, set by `race::record_replay_header`; read by `drive::write_replay_frames`, the viewer (`race::viewer`).
    pub header_one_player: PlainDword,
    /// +0x3c (0x719B38): `dword_719B38`: the first car's co-driver sex (`race::replay`, `race::viewer_load_stage`).
    pub header_codriver_sex_0: PlainDword,
    /// +0x40 (0x719B3C): `dword_719B3C`: the second car's co-driver sex (`race::replay`, `race::viewer_load_stage`).
    pub header_codriver_sex_1: PlainDword,
    /// +0x44 (0x719B40): `dword_719B40`: the header's start place, offset 0x2C (`race::race_end`).
    pub header_start_place: PlainDword,
    /// +0x48 (0x719B44): `dword_719B44`: the header's arcade cars, offset 0x30 (`race::race_end`).
    pub header_arcade_cars: PlainDword,
    /// +0x4c (0x719B48): `dword_719B48`: the header's random seed, offset 0x34 (`race::race_end`).
    pub header_seed: PlainDword,
    /// +0x50 (0x719B4C): `dword_719B4C`: the header's game-over flag of car 0, offset 0x38 (`race::race_end`).
    pub header_game_over_0: PlainDword,
    /// +0x54 (0x719B50): `dword_719B50`: the header's game-over flag of car 1, offset 0x3C (`race::race_end`).
    pub header_game_over_1: PlainDword,
    /// +0x58 (0x719B54): `dword_719B54`: the replay's sealed header part, 0x100 bytes: checksum dword, version byte at +4, copied settings (some unaligned, `race::replay::copy_header_settings`), the player name at +0xD (0x20 bytes), sealed in blocks of 8 (`race::seal_replay_header`, `race::replay`).
    pub sealed_header: [PlainDword; 64],
    /// +0x158 (0x719C54): `dword_719C54`: set to 6 when the header holds the patch switches (`race::replay_patches`, `race::replay`).
    pub patches_in_header: PlainDword,
    /// +0x15c (0x719C58): `unk_719C58`: the 13 patch switches copied into the header (0x34 bytes, `race::replay_patches`, `race::replay`).
    pub header_patches: [PlainDword; 13],
    /// +0x190 (0x719C8C): `byte_719C8C`: the low byte of the first dword is the first car's door (race) number the viewer reads (`race::viewer_load_stage`); the rest is read by no code.
    pub unknown_7d0: [PlainDword; 162],
    /// +0x418 (0x719F14): `dword_719F14`: the wished camera per head byte, indexed by a byte of the frame head (`drive::replay_cameras`).
    pub replay_camera_wishes: [PlainDword; 47],
    /// +0x4d4 (0x719FD0): `dword_719FD0` / `dword_719FD4`: the frame of the last camera draw for cars 0 and 1; cleared at stage load (`drive::replay_cameras`, `race::loadlevel`).
    pub camera_last_frame: [PlainDword; 2],
}

impl ReplayWork {
    /// The work with the image's first values.
    pub const fn new() -> Self {
        Self {
            unknown_640: [
                PlainDword::new(0x3C0),
                PlainDword::new(0x1A10),
                PlainDword::new(0x640),
                PlainDword::new(0x400),
                PlainDword::new(0x2810),
                PlainDword::new(0x2810),
            ],
            replay_camera_wishes: [
                PlainDword::new(8),
                PlainDword::new(0xA),
                PlainDword::new(7),
                PlainDword::new(9),
                PlainDword::new(0),
                PlainDword::new(1),
                PlainDword::new(2),
                PlainDword::new(3),
                PlainDword::new(4),
                PlainDword::new(5),
                PlainDword::new(6),
                PlainDword::new(0x2E),
                PlainDword::new(0x2F),
                PlainDword::new(0x30),
                PlainDword::new(0x31),
                PlainDword::new(0x23),
                PlainDword::new(0x24),
                PlainDword::new(0x25),
                PlainDword::new(0x10),
                PlainDword::new(0x11),
                PlainDword::new(0x12),
                PlainDword::new(0x13),
                PlainDword::new(0x14),
                PlainDword::new(0x15),
                PlainDword::new(0x16),
                PlainDword::new(0x17),
                PlainDword::new(0x18),
                PlainDword::new(0x19),
                PlainDword::new(0x1A),
                PlainDword::new(0x1B),
                PlainDword::new(0x1C),
                PlainDword::new(0x1D),
                PlainDword::new(0x1E),
                PlainDword::new(0x1F),
                PlainDword::new(0x20),
                PlainDword::new(0x21),
                PlainDword::new(0x22),
                PlainDword::new(0x32),
                PlainDword::new(0x33),
                PlainDword::new(0x26),
                PlainDword::new(0x27),
                PlainDword::new(0x28),
                PlainDword::new(0x29),
                PlainDword::new(0x2A),
                PlainDword::new(0x2B),
                PlainDword::new(0x2C),
                PlainDword::new(0x2D),
            ],
            ..ZeroCells::ZERO
        }
    }
}

/// The piece's host address (the alignment checks).
#[cfg(test)]
pub fn piece_addr() -> usize {
    core::ptr::from_ref(race().replay.replay_work()).addr()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn piece_is_aligned() {
        assert_eq!(piece_addr() % 4, 0);
    }
}
