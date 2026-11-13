//! The frame's draw work (IDA 0x64158C .. 0x641AA4; phase 7, typed): the counts of a frame
//! (flares, lamps, cars drawn and culled), the sun's place and size, the mirror, the views'
//! layouts and scales, the lead car's camera kept, each car's listed sound, the car parts'
//! matrices and decal places, and the car shadow's work. The fields are in the image's order. Names are readings unless IDA's.

use crate::cells::ZeroCells;
use crate::session::{PlainDword, PlainFlag, PlainFloat};

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct FrameWork {
    /// +0x0 (0x64158C): `g_flareCount`: the flares this frame; now each view's (`race::lamp_work::LampWork`); unused.
    pub flare_count: PlainDword,
    /// +0x4 (0x641590): `g_lampEntryCount`; read by nothing (no longer written); unused.
    pub lamp_entry_count: PlainDword,
    /// +0x8 (0x641594): `g_carsDrawn`; now each view's (`ctx.view.cars.cars_drawn`); unused.
    pub cars_drawn: PlainDword,
    /// +0xc (0x641598): `g_carsCulled`; read by nothing, no longer written; unused.
    pub cars_culled: PlainDword,
    /// +0x10 (0x64159C): (`race::car_draw`); now each view's (`ctx.view.cars.car_kept`); unused.
    pub car_kept: [PlainDword; 2],
    /// +0x18 (0x6415A4): the car's depth (0x50000 at first, `race::car_draw`); now each view's (`ctx.view.cars.car_depth`); unused.
    pub car_depth: PlainDword,
    /// +0x1c (0x6415A8): (`race::frame_draw`).
    pub sun_depth: PlainFloat,
    /// +0x20 (0x6415AC): the sun's direction (`race::place_sun`), read by nothing; no longer written.
    pub sun_floats: [PlainFloat; 3],
    /// +0x2c (0x6415B8): the sun's turned point (`race::place_sun`), read by nothing; no longer written.
    pub sun_points: [PlainDword; 3],
    /// +0x38 (0x6415C4): `g_sunOnScreen` (`scene::draw_polygon`'s hit test); now each view's (`ViewSun`); unused.
    pub sun_on_screen: PlainDword,
    /// +0x3c (0x6415C8): the sun's point on the screen, x and y (`scene::draw_polygon`); now each view's (`ViewSun::hit`); unused.
    pub sun_hit: [PlainDword; 2],
    /// +0x44 (0x6415D0): its x and y as floats' bits (`drive::set_listener` reads them); now each view's (`ViewSun::place`); unused.
    pub sun_place: [PlainDword; 2],
    /// +0x4c (0x6415D8): the view's value (`race::frame_draw`, `drive::set_listener`); now each view's (`ViewSun::view_out`); unused.
    pub view_out: PlainDword,
    /// +0x50 (0x6415DC): `g_sunSize`; now each view's (`ViewSun::size`); unused.
    pub sun_size: PlainDword,
    /// +0x54 (0x6415E0): the mirror's picture wanted (`d3d::copy_screen_to_texture`, `race::draw_race_scene`).
    pub mirror_wanted: PlainFlag,
    /// +0x58 (0x6415E4): `g_viewLight`; now each view's (`scene::object_light`); unused.
    pub view_light: PlainDword,
    /// +0x5c (0x6415E8): (`race::frame_views`).
    pub mirror_switch: PlainDword,
    /// +0x60 (0x6415EC): (`race::frame_views`).
    pub view_kept: [PlainDword; 2],
    /// +0x68 (0x6415F4): `g_viewTexturesForgotten`: bits.
    pub view_textures_forgotten: PlainDword,
    /// +0x6c (0x6415F8): (`scene::textures`, `race::draw_race_scene`).
    pub kind_textures_done: PlainDword,
    /// +0x70 (0x6415FC): `g_mirrorPlaces` (and IDA's `dword_641600`): 12 entries of two dwords,
    /// by `race::mirror_entry`.
    pub mirror_places: [PlainDword; 24],
    /// +0xd0 (0x64165C): the lead car's camera kept (`drive::lead_car_values`); [0] also `effects::stage_record`'s number, [7] `effects::draw_plane`'s switch and `effects::pong`'s shown.
    pub lead_kept: [PlainDword; 8],
    /// +0xf0 (0x64167C): `g_faceHideMask`; now each view's (`race::face_work::FaceWork`); unused.
    pub face_hide_mask: PlainDword,
    /// +0xf4 (0x641680): each car's listed sound, -1 none (`physics::cue_listed_car_sound`, `drive::choose_shown_cars`).
    pub listed_car_sounds: [PlainDword; 8],
    /// +0x114 (0x6416A0): (`race::frame_views`).
    pub view_layouts: [PlainDword; 6],
    /// +0x12c (0x6416B8): (`race::show_replay_still`, `race::skidmarks`, `race::frame_views`).
    pub drawing: PlainDword,
    /// +0x130 (0x6416BC): `dword_6416BC`: 1 while the replay's still frame is shown (set and cleared by `race::show_replay_still`); `race::lay_skidmarks_if_on` lays no skid marks while it is set.
    pub replay_still_on: PlainDword,
    /// +0x134 (0x6416C0): by layout (`race::frame_views`).
    pub view_scale_x: [PlainFloat; 5],
    /// +0x148 (0x6416D4): by layout.
    pub view_scale_y: [PlainFloat; 5],
    /// +0x15c (0x6416E8): the car's place for the camera (`physics::update_camera`, `physics::place_camera`).
    pub camera_car: [PlainDword; 3],
    /// +0x168 (0x6416F4): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_168: PlainDword,
    /// +0x16c (0x6416F8): `g_carDecalPlaces`.
    pub car_decal_places: [PlainDword; 18],
    /// +0x1b4 (0x641740): (`race::car_draw`).
    pub decal_limit: PlainFloat,
    /// +0x1b8 (0x641744): (`race::frame_draw`).
    pub part_place: [PlainDword; 3],
    /// +0x1c4 (0x641750): (`race::frame_draw`); written only, by nothing now (a dead store); unused.
    pub part_kind: PlainDword,
    /// +0x1c8 (0x641754): the car's parts present (`race::frame_draw`); now each view's (`race::car_draw_state::CarDrawState`); unused.
    pub parts_present: PlainDword,
    /// +0x1cc (0x641758): (`race::frame_draw`); now each view's (`race::car_draw_state::CarDrawState`); unused.
    pub light_left: PlainDword,
    /// +0x1d0 (0x64175C): (`race::frame_draw`); now each view's (`race::car_draw_state::CarDrawState`); unused.
    pub light_right: PlainDword,
    /// +0x1d4 (0x641760): the car's placed matrix and the next (`race::car_parts`); the drawing's
    /// are the view's (`CarDrawState::car_turn`).
    pub placed_matrices: [PlainFloat; 32],
    /// +0x254 (0x6417E0): the car parts' four matrices (`race::car_parts`); the drawing's are the
    /// view's (`CarDrawState::part_matrices`).
    pub part_matrices: [PlainFloat; 64],
    /// +0x354 (0x6418E0): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_354: [PlainDword; 18],
    /// +0x39c (0x641928): (`race::car_shadow`). Unused since the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub shadow_strength: PlainDword,
    /// +0x3a0 (0x64192C): `g_shadowPointCount`. Unused since the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub shadow_point_count: PlainDword,
    /// +0x3a4 (0x641930): (`race::car_shadow`). Unused since the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub shadow_texture: PlainDword,
    /// +0x3a8 (0x641934): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_3a8: PlainDword,
    /// +0x3ac (0x641938): (`race::car_shadow`). Unused since the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub shadow_light: [PlainDword; 3],
    /// +0x3b8 (0x641944): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_3b8: PlainDword,
    /// +0x3bc (0x641948): the shadow's corners, x and z twice (`race::car_shadow`). Unused since the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub shadow_corners: [PlainDword; 4],
    /// +0x3cc (0x641958): `race::car_shadow`'s work dwords. Unused since the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub shadow_work: [PlainDword; 14],
    /// +0x404 (0x641990): (`race::car_shadow`). Unused since the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub shadow_scene: PlainDword,
    /// +0x408 (0x641994): `g_shadowCircle`, `flt_641998`: the shadow circle's radius and unit, 2^23 and 2^24, never written (`race::car_shadow::CIRCLE`).
    pub shadow_circle: [PlainFloat; 2],
    /// +0x410 (0x64199C): the shadow's list (used by address, `race::car_shadow`). Unused since the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub shadow_list: [PlainDword; 64],
    /// +0x510 (0x641A9C): (`race::car_shadow`). Unused since the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub shadow_list_count: PlainDword,
    /// +0x514 (0x641AA0): the ground's height (`race::car_shadow`). Unused since the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub shadow_ground: PlainDword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece, with the image's first values.
impl FrameWork {
    pub const fn new() -> Self {
        Self {
            sun_depth: PlainFloat::new(0.024543693),
            mirror_places: [
                PlainDword::new(0x25D),
                PlainDword::new(0x30),
                PlainDword::new(0xD6),
                PlainDword::new(0x24),
                PlainDword::new(0x25D),
                PlainDword::new(4),
                PlainDword::new(0x25D),
                PlainDword::new(0x30),
                PlainDword::new(0x268),
                PlainDword::new(0x24),
                PlainDword::new(0x25D),
                PlainDword::new(0x141),
                PlainDword::new(0x25D),
                PlainDword::new(0x10),
                PlainDword::new(0xD6),
                PlainDword::new(4),
                PlainDword::new(0x25D),
                PlainDword::new(4),
                PlainDword::new(0x25D),
                PlainDword::new(0x10),
                PlainDword::new(0x268),
                PlainDword::new(4),
                PlainDword::new(0x25D),
                PlainDword::new(0x131),
            ],
            view_layouts: [
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(3),
                PlainDword::new(4),
                PlainDword::new(1),
                PlainDword::new(2),
            ],
            view_scale_x: [
                PlainFloat::new(400.0),
                PlainFloat::new(400.0),
                PlainFloat::new(400.0),
                PlainFloat::new(200.0),
                PlainFloat::new(600.0),
            ],
            view_scale_y: [
                PlainFloat::new(300.0),
                PlainFloat::new(150.0),
                PlainFloat::new(450.0),
                PlainFloat::new(300.0),
                PlainFloat::new(300.0),
            ],
            car_decal_places: [
                PlainDword::new(0xA),
                PlainDword::new(0xB),
                PlainDword::new(0x18),
                PlainDword::new(0x19),
                PlainDword::new(0xC),
                PlainDword::new(0xD),
                PlainDword::new(0x16),
                PlainDword::new(0x17),
                PlainDword::new(0x1A),
                PlainDword::new(0x1B),
                PlainDword::new(0xE),
                PlainDword::new(0xF),
                PlainDword::new(0x10),
                PlainDword::new(0x11),
                PlainDword::new(0x13),
                PlainDword::new(0x14),
                PlainDword::new(0x12),
                PlainDword::new(0),
            ],
            decal_limit: PlainFloat::new(5.0),
            part_place: [
                PlainDword::new(0xFFFFA000),
                PlainDword::new(0xC000),
                PlainDword::new(0x20000),
            ],
            shadow_circle: [PlainFloat::new(8388608.0), PlainFloat::new(16777216.0)],
            ..ZeroCells::ZERO
        }
    }
}
