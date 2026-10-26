//! The listener, mark and known-kind tables (IDA 0x5C5624 .. 0x5C5CE0; phase 7, typed): the sound listener's place, points and switches (`drive::set_listener`), the mark set-up's numbers, and the per-kind tables of 34 known cars (pictures, object kinds, file and icon numbers, view, wheel, hit and part flags). `#[repr(C)]` with the image's layout, over the race block's bytes. Names are readings unless IDA's.

use crate::session::{Dword, Float};

#[repr(C)]
pub struct KindTables {
    /// +0x0 (0x5C5624): `flt_5C5624`: a float copied from `dword_70F5C4` (`drive::set_listener`; a reading).
    pub listener_float_a: Float,
    /// +0x4 (0x5C5628): `flt_5C5628`: a float copied from `dword_7102F4` (`drive::set_listener`; a reading).
    pub listener_float_b: Float,
    /// +0x8 (0x5C562C): `dword_5C562C`: the work car's file number (`marks::init_mark_step`).
    pub mark_car_file_number: Dword,
    /// +0xc (0x5C5630): `flt_5C5630` ..: the viewed car's three whole numbers +0x134 .. +0x13C as floats (`drive::set_listener`).
    pub listener_car_place: [Float; 3],
    /// +0x18 (0x5C563C): `dword_5C563C`: 1 in the mark set-up (`marks::init_mark_step`).
    pub mark_set_flag: Dword,
    /// +0x1c (0x5C5640): `dword_5C5640`: 0x50 in the mark set-up (`marks::init_mark_step`).
    pub mark_set_count: Dword,
    /// +0x20 (0x5C5644): not named.
    pub unknown_20: [Dword; 7],
    /// +0x3c (0x5C5660): `flt_5C5660`: `dword_70FA3C` / 2^29 + `flt_637644` (`drive::set_listener`; a reading).
    pub listener_scale: Float,
    /// +0x40 (0x5C5664): not named.
    pub unknown_40: [Dword; 7],
    /// +0x5c (0x5C5680): `dword_5C5680`: the old `dword_70F628`, 0 with `dword_601024` (`drive::set_listener`; a reading).
    pub listener_gear_changed: Dword,
    /// +0x60 (0x5C5684): `dword_5C5684`: `dword_70FA2C` != 0, 0 with `dword_601024` (`drive::set_listener`; a reading).
    pub listener_flag: Dword,
    /// +0x64 (0x5C5688): `dword_5C5688`: 1 after the race sound objects are shared, else 0 (`drive::share_all_cars`).
    pub sound_objects_shown: Dword,
    /// +0x68 (0x5C568C): `dword_5C568C` ..: the first light point turned, x negated, as whole numbers (`drive::set_listener`). Written with the next three groups as 12 dwords.
    pub listener_point_a_negated: [Dword; 3],
    /// +0x74 (0x5C5698): `dword_5C5698` ..: the first light point turned (`drive::set_listener`).
    pub listener_point_a: [Dword; 3],
    /// +0x80 (0x5C56A4): `dword_5C56A4` ..: the second light point turned, x negated (`drive::set_listener`).
    pub listener_point_b_negated: [Dword; 3],
    /// +0x8c (0x5C56B0): `dword_5C56B0` ..: the second light point turned (`drive::set_listener`).
    pub listener_point_b: [Dword; 3],
    /// +0x98 (0x5C56BC): `dword_5C56BC`: a switch by `dword_6415E0`, `dword_70FCEC`, `dword_70FCF0` (`drive::set_listener`; a reading).
    pub listener_switch_0: Dword,
    /// +0x9c (0x5C56C0): `dword_5C56C0`: a switch (`drive::set_listener`; a reading).
    pub listener_switch_2: Dword,
    /// +0xa0 (0x5C56C4): `dword_5C56C4`: a switch (`drive::set_listener`; a reading).
    pub listener_switch_3: Dword,
    /// +0xa4 (0x5C56C8): `dword_5C56C8`: a switch (`drive::set_listener`; a reading).
    pub listener_switch_6: Dword,
    /// +0xa8 (0x5C56CC): `dword_5C56CC`: a switch (`drive::set_listener`; a reading).
    pub listener_switch_7: Dword,
    /// +0xac (0x5C56D0): `dword_5C56D0`: a switch (`drive::set_listener`; a reading).
    pub listener_switch_5: Dword,
    /// +0xb0 (0x5C56D4): not named.
    pub unknown_b0: Dword,
    /// +0xb4 (0x5C56D8): `dword_5C56D8` ..: the first figure and `dword_6415D0`, `dword_6415D4` (`drive::set_listener`).
    pub listener_figures: [Dword; 3],
    /// +0xc0 (0x5C56E4): `dword_5C56E4`: a switch (`drive::set_listener`; a reading).
    pub listener_switch_4: Dword,
    /// +0xc4 (0x5C56E8): `dword_5C56E8`: a switch (`drive::set_listener`; a reading).
    pub listener_switch_1: Dword,
    /// +0xc8 (0x5C56EC): not named.
    pub unknown_c8: [Dword; 7],
    /// +0xe4 (0x5C5708): `dword_5C5708`: a known kind's first picture, 0x22 kinds (`scene::known_kind_picture`).
    pub known_kind_pictures: [Dword; 34],
    /// +0x16c (0x5C5790): `g_knownObjectKinds`: the 0x22 known kinds (`scene::known_kind`, `scene::load_object`, `scene::list_object_face`).
    pub known_object_kinds: [Dword; 34],
    /// +0x1f4 (0x5C5818): `dword_5C5818`: per kind, for `race::car_kind_10` (`race::objects`).
    pub kind_10_table: [Dword; 34],
    /// +0x27c (0x5C58A0): `dword_5C58A0`: per kind, for `race::car_kind_11` (`race::objects`).
    pub kind_11_table: [Dword; 34],
    /// +0x304 (0x5C5928): `g_carFileNumbers`: each known kind's file number (`race::mud`).
    pub car_file_numbers: [Dword; 34],
    /// +0x38c (0x5C59B0): `dword_5C59B0`: per kind, the light record index (`race::frame_draw`; a reading).
    pub kind_light_records: [Dword; 34],
    /// +0x414 (0x5C5A38): `g_carIconNumbers`: each known kind's icon number (`scene::car_icon_number`).
    pub car_icon_numbers: [Dword; 34],
    /// +0x49c (0x5C5AC0): `dword_5C5AC0`: per kind, non-zero when it has views / keeps the light figure (`drive::next_camera_view`, `physics::update_camera`, `race::race_shell`).
    pub kind_has_views: [Dword; 34],
    /// +0x524 (0x5C5B48): `dword_5C5B48`: per kind, non-zero when its wheels turn (`physics::turn_wheel_angles`).
    pub kind_turns_wheels: [Dword; 34],
    /// +0x5ac (0x5C5BD0): `dword_5C5BD0`: per kind, non-zero when its hits sound (`physics::sound_hit_parts`).
    pub kind_hit_sounds: [Dword; 34],
    /// +0x634 (0x5C5C58): `dword_5C5C58`: per kind, non-zero when its parts show (`race::frame_draw`).
    pub kind_shows_parts: [Dword; 34],
}

