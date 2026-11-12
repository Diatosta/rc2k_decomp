//! The known-kind tables (IDA 0x5C5708 .. 0x5C5CE0; phase 7, typed; the listener's figures and the mark set-up's numbers before them are the marks' settings' tail since 2026-11-02, `race::effects_settings_store`): the per-kind tables of 34 known cars (pictures, object kinds, file and icon numbers, view, wheel, hit and part flags). `#[repr(C)]` with the image's layout, over the race block's bytes. Names are readings unless IDA's.

use crate::race::first_cells::dwords;
use crate::session::Dword;

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct KindTables {
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
    /// +0x38c (0x5C59B0): `dword_5C59B0`: per car kind, the index of its light record (`race::frame_draw::scene_objects` picks the record by it).
    pub kind_light_records: [Dword; 34],
    /// +0x414 (0x5C5A38): `g_carIconNumbers`: each known kind's icon number (`scene::car_icon_number`).
    pub car_icon_numbers: [Dword; 34],
    /// +0x49c (0x5C5AC0): `dword_5C5AC0`: per kind, non-zero when it has views / keeps the light figure (`drive::next_camera_view`, `physics::update_camera`, `race::race_shell`).
    pub kind_has_views: [Dword; 34],
    /// +0x524 (0x5C5B48): `dword_5C5B48`: per kind, non-zero when the car collects mud (`physics::accumulate_mud` adds the mud amounts only for such kinds; the amounts are `mud_thrown_in` / `mud_splashes_in`, used by `race::update_mud_texture`).
    pub kind_gets_mud: [Dword; 34],
    /// +0x5ac (0x5C5BD0): `dword_5C5BD0`: per kind, non-zero when its hits sound (`physics::sound_hit_parts`).
    pub kind_hit_sounds: [Dword; 34],
    /// +0x634 (0x5C5C58): `dword_5C5C58`: per kind, non-zero when its parts show (`race::frame_draw`).
    pub kind_shows_parts: [Dword; 34],
}

impl KindTables {
    /// The known kinds' tables as the game starts.
    pub const fn new() -> Self {
        KindTables {
            known_kind_pictures: dwords([
                0x3E8, 0x3F0, 0x3F8, 0x400, 0x408, 0x410, 0x4D0, 0x420, 0x428, 0x430, 0x438, 0x440,
                0x448, 0x450, 0x458, 0x460, 0x468, 0x470, 0x478, 0x480, 0x488, 0x490, 0x498, 0x4A0,
                0x418, 0x4A8, 0x4CB, 0x4CA, 0x4C9, 0x4C9, 0x4C3, 0x4C3, 0x4CC, 0x4F0,
            ]),
            known_object_kinds: dwords([
                0x191, 0x193, 0x195, 0x197, 0x199, 0x19B, 0x1C3, 0x19F, 0x1A1, 0x1A3, 0x1A5, 0x1A7,
                0x1A9, 0x1AB, 0x1AD, 0x1AF, 0x1B1, 0x1B3, 0x1B5, 0x1B7, 0x1B9, 0x1BB, 0x1BD, 0x1BF,
                0x19D, 0x1C1, 0x1C5, 0x1C7, 0x1C9, 0x1CB, 0x1CF, 0x1CF, 0x1D1, 0x23A,
            ]),
            kind_10_table: dwords([
                0x192, 0x194, 0x196, 0x198, 0x19A, 0x19C, 0x1C4, 0x1A0, 0x1A2, 0x1A4, 0x1A6, 0x1A8,
                0x1AA, 0x1AC, 0x1AE, 0x1B0, 0x1B2, 0x1B4, 0x1B6, 0x1B8, 0x1BA, 0x1BC, 0x1BE, 0x1C0,
                0x19E, 0x1C2, 0x1C6, 0x1C8, 0x1CA, 0x1CC, 0x1D0, 0x1D0, 0x1D1, 0x23A,
            ]),
            kind_11_table: dwords([
                0x1F5, 0x1F7, 0x1F9, 0x1FB, 0x1FD, 0x1FF, 0x227, 0x203, 0x205, 0x207, 0x209, 0x20B,
                0x20D, 0x20F, 0x211, 0x213, 0x215, 0x217, 0x219, 0x21B, 0x21D, 0x21F, 0x221, 0x223,
                0x201, 0x225, 0x229, 0x22B, 0x22D, 0x22F, 0x233, 0x233, 0x235, 0x23B,
            ]),
            car_file_numbers: dwords([
                0, 1, 2, 3, 4, 5, 0xB, 7, 8, 9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF, 0x10, 0x11, 0x12,
                0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1E, 0x1F,
                0x21,
            ]),
            kind_light_records: dwords([
                0, 1, 2, 3, 4, 5, 0xB, 7, 8, 9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF, 0x10, 0x11, 0x12,
                0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1E, 0x1F,
                0x21,
            ]),
            car_icon_numbers: dwords([
                0, 1, 2, 3, 4, 5, 0x19, 7, 8, 9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF, 0x10, 0x11, 0x12,
                0x13, 0x14, 0x15, 0x16, 0x17, 6, 0x18, 0x1A, 0x1B, 0x1C, 0x1D, 0x1F, 0x1F, 0x20,
                0x21,
            ]),
            kind_has_views: dwords([
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 1,
            ]),
            kind_gets_mud: dwords([
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 1,
            ]),
            kind_hit_sounds: dwords([
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 1,
            ]),
            kind_shows_parts: dwords([
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 1,
            ]),
        }
    }
}
