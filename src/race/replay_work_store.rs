//! The replay's work (IDA 0x7194BC .. 0x719FD8; phase 7, typed): the input tape (16 bytes a tenth for
//! cars 0 and 1), the car header (stage, mode, players, car kinds, start place, seed, game-over
//! flags, co-driver sexes), the sealed header part (0x100 bytes, some fields unaligned), the patch
//! switches and the co-driver's door kept in the header, and the replay cameras' wishes and last
//! draw frames. `#[repr(C)]` with the image's layout, over the race block's bytes. Names are readings unless IDA's.

use crate::race::owner::race;
use crate::session::Dword;

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ReplayWork {
    /// +0x0 (0x7194BC): `dword_7194BC` / `dword_7194C0`: the replay's input tape, 16 bytes a tenth, 8 a car for cars 0 and 1, 100 tenths (`drive::record_inputs`, `drive::play_inputs`); extent to `unk_719AFC`.
    pub replay_tape: [Dword; 400],
    /// +0x640 (0x719AFC): only pointed to by data offsets in the table at 0x5ECBC6 .. (a pointer table nothing ported reads).
    pub unknown_640: [Dword; 6],
    /// +0x658 (0x719B14): `dword_719B14`: the car header's stage setting, header offset 0 (`race::replay`, `race::race_end`, `race::viewer`).
    pub header_stage: Dword,
    /// +0x65c (0x719B18): `dword_719B18`: the header's mode, offset 4; the viewer copies it to the run kind (`race::viewer_load_stage`).
    pub header_mode: Dword,
    /// +0x660 (0x719B1C): `dword_719B1C`: the header's players, offset 8 (`race::race_end`).
    pub header_players: Dword,
    /// +0x664 (0x719B20): `dword_719B20`: the header's cars setting, offset 0xC (`race::race_end`).
    pub header_cars: Dword,
    /// +0x668 (0x719B24): `dword_719B24`: the header's car kind 0 (the first car's, limits 0x16 .. 0x21 checked), offset 0x10 (`race::viewer`, `race::race_end`).
    pub header_kind_0: Dword,
    /// +0x66c (0x719B28): `dword_719B28`: the header's car kind 1, offset 0x14 (`race::viewer_load_stage`).
    pub header_kind_1: Dword,
    /// +0x670 (0x719B2C): `dword_719B2C`: the header's car kind 2, offset 0x18 (`race::race_end`).
    pub header_kind_2: Dword,
    /// +0x674 (0x719B30): `dword_719B30`: the header's car kind 3, offset 0x1C (`race::race_end`).
    pub header_kind_3: Dword,
    /// +0x678 (0x719B34): `dword_719B34`: one player (1) or two, set by `race::record_replay_header`; read by `drive::write_replay_frames`, the viewer (`race::viewer`).
    pub header_one_player: Dword,
    /// +0x67c (0x719B38): `dword_719B38`: the first car's co-driver sex (`race::replay`, `race::viewer_load_stage`).
    pub header_codriver_sex_0: Dword,
    /// +0x680 (0x719B3C): `dword_719B3C`: the second car's co-driver sex (`race::replay`, `race::viewer_load_stage`).
    pub header_codriver_sex_1: Dword,
    /// +0x684 (0x719B40): `dword_719B40`: the header's start place, offset 0x2C (`race::race_end`).
    pub header_start_place: Dword,
    /// +0x688 (0x719B44): `dword_719B44`: the header's arcade cars, offset 0x30 (`race::race_end`).
    pub header_arcade_cars: Dword,
    /// +0x68c (0x719B48): `dword_719B48`: the header's random seed, offset 0x34 (`race::race_end`).
    pub header_seed: Dword,
    /// +0x690 (0x719B4C): `dword_719B4C`: the header's game-over flag of car 0, offset 0x38 (`race::race_end`).
    pub header_game_over_0: Dword,
    /// +0x694 (0x719B50): `dword_719B50`: the header's game-over flag of car 1, offset 0x3C (`race::race_end`).
    pub header_game_over_1: Dword,
    /// +0x698 (0x719B54): `dword_719B54`: the replay's sealed header part, 0x100 bytes: checksum dword, version byte at +4, copied settings (some unaligned, `race::replay::copy_header_settings`), the player name at +0xD (0x20 bytes), sealed in blocks of 8 (`race::seal_replay_header`, `race::replay`).
    pub sealed_header: [Dword; 64],
    /// +0x798 (0x719C54): `dword_719C54`: set to 6 when the header holds the patch switches (`race::replay_patches`, `race::replay`).
    pub patches_in_header: Dword,
    /// +0x79c (0x719C58): `unk_719C58`: the 13 patch switches copied into the header (0x34 bytes, `race::replay_patches`, `race::replay`).
    pub header_patches: [Dword; 13],
    /// +0x7d0 (0x719C8C): `byte_719C8C`: the low byte of the first dword is the first car's door (race) number the viewer reads (`race::viewer_load_stage`); the rest is read by no code.
    pub unknown_7d0: [Dword; 162],
    /// +0xa58 (0x719F14): `dword_719F14`: the wished camera per head byte, indexed by a byte of the frame head (`drive::replay_cameras`).
    pub replay_camera_wishes: [Dword; 47],
    /// +0xb14 (0x719FD0): `dword_719FD0` / `dword_719FD4`: the frame of the last camera draw for cars 0 and 1; cleared at stage load (`drive::replay_cameras`, `race::loadlevel`).
    pub camera_last_frame: [Dword; 2],
}

/// The piece: a field of `CARS_AND_REPLAY` since 2026-10-29 (it left the race
/// block with its group, phase 13).
pub struct InGroup;

impl core::ops::Deref for InGroup {
    type Target = ReplayWork;
    fn deref(&self) -> &ReplayWork {
        &race().replay.cars_and_replay.replay_work
    }
}

pub static REPLAY_WORK: InGroup = InGroup;

/// IDA 0x7194BC, the first byte of the piece.
pub const START: usize = 0x7194BC;
/// IDA 0x719FD8, the end of the piece.
pub const END: usize = 0x719FD8;

const _: () = assert!(END - START == core::mem::size_of::<ReplayWork>());

/// Where IDA 0x7194BC is now (its group's static).
#[cfg(test)]
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(core::ptr::from_ref(
        &race().replay.cars_and_replay.replay_work,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn piece_sits_on_the_race_block() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            piece_addr(),
            crate::framework::game_addr(crate::framework::rva(START))
        );
        assert_eq!(offset_of!(ReplayWork, replay_tape), 0x7194BC - START);
        assert_eq!(offset_of!(ReplayWork, header_kind_1), 0x719B28 - START);
        assert_eq!(offset_of!(ReplayWork, header_start_place), 0x719B40 - START);
        assert_eq!(offset_of!(ReplayWork, patches_in_header), 0x719C54 - START);
    }
}
