//! The race's variables before the draw state (IDA 0x6235E8 .. 0x623F80; phase 7, typed): the
//! race's settings as the race code reads them (stage, direction, car kinds, tyres, run kind,
//! split, settings words; `race::settings_copy` fills them), the checkpoints and the best
//! line's work, the loading bar, the mud texture's state, and the flat quads' corners,
//! colours and vertices. `#[repr(C)]` with the image's layout, over the race block's bytes
//! (`RACE_VARS`), as `CAR`. Names are readings unless IDA's.

#[cfg(test)]
use crate::race::owner::race;
use crate::session::{PlainDword, PlainFloat};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct RaceVars {
    /// +0x0 (0x6235E8): a scene switch (`race::show_replay_still`; read by `physics::car_lighting`, `race::skid_laying`).
    pub scene_switch: PlainDword,
    /// +0x4 (0x6235EC): the held count (`drive::hold_count_tick`; `drive::pause_until_key` sets 0x19).
    pub hold_count: PlainDword,
    /// +0x8 (0x6235F0): `drive::pause_until_key`'s wait (0x1E, counted down).
    pub pause_wait: PlainDword,
    /// +0xc (0x6235F4): put back into `dword_606294` at the race's end (`race::race_end`).
    pub kept_scale: PlainFloat,
    /// +0x10 (0x6235F8): the race's tick count (`drive::race_tick`).
    pub race_ticks: PlainDword,
    /// +0x14 (0x6235FC): each car's tyres (the players' `settings.cars[n].tyre`); the car's gravel
    /// texture switch (`scene::load_texture_picture` reads eight).
    pub car_tyres: [PlainDword; 8],
    /// +0x34 (0x62361C): `g_faceMark`: -1 from `scene::list_object_faces`, read by nothing; no longer written.
    pub face_mark: PlainDword,
    /// +0x38 (0x623620): `g_faceNumber` ("objfaced"); now each view's (`race::face_work::FaceWork`; the bad polygon's log reads the view's); this one not used.
    pub face_number: PlainDword,
    /// +0x3c (0x623624): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_3c: [PlainDword; 4],
    /// +0x4c (0x623634): the loading bar's span, from and to (`scene::step_loading_bar`; `race::loadlevel`'s object span).
    pub bar_span: [PlainDword; 2],
    /// +0x54 (0x62363C): `g_lineFaceTotal` ("numblo", `race::best_line`).
    pub line_face_total: PlainDword,
    /// +0x58 (0x623640): `g_checkpointCount`.
    pub checkpoint_count: PlainDword,
    /// +0x5c (0x623644): `g_lastCheckpoint` ("numsortedm1").
    pub last_checkpoint: PlainDword,
    /// +0x60 (0x623648): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_60: [PlainDword; 3],
    /// +0x6c (0x623654): `g_lineFaceKind`.
    pub line_face_kind: PlainDword,
    /// +0x70 (0x623658): a point's x, z (`race::best_line`).
    pub line_from: [PlainDword; 2],
    /// +0x78 (0x623660): `g_replayCameraCount1`, `2`.
    pub replay_camera_counts: [PlainDword; 2],
    /// +0x80 (0x623668): the nearest distance, then the one (`race::best_line`).
    pub line_nearest: [PlainDword; 2],
    /// +0x88 (0x623670): `g_surfaceShares`.
    pub surface_shares: [PlainFloat; 16],
    /// +0xc8 (0x6236B0): `g_surfaceWeight`, a whole number.
    pub surface_weight: PlainDword,
    /// +0xcc (0x6236B4): `g_mapAnchorCount`.
    pub map_anchor_count: PlainDword,
    /// +0xd0 (0x6236B8): the map: the stretch's two anchors and what is made of them
    /// (`race::best_line`, its offsets 0xB8 .. 0xF8 and one dword more).
    pub map_work: [PlainDword; 18],
    /// +0x118 (0x623700): the loading bar's share loaded, 0 .. 100.
    pub bar_share: PlainDword,
    /// +0x11c (0x623704): the loading bar's function (0: none).
    pub bar_function: PlainDword,
    /// +0x120 (0x623708): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_120: [PlainDword; 82],
    /// +0x268 (0x623850): `g_stageNumber` (`settings.stage`).
    pub stage_number: PlainDword,
    /// +0x26c (0x623854): the stage's direction (`settings.direction`).
    pub direction: PlainDword,
    /// +0x270 (0x623858): `g_raceCarKinds`: [0] the current (`settings.car_kind_current`), [1] the first (`car_kind_first`), [2 ..] `[shauncarnum3]` ...
    pub race_car_kinds: [PlainDword; 8],
    /// +0x290 (0x623878): each car's colours word (`net::set_up_race`; the stored blocks' gearbox kind; the players' `settings.cars[n].gearing`).
    pub car_colours: [PlainDword; 8],
    /// +0x2b0 (0x623898): the textures' mip setting (`scene::set_filters`, `scene::try_allocate_textures`; from `dword_4F50D0`).
    pub mip_setting: PlainDword,
    /// +0x2b4 (0x62389C): the race's run kind (`settings.run_kind`; 2: the cars by place; splits, views, hidden kinds).
    pub run_kind: PlainDword,
    /// +0x2b8 (0x6238A0): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_2b8: PlainDword,
    /// +0x2bc (0x6238A4): `settings.kept`: the screen split / players (`race::frame`, `hud::overlay_pass`, the gear choice).
    pub split: PlainDword,
    /// +0x2c0 (0x6238A8): the graphics options (`settings.lens_flare`, `reflections`, `car_shadows`, `car_detail`, `headlights`; the ini keys `[shaunlensflare]`, `[shaunreflections]`, `[shauncarshadow]`, `[shauncardetail]`, `[shaunheadlights]`): [0] the lens flare (through `listener_figures[0]`, which gates and fades `effects::draw_lens_flare`), [1] the faces' shine, [2] car shadows, [3] see-through windows 0 / 1 / 2 (none, the viewed car, all cars), [4] all cars' lamps light objects.
    pub settings_words: [PlainDword; 5],
    /// +0x2d4 (0x6238BC): `g_rivals` (`settings.rivals`: the standings summary's block id).
    pub rivals: PlainDword,
    /// +0x2d8 (0x6238C0): `settings.no_replay`.
    pub no_replay: PlainDword,
    /// +0x2dc (0x6238C4): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_2dc: [PlainDword; 2],
    /// +0x2e4 (0x6238CC): `results.race_level`.
    pub race_level: PlainDword,
    /// +0x2e8 (0x6238D0): `results.race_stage`, less 1.
    pub race_stage: PlainDword,
    /// +0x2ec (0x6238D4): the cars' start numbers (`net::set_up_race`).
    pub start_numbers: [PlainDword; 8],
    /// +0x30c (0x6238F4): "files\b\mudfile.0xx" (`race::mud`; the digits at +0x11).
    pub mud_path: crate::session::PlainBytes<20>,
    /// +0x320 (0x623908): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_320: PlainDword,
    /// +0x324 (0x62390C): [0] turned into [1] (`race::mud_texture_turn`).
    pub mud_turn_a: [PlainDword; 2],
    /// +0x32c (0x623914): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_32c: PlainDword,
    /// +0x330 (0x623918): the same.
    pub mud_turn_b: [PlainDword; 2],
    /// +0x338 (0x623920): (`race::update_mud_texture`).
    pub mud_throws: PlainDword,
    /// +0x33c (0x623924): (`race::update_mud_texture`).
    pub mud_splashes: PlainDword,
    /// +0x340 (0x623928): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_340: [PlainDword; 2],
    /// +0x348 (0x623930): `g_mudColours`.
    pub mud_colours: [PlainDword; 256],
    /// +0x748 (0x623D30): (`race::throw_mud_texel`).
    pub mud_extra_sum: PlainDword,
    /// +0x74c (0x623D34): (`race::mud_texture_turn`).
    pub mud_packed: PlainDword,
    /// +0x750 (0x623D38): (`race::splash_mud`).
    pub mud_column: PlainDword,
    /// +0x754 (0x623D3C): (`race::splash_mud`).
    pub mud_row: PlainDword,
    /// +0x758 (0x623D40): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_758: [PlainDword; 4],
    /// +0x768 (0x623D50): `g_mud`: the mud file's memory (the game's address; unused
    /// since the mud is Rust's, `race::mud::MUD`).
    pub mud: PlainDword,
    /// +0x76c (0x623D54): the texture locked (`race::update_mud_texture`).
    pub mud_locked: PlainDword,
    /// +0x770 (0x623D58): `g_mudFrames` (the mud state's first of 18 dwords).
    pub mud_frames: PlainDword,
    /// +0x774 (0x623D5C): `g_mudTexture`: the mud texture worked on.
    pub mud_texture: PlainDword,
    /// +0x778 (0x623D60): `g_mudThrows`: each texture's thrown mud.
    pub mud_thrown_in: [PlainFloat; 4],
    /// +0x788 (0x623D70): `g_mudSplashes`: each texture's splashes.
    pub mud_splashes_in: [PlainFloat; 4],
    /// +0x798 (0x623D80): `g_mudThrownCounts`.
    pub mud_thrown_counts: [PlainDword; 4],
    /// +0x7a8 (0x623D90): `g_mudSplashCounts`.
    pub mud_splash_counts: [PlainDword; 4],
    /// +0x7b8 (0x623DA0): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_7b8: [PlainDword; 2],
    /// +0x7c0 (0x623DA8): [0] the drive's, [1] the slip's (`physics::accumulate_mud`).
    pub mud_scale_a: [PlainFloat; 2],
    /// +0x7c8 (0x623DB0): the same, the second.
    pub mud_scale_b: [PlainFloat; 2],
    /// +0x7d0 (0x623DB8): `g_polygonMark`: read only by unreached code (0x44EB04); the port
    /// does not store it.
    pub polygon_mark: PlainDword,
    /// +0x7d4 (0x623DBC): (`scene::draw_polygon`, `race::skidmarks`).
    pub fade_uv: [PlainDword; 2],
    /// +0x7dc (0x623DC4): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_7dc: [PlainDword; 2],
    /// +0x7e4 (0x623DCC): a factor of the view's scale (`scene::set_view_window`).
    pub scale_factor: PlainFloat,
    /// +0x7e8 (0x623DD0): (`scene::place_object_points`, `scene::set_object_material`); now each view's (`ObjectPlace::move_by`); unused.
    pub move_by: PlainDword,
    /// +0x7ec (0x623DD4): the same's fraction; now each view's (`ObjectPlace::move_fraction`); unused.
    pub move_fraction: PlainFloat,
    /// +0x7f0 (0x623DD8): a texture mapping (`race::course_faces`); now each view's (`race::face_work::FaceWork`); unused.
    pub mapping_a: [PlainDword; 2],
    /// +0x7f8 (0x623DE0): another; now each view's (`race::face_work::FaceWork`); unused.
    pub mapping_b: [PlainDword; 2],
    /// +0x800 (0x623DE8): the quad's colour and see-through; now each view's (`race::quad_work::QuadWork`); unused.
    pub quad_colour: [PlainDword; 2],
    /// +0x808 (0x623DF0): the quad's texture coordinates (`race::frame_flush`); now each view's (`race::quad_work::QuadWork`); unused.
    pub quad_uv: [PlainDword; 8],
    /// +0x828 (0x623E10): [0] left, [1] right, [4] top, [5] bottom; now each view's (`race::quad_work::QuadWork`); unused.
    pub quad_corners: [PlainDword; 8],
    /// +0x848 (0x623E30): IDA touches it only in code outside any function (0x44A5F1 .. 0x44A61E, dead); the port never reaches it.
    pub unused_848: [PlainDword; 4],
    /// +0x858 (0x623E40): four vertices of 0x20 (`scene::draw_flat_quad`, `race::course_faces`).
    /// No longer used by the drawing: each view's are `PolygonWork::quad_vertices`.
    pub quad_vertices: [PlainDword; 32],
    /// +0x8d8 (0x623EC0): IDA reads and writes it only in code outside any function (0x44FABB .. 0x44FBC8, dead); the port never reaches it.
    pub unused_8d8: [PlainDword; 48],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl RaceVars {
    /// The piece, with the image's first values.
    pub const fn new() -> Self {
        crate::cells::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START))
    }
}

