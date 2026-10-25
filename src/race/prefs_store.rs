//! The stage's prefs buffer and the scene settings in it (IDA 0x608CB4 .. 0x60AD20): `g_stagePrefs`, the stage's `localdat` read in place (0x206C bytes): the texture kinds, slot kinds, the scene's colours and fog settings, the sun and sky, the face tables, lamp and picture lists, the grid reach, object tints, the draw-distance tables, the mud colour and the surface noise figures. `#[repr(C)]` with the image's layout, over the race block's bytes.
//! Names are readings unless IDA's.

use crate::session::Dword;

#[repr(C)]
pub struct StagePrefs {
    /// +0x0 (0x608CB4): the first 0x100 bytes of `g_stagePrefs`, the stage's `localdat` (0x206C bytes read in place; `race::stage_prefs`); not used by name.
    pub prefs_head: [Dword; 64],
    /// +0x100 (0x608DB4): `g_textureKinds` (`race::stage_prefs`, `race::level_init`).
    pub texture_kinds: [Dword; 256],
    /// +0x500 (0x6091B4): `dword_6091B4`: the default kinds of slots 1 .. 0x3F (`race::objects`, `race::course_faces`).
    pub slot_kind_defaults: [Dword; 56],
    /// +0x5e0 (0x609294): `dword_609294` .. `dword_6092B0`: the eight fixed kinds (`race::objects`).
    pub fixed_kinds: [Dword; 8],
    /// +0x600 (0x6092B4): the scene's dark colour; also the towards-dark colour (`scene::vertex_light`, `scene::fill_light_tables`, `scene::set_up_front_end_scene`).
    pub dark_colour: [Dword; 3],
    /// +0x60c (0x6092C0): the scene's lit colour; also the towards-light colour (`scene::vertex_light`, `scene::set_up_front_end_scene`).
    pub lit_colour: [Dword; 3],
    /// +0x618 (0x6092CC): the fog's colour (`race::frame_draw::set_up_fog`; cleared in `scene::set_up_front_end_scene`).
    pub fog_colour: [Dword; 3],
    /// +0x624 (0x6092D8): not named.
    pub unknown_624: [Dword; 3],
    /// +0x630 (0x6092E4): the light's level for the lit objects (`scene::draw_described_object`, `race::car_draw`).
    pub light_level: Dword,
    /// +0x634 (0x6092E8): the light's strength for the lit objects (`scene::draw_described_object`).
    pub light_strength_kept: Dword,
    /// +0x638 (0x6092EC): not named.
    pub unknown_638: [Dword; 5],
    /// +0x64c (0x609300): the scene's size, 0x100 at start (`scene::reload_system_textures`, `scene::set_up_front_end_scene`).
    pub scene_size_a: Dword,
    /// +0x650 (0x609304): the scene's size, 0x100 at start (same).
    pub scene_size_b: Dword,
    /// +0x654 (0x609308): fog setting, picked by the draw distance (`race::frame_draw`).
    pub fog_low: Dword,
    /// +0x658 (0x60930C): fog setting: where it ends (`race::frame_draw`).
    pub fog_end: Dword,
    /// +0x65c (0x609310): fog setting: near (`race::frame_draw`).
    pub fog_near: Dword,
    /// +0x660 (0x609314): not named.
    pub unknown_660: Dword,
    /// +0x664 (0x609318): the sky's height, << 8 (`race::frame_draw::sky_matrix`).
    pub sky_height: Dword,
    /// +0x668 (0x60931C): `dword_60931C`: a lighting switch (a reading; `scene::vertex_light`, `scene::fill_light_tables`).
    pub lit_b: Dword,
    /// +0x66c (0x609320): fog setting, picked by the draw distance (`race::frame_draw`).
    pub fog_high: Dword,
    /// +0x670 (0x609324): the shine handed to `draw_race_scene` (`race::draw_race_scene`).
    pub shine_from: Dword,
    /// +0x674 (0x609328): the car shadow's alpha, >> 1 (a reading; `race::car_shadow`).
    pub shadow_alpha: Dword,
    /// +0x678 (0x60932C): flat-depth switch of the projection (`scene::project_point`, `race::skidmarks`, `race::course_faces`).
    pub flat_depth: Dword,
    /// +0x67c (0x609330): the shown object's flag (`scene::draw_shown_object`).
    pub shown: Dword,
    /// +0x680 (0x609334): the sky's turn (`race::frame_draw`).
    pub sky_turn: Dword,
    /// +0x684 (0x609338): the sun's turn (`race::frame_draw`, `race::car_shadow`, `race::car_draw`).
    pub sun_turn: Dword,
    /// +0x688 (0x60933C): the sun's height (`race::frame_draw`, `race::car_shadow`).
    pub sun_height: Dword,
    /// +0x68c (0x609340): not named.
    pub unknown_68c: Dword,
    /// +0x690 (0x609344): `qword_609344`: cleared by the stage prefs, tested by `race::draw_race_scene`, scaling factor of `physics::surface_noise`.
    pub blit_cleared: [Dword; 2],
    /// +0x698 (0x60934C): which surface noise is kept (`physics::surface_noise`).
    pub noise_pick: Dword,
    /// +0x69c (0x609350): refresh flags (`race::loadlevel`, `race::draw_race_scene`).
    pub refresh_flags_b: Dword,
    /// +0x6a0 (0x609354): refresh flags (`race::loadlevel`, `race::draw_race_scene`).
    pub refresh_flags_a: Dword,
    /// +0x6a4 (0x609358): fog setting: where it starts (`race::frame_draw`).
    pub fog_start: Dword,
    /// +0x6a8 (0x60935C): the lit objects' light strength (`race::car_draw`, `race::frame_draw`).
    pub light_strength: Dword,
    /// +0x6ac (0x609360): fog setting: far (`race::frame_draw`).
    pub fog_far: Dword,
    /// +0x6b0 (0x609364): not named.
    pub unknown_6b0: [Dword; 7],
    /// +0x6cc (0x609380): `g_cdatNumber` (`race::course_data`).
    pub cdat_number: Dword,
    /// +0x6d0 (0x609384): `byte_609384`, 2248 bytes: texture bytes by frame place (`race::course_faces`).
    pub face_places: [Dword; 562],
    /// +0xf98 (0x609C4C): `byte_609C4C`, 2304 bytes: each object's animation speed byte (`race::course_faces`).
    pub face_speeds: [Dword; 576],
    /// +0x1898 (0x60A54C): `dword_60A54C` (`scene::fill_light_tables`).
    pub lamp_table_a: [Dword; 3],
    /// +0x18a4 (0x60A558): `dword_60A558` (`scene::fill_light_tables`).
    pub lamp_table_b: [Dword; 3],
    /// +0x18b0 (0x60A564): `dword_60A564` .. `dword_60A56C` (`scene::fill_light_tables`, `race::stage_prefs`).
    pub car_light_table: [Dword; 3],
    /// +0x18bc (0x60A570): `dword_60A570` and two bytes: the glare colour (`scene::fill_light_tables`, `race::frame_views`).
    pub lamp_table_c: Dword,
    /// +0x18c0 (0x60A574): not named.
    pub unknown_18c0: [Dword; 2],
    /// +0x18c8 (0x60A57C): `dword_60A57C`: picture lists (`scene::texture_levels`, `race::set_all_plates`), extent to the next list.
    pub picture_lists_a: [Dword; 64],
    /// +0x19c8 (0x60A67C): `dword_60A67C`: more picture lists; 0x40 kinds swapped in (`race::stage_prefs`, `race::set_all_plates`).
    pub picture_lists_b: [Dword; 64],
    /// +0x1ac8 (0x60A77C): picture lists, the third piece (`scene::texture_levels`, `race::set_all_plates`), extent to `grid_reach`.
    pub picture_lists_c: [Dword; 64],
    /// +0x1bc8 (0x60A87C): `g_gridReach`: how far the grid reaches (`race::frame_draw`, `scene::set_view_window`, `race::sound_spots`).
    pub grid_reach: Dword,
    /// +0x1bcc (0x60A880): `g_objectTints` (`race::course_faces`).
    pub object_tints: [Dword; 256],
    /// +0x1fcc (0x60AC80): `dword_60AC80`: a draw-distance setting (`scene::set_view_window`, `race::frame_draw`).
    pub nearer: Dword,
    /// +0x1fd0 (0x60AC84): `g_drawDistanceLevel` (`race::frame_draw`).
    pub draw_distance_level: Dword,
    /// +0x1fd4 (0x60AC88): `dword_60AC88`: the grid reach per level (`race::frame_draw`).
    pub reach_by_level: [Dword; 5],
    /// +0x1fe8 (0x60AC9C): table for `fog_end` (`race::frame_draw`).
    pub fog_end_by_level: [Dword; 5],
    /// +0x1ffc (0x60ACB0): table for `fog_start` (`race::frame_draw`).
    pub fog_start_by_level: [Dword; 5],
    /// +0x2010 (0x60ACC4): table for `fog_low` (`race::frame_draw`).
    pub fog_low_by_level: [Dword; 5],
    /// +0x2024 (0x60ACD8): table for `fog_high` (`race::frame_draw`).
    pub fog_high_by_level: [Dword; 5],
    /// +0x2038 (0x60ACEC): table for `nearer` (`race::frame_draw`).
    pub nearer_by_level: [Dword; 5],
    /// +0x204c (0x60AD00): `g_mudColour`: red, green, blue (`race::tint_to_mud_colour`).
    pub mud_colour: [Dword; 3],
    /// +0x2058 (0x60AD0C): the shine's scale (`race::car_draw`).
    pub shine_scale: Dword,
    /// +0x205c (0x60AD10): the rough floor of the surface noise (`physics::surface_noise`).
    pub surface_floor: Dword,
    /// +0x2060 (0x60AD14): not named.
    pub unknown_2060: Dword,
    /// +0x2064 (0x60AD18): the second noise's scale pair (`physics::surface_noise`).
    pub surface_scale: [Dword; 2],
}

