//! The race's variables before the draw state: the race's settings as the race code reads them
//! (stage, direction, car kinds, tyres, run kind, split, settings words; `race::settings_copy`
//! fills them), the checkpoints and the best line's work, the mud texture's
//! state, and the flat quads' corners, colours and vertices. `#[repr(C)]`, laid over the race
//! block bytes.

use crate::memory::cells::ZeroCells;

use crate::memory::session::{PlainDword, PlainFloat};

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct RaceVars {
    /// A scene switch (`race::show_replay_still`; read by `physics::car_lighting`,
    /// `race::skid_laying`).
    pub scene_switch: PlainDword,
    /// Unused; it only keeps the layout.
    pub unused_4: PlainDword,
    /// `drive::pause_until_key`'s wait (0x1E, counted down).
    pub pause_wait: PlainDword,
    /// A scale put back at the race's end (`race::race_end`).
    pub kept_scale: PlainFloat,
    /// The race's tick count (`drive::race_tick`).
    pub race_ticks: PlainDword,
    /// Each car's tyres (the players' `settings.cars[n].tyre`); the car's gravel
    /// texture switch (`scene::load_texture_picture` reads eight).
    pub car_tyres: [PlainDword; 8],
    /// -1 from `scene::list_object_faces`, read by nothing; no longer written.
    pub face_mark: PlainDword,
    /// The face number; now each view's (`race::face_work::FaceWork`; the bad polygon's log
    /// reads the view's); this one not used.
    pub face_number: PlainDword,
    /// Unused; it only keeps the layout.
    pub unused_3c: [PlainDword; 4],
    /// The best line's face total (`race::best_line`).
    pub line_face_total: PlainDword,
    /// The checkpoint count.
    pub checkpoint_count: PlainDword,
    /// The last checkpoint's number.
    pub last_checkpoint: PlainDword,
    /// Unused; it only keeps the layout.
    pub unused_60: [PlainDword; 3],
    /// The best line's face kind.
    pub line_face_kind: PlainDword,
    /// The replay's camera counts, 1 and 2.
    pub replay_camera_counts: [PlainDword; 2],
    /// The surface shares, as the whole numbers `race::surface_shares` leaves (it adds them
    /// as floats on its own).
    pub surface_shares: [PlainDword; 16],
    /// The map's anchor count.
    pub map_anchor_count: PlainDword,
    /// Unused; it only keeps the layout.
    pub unused_120: [PlainDword; 82],
    /// The stage number (`settings.stage`).
    pub stage_number: PlainDword,
    /// The stage's direction (`settings.direction`).
    pub direction: PlainDword,
    /// Each car's kind: [0] the current (`settings.car_kind_current`), [1] the first
    /// (`car_kind_first`), then the others.
    pub race_car_kinds: [PlainDword; 8],
    /// Each car's colours word (`net::set_up_race`; the stored blocks' gearbox kind; the
    /// players' `settings.cars[n].gearing`).
    pub car_colours: [PlainDword; 8],
    /// The textures' mip setting (`scene::set_filters`, `scene::try_allocate_textures`).
    pub mip_setting: PlainDword,
    /// The race's run kind (`settings.run_kind`; 2: the cars by place; splits, views,
    /// hidden kinds).
    pub run_kind: PlainDword,
    /// Unused; it only keeps the layout.
    pub unused_2b8: PlainDword,
    /// `settings.kept`: the screen split / players (`race::frame`, `hud::overlay_pass`, the
    /// gear choice).
    pub split: PlainDword,
    /// The graphics options (`settings.lens_flare`, `reflections`, `car_shadows`,
    /// `car_detail`, `headlights`; the ini keys `[shaunlensflare]`, `[shaunreflections]`,
    /// `[shauncarshadow]`, `[shauncardetail]`, `[shaunheadlights]`): [0] the lens flare (through
    /// `listener_figures[0]`, which gates and fades `effects::draw_lens_flare`), [1] the faces'
    /// shine, [2] car shadows, [3] see-through windows 0 / 1 / 2 (none, the viewed car, all cars),
    /// [4] all cars' lamps light objects.
    pub settings_words: [PlainDword; 5],
    /// The standings summary's block id (`settings.rivals`).
    pub rivals: PlainDword,
    /// Whether no replay is kept (`settings.no_replay`).
    pub no_replay: PlainDword,
    /// Unused; it only keeps the layout.
    pub unused_2dc: [PlainDword; 2],
    /// The race's level (`results.race_level`).
    pub race_level: PlainDword,
    /// The race's stage, less 1 (`results.race_stage`).
    pub race_stage: PlainDword,
    /// The cars' start numbers (`net::set_up_race`).
    pub start_numbers: [PlainDword; 8],
    /// The mud file's path, "files\b\mudfile.0xx" (`race::mud`; the digits start at byte 0x11).
    pub mud_path: crate::memory::session::PlainBytes<20>,
    /// Unused; it only keeps the layout.
    pub unused_320: PlainDword,
    /// [0] turned into [1] (`race::mud_texture_turn`).
    pub mud_turn_a: [PlainDword; 2],
    /// Unused; it only keeps the layout.
    pub unused_32c: PlainDword,
    /// The same.
    pub mud_turn_b: [PlainDword; 2],
    /// The texture's thrown mud units (`race::update_mud_texture`).
    pub mud_throws: PlainDword,
    /// The texture's splash units (`race::update_mud_texture`).
    pub mud_splashes: PlainDword,
    /// Unused; it only keeps the layout.
    pub unused_340: [PlainDword; 2],
    /// The mud's splash colours.
    pub mud_colours: [PlainDword; 256],
    /// The extra throws' running sum (`race::throw_mud_texel`).
    pub mud_extra_sum: PlainDword,
    /// The packed mud state (`race::mud_texture_turn`).
    pub mud_packed: PlainDword,
    /// The splash's column count (`race::splash_mud`).
    pub mud_column: PlainDword,
    /// The splash's row count (`race::splash_mud`).
    pub mud_row: PlainDword,
    /// Unused; it only keeps the layout.
    pub unused_758: [PlainDword; 4],
    /// The mud file's memory; unused since the mud is Rust's (`StageData::mud`).
    pub mud: PlainDword,
    /// The texture locked (`race::update_mud_texture`).
    pub mud_locked: PlainDword,
    /// The first of the mud state's 18 dwords.
    pub mud_frames: PlainDword,
    /// The mud texture worked on.
    pub mud_texture: PlainDword,
    /// Each texture's thrown mud.
    pub mud_thrown_in: [PlainFloat; 4],
    /// Each texture's splashes.
    pub mud_splashes_in: [PlainFloat; 4],
    /// Each texture's thrown count.
    pub mud_thrown_counts: [PlainDword; 4],
    /// Each texture's splash count.
    pub mud_splash_counts: [PlainDword; 4],
    /// Unused; it only keeps the layout.
    pub unused_7b8: [PlainDword; 2],
    /// [0] the drive's, [1] the slip's (`physics::accumulate_mud`).
    pub mud_scale_a: [PlainFloat; 2],
    /// The same, the second.
    pub mud_scale_b: [PlainFloat; 2],
    /// A polygon mark that nothing reads; it is not stored.
    pub polygon_mark: PlainDword,
    /// The fade texture coordinates (`scene::draw_polygon`, `race::skidmarks`).
    pub fade_uv: [PlainDword; 2],
    /// Unused; it only keeps the layout.
    pub unused_7dc: [PlainDword; 2],
    /// A factor of the view's scale (`scene::set_view_window`).
    pub scale_factor: PlainFloat,
    /// How far the object moves (`scene::place_object_points`,
    /// `scene::set_object_material`); now each view's
    /// (`ObjectPlace::move_by`); unused.
    pub move_by: PlainDword,
    /// The move's fraction; now each view's (`ObjectPlace::move_fraction`); unused.
    pub move_fraction: PlainFloat,
    /// A texture mapping (`race::course_faces`); now each view's
    /// (`race::face_work::FaceWork`); unused.
    pub mapping_a: [PlainDword; 2],
    /// Another; now each view's (`race::face_work::FaceWork`); unused.
    pub mapping_b: [PlainDword; 2],
    /// The quad's colour and see-through; now each view's (`race::quad_work::QuadWork`);
    /// unused.
    pub quad_colour: [PlainDword; 2],
    /// The quad's texture coordinates (`race::frame_flush`); now each view's
    /// (`race::quad_work::QuadWork`); unused.
    pub quad_uv: [PlainDword; 8],
    /// [0] left, [1] right, [4] top, [5] bottom; now each view's
    /// (`race::quad_work::QuadWork`); unused.
    pub quad_corners: [PlainDword; 8],
    /// Unused.
    pub unused_848: [PlainDword; 4],
    /// Four vertices of 0x20 (`scene::draw_flat_quad`, `race::course_faces`).
    /// No longer used by the drawing: each view's are `PolygonWork::quad_vertices`.
    pub quad_vertices: [PlainDword; 32],
    /// Unused.
    pub unused_8d8: [PlainDword; 48],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl RaceVars {
    pub const fn new() -> Self {
        Self {
            kept_scale: PlainFloat::new(384.0),
            unused_120: {
                let mut a: [PlainDword; 82] = ZeroCells::ZERO;
                a[22] = PlainDword::new(0xA);
                a[23] = PlainDword::new(0xA);
                a[24] = PlainDword::new(0xA);
                a[25] = PlainDword::new(0xA);
                a[26] = PlainDword::new(0xA);
                a[27] = PlainDword::new(0xA);
                a[32] = PlainDword::new(0xA);
                a[33] = PlainDword::new(0xA);
                a[34] = PlainDword::new(0xA);
                a[35] = PlainDword::new(0xA);
                a[36] = PlainDword::new(0xA);
                a[37] = PlainDword::new(0xA);
                a[42] = PlainDword::new(0xA);
                a[43] = PlainDword::new(0xA);
                a[44] = PlainDword::new(0xA);
                a[45] = PlainDword::new(0xA);
                a[46] = PlainDword::new(0xA);
                a[47] = PlainDword::new(0xA);
                a[52] = PlainDword::new(0xA);
                a[53] = PlainDword::new(0xA);
                a[54] = PlainDword::new(0xA);
                a[55] = PlainDword::new(0xA);
                a[56] = PlainDword::new(0xA);
                a[57] = PlainDword::new(0xA);
                a[62] = PlainDword::new(0xA);
                a[63] = PlainDword::new(0xA);
                a[64] = PlainDword::new(0xA);
                a[65] = PlainDword::new(0xA);
                a[66] = PlainDword::new(0xA);
                a[67] = PlainDword::new(0xA);
                a[72] = PlainDword::new(0xA);
                a[73] = PlainDword::new(0xA);
                a[74] = PlainDword::new(0xA);
                a[75] = PlainDword::new(0xA);
                a[76] = PlainDword::new(0xA);
                a[77] = PlainDword::new(0xA);
                a
            },
            race_car_kinds: [
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
            ],
            settings_words: [
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(0),
                PlainDword::new(0),
            ],
            start_numbers: [
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
            ],
            mud_path: crate::memory::session::PlainBytes::new([
                0x66, 0x69, 0x6C, 0x65, 0x73, 0x5C, 0x62, 0x5C, 0x6D, 0x75, 0x64, 0x66, 0x69, 0x6C,
                0x65, 0x2E, 0x30, 0x78, 0x78, 0,
            ]),
            mud_scale_b: [PlainFloat::new(0.075), PlainFloat::new(0.008)],
            fade_uv: [PlainDword::new(0x3F780000), PlainDword::new(0x3D000000)],
            unused_7dc: [PlainDword::new(0xF80000), PlainDword::new(0x700000)],
            scale_factor: PlainFloat::new(6.1035157e-06),
            move_by: PlainDword::new(0xC),
            ..ZeroCells::ZERO
        }
    }
}
