//! The stage's prefs buffer and the scene settings in it (IDA 0x608CB4 .. 0x60AD20): `g_stagePrefs`, the stage's `localdat` read in place (0x206C bytes): the texture kinds, slot kinds, the scene's colours and fog settings, the sun and sky, the face tables, lamp and picture lists, the grid reach, object tints, the draw-distance tables, the mud colour and the surface noise figures. `#[repr(C)]` with the image's layout, over the race block's bytes.
//! Plain cells: a drawing store (`session::DrawShared`), written on the game's thread between
//! frames and read by the drawing parts through their `workers::DrawReads`. Names are readings
//! unless IDA's.

use crate::session::PlainDword;

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct StagePrefs {
    /// +0x0 (0x608CB4): the first 0x100 bytes of `g_stagePrefs`, the stage's `localdat` (0x206C bytes read in place; `race::stage_prefs`); not used by name.
    pub prefs_head: [PlainDword; 64],
    /// +0x100 (0x608DB4): `g_textureKinds` (`race::stage_prefs`, `race::level_init`).
    pub texture_kinds: [PlainDword; 256],
    /// +0x500 (0x6091B4): `dword_6091B4`: the default kinds of slots 1 .. 0x3F (`race::objects`, `race::course_faces`).
    pub slot_kind_defaults: [PlainDword; 56],
    /// +0x5e0 (0x609294): `dword_609294` .. `dword_6092B0`: the eight fixed kinds (`race::objects`).
    pub fixed_kinds: [PlainDword; 8],
    /// +0x600 (0x6092B4): the scene's dark colour; also the towards-dark colour (`scene::vertex_light`, `scene::fill_light_tables`, `scene::set_up_front_end_scene`).
    pub dark_colour: [PlainDword; 3],
    /// +0x60c (0x6092C0): the scene's lit colour; also the towards-light colour (`scene::vertex_light`, `scene::set_up_front_end_scene`).
    pub lit_colour: [PlainDword; 3],
    /// +0x618 (0x6092CC): the fog's colour (`race::frame_draw::set_up_fog`; cleared in `scene::set_up_front_end_scene`).
    pub fog_colour: [PlainDword; 3],
    /// +0x624 (0x6092D8): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_624: [PlainDword; 3],
    /// +0x630 (0x6092E4): the light's level for the lit objects (`scene::draw_described_object`, `race::car_draw`).
    pub light_level: PlainDword,
    /// +0x634 (0x6092E8): the light's strength for the lit objects (`scene::draw_described_object`).
    pub light_strength_kept: PlainDword,
    /// +0x638 (0x6092EC): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_638: [PlainDword; 5],
    /// +0x64c (0x609300): the scene's size, 0x100 at start (`scene::reload_system_textures`, `scene::set_up_front_end_scene`).
    pub scene_size_a: PlainDword,
    /// +0x650 (0x609304): the scene's size, 0x100 at start (same).
    pub scene_size_b: PlainDword,
    /// +0x654 (0x609308): fog setting, picked by the draw distance (`race::frame_draw`).
    pub fog_low: PlainDword,
    /// +0x658 (0x60930C): fog setting: where it ends (`race::frame_draw`).
    pub fog_end: PlainDword,
    /// +0x65c (0x609310): fog setting: near (`race::frame_draw`).
    pub fog_near: PlainDword,
    /// +0x660 (0x609314): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_660: PlainDword,
    /// +0x664 (0x609318): the sky's height, << 8 (`race::frame_draw::sky_matrix`).
    pub sky_height: PlainDword,
    /// +0x668 (0x60931C): `dword_60931C`: a lighting switch (`scene::vertex_light`, `scene::fill_light_tables`; its exact meaning is not shown by those uses).
    pub lit_b: PlainDword,
    /// +0x66c (0x609320): fog setting, picked by the draw distance (`race::frame_draw`).
    pub fog_high: PlainDword,
    /// +0x670 (0x609324): the shine handed to `draw_race_scene` (`race::draw_race_scene`).
    pub shine_from: PlainDword,
    /// +0x674 (0x609328): the car shadow's alpha, >> 1 (`race::car_shadow`).
    pub shadow_alpha: PlainDword,
    /// +0x678 (0x60932C): flat-depth switch of the projection (`scene::project_point`, `race::skidmarks`, `race::course_faces`).
    pub flat_depth: PlainDword,
    /// +0x67c (0x609330): the shown object's flag (`scene::draw_shown_object`).
    pub shown: PlainDword,
    /// +0x680 (0x609334): the sky's turn (`race::frame_draw`).
    pub sky_turn: PlainDword,
    /// +0x684 (0x609338): the sun's turn (`race::frame_draw`, `race::car_shadow`, `race::car_draw`).
    pub sun_turn: PlainDword,
    /// +0x688 (0x60933C): the sun's height (`race::frame_draw`, `race::car_shadow`).
    pub sun_height: PlainDword,
    /// +0x68c (0x609340): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_68c: PlainDword,
    /// +0x690 (0x609344): `qword_609344`: cleared by the stage prefs, tested by `race::draw_race_scene`, scaling factor of `physics::surface_noise`.
    pub blit_cleared: [PlainDword; 2],
    /// +0x698 (0x60934C): which surface noise is kept (`physics::surface_noise`).
    pub noise_pick: PlainDword,
    /// +0x69c (0x609350): refresh flags (`race::loadlevel`, `race::draw_race_scene`).
    pub refresh_flags_b: PlainDword,
    /// +0x6a0 (0x609354): refresh flags (`race::loadlevel`, `race::draw_race_scene`).
    pub refresh_flags_a: PlainDword,
    /// +0x6a4 (0x609358): fog setting: where it starts (`race::frame_draw`).
    pub fog_start: PlainDword,
    /// +0x6a8 (0x60935C): the lit objects' light strength (`race::car_draw`, `race::frame_draw`).
    pub light_strength: PlainDword,
    /// +0x6ac (0x609360): fog setting: far (`race::frame_draw`).
    pub fog_far: PlainDword,
    /// +0x6b0 (0x609364): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_6b0: [PlainDword; 7],
    /// +0x6cc (0x609380): `g_cdatNumber` (`race::course_data`).
    pub cdat_number: PlainDword,
    /// +0x6d0 (0x609384): `byte_609384`, 2248 bytes: texture bytes by frame place (`race::course_faces`).
    pub face_places: [PlainDword; 562],
    /// +0xf98 (0x609C4C): `byte_609C4C`, 2304 bytes: each object's animation speed byte (`race::course_faces`).
    pub face_speeds: [PlainDword; 576],
    /// +0x1898 (0x60A54C): `dword_60A54C` (`scene::fill_light_tables`).
    pub lamp_table_a: [PlainDword; 3],
    /// +0x18a4 (0x60A558): `dword_60A558` (`scene::fill_light_tables`).
    pub lamp_table_b: [PlainDword; 3],
    /// +0x18b0 (0x60A564): `dword_60A564` .. `dword_60A56C` (`scene::fill_light_tables`, `race::stage_prefs`).
    pub car_light_table: [PlainDword; 3],
    /// +0x18bc (0x60A570): `dword_60A570` and two bytes: the glare colour (`scene::fill_light_tables`, `race::frame_views`).
    pub lamp_table_c: PlainDword,
    /// +0x18c0 (0x60A574): `byte_60A574`, `byte_60A578`: the sun glare's two colour bytes (the low byte of each dword; `race::frame_views`, with `lamp_table_c`).
    pub glare_bytes: [PlainDword; 2],
    /// +0x18c8 (0x60A57C): `dword_60A57C`: picture lists (`scene::texture_levels`, `race::set_all_plates`), extent to the next list.
    pub picture_lists_a: [PlainDword; 64],
    /// +0x19c8 (0x60A67C): `dword_60A67C`: more picture lists; 0x40 kinds swapped in (`race::stage_prefs`, `race::set_all_plates`).
    pub picture_lists_b: [PlainDword; 64],
    /// +0x1ac8 (0x60A77C): picture lists, the third piece (`scene::texture_levels`, `race::set_all_plates`), extent to `grid_reach`.
    pub picture_lists_c: [PlainDword; 64],
    /// +0x1bc8 (0x60A87C): `g_gridReach`: how far the grid reaches (`race::frame_draw`, `scene::set_view_window`, `race::sound_spots`).
    pub grid_reach: PlainDword,
    /// +0x1bcc (0x60A880): `g_objectTints` (`race::course_faces`).
    pub object_tints: [PlainDword; 256],
    /// +0x1fcc (0x60AC80): `dword_60AC80`: a draw-distance setting (`scene::set_view_window`, `race::frame_draw`).
    pub nearer: PlainDword,
    /// +0x1fd0 (0x60AC84): `g_drawDistanceLevel` (`race::frame_draw`).
    pub draw_distance_level: PlainDword,
    /// +0x1fd4 (0x60AC88): `dword_60AC88`: the grid reach per level (`race::frame_draw`).
    pub reach_by_level: [PlainDword; 5],
    /// +0x1fe8 (0x60AC9C): table for `fog_end` (`race::frame_draw`).
    pub fog_end_by_level: [PlainDword; 5],
    /// +0x1ffc (0x60ACB0): table for `fog_start` (`race::frame_draw`).
    pub fog_start_by_level: [PlainDword; 5],
    /// +0x2010 (0x60ACC4): table for `fog_low` (`race::frame_draw`).
    pub fog_low_by_level: [PlainDword; 5],
    /// +0x2024 (0x60ACD8): table for `fog_high` (`race::frame_draw`).
    pub fog_high_by_level: [PlainDword; 5],
    /// +0x2038 (0x60ACEC): table for `nearer` (`race::frame_draw`).
    pub nearer_by_level: [PlainDword; 5],
    /// +0x204c (0x60AD00): `g_mudColour`: red, green, blue (`race::tint_to_mud_colour`).
    pub mud_colour: [PlainDword; 3],
    /// +0x2058 (0x60AD0C): the shine's scale (`race::car_draw`).
    pub shine_scale: PlainDword,
    /// +0x205c (0x60AD10): the rough floor of the surface noise (`physics::surface_noise`).
    pub surface_floor: PlainDword,
    /// +0x2060 (0x60AD14): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_2060: PlainDword,
    /// +0x2064 (0x60AD18): the second noise's scale pair (`physics::surface_noise`).
    pub surface_scale: [PlainDword; 2],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece (a field of `race::owner::Race::course`), with the image's first values.
impl StagePrefs {
    pub const fn new() -> Self {
        crate::cells::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START))
    }
}