/// The piece: the race block's bytes at IDA 0x608CB4 seen as `StagePrefs`. They stay there
/// while code reaches them by RVA from `race::store::asm_base()` and by IDA address
/// (`moved::place`): one copy for all three.
pub struct InRaceBlock;

impl core::ops::Deref for InRaceBlock {
    type Target = StagePrefs;
    fn deref(&self) -> &StagePrefs {
        // SAFETY: the race block lives as long as the program and is aligned to 4 (the
        // test checks); every field is an atomic or a cell, as the block's bytes were shared.
        unsafe { &*(piece_addr() as *const StagePrefs) }
    }
}

pub static STAGE_PREFS: InRaceBlock = InRaceBlock;

/// IDA 0x608CB4, the first byte of the piece.
pub const START: usize = 0x608CB4;
/// IDA 0x60AD20, the end of the piece.
pub const END: usize = 0x60AD20;

const _: () = assert!(END - START == core::mem::size_of::<StagePrefs>());

/// Where IDA 0x608CB4 is now (in the race block).
pub fn piece_addr() -> usize {
    crate::race::store::home(START)
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
        assert_eq!(offset_of!(StagePrefs, prefs_head), 0x608CB4 - START);
        assert_eq!(offset_of!(StagePrefs, fog_colour), 0x6092CC - START);
        assert_eq!(offset_of!(StagePrefs, fog_end), 0x60930C - START);
        assert_eq!(offset_of!(StagePrefs, shadow_alpha), 0x609328 - START);
        assert_eq!(offset_of!(StagePrefs, blit_cleared), 0x609344 - START);
        assert_eq!(offset_of!(StagePrefs, fog_far), 0x609360 - START);
        assert_eq!(offset_of!(StagePrefs, car_light_table), 0x60A564 - START);
        assert_eq!(offset_of!(StagePrefs, object_tints), 0x60A880 - START);
        assert_eq!(offset_of!(StagePrefs, fog_low_by_level), 0x60ACC4 - START);
        assert_eq!(offset_of!(StagePrefs, surface_scale), 0x60AD18 - START);
    }
}
