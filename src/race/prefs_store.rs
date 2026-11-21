//! The stage's prefs buffer and the scene settings in it: `stage_prefs`, the stage's `localdat`
//! read in place (0x206C bytes): the texture kinds, slot kinds, the scene's colours and fog
//! settings, the sun and sky, the face tables, lamp and picture lists, the grid reach, object
//! tints, the draw-distance tables, the mud colour and the surface noise figures. `#[repr(C)]`
//! with a fixed layout. A plain record (`session::DrawArc`), written on the game's thread
//! between frames and read there and by the drawing parts, which share it as it stood when their
//! scope opened (`StagePrefsRef`).

use crate::memory::session::{ArcRead, DrawArc};

#[repr(C)]
#[derive(Clone)]
pub struct StagePrefs {
    /// The first 0x100 bytes of `stage_prefs`, the stage's `localdat` (0x206C bytes read in
    /// place; `race::stage_prefs`); not used by name.
    pub prefs_head: [u32; 64],
    /// The 256 textures' kinds (`race::stage_prefs`, `race::level_init`).
    pub texture_kinds: [u32; 256],
    /// The default kinds of slots 1 .. 0x3F (`race::objects`, `race::course_faces`).
    pub slot_kind_defaults: [u32; 56],
    /// The eight fixed kinds (`race::objects`).
    pub fixed_kinds: [u32; 8],
    /// The scene's dark colour; also the towards-dark colour (`scene::vertex_light`,
    /// `scene::fill_light_tables`, `scene::set_up_front_end_scene`).
    pub dark_colour: [u32; 3],
    /// The scene's lit colour; also the towards-light colour (`scene::vertex_light`,
    /// `scene::set_up_front_end_scene`).
    pub lit_colour: [u32; 3],
    /// The fog's colour (`race::frame_draw::set_up_fog`; cleared in
    /// `scene::set_up_front_end_scene`).
    pub fog_colour: [u32; 3],
    /// Unused; it only keeps the layout.
    pub reserved_624: [u32; 3],
    /// The light's level for the lit objects (`scene::draw_described_object`,
    /// `race::car_draw`).
    pub light_level: u32,
    /// The light's strength for the lit objects (`scene::draw_described_object`).
    pub light_strength_kept: u32,
    /// Unused; it only keeps the layout.
    pub reserved_638: [u32; 5],
    /// The scene's size, 0x100 at start (`scene::reload_system_textures`,
    /// `scene::set_up_front_end_scene`).
    pub scene_size_a: u32,
    /// The scene's size, 0x100 at start (same).
    pub scene_size_b: u32,
    /// Fog setting, picked by the draw distance (`race::frame_draw`).
    pub fog_low: u32,
    /// Fog setting: where it ends (`race::frame_draw`).
    pub fog_end: u32,
    /// Fog setting: near (`race::frame_draw`).
    pub fog_near: u32,
    /// Unused; it only keeps the layout.
    pub reserved_660: u32,
    /// The sky's height, << 8 (`race::frame_draw::sky_matrix`).
    pub sky_height: u32,
    /// A lighting switch (`scene::vertex_light`, `scene::fill_light_tables`; its exact
    /// meaning is not shown by those uses).
    pub lit_b: u32,
    /// Fog setting, picked by the draw distance (`race::frame_draw`).
    pub fog_high: u32,
    /// The shine handed to `draw_race_scene` (`race::draw_race_scene`).
    pub shine_from: u32,
    /// The car shadow's alpha, >> 1 (`race::car_shadow`).
    pub shadow_alpha: u32,
    /// Flat-depth switch of the projection (`scene::project_point`, `race::skidmarks`,
    /// `race::course_faces`).
    pub flat_depth: u32,
    /// The shown object's flag (`scene::draw_shown_object`).
    pub shown: u32,
    /// The sky's turn (`race::frame_draw`).
    pub sky_turn: u32,
    /// The sun's turn (`race::frame_draw`, `race::car_shadow`, `race::car_draw`).
    pub sun_turn: u32,
    /// The sun's height (`race::frame_draw`, `race::car_shadow`).
    pub sun_height: u32,
    /// Unused; it only keeps the layout.
    pub reserved_68c: u32,
    /// The stage's precipitation range (low, high): the first noise map's sample is scaled into it
    /// (`physics::surface_noise`) for the rain and snow amount, and a screen picture for the
    /// windscreen is wanted only while it is not 0 .. 0 (`race::draw_race_scene`). The
    /// `noprecipitation` switch clears it (`race::stage_prefs`).
    pub precipitation_range: [u32; 2],
    /// Which surface noise is kept (`physics::surface_noise`).
    pub noise_pick: u32,
    /// Refresh flags (`race::loadlevel`, `race::draw_race_scene`).
    pub refresh_flags_b: u32,
    /// Refresh flags (`race::loadlevel`, `race::draw_race_scene`).
    pub refresh_flags_a: u32,
    /// Fog setting: where it starts (`race::frame_draw`).
    pub fog_start: u32,
    /// The lit objects' light strength (`race::car_draw`, `race::frame_draw`).
    pub light_strength: u32,
    /// Fog setting: far (`race::frame_draw`).
    pub fog_far: u32,
    /// Unused; it only keeps the layout.
    pub reserved_6b0: [u32; 7],
    /// The course data's number (`race::course_data`).
    pub cdat_number: u32,
    /// 2248 bytes: texture bytes by frame place (`race::course_faces`).
    pub face_places: [u32; 562],
    /// 2304 bytes: each object's animation speed byte (`race::course_faces`).
    pub face_speeds: [u32; 576],
    /// A lighting table (`scene::fill_light_tables`).
    pub lamp_table_a: [u32; 3],
    /// A lighting table (`scene::fill_light_tables`).
    pub lamp_table_b: [u32; 3],
    /// The car's light table (`scene::fill_light_tables`, `race::stage_prefs`).
    pub car_light_table: [u32; 3],
    /// The glare's red (`scene::fill_light_tables`, `race::frame_views`).
    pub lamp_table_c: u32,
    /// The sun glare's two colour bytes (the low byte of each dword; `race::frame_views`, with
    /// `lamp_table_c`).
    pub glare_bytes: [u32; 2],
    /// Picture lists (`scene::texture_levels`, `race::set_all_plates`), extent to the next
    /// list.
    pub picture_lists_a: [u32; 64],
    /// More picture lists; 0x40 kinds swapped in (`race::stage_prefs`,
    /// `race::set_all_plates`).
    pub picture_lists_b: [u32; 64],
    /// Picture lists, the third piece (`scene::texture_levels`, `race::set_all_plates`),
    /// extent to `grid_reach`.
    pub picture_lists_c: [u32; 64],
    /// How far the grid reaches (`race::frame_draw`, `scene::set_view_window`,
    /// `race::sound_spots`).
    pub grid_reach: u32,
    /// The objects' tints (`race::course_faces`).
    pub object_tints: [u32; 256],
    /// A draw-distance setting (`scene::set_view_window`, `race::frame_draw`).
    pub nearer: u32,
    /// The draw distance's level (`race::frame_draw`).
    pub draw_distance_level: u32,
    /// The grid reach per level (`race::frame_draw`).
    pub reach_by_level: [u32; 5],
    /// Table for `fog_end` (`race::frame_draw`).
    pub fog_end_by_level: [u32; 5],
    /// Table for `fog_start` (`race::frame_draw`).
    pub fog_start_by_level: [u32; 5],
    /// Table for `fog_low` (`race::frame_draw`).
    pub fog_low_by_level: [u32; 5],
    /// Table for `fog_high` (`race::frame_draw`).
    pub fog_high_by_level: [u32; 5],
    /// Table for `nearer` (`race::frame_draw`).
    pub nearer_by_level: [u32; 5],
    /// Red, green, blue (`race::tint_to_mud_colour`).
    pub mud_colour: [u32; 3],
    /// The shine's scale (`race::car_draw`).
    pub shine_scale: u32,
    /// The rough floor of the surface noise (`physics::surface_noise`).
    pub surface_floor: u32,
    /// Unused; it only keeps the layout.
    pub reserved_2060: u32,
    /// The second noise's scale pair (`physics::surface_noise`).
    pub surface_scale: [u32; 2],
}

