//! The stage's prefs buffer and the scene settings in it: `stage_prefs`, the stage's `localdat`
//! read in place (0x206C bytes): the texture kinds, slot kinds, the scene's colours and fog
//! settings, the sun and sky, the face tables, lamp and picture lists, the grid reach, object
//! tints, the draw-distance tables, the mud colour and the surface noise figures. `#[repr(C)]`
//! with a fixed layout. Plain cells: a drawing store
//! (`session::DrawShared`), written on the game's thread between frames and read by the drawing
//! parts through their `workers::DrawReads`.
//!

use crate::cells::ZeroCells;
use crate::session::PlainDword;

#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells, rc2k_derive::ByteView)]
pub struct StagePrefs {
    /// The first 0x100 bytes of `stage_prefs`, the stage's `localdat` (0x206C bytes read in
    /// place; `race::stage_prefs`); not used by name.
    pub prefs_head: [PlainDword; 64],
    /// The 256 textures' kinds (`race::stage_prefs`, `race::level_init`).
    pub texture_kinds: [PlainDword; 256],
    /// The default kinds of slots 1 .. 0x3F (`race::objects`, `race::course_faces`).
    pub slot_kind_defaults: [PlainDword; 56],
    /// The eight fixed kinds (`race::objects`).
    pub fixed_kinds: [PlainDword; 8],
    /// The scene's dark colour; also the towards-dark colour (`scene::vertex_light`,
    /// `scene::fill_light_tables`, `scene::set_up_front_end_scene`).
    pub dark_colour: [PlainDword; 3],
    /// The scene's lit colour; also the towards-light colour (`scene::vertex_light`,
    /// `scene::set_up_front_end_scene`).
    pub lit_colour: [PlainDword; 3],
    /// The fog's colour (`race::frame_draw::set_up_fog`; cleared in
    /// `scene::set_up_front_end_scene`).
    pub fog_colour: [PlainDword; 3],
    /// Unused; it only keeps the layout.
    pub unused_624: [PlainDword; 3],
    /// The light's level for the lit objects (`scene::draw_described_object`,
    /// `race::car_draw`).
    pub light_level: PlainDword,
    /// The light's strength for the lit objects (`scene::draw_described_object`).
    pub light_strength_kept: PlainDword,
    /// Unused; it only keeps the layout.
    pub unused_638: [PlainDword; 5],
    /// The scene's size, 0x100 at start (`scene::reload_system_textures`,
    /// `scene::set_up_front_end_scene`).
    pub scene_size_a: PlainDword,
    /// The scene's size, 0x100 at start (same).
    pub scene_size_b: PlainDword,
    /// Fog setting, picked by the draw distance (`race::frame_draw`).
    pub fog_low: PlainDword,
    /// Fog setting: where it ends (`race::frame_draw`).
    pub fog_end: PlainDword,
    /// Fog setting: near (`race::frame_draw`).
    pub fog_near: PlainDword,
    /// Unused; it only keeps the layout.
    pub unused_660: PlainDword,
    /// The sky's height, << 8 (`race::frame_draw::sky_matrix`).
    pub sky_height: PlainDword,
    /// A lighting switch (`scene::vertex_light`, `scene::fill_light_tables`; its exact
    /// meaning is not shown by those uses).
    pub lit_b: PlainDword,
    /// Fog setting, picked by the draw distance (`race::frame_draw`).
    pub fog_high: PlainDword,
    /// The shine handed to `draw_race_scene` (`race::draw_race_scene`).
    pub shine_from: PlainDword,
    /// The car shadow's alpha, >> 1 (`race::car_shadow`).
    pub shadow_alpha: PlainDword,
    /// Flat-depth switch of the projection (`scene::project_point`, `race::skidmarks`,
    /// `race::course_faces`).
    pub flat_depth: PlainDword,
    /// The shown object's flag (`scene::draw_shown_object`).
    pub shown: PlainDword,
    /// The sky's turn (`race::frame_draw`).
    pub sky_turn: PlainDword,
    /// The sun's turn (`race::frame_draw`, `race::car_shadow`, `race::car_draw`).
    pub sun_turn: PlainDword,
    /// The sun's height (`race::frame_draw`, `race::car_shadow`).
    pub sun_height: PlainDword,
    /// Unused; it only keeps the layout.
    pub unused_68c: PlainDword,
    /// Cleared by the stage prefs, tested by `race::draw_race_scene`, scaling factor of
    /// `physics::surface_noise`.
    pub blit_cleared: [PlainDword; 2],
    /// Which surface noise is kept (`physics::surface_noise`).
    pub noise_pick: PlainDword,
    /// Refresh flags (`race::loadlevel`, `race::draw_race_scene`).
    pub refresh_flags_b: PlainDword,
    /// Refresh flags (`race::loadlevel`, `race::draw_race_scene`).
    pub refresh_flags_a: PlainDword,
    /// Fog setting: where it starts (`race::frame_draw`).
    pub fog_start: PlainDword,
    /// The lit objects' light strength (`race::car_draw`, `race::frame_draw`).
    pub light_strength: PlainDword,
    /// Fog setting: far (`race::frame_draw`).
    pub fog_far: PlainDword,
    /// Unused; it only keeps the layout.
    pub unused_6b0: [PlainDword; 7],
    /// The course data's number (`race::course_data`).
    pub cdat_number: PlainDword,
    /// 2248 bytes: texture bytes by frame place (`race::course_faces`).
    pub face_places: [PlainDword; 562],
    /// 2304 bytes: each object's animation speed byte (`race::course_faces`).
    pub face_speeds: [PlainDword; 576],
    /// A lighting table (`scene::fill_light_tables`).
    pub lamp_table_a: [PlainDword; 3],
    /// A lighting table (`scene::fill_light_tables`).
    pub lamp_table_b: [PlainDword; 3],
    /// The car's light table (`scene::fill_light_tables`, `race::stage_prefs`).
    pub car_light_table: [PlainDword; 3],
    /// The glare's red (`scene::fill_light_tables`, `race::frame_views`).
    pub lamp_table_c: PlainDword,
    /// The sun glare's two colour bytes (the low byte of each dword; `race::frame_views`, with
    /// `lamp_table_c`).
    pub glare_bytes: [PlainDword; 2],
    /// Picture lists (`scene::texture_levels`, `race::set_all_plates`), extent to the next
    /// list.
    pub picture_lists_a: [PlainDword; 64],
    /// More picture lists; 0x40 kinds swapped in (`race::stage_prefs`,
    /// `race::set_all_plates`).
    pub picture_lists_b: [PlainDword; 64],
    /// Picture lists, the third piece (`scene::texture_levels`, `race::set_all_plates`),
    /// extent to `grid_reach`.
    pub picture_lists_c: [PlainDword; 64],
    /// How far the grid reaches (`race::frame_draw`, `scene::set_view_window`,
    /// `race::sound_spots`).
    pub grid_reach: PlainDword,
    /// The objects' tints (`race::course_faces`).
    pub object_tints: [PlainDword; 256],
    /// A draw-distance setting (`scene::set_view_window`, `race::frame_draw`).
    pub nearer: PlainDword,
    /// The draw distance's level (`race::frame_draw`).
    pub draw_distance_level: PlainDword,
    /// The grid reach per level (`race::frame_draw`).
    pub reach_by_level: [PlainDword; 5],
    /// Table for `fog_end` (`race::frame_draw`).
    pub fog_end_by_level: [PlainDword; 5],
    /// Table for `fog_start` (`race::frame_draw`).
    pub fog_start_by_level: [PlainDword; 5],
    /// Table for `fog_low` (`race::frame_draw`).
    pub fog_low_by_level: [PlainDword; 5],
    /// Table for `fog_high` (`race::frame_draw`).
    pub fog_high_by_level: [PlainDword; 5],
    /// Table for `nearer` (`race::frame_draw`).
    pub nearer_by_level: [PlainDword; 5],
    /// Red, green, blue (`race::tint_to_mud_colour`).
    pub mud_colour: [PlainDword; 3],
    /// The shine's scale (`race::car_draw`).
    pub shine_scale: PlainDword,
    /// The rough floor of the surface noise (`physics::surface_noise`).
    pub surface_floor: PlainDword,
    /// Unused; it only keeps the layout.
    pub unused_2060: PlainDword,
    /// The second noise's scale pair (`physics::surface_noise`).
    pub surface_scale: [PlainDword; 2],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece (a field of `race::owner::Race::course`), with its first values.
impl StagePrefs {
    pub const fn new() -> Self {
        Self {
            lit_colour: [
                PlainDword::new(0xFF),
                PlainDword::new(0xFF),
                PlainDword::new(0xFF),
            ],
            unused_624: {
                let mut a: [PlainDword; 3] = ZeroCells::ZERO;
                a[2] = PlainDword::new(1);
                a
            },
            unused_638: {
                let mut a: [PlainDword; 5] = ZeroCells::ZERO;
                a[0] = PlainDword::new(1);
                a
            },
            scene_size_a: PlainDword::new(0x100),
            scene_size_b: PlainDword::new(0x100),
            car_light_table: {
                let mut a: [PlainDword; 3] = ZeroCells::ZERO;
                a[0] = PlainDword::new(0xFF);
                a
            },
            lamp_table_c: PlainDword::new(0xFF),
            glare_bytes: [PlainDword::new(0xFF), PlainDword::new(0xFF)],
            grid_reach: PlainDword::new(6),
            mud_colour: [
                PlainDword::new(0x80),
                PlainDword::new(0x40),
                PlainDword::new(0x20),
            ],
            shine_scale: PlainDword::new(0x200),
            ..ZeroCells::ZERO
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rc2k_core::race::stage_files as files;

    /// The offsets the core parses the file by are the fields'.
    #[test]
    fn the_cores_offsets_are_the_fields() {
        assert_eq!(size_of::<StagePrefs>(), files::PREFS_SIZE);
        assert_eq!(
            std::mem::offset_of!(StagePrefs, texture_kinds),
            files::TEXTURE_KINDS_AT
        );
        assert_eq!(
            std::mem::offset_of!(StagePrefs, blit_cleared),
            files::BLIT_CLEARED_AT
        );
        assert_eq!(
            std::mem::offset_of!(StagePrefs, car_light_table),
            files::CAR_LIGHT_TABLE_AT
        );
        assert_eq!(
            std::mem::offset_of!(StagePrefs, picture_lists_b),
            files::PICTURE_LISTS_B_AT
        );
    }

    /// The fields in order are the whole piece, dword for dword.
    #[test]
    fn fields_in_order_are_the_dwords() {
        let prefs = StagePrefs::new();
        for (k, cell) in
            crate::cells::plain_dwords_in::<_, { size_of::<StagePrefs>() / 4 }>(&prefs, 0)
                .iter()
                .enumerate()
        {
            cell.set(0x1000 + k as u32);
        }
        let flat = crate::cells::flatten(&prefs);
        assert_eq!(flat.len(), size_of::<StagePrefs>());
        for (k, b) in flat.as_chunks::<4>().0.iter().enumerate() {
            assert_eq!(u32::from_le_bytes(*b), 0x1000 + k as u32);
        }
    }
}