/// The piece: the race block's bytes at IDA 0x5C5624 seen as `KindTables`. They stay there
/// while code reaches them by RVA from `race::store::asm_base()` and by IDA address
/// (`moved::place`): one copy for all three.
pub struct InRaceBlock;

impl core::ops::Deref for InRaceBlock {
    type Target = KindTables;
    fn deref(&self) -> &KindTables {
        // SAFETY: the race block lives as long as the program and is aligned to 4 (the
        // test checks); every field is an atomic or a cell, as the block's bytes were shared.
        unsafe { &*(crate::pointers::host(piece_addr()) as *const KindTables) }
    }
}

pub static KIND_TABLES: InRaceBlock = InRaceBlock;

/// IDA 0x5C5624, the first byte of the piece.
pub const START: usize = 0x5C5624;
/// IDA 0x5C5CE0, the end of the piece.
pub const END: usize = 0x5C5CE0;

const _: () = assert!(END - START == core::mem::size_of::<KindTables>());

/// Where IDA 0x5C5624 is now (in the race block).
pub fn piece_addr() -> u32 {
    crate::race::store::home(START) as u32
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
        assert_eq!(offset_of!(KindTables, listener_float_a), 0x5C5624 - START);
        assert_eq!(offset_of!(KindTables, listener_scale), 0x5C5660 - START);
        assert_eq!(
            offset_of!(KindTables, listener_point_b_negated),
            0x5C56A4 - START
        );
        assert_eq!(offset_of!(KindTables, listener_switch_7), 0x5C56CC - START);
        assert_eq!(offset_of!(KindTables, known_object_kinds), 0x5C5790 - START);
        assert_eq!(offset_of!(KindTables, kind_has_views), 0x5C5AC0 - START);
    }
}