/// The prefs as the stage's `localdat` file lays them out: the fields in order, each dword
/// little-endian (the layout of `rc2k_core::race::stage_files`).
impl StagePrefs {
    /// Every field in order, each as its name's path (`.name`, or `.name[0]` for a table) and
    /// its words.
    fn runs(&self) -> [(&'static str, &[u32]); 62] {
        [
            (".prefs_head[0]", &self.prefs_head),
            (".texture_kinds[0]", &self.texture_kinds),
            (".slot_kind_defaults[0]", &self.slot_kind_defaults),
            (".fixed_kinds[0]", &self.fixed_kinds),
            (".dark_colour[0]", &self.dark_colour),
            (".lit_colour[0]", &self.lit_colour),
            (".fog_colour[0]", &self.fog_colour),
            (".reserved_624[0]", &self.reserved_624),
            (".light_level", std::slice::from_ref(&self.light_level)),
            (
                ".light_strength_kept",
                std::slice::from_ref(&self.light_strength_kept),
            ),
            (".reserved_638[0]", &self.reserved_638),
            (".scene_size_a", std::slice::from_ref(&self.scene_size_a)),
            (".scene_size_b", std::slice::from_ref(&self.scene_size_b)),
            (".fog_low", std::slice::from_ref(&self.fog_low)),
            (".fog_end", std::slice::from_ref(&self.fog_end)),
            (".fog_near", std::slice::from_ref(&self.fog_near)),
            (".reserved_660", std::slice::from_ref(&self.reserved_660)),
            (".sky_height", std::slice::from_ref(&self.sky_height)),
            (".lit_b", std::slice::from_ref(&self.lit_b)),
            (".fog_high", std::slice::from_ref(&self.fog_high)),
            (".shine_from", std::slice::from_ref(&self.shine_from)),
            (".shadow_alpha", std::slice::from_ref(&self.shadow_alpha)),
            (".flat_depth", std::slice::from_ref(&self.flat_depth)),
            (".shown", std::slice::from_ref(&self.shown)),
            (".sky_turn", std::slice::from_ref(&self.sky_turn)),
            (".sun_turn", std::slice::from_ref(&self.sun_turn)),
            (".sun_height", std::slice::from_ref(&self.sun_height)),
            (".reserved_68c", std::slice::from_ref(&self.reserved_68c)),
            (".precipitation_range[0]", &self.precipitation_range),
            (".noise_pick", std::slice::from_ref(&self.noise_pick)),
            (
                ".refresh_flags_b",
                std::slice::from_ref(&self.refresh_flags_b),
            ),
            (
                ".refresh_flags_a",
                std::slice::from_ref(&self.refresh_flags_a),
            ),
            (".fog_start", std::slice::from_ref(&self.fog_start)),
            (
                ".light_strength",
                std::slice::from_ref(&self.light_strength),
            ),
            (".fog_far", std::slice::from_ref(&self.fog_far)),
            (".reserved_6b0[0]", &self.reserved_6b0),
            (".cdat_number", std::slice::from_ref(&self.cdat_number)),
            (".face_places[0]", &self.face_places),
            (".face_speeds[0]", &self.face_speeds),
            (".lamp_table_a[0]", &self.lamp_table_a),
            (".lamp_table_b[0]", &self.lamp_table_b),
            (".car_light_table[0]", &self.car_light_table),
            (".lamp_table_c", std::slice::from_ref(&self.lamp_table_c)),
            (".glare_bytes[0]", &self.glare_bytes),
            (".picture_lists_a[0]", &self.picture_lists_a),
            (".picture_lists_b[0]", &self.picture_lists_b),
            (".picture_lists_c[0]", &self.picture_lists_c),
            (".grid_reach", std::slice::from_ref(&self.grid_reach)),
            (".object_tints[0]", &self.object_tints),
            (".nearer", std::slice::from_ref(&self.nearer)),
            (
                ".draw_distance_level",
                std::slice::from_ref(&self.draw_distance_level),
            ),
            (".reach_by_level[0]", &self.reach_by_level),
            (".fog_end_by_level[0]", &self.fog_end_by_level),
            (".fog_start_by_level[0]", &self.fog_start_by_level),
            (".fog_low_by_level[0]", &self.fog_low_by_level),
            (".fog_high_by_level[0]", &self.fog_high_by_level),
            (".nearer_by_level[0]", &self.nearer_by_level),
            (".mud_colour[0]", &self.mud_colour),
            (".shine_scale", std::slice::from_ref(&self.shine_scale)),
            (".surface_floor", std::slice::from_ref(&self.surface_floor)),
            (".reserved_2060", std::slice::from_ref(&self.reserved_2060)),
            (".surface_scale[0]", &self.surface_scale),
        ]
    }

    /// Every field's words in order, to write.
    fn runs_mut(&mut self) -> [&mut [u32]; 62] {
        [
            &mut self.prefs_head,
            &mut self.texture_kinds,
            &mut self.slot_kind_defaults,
            &mut self.fixed_kinds,
            &mut self.dark_colour,
            &mut self.lit_colour,
            &mut self.fog_colour,
            &mut self.reserved_624,
            std::slice::from_mut(&mut self.light_level),
            std::slice::from_mut(&mut self.light_strength_kept),
            &mut self.reserved_638,
            std::slice::from_mut(&mut self.scene_size_a),
            std::slice::from_mut(&mut self.scene_size_b),
            std::slice::from_mut(&mut self.fog_low),
            std::slice::from_mut(&mut self.fog_end),
            std::slice::from_mut(&mut self.fog_near),
            std::slice::from_mut(&mut self.reserved_660),
            std::slice::from_mut(&mut self.sky_height),
            std::slice::from_mut(&mut self.lit_b),
            std::slice::from_mut(&mut self.fog_high),
            std::slice::from_mut(&mut self.shine_from),
            std::slice::from_mut(&mut self.shadow_alpha),
            std::slice::from_mut(&mut self.flat_depth),
            std::slice::from_mut(&mut self.shown),
            std::slice::from_mut(&mut self.sky_turn),
            std::slice::from_mut(&mut self.sun_turn),
            std::slice::from_mut(&mut self.sun_height),
            std::slice::from_mut(&mut self.reserved_68c),
            &mut self.precipitation_range,
            std::slice::from_mut(&mut self.noise_pick),
            std::slice::from_mut(&mut self.refresh_flags_b),
            std::slice::from_mut(&mut self.refresh_flags_a),
            std::slice::from_mut(&mut self.fog_start),
            std::slice::from_mut(&mut self.light_strength),
            std::slice::from_mut(&mut self.fog_far),
            &mut self.reserved_6b0,
            std::slice::from_mut(&mut self.cdat_number),
            &mut self.face_places,
            &mut self.face_speeds,
            &mut self.lamp_table_a,
            &mut self.lamp_table_b,
            &mut self.car_light_table,
            std::slice::from_mut(&mut self.lamp_table_c),
            &mut self.glare_bytes,
            &mut self.picture_lists_a,
            &mut self.picture_lists_b,
            &mut self.picture_lists_c,
            std::slice::from_mut(&mut self.grid_reach),
            &mut self.object_tints,
            std::slice::from_mut(&mut self.nearer),
            std::slice::from_mut(&mut self.draw_distance_level),
            &mut self.reach_by_level,
            &mut self.fog_end_by_level,
            &mut self.fog_start_by_level,
            &mut self.fog_low_by_level,
            &mut self.fog_high_by_level,
            &mut self.nearer_by_level,
            &mut self.mud_colour,
            std::slice::from_mut(&mut self.shine_scale),
            std::slice::from_mut(&mut self.surface_floor),
            std::slice::from_mut(&mut self.reserved_2060),
            &mut self.surface_scale,
        ]
    }

    /// The whole record in the file's layout.
    pub fn to_le_bytes(&self) -> Vec<u8> {
        self.runs()
            .iter()
            .flat_map(|(_, words)| words.iter())
            .flat_map(|word| word.to_le_bytes())
            .collect()
    }

    /// The whole record set from `bytes` in the file's layout.
    ///
    /// # Errors
    /// When `bytes` is not exactly the record's size; nothing is written then.
    pub fn set_from_le_bytes(&mut self, bytes: &[u8]) -> Result<(), crate::errors::BadInput> {
        if bytes.len() != size_of::<Self>() {
            return Err(crate::errors::BadInput(
                "stage preferences of the wrong size",
            ));
        }
        let mut words = bytes
            .as_chunks::<4>()
            .0
            .iter()
            .map(|b| u32::from_le_bytes(*b));
        for run in self.runs_mut() {
            for word in run {
                *word = words.next().expect("the size was checked");
            }
        }
        Ok(())
    }

    /// The hash of the record by field path and value: one hash for each field's run, added up
    /// (the stage's logged line).
    #[cfg(any(test, feature = "remote"))]
    pub fn hash(&self) -> u64 {
        self.runs().iter().fold(0u64, |sum, (path, words)| {
            let bytes: Vec<u8> = words.iter().flat_map(|w| w.to_le_bytes()).collect();
            sum.wrapping_add(crate::drive::sim_hash::byte_run_hash(path, &bytes))
        })
    }
}

/// The piece (a field of `race::owner::Race::course`), with its first values.
impl StagePrefs {
    pub const fn new() -> Self {
        Self {
            prefs_head: [0; 64],
            texture_kinds: [0; 256],
            slot_kind_defaults: [0; 56],
            fixed_kinds: [0; 8],
            dark_colour: [0; 3],
            lit_colour: [0xFF, 0xFF, 0xFF],
            fog_colour: [0; 3],
            reserved_624: [0, 0, 1],
            light_level: 0,
            light_strength_kept: 0,
            reserved_638: [1, 0, 0, 0, 0],
            scene_size_a: 0x100,
            scene_size_b: 0x100,
            fog_low: 0,
            fog_end: 0,
            fog_near: 0,
            reserved_660: 0,
            sky_height: 0,
            lit_b: 0,
            fog_high: 0,
            shine_from: 0,
            shadow_alpha: 0,
            flat_depth: 0,
            shown: 0,
            sky_turn: 0,
            sun_turn: 0,
            sun_height: 0,
            reserved_68c: 0,
            precipitation_range: [0; 2],
            noise_pick: 0,
            refresh_flags_b: 0,
            refresh_flags_a: 0,
            fog_start: 0,
            light_strength: 0,
            fog_far: 0,
            reserved_6b0: [0; 7],
            cdat_number: 0,
            face_places: [0; 562],
            face_speeds: [0; 576],
            lamp_table_a: [0; 3],
            lamp_table_b: [0; 3],
            car_light_table: [0xFF, 0, 0],
            lamp_table_c: 0xFF,
            glare_bytes: [0xFF, 0xFF],
            picture_lists_a: [0; 64],
            picture_lists_b: [0; 64],
            picture_lists_c: [0; 64],
            grid_reach: 6,
            object_tints: [0; 256],
            nearer: 0,
            draw_distance_level: 0,
            reach_by_level: [0; 5],
            fog_end_by_level: [0; 5],
            fog_start_by_level: [0; 5],
            fog_low_by_level: [0; 5],
            fog_high_by_level: [0; 5],
            nearer_by_level: [0; 5],
            mud_colour: [0x80, 0x40, 0x20],
            shine_scale: 0x200,
            surface_floor: 0,
            reserved_2060: 0,
            surface_scale: [0; 2],
        }
    }
}

/// The prefs as one reader finds them: the game's thread's own (read through the store when
/// used), or a drawing part's share of them.
#[derive(Clone, Copy)]
pub enum StagePrefsRef<'a> {
    Store(&'a DrawArc<StagePrefs>),
    Copy(&'a StagePrefs),
}

impl StagePrefsRef<'_> {
    /// `f` on the prefs as they stand (a short read: nothing in `f` may write the store).
    pub fn read<R>(self, f: impl FnOnce(&StagePrefs) -> R) -> R {
        match self {
            Self::Store(store) => f(&store.borrow()),
            Self::Copy(prefs) => f(prefs),
        }
    }
}

impl<'a> From<&'a ArcRead<StagePrefs>> for StagePrefsRef<'a> {
    fn from(share: &'a ArcRead<StagePrefs>) -> Self {
        Self::Copy(share)
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
            std::mem::offset_of!(StagePrefs, precipitation_range),
            files::PRECIPITATION_RANGE_AT
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

    /// The fields in order are the whole piece, dword for dword, and the file's bytes put them
    /// back.
    #[test]
    fn fields_in_order_are_the_dwords() {
        let mut prefs = StagePrefs::new();
        let mut k = 0u32;
        for run in prefs.runs_mut() {
            for word in run {
                *word = 0x1000 + k;
                k += 1;
            }
        }
        let flat = prefs.to_le_bytes();
        assert_eq!(flat.len(), size_of::<StagePrefs>());
        assert_eq!(k as usize * 4, flat.len());
        for (k, b) in flat.as_chunks::<4>().0.iter().enumerate() {
            assert_eq!(u32::from_le_bytes(*b), 0x1000 + k as u32);
        }
        let mut other = StagePrefs::new();
        other.set_from_le_bytes(&flat).unwrap();
        assert_eq!(other.to_le_bytes(), flat);
        assert_eq!(other.hash(), prefs.hash());
        assert!(other.set_from_le_bytes(&flat[1..]).is_err());
    }

    /// The field order is the record's layout: each field starts where the words before it end.
    #[test]
    fn the_runs_follow_the_layout() {
        let prefs = StagePrefs::new();
        let base = std::ptr::from_ref(&prefs).addr();
        let mut at = 0;
        for (path, words) in prefs.runs() {
            assert_eq!(words.as_ptr().addr() - base, at, "{path}");
            at += 4 * words.len();
        }
        assert_eq!(at, size_of::<StagePrefs>());
    }
}