/// IDA 0x6235E8, the first byte of the piece.
pub const START: usize = 0x6235E8;
/// IDA 0x623F80, the end of the piece.
pub const END: usize = 0x623F80;

const _: () = assert!(END - START == core::mem::size_of::<RaceVars>());

/// The piece's host address (the alignment checks).
#[cfg(test)]
pub fn piece_addr() -> usize {
    core::ptr::from_ref(race().shared.vars.get()).addr()
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            race().shared.vars.get(),
        );
    }

    #[test]
    fn fields_are_at_the_games_offsets() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(offset_of!(RaceVars, scene_switch), 0x0);
        assert_eq!(offset_of!(RaceVars, face_mark), 0x34);
        assert_eq!(offset_of!(RaceVars, line_face_kind), 0x6c);
        assert_eq!(offset_of!(RaceVars, map_anchor_count), 0xcc);
        assert_eq!(offset_of!(RaceVars, race_car_kinds), 0x270);
        assert_eq!(offset_of!(RaceVars, rivals), 0x2d4);
        assert_eq!(offset_of!(RaceVars, mud_turn_a), 0x324);
        assert_eq!(offset_of!(RaceVars, mud_packed), 0x74c);
        assert_eq!(offset_of!(RaceVars, mud_texture), 0x774);
        assert_eq!(offset_of!(RaceVars, mud_scale_b), 0x7c8);
        assert_eq!(offset_of!(RaceVars, mapping_a), 0x7f0);
    }
}