impl StagePrefs {
    /// The piece as its dwords in order: the stage's `localdat` read in place.
    pub fn dwords(&self) -> &[PlainDword; (END - START) / 4] {
        crate::cells::plain_dwords_in(self, 0)
    }
}

/// IDA 0x608CB4, the first byte of the piece.
pub const START: usize = 0x608CB4;
/// IDA 0x60AD20, the end of the piece.
pub const END: usize = 0x60AD20;

const _: () = assert!(END - START == core::mem::size_of::<StagePrefs>());

/// The piece's host address (the alignment checks).
#[cfg(test)]
pub fn piece_addr() -> usize {
    core::ptr::from_ref(crate::race::owner::RACE.course.stage_prefs.get()).addr()
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            crate::race::owner::RACE.course.stage_prefs.get(),
        );
    }

    #[test]
    fn fields_are_at_the_games_offsets() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(offset_of!(StagePrefs, prefs_head), 0x0);
        assert_eq!(offset_of!(StagePrefs, fog_colour), 0x618);
        assert_eq!(offset_of!(StagePrefs, fog_end), 0x658);
        assert_eq!(offset_of!(StagePrefs, shadow_alpha), 0x674);
        assert_eq!(offset_of!(StagePrefs, blit_cleared), 0x690);
        assert_eq!(offset_of!(StagePrefs, fog_far), 0x6ac);
        assert_eq!(offset_of!(StagePrefs, car_light_table), 0x18b0);
        assert_eq!(offset_of!(StagePrefs, object_tints), 0x1bcc);
        assert_eq!(offset_of!(StagePrefs, fog_low_by_level), 0x2010);
        assert_eq!(offset_of!(StagePrefs, surface_scale), 0x2064);
    }
}
