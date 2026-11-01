//! The scene's lighting and fog state in the race block: the lit place, the turned car light, the view
//! and draw flags, the best-line checkpoint place, the fog values and its 0x1000-byte table, the
//! noise samples, the camera frame values and the polygon hit work. Stops before the error list
//! at 0x637660 (0x2000 bytes, `race::level_init`), and leaves out the lone word at 0x636526.
//! Names are readings unless IDA's.

use crate::session::{Dword, Float};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct LightState {
    /// +0x0 (0x636588): `g_litPlace`, `dword_63658C`, `dword_636590`: the place (x, y, z) being lit, a car or a point (`scene::add_point_lights`, `race::frame_draw`, `race::course_faces`).
    pub lit_place: [Dword; 3],
    /// +0xc (0x636594): not named.
    pub unknown_c: [Dword; 9],
    /// +0x30 (0x6365B8): `dword_6365B8` .. `dword_6365C0`: the turned light direction seen by the car lights (`scene::add_car_light`, `race::frame_draw`).
    pub turned_light: [Dword; 3],
    /// +0x3c (0x6365C4): `dword_6365C4`: the scene mode / lowered-point flag read by the object and face drawers (`scene::project_point`, `scene::draw_object_face`, `scene::draw_car_object`) (a reading).
    pub draw_mode: Dword,
    /// +0x40 (0x6365C8): `dword_6365C8`: the view kind (2 in described objects) (`scene::draw_object_face`, `scene::fog_polygon`, `race::frame_draw`).
    pub view_kind: Dword,
    /// +0x44 (0x6365CC): `dword_6365CC`: the scene state / loading-bar mark (`menu::frame_callback`, `race::loading_bar_frame`).
    pub scene_state: Dword,
    /// +0x48 (0x6365D0): `dword_6365D0`: zero lets the frame turn the light by the views (`race::frame_draw`) (a reading).
    pub light_turn_off: Dword,
    /// +0x4c (0x6365D4): not named.
    pub unknown_4c: [Dword; 4],
    /// +0x5c (0x6365E4): `dword_6365E4`: the checkpoint the car is on, on the best line (`race::start_place`, `physics::reset_car`, `race::best_line`).
    pub line_place: Dword,
    /// +0x60 (0x6365E8): not named.
    pub unknown_60: Dword,
    /// +0x64 (0x6365EC): `dword_6365EC`: the fraction on to the next checkpoint, / 65536 (`race::start_place`, `physics::reset_car_on_line`).
    pub line_fraction: Dword,
    /// +0x68 (0x6365F0): `dword_6365F0`: the fog's near distance, written by the frame set-up (`race::frame_draw`).
    pub fog_near: Dword,
    /// +0x6c (0x6365F4): `dword_6365F4`: the fog's far value; also read as the glare alpha input (`race::frame_draw`, `race::frame_views`).
    pub fog_far: Dword,
    /// +0x70 (0x6365F8): not named.
    pub unknown_70: [Dword; 4],
    /// +0x80 (0x636608): `dword_636608`: the face bits of the object being drawn (`scene::draw_object_face`).
    pub face_bits: Dword,
    /// +0x84 (0x63660C): `dword_63660C`: the faded flag (`scene::draw_polygon`, `scene::draw_object_face`).
    pub faded: Dword,
    /// +0x88 (0x636610): `dword_636610`: the fog's next step, cleared by `scene::clear_fog` (`scene::fog_polygon`, `race::frame_draw`).
    pub fog_next: Dword,
    /// +0x8c (0x636614): not named.
    pub unknown_8c: [Dword; 2],
    /// +0x94 (0x63661C): `g_fogTable`: 0x1000 bytes of fog levels, taken in place (`scene::fog_at`, `scene::fog_polygon`, `scene::clear_fog`, `race::frame_draw`).
    pub fog_table: [Dword; 1024],
    /// +0x1094 (0x63761C): `g_fogTableLength` (`scene::fog_at`, `scene::fog_polygon`, `race::frame_draw`).
    pub fog_table_length: Dword,
    /// +0x1098 (0x637620): `dword_637620` .. `dword_637634`: the four corners, then the x blends, of the noise map (`race::frame_draw::sample_noise`).
    pub noise_samples: [Dword; 6],
    /// +0x10b0 (0x637638): `dword_637638`: camera frame counter (`physics::camera_frame`) (a reading).
    pub camera_frame_a: Dword,
    /// +0x10b4 (0x63763C): `dword_63763C`: camera frame value held under a limit (`physics::camera_frame`) (a reading).
    pub camera_frame_b: Dword,
    /// +0x10b8 (0x637640): `dword_637640` (`physics::camera_frame`) (a reading).
    pub camera_frame_c: Dword,
    /// +0x10bc (0x637644): `flt_637644`: read by `drive::set_listener`, cleared by `physics::camera_frame` (a reading).
    pub listener_camera: Float,
    /// +0x10c0 (0x637648): `dword_637648` .. `dword_63765C`: the polygon hit test's work (`scene::draw_polygon`).
    pub hit_work: [Dword; 6],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece: a static of its own since 2026-10-29 (it left the race block, phase 13:
/// `race::store::LEFT`), with the image's first values.
pub static LIGHT_STATE: LightState =
    crate::pointers::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START));

/// IDA 0x636588, the first byte of the piece.
pub const START: usize = 0x636588;
/// IDA 0x637660, the end of the piece.
pub const END: usize = 0x637660;

const _: () = assert!(END - START == core::mem::size_of::<LightState>());

/// Where IDA 0x636588 is now (its own static).
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(core::ptr::from_ref(&LIGHT_STATE))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&LIGHT_STATE, START, END);
    }

    #[test]
    fn piece_has_its_home() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            piece_addr(),
            crate::framework::game_addr(crate::framework::rva(START))
        );
        assert_eq!(offset_of!(LightState, lit_place), 0x636588 - START);
        assert_eq!(offset_of!(LightState, line_place), 0x6365E4 - START);
        assert_eq!(offset_of!(LightState, fog_next), 0x636610 - START);
        assert_eq!(offset_of!(LightState, camera_frame_c), 0x637640 - START);
    }
}
