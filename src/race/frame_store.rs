//! The frame's draw work (IDA 0x64158C .. 0x641AA4; phase 7, typed): the counts of a frame
//! (flares, lamps, cars drawn and culled), the sun's place and size, the mirror, the views'
//! layouts and scales, the lead car's camera kept, each car's listed sound, the car parts'
//! matrices and decal places, and the car shadow's work. `#[repr(C)]` with the image's
//! layout, over the race block's bytes (`FRAME_WORK`), as `CAR`. Names are readings unless IDA's.

#[cfg(test)]
use crate::race::owner::race;
use crate::session::{Dword, Float};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct FrameWork {
    /// +0x0 (0x64158C): `g_flareCount`: the flares this frame; now each view's (`race::lamp_work::LampWork`); unused.
    pub flare_count: Dword,
    /// +0x4 (0x641590): `g_lampEntryCount`; read by nothing (no longer written); unused.
    pub lamp_entry_count: Dword,
    /// +0x8 (0x641594): `g_carsDrawn`; now each view's (`ctx.view.cars.cars_drawn`); unused.
    pub cars_drawn: Dword,
    /// +0xc (0x641598): `g_carsCulled`.
    pub cars_culled: Dword,
    /// +0x10 (0x64159C): (`race::car_draw`); now each view's (`ctx.view.cars.car_kept`); unused.
    pub car_kept: [Dword; 2],
    /// +0x18 (0x6415A4): the car's depth (0x50000 at first, `race::car_draw`); now each view's (`ctx.view.cars.car_depth`); unused.
    pub car_depth: Dword,
    /// +0x1c (0x6415A8): (`race::frame_draw`).
    pub sun_depth: Float,
    /// +0x20 (0x6415AC): the sun's direction (`race::place_sun`), read by nothing; no longer written.
    pub sun_floats: [Float; 3],
    /// +0x2c (0x6415B8): the sun's turned point (`race::place_sun`), read by nothing; no longer written.
    pub sun_points: [Dword; 3],
    /// +0x38 (0x6415C4): `g_sunOnScreen` (`scene::draw_polygon`'s hit test); now each view's (`ViewSun`); unused.
    pub sun_on_screen: Dword,
    /// +0x3c (0x6415C8): the sun's point on the screen, x and y (`scene::draw_polygon`); now each view's (`ViewSun::hit`); unused.
    pub sun_hit: [Dword; 2],
    /// +0x44 (0x6415D0): its x and y as floats' bits (`drive::set_listener` reads them); now each view's (`ViewSun::place`); unused.
    pub sun_place: [Dword; 2],
    /// +0x4c (0x6415D8): the view's value (`race::frame_draw`, `drive::set_listener`); now each view's (`ViewSun::view_out`); unused.
    pub view_out: Dword,
    /// +0x50 (0x6415DC): `g_sunSize`; now each view's (`ViewSun::size`); unused.
    pub sun_size: Dword,
    /// +0x54 (0x6415E0): the mirror's picture wanted (`d3d::copy_screen_to_texture`, `race::draw_race_scene`).
    pub mirror_wanted: Dword,
    /// +0x58 (0x6415E4): `g_viewLight`; now each view's (`scene::object_light`); unused.
    pub view_light: Dword,
    /// +0x5c (0x6415E8): (`race::frame_views`).
    pub mirror_switch: Dword,
    /// +0x60 (0x6415EC): (`race::frame_views`).
    pub view_kept: [Dword; 2],
    /// +0x68 (0x6415F4): `g_viewTexturesForgotten`: bits.
    pub view_textures_forgotten: Dword,
    /// +0x6c (0x6415F8): (`scene::textures`, `race::draw_race_scene`).
    pub kind_textures_done: Dword,
    /// +0x70 (0x6415FC): `g_mirrorPlaces` (and IDA's `dword_641600`): 12 entries of two dwords,
    /// by `race::mirror_entry`.
    pub mirror_places: [Dword; 24],
    /// +0xd0 (0x64165C): the lead car's camera kept (`drive::lead_car_values`); [0] also `effects::stage_record`'s number, [7] `effects::draw_plane`'s switch and `effects::pong`'s shown.
    pub lead_kept: [Dword; 8],
    /// +0xf0 (0x64167C): `g_faceHideMask`; now each view's (`race::face_work::FaceWork`); unused.
    pub face_hide_mask: Dword,
    /// +0xf4 (0x641680): each car's listed sound, -1 none (`physics::cue_listed_car_sound`, `drive::choose_shown_cars`).
    pub listed_car_sounds: [Dword; 8],
    /// +0x114 (0x6416A0): (`race::frame_views`).
    pub view_layouts: [Dword; 6],
    /// +0x12c (0x6416B8): (`race::show_replay_still`, `race::skidmarks`, `race::frame_views`).
    pub drawing: Dword,
    /// +0x130 (0x6416BC): `dword_6416BC`: 1 while the replay's still frame is shown (set and cleared by `race::show_replay_still`); `race::lay_skidmarks_if_on` lays no skid marks while it is set.
    pub replay_still_on: Dword,
    /// +0x134 (0x6416C0): by layout (`race::frame_views`).
    pub view_scale_x: [Float; 5],
    /// +0x148 (0x6416D4): by layout.
    pub view_scale_y: [Float; 5],
    /// +0x15c (0x6416E8): the car's place for the camera (`physics::update_camera`, `physics::place_camera`).
    pub camera_car: [Dword; 3],
    /// +0x168 (0x6416F4): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_168: Dword,
    /// +0x16c (0x6416F8): `g_carDecalPlaces`.
    pub car_decal_places: [Dword; 18],
    /// +0x1b4 (0x641740): (`race::car_draw`).
    pub decal_limit: Float,
    /// +0x1b8 (0x641744): (`race::frame_draw`).
    pub part_place: [Dword; 3],
    /// +0x1c4 (0x641750): (`race::frame_draw`); written only, by nothing now (a dead store); unused.
    pub part_kind: Dword,
    /// +0x1c8 (0x641754): the car's parts present (`race::frame_draw`); now each view's (`race::car_draw_state::CarDrawState`); unused.
    pub parts_present: Dword,
    /// +0x1cc (0x641758): (`race::frame_draw`); now each view's (`race::car_draw_state::CarDrawState`); unused.
    pub light_left: Dword,
    /// +0x1d0 (0x64175C): (`race::frame_draw`); now each view's (`race::car_draw_state::CarDrawState`); unused.
    pub light_right: Dword,
    /// +0x1d4 (0x641760): the car's placed matrix and the next (`race::car_parts`); the drawing's
    /// are the view's (`CarDrawState::car_turn`).
    pub placed_matrices: [Float; 32],
    /// +0x254 (0x6417E0): the car parts' four matrices (`race::car_parts`); the drawing's are the
    /// view's (`CarDrawState::part_matrices`).
    pub part_matrices: [Float; 64],
    /// +0x354 (0x6418E0): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_354: [Dword; 18],
    /// +0x39c (0x641928): (`race::car_shadow`). Unused since the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub shadow_strength: Dword,
    /// +0x3a0 (0x64192C): `g_shadowPointCount`. Unused since the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub shadow_point_count: Dword,
    /// +0x3a4 (0x641930): (`race::car_shadow`). Unused since the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub shadow_texture: Dword,
    /// +0x3a8 (0x641934): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_3a8: Dword,
    /// +0x3ac (0x641938): (`race::car_shadow`). Unused since the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub shadow_light: [Dword; 3],
    /// +0x3b8 (0x641944): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_3b8: Dword,
    /// +0x3bc (0x641948): the shadow's corners, x and z twice (`race::car_shadow`). Unused since the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub shadow_corners: [Dword; 4],
    /// +0x3cc (0x641958): `race::car_shadow`'s work dwords. Unused since the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub shadow_work: [Dword; 14],
    /// +0x404 (0x641990): (`race::car_shadow`). Unused since the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub shadow_scene: Dword,
    /// +0x408 (0x641994): `g_shadowCircle`, `flt_641998`: the shadow circle's radius and unit, 2^23 and 2^24, never written (`race::car_shadow::CIRCLE`).
    pub shadow_circle: [Float; 2],
    /// +0x410 (0x64199C): the shadow's list (used by address, `race::car_shadow`). Unused since the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub shadow_list: [Dword; 64],
    /// +0x510 (0x641A9C): (`race::car_shadow`). Unused since the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub shadow_list_count: Dword,
    /// +0x514 (0x641AA0): the ground's height (`race::car_shadow`). Unused since the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub shadow_ground: Dword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece: a static of its own since 2026-10-29 (it left the race block, phase 13:
/// `race::store::LEFT`), with the image's first values.
impl FrameWork {
    pub const fn new() -> Self {
        crate::cells::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START))
    }
}

/// IDA 0x64158C, the first byte of the piece.
pub const START: usize = 0x64158C;
/// IDA 0x641AA4, the end of the piece.
pub const END: usize = 0x641AA4;

const _: () = assert!(END - START == core::mem::size_of::<FrameWork>());

/// Where IDA 0x64158C is now (its own static).
#[cfg(test)]
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(core::ptr::from_ref(&race().frame.frame_work))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            &race().frame.frame_work,
        );
    }

    #[test]
    fn fields_are_at_the_games_offsets() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(offset_of!(FrameWork, flare_count), 0x0);
        assert_eq!(offset_of!(FrameWork, sun_depth), 0x1c);
        assert_eq!(offset_of!(FrameWork, view_out), 0x4c);
        assert_eq!(offset_of!(FrameWork, view_textures_forgotten), 0x68);
        assert_eq!(offset_of!(FrameWork, view_layouts), 0x114);
        assert_eq!(offset_of!(FrameWork, car_decal_places), 0x16c);
        assert_eq!(offset_of!(FrameWork, light_right), 0x1d0);
        assert_eq!(offset_of!(FrameWork, shadow_light), 0x3ac);
        assert_eq!(offset_of!(FrameWork, shadow_list_count), 0x510);
    }
}
