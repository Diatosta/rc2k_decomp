//! The known-kind tables (IDA 0x5C56EC .. 0x5C5CE0; phase 7, typed; the listener's figures and the mark set-up's numbers before them are the marks' settings' tail since 2026-11-02, `race::effects_settings_store`): the per-kind tables of 34 known cars (pictures, object kinds, file and icon numbers, view, wheel, hit and part flags). `#[repr(C)]` with the image's layout, over the race block's bytes. Names are readings unless IDA's.

use crate::session::Dword;

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct KindTables {
    /// +0xc8 (0x5C56EC): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_c8: [Dword; 7],
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

/// IDA 0x5C56EC, the first byte of the piece (after the marks' settings).
pub const START: usize = 0x5C56EC;
/// IDA 0x5C5CE0, the end of the piece.
pub const END: usize = 0x5C5CE0;

const _: () = assert!(END - START == core::mem::size_of::<KindTables>());

/// Where IDA 0x5C5624 is now (its group's static).
#[cfg(test)]
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(core::ptr::from_ref(
        &crate::race::owner::race()
            .replay
            .viewer_and_kinds
            .kind_tables,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn fields_are_at_the_games_offsets() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(offset_of!(KindTables, unused_c8), 0x0);
        assert_eq!(offset_of!(KindTables, known_object_kinds), 0xa4);
        assert_eq!(offset_of!(KindTables, kind_has_views), 0x3d4);
    }
}
