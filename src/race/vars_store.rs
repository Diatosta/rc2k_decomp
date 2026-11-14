//! The race's variables before the draw state: the race's settings as the race code reads them
//! (stage, direction, car kinds, tyres, run kind, split, settings words; `race::settings_copy`
//! fills them), the checkpoints and the best line's work, the loading bar, the mud texture's
//! state, and the flat quads' corners, colours and vertices. `#[repr(C)]`, laid over the race
//! block bytes.

use crate::cells::ZeroCells;

use crate::session::{PlainDword, PlainFloat};

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct RaceVars {
    /// +0x0: a scene switch (`race::show_replay_still`; read by `physics::car_lighting`,
    /// `race::skid_laying`).
    pub scene_switch: PlainDword,
    /// +0x4: the held count (`drive::hold_count_tick`; `drive::pause_until_key` sets 0x19).
    pub hold_count: PlainDword,
    /// +0x8: `drive::pause_until_key`'s wait (0x1E, counted down).
    pub pause_wait: PlainDword,
    /// +0xc: a scale put back at the race's end (`race::race_end`).
    pub kept_scale: PlainFloat,
    /// +0x10: the race's tick count (`drive::race_tick`).
    pub race_ticks: PlainDword,
    /// +0x14: each car's tyres (the players' `settings.cars[n].tyre`); the car's gravel
    /// texture switch (`scene::load_texture_picture` reads eight).
    pub car_tyres: [PlainDword; 8],
    /// +0x34: -1 from `scene::list_object_faces`, read by nothing; no longer written.
    pub face_mark: PlainDword,
    /// +0x38: the face number; now each view's (`race::face_work::FaceWork`; the bad polygon's log
    /// reads the view's); this one not used.
    pub face_number: PlainDword,
    /// +0x3c: unused; it only keeps the layout.
    pub unused_3c: [PlainDword; 4],
    /// +0x4c: the loading bar's span, from and to (`scene::step_loading_bar`; `race::loadlevel`'s
    /// object span).
    pub bar_span: [PlainDword; 2],
    /// +0x54: the best line's face total (`race::best_line`).
    pub line_face_total: PlainDword,
    /// +0x58: the checkpoint count.
    pub checkpoint_count: PlainDword,
    /// +0x5c: the last checkpoint's number.
    pub last_checkpoint: PlainDword,
    /// +0x60: unused; it only keeps the layout.
    pub unused_60: [PlainDword; 3],
    /// +0x6c: the best line's face kind.
    pub line_face_kind: PlainDword,
    /// +0x70: a point's x, z (`race::best_line`).
    pub line_from: [PlainDword; 2],
    /// +0x78: the replay's camera counts, 1 and 2.
    pub replay_camera_counts: [PlainDword; 2],
    /// +0x80: the nearest distance, then the one (`race::best_line`).
    pub line_nearest: [PlainDword; 2],
    /// +0x88: the surface shares, as the whole numbers `race::surface_shares` leaves (it adds them
    /// as floats on its own).
    pub surface_shares: [PlainDword; 16],
    /// +0xc8: the surface weight, a whole number.
    pub surface_weight: PlainDword,
    /// +0xcc: the map's anchor count.
    pub map_anchor_count: PlainDword,
    /// +0xd0: the map: the stretch's two anchors and what is made of them
    /// (`race::best_line`).
    pub map_work: [PlainDword; 18],
    /// +0x118: the loading bar's share loaded, 0 .. 100.
    pub bar_share: PlainDword,
    /// +0x11c: the loading bar's function (0: none).
    pub bar_function: PlainDword,
    /// +0x120: unused; it only keeps the layout.
    pub unused_120: [PlainDword; 82],
    /// +0x268: the stage number (`settings.stage`).
    pub stage_number: PlainDword,
    /// +0x26c: the stage's direction (`settings.direction`).
    pub direction: PlainDword,
    /// +0x270: each car's kind: [0] the current (`settings.car_kind_current`), [1] the first
    /// (`car_kind_first`), then the others.
    pub race_car_kinds: [PlainDword; 8],
    /// +0x290: each car's colours word (`net::set_up_race`; the stored blocks' gearbox kind; the
    /// players' `settings.cars[n].gearing`).
    pub car_colours: [PlainDword; 8],
    /// +0x2b0: the textures' mip setting (`scene::set_filters`, `scene::try_allocate_textures`).
    pub mip_setting: PlainDword,
    /// +0x2b4: the race's run kind (`settings.run_kind`; 2: the cars by place; splits, views,
    /// hidden kinds).
    pub run_kind: PlainDword,
    /// +0x2b8: unused; it only keeps the layout.
    pub unused_2b8: PlainDword,
    /// +0x2bc: `settings.kept`: the screen split / players (`race::frame`, `hud::overlay_pass`, the
    /// gear choice).
    pub split: PlainDword,
    /// +0x2c0: the graphics options (`settings.lens_flare`, `reflections`, `car_shadows`,
    /// `car_detail`, `headlights`; the ini keys `[shaunlensflare]`, `[shaunreflections]`,
    /// `[shauncarshadow]`, `[shauncardetail]`, `[shaunheadlights]`): [0] the lens flare (through
    /// `listener_figures[0]`, which gates and fades `effects::draw_lens_flare`), [1] the faces'
    /// shine, [2] car shadows, [3] see-through windows 0 / 1 / 2 (none, the viewed car, all cars),
    /// [4] all cars' lamps light objects.
    pub settings_words: [PlainDword; 5],
    /// +0x2d4: the standings summary's block id (`settings.rivals`).
    pub rivals: PlainDword,
    /// +0x2d8: whether no replay is kept (`settings.no_replay`).
    pub no_replay: PlainDword,
    /// +0x2dc: unused; it only keeps the layout.
    pub unused_2dc: [PlainDword; 2],
    /// +0x2e4: the race's level (`results.race_level`).
    pub race_level: PlainDword,
    /// +0x2e8: the race's stage, less 1 (`results.race_stage`).
    pub race_stage: PlainDword,
    /// +0x2ec: the cars' start numbers (`net::set_up_race`).
    pub start_numbers: [PlainDword; 8],
    /// +0x30c: the mud file's path, "files\b\mudfile.0xx" (`race::mud`; the digits at +0x11).
    pub mud_path: crate::session::PlainBytes<20>,
    /// +0x320: unused; it only keeps the layout.
    pub unused_320: PlainDword,
    /// +0x324: [0] turned into [1] (`race::mud_texture_turn`).
    pub mud_turn_a: [PlainDword; 2],
    /// +0x32c: unused; it only keeps the layout.
    pub unused_32c: PlainDword,
    /// +0x330: the same.
    pub mud_turn_b: [PlainDword; 2],
    /// +0x338: the texture's thrown mud units (`race::update_mud_texture`).
    pub mud_throws: PlainDword,
    /// +0x33c: the texture's splash units (`race::update_mud_texture`).
    pub mud_splashes: PlainDword,
    /// +0x340: unused; it only keeps the layout.
    pub unused_340: [PlainDword; 2],
    /// +0x348: the mud's splash colours.
    pub mud_colours: [PlainDword; 256],
    /// +0x748: the extra throws' running sum (`race::throw_mud_texel`).
    pub mud_extra_sum: PlainDword,
    /// +0x74c: the packed mud state (`race::mud_texture_turn`).
    pub mud_packed: PlainDword,
    /// +0x750: the splash's column count (`race::splash_mud`).
    pub mud_column: PlainDword,
    /// +0x754: the splash's row count (`race::splash_mud`).
    pub mud_row: PlainDword,
    /// +0x758: unused; it only keeps the layout.
    pub unused_758: [PlainDword; 4],
    /// +0x768: the mud file's memory; unused since the mud is Rust's (`race::mud::MUD`).
    pub mud: PlainDword,
    /// +0x76c: the texture locked (`race::update_mud_texture`).
    pub mud_locked: PlainDword,
    /// +0x770: the first of the mud state's 18 dwords.
    pub mud_frames: PlainDword,
    /// +0x774: the mud texture worked on.
    pub mud_texture: PlainDword,
    /// +0x778: each texture's thrown mud.
    pub mud_thrown_in: [PlainFloat; 4],
    /// +0x788: each texture's splashes.
    pub mud_splashes_in: [PlainFloat; 4],
    /// +0x798: each texture's thrown count.
    pub mud_thrown_counts: [PlainDword; 4],
    /// +0x7a8: each texture's splash count.
    pub mud_splash_counts: [PlainDword; 4],
    /// +0x7b8: unused; it only keeps the layout.
    pub unused_7b8: [PlainDword; 2],
    /// +0x7c0: [0] the drive's, [1] the slip's (`physics::accumulate_mud`).
    pub mud_scale_a: [PlainFloat; 2],
    /// +0x7c8: the same, the second.
    pub mud_scale_b: [PlainFloat; 2],
    /// +0x7d0: a polygon mark that nothing reads; it is not stored.
    pub polygon_mark: PlainDword,
    /// +0x7d4: the fade texture coordinates (`scene::draw_polygon`, `race::skidmarks`).
    pub fade_uv: [PlainDword; 2],
    /// +0x7dc: unused; it only keeps the layout.
    pub unused_7dc: [PlainDword; 2],
    /// +0x7e4: a factor of the view's scale (`scene::set_view_window`).
    pub scale_factor: PlainFloat,
    /// +0x7e8: how far the object moves (`scene::place_object_points`,
    /// `scene::set_object_material`); now each view's
    /// (`ObjectPlace::move_by`); unused.
    pub move_by: PlainDword,
    /// +0x7ec: the move's fraction; now each view's (`ObjectPlace::move_fraction`); unused.
    pub move_fraction: PlainFloat,
    /// +0x7f0: a texture mapping (`race::course_faces`); now each view's
    /// (`race::face_work::FaceWork`); unused.
    pub mapping_a: [PlainDword; 2],
    /// +0x7f8: another; now each view's (`race::face_work::FaceWork`); unused.
    pub mapping_b: [PlainDword; 2],
    /// +0x800: the quad's colour and see-through; now each view's (`race::quad_work::QuadWork`);
    /// unused.
    pub quad_colour: [PlainDword; 2],
    /// +0x808: the quad's texture coordinates (`race::frame_flush`); now each view's
    /// (`race::quad_work::QuadWork`); unused.
    pub quad_uv: [PlainDword; 8],
    /// +0x828: [0] left, [1] right, [4] top, [5] bottom; now each view's
    /// (`race::quad_work::QuadWork`); unused.
    pub quad_corners: [PlainDword; 8],
    /// +0x848: unused.
    pub unused_848: [PlainDword; 4],
    /// +0x858: four vertices of 0x20 (`scene::draw_flat_quad`, `race::course_faces`).
    /// No longer used by the drawing: each view's are `PolygonWork::quad_vertices`.
    pub quad_vertices: [PlainDword; 32],
    /// +0x8d8: unused.
    pub unused_8d8: [PlainDword; 48],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl RaceVars {
    pub const fn new() -> Self {
        Self {
            kept_scale: PlainFloat::new(384.0),
            bar_span: [PlainDword::new(0), PlainDword::new(0x64)],
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
            mud_path: crate::session::PlainBytes::new([
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
