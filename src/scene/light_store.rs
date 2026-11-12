//! The scene's lighting and fog state in the race block: the lit place, the turned car light, the view
//! and draw flags, the best-line checkpoint place, the fog values and its 0x1000-byte table, the
//! noise samples, the camera frame values and the polygon hit work. Stops before the error list
//! at 0x637660 (0x2000 bytes, `race::level_init`), and leaves out the lone word at 0x636526.
//! Names are readings unless IDA's.

use crate::session::{Dword, Float};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct LightState {
    /// +0x0 (0x636588): `g_litPlace`, `dword_63658C`, `dword_636590`: the place (x, y, z) being lit, a car or a point (`scene::add_point_lights`, `race::frame_draw`, `race::course_faces`); now each view's (`scene::object_light`); unused.
    pub lit_place: [Dword; 3],
    /// +0xc (0x636594): nine dwords before the turned light; the original's code does not reference them and nothing in the port does (a gap).
    pub unknown_c: [Dword; 9],
    /// +0x30 (0x6365B8): `dword_6365B8` .. `dword_6365C0`: the turned light direction seen by the car lights (`scene::add_car_light`, `race::frame_draw`); now each view's (`scene::object_light`); unused.
    pub turned_light: [Dword; 3],
    /// +0x3c (0x6365C4): `dword_6365C4`: the draw mode: 0 normal (set by `scene::draw_car_object`, the carousel and the highlighted object); 1 lowers each projected point by a fixed amount (`scene::project_point`) and, with 2, skips the sky (`race::frame_draw::view::draw_sky`); non-0 changes how `scene::draw_object_face` and `scene::draw_scene_object` draw. Nothing in the port sets 1 or 2, so they come from the image's first value only (probably).
    pub draw_mode: Dword,
    /// +0x40 (0x6365C8): `dword_6365C8`: the view kind (2 in described objects) (`scene::draw_object_face`, `scene::fog_polygon`, `race::frame_draw`); now each view's (`scene::object_light`); unused.
    pub view_kind: Dword,
    /// +0x44 (0x6365CC): `dword_6365CC`: the scene state / loading-bar mark (`menu::frame_callback`, `race::loading_bar_frame`).
    pub scene_state: Dword,
    /// +0x48 (0x6365D0): `dword_6365D0`: `race::place_object_matrix`'s switch: 0 turns the object's matrix by the angles in `OBJECT_WORK` (y, x, z); non-0 uses the material rules (facing the view, turned by the view's angles) (`race::frame_draw::scene_objects`).
    pub light_turn_off: Dword,
    /// +0x4c (0x6365D4): four dwords after the light switch; no code references them (a gap).
    pub unknown_4c: [Dword; 4],
    /// +0x5c (0x6365E4): `dword_6365E4`: the checkpoint the car is on, on the best line (`race::start_place`, `physics::reset_car`, `race::best_line`).
    pub line_place: Dword,
    /// +0x60 (0x6365E8): one dword between the line place and its fraction; no code references it (a gap).
    pub unknown_60: Dword,
    /// +0x64 (0x6365EC): `dword_6365EC`: the fraction on to the next checkpoint, / 65536 (`race::start_place`, `physics::reset_car_on_line`).
    pub line_fraction: Dword,
    /// +0x68 (0x6365F0): `dword_6365F0`: the fog's near distance; never read, no longer written (`scene::fog`).
    pub fog_near: Dword,
    /// +0x6c (0x6365F4): `dword_6365F4`: the fog's far value, now each view's (`scene::fog::Fog::far`); unused.
    pub fog_far: Dword,
    /// +0x70 (0x6365F8): four dwords after the fog's far value; no code references them (a gap).
    pub unknown_70: [Dword; 4],
    /// +0x80 (0x636608): `dword_636608`: the face bits of the object being drawn (`scene::draw_object_face`); now each view's (`scene::object_light`); unused.
    pub face_bits: Dword,
    /// +0x84 (0x63660C): `dword_63660C`: the faded flag; the drawing's is the view's (`race::face_work`), this one not used.
    pub faded: Dword,
    /// +0x88 (0x636610): `dword_636610`: the fog's next step, now each view's (`scene::fog::Fog::next`); unused.
    pub fog_next: Dword,
    /// +0x8c (0x636614): two dwords before the fog table; no code references them (a gap).
    pub unknown_8c: [Dword; 2],
    /// +0x94 (0x63661C): `g_fogTable`: 0x1000 bytes of fog levels, now each view's (`scene::fog::Fog`); unused.
    pub fog_table: [Dword; 1024],
    /// +0x1094 (0x63761C): `g_fogTableLength`, now each view's (`scene::fog::Fog::length`); unused.
    pub fog_table_length: Dword,
    /// +0x1098 (0x637620): `dword_637620` .. `dword_637634`: the four corners, then the x blends, of the noise map: `race::sample_noise`'s own steps, read by nothing else; no longer written (`rc2k_core::race::view::sample_noise` returns the value).
    pub noise_samples: [Dword; 6],
    /// +0x10b0 (0x637638): `dword_637638`: the camera shake's first component, added to view A's / view B's camera offset (`physics::take_first_view`, `physics::take_second_view`). Never written (0 from the image, nothing stores to it; the original's `physics::camera_frame` cleared it each call): the shake is all in `camera_shake_y`. Probably x.
    pub camera_shake_x: Dword,
    /// +0x10b4 (0x63763C): `dword_63763C`: the camera shake's second component: the car's bump value (`unknown_90c`) times the physics table's scale, held within plus / minus its limit, 0 when the car stands still; added to the camera offsets. Probably y, the vertical (`physics::camera_frame`).
    pub camera_shake_y: Dword,
    /// +0x10b8 (0x637640): `dword_637640`: the camera shake's third component; never written (0 from the image; the original's `physics::camera_frame` cleared it each call), added to the camera offsets. Probably z.
    pub camera_shake_z: Dword,
    /// +0x10bc (0x637644): `flt_637644`: the shake's strength for the sound: a random number / 65536 / 4096 times 3 times `camera_shake_y`; 0 outside the shaking camera modes. Read by `drive::set_listener`: added to the steering angle / 0x20000000 to make `listener_scale` (`physics::camera_frame`).
    pub camera_shake_listener: Float,
    /// +0x10c0 (0x637648): `dword_637648` .. `dword_63765C`: the polygon hit test's work (`scene::draw_polygon`).
    pub hit_work: [Dword; 6],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl LightState {
    /// The piece with the image's first values.
    pub const fn new() -> Self {
        crate::pointers::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START))
    }
}

/// IDA 0x636588, the first byte of the piece.
pub const START: usize = 0x636588;
/// IDA 0x637660, the end of the piece.
pub const END: usize = 0x637660;

const _: () = assert!(END - START == core::mem::size_of::<LightState>());

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::owner::scene;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(&scene().lights.state);
    }

    #[test]
    fn piece_has_its_home() {
        assert_eq!(offset_of!(LightState, lit_place), 0x0);
        assert_eq!(offset_of!(LightState, line_place), 0x5c);
        assert_eq!(offset_of!(LightState, fog_next), 0x88);
        assert_eq!(offset_of!(LightState, camera_shake_z), 0x10b8);
    }
}
