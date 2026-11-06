//! The ground and face work (IDA 0x63967C .. 0x63A9B8; phase 7, typed): the frame count, the normal work of a
//! face, the face under a point (the point, the three corners, the differences, the shares and the height
//! physics::ground, plane_height, the car shadow and the lighting share), the object drawing state (texture set,
//! fade, see-through, polygon counts, lamp strength) and the reflection work with its first kept table (normals;
//! the point and coordinate tables follow at +0x1000 each). #[repr(C)] with the image's layout, over the race
//! block's bytes. Names are readings unless IDA's.

use crate::session::{Bytes, Dword, Float};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct GroundWork {
    /// +0x0 (0x63967C): `dword_63967C`: a count kept up by one for each frame drawn (`race::draw_race_scene`).
    pub frames_drawn: Dword,
    /// +0x4 (0x639680): not named.
    pub unknown_4: Dword,
    /// +0x8 (0x639684): `dword_639684`: set to 0 at the end of loading a stage (`race::stage_loading`, a reading: 1 while one loads).
    pub stage_loading: Dword,
    /// +0xc (0x639688): not named.
    pub unknown_c: [Dword; 6],
    /// +0x24 (0x6396A0): `face_normal`'s work: the three points (9 dwords) and the six differences (`race::norms`, `scene::face_faces_away`).
    pub normal_work: [Dword; 15],
    /// +0x60 (0x6396DC): not named.
    pub unknown_60: [Dword; 8],
    /// +0x80 (0x6396FC): the point's x, copied from `ground_x` by `physics::face_shares` (`physics::ground`).
    pub face_point_x: Dword,
    /// +0x84 (0x639700): not named.
    pub unknown_84: Dword,
    /// +0x88 (0x639704): the point's z, copied from `ground_z` by `physics::face_shares` (`physics::ground`).
    pub face_point_z: Dword,
    /// +0x8c (0x639708): the face's three corners (x, y, z each) copied by `physics::face_shares`; the y's are `physics::plane_height`'s inputs (`physics::ground`).
    pub face_corners: [Dword; 9],
    /// +0xb0 (0x63972C): not named.
    pub unknown_b0: [Dword; 3],
    /// +0xbc (0x639738): point x - corner 0 x, corner 1 x - corner 0 x, corner 2 x - corner 0 x (`physics::ground`).
    pub corner_x_differences: [Dword; 3],
    /// +0xc8 (0x639744): not named.
    pub unknown_c8: Dword,
    /// +0xcc (0x639748): corner 1 y - corner 0 y, corner 2 y - corner 0 y (`physics::ground`).
    pub corner_y_differences: [Dword; 2],
    /// +0xd4 (0x639750): point z - corner 0 z, corner 1 z - corner 0 z, corner 2 z - corner 0 z (`physics::ground`).
    pub corner_z_differences: [Dword; 3],
    /// +0xe0 (0x63975C): `flt_63975C`: a float `physics::face_shares` leaves (a reading, output 1) (`physics::ground`).
    pub face_share_b: Float,
    /// +0xe4 (0x639760): `flt_639760`: a float `physics::face_shares` leaves (a reading, output 2) (`physics::ground`).
    pub face_share_c: Float,
    /// +0xe8 (0x639764): `flt_639764`: a float `physics::face_shares` leaves (a reading, output 0) (`physics::ground`).
    pub face_share_a: Float,
    /// +0xec (0x639768): `dword_639768`: written by `physics::face_shares` only with the SSE switch (a reading, the reciprocal) (`physics::ground`).
    pub face_share_scale: Float,
    /// +0xf0 (0x63976C): unreferenced bytes (0x94 of them).
    pub _gap_f0: Bytes<148>,
    /// +0x184 (0x639800): `dword_639800`: a share as a whole number (`fistp`) from `physics::face_shares`; read by `physics::car_lighting`, copied to `dword_62DB08`.
    pub share_s: Dword,
    /// +0x188 (0x639804): `dword_639804`: the second share as a whole number (`fistp`) from `physics::face_shares`; read by `physics::car_lighting`, copied to `dword_62DB0C`.
    pub share_t: Dword,
    /// +0x18c (0x639808): unreferenced bytes (0x8 of them).
    pub _gap_18c: Bytes<8>,
    /// +0x194 (0x639810): `flt_639810`: a float from `physics::face_shares`, a `physics::plane_height` factor; `race::car_shadow` writes it; `physics::probe_ground` copies it on.
    pub plane_f: Float,
    /// +0x198 (0x639814): `flt_639814`: a float from `physics::face_shares`, a `physics::plane_height` factor; `physics::probe_ground` copies it on.
    pub plane_g: Float,
    /// +0x19c (0x639818): unreferenced bytes (0x8 of them).
    pub _gap_19c: Bytes<8>,
    /// +0x1a4 (0x639820): `dword_639820`: set to 1 by `physics::face_under_point` (`physics::ground`, a reading).
    pub face_under_point_done: Dword,
    /// +0x1a8 (0x639824): `dword_639824`: the x of the place whose ground is asked for (`physics::ground`, `physics::probe_ground`, `physics::car_lighting`, `race::car_shadow`, `race::start_place`).
    pub ground_x: Dword,
    /// +0x1ac (0x639828): `dword_639828`: the ground's height `physics::plane_height` works out (`physics::camera_clear_of_ground`, `drive::simple_car_step`).
    pub ground_height: Dword,
    /// +0x1b0 (0x63982C): `dword_63982C`: the z of the place whose ground is asked for (as `ground_x`).
    pub ground_z: Dword,
    /// +0x1b4 (0x639830): `g_groundFound`: 1 when `physics::ground_search` finds a face, else 0 (`physics::count_ground_probe`).
    pub ground_found: Dword,
    /// +0x1b8 (0x639834): not named.
    pub unknown_1b8: [Dword; 23],
    /// +0x214 (0x639890): the three corner point numbers of the face now looked at (`physics::face_shares`, `race::car_shadow::raise_onto_face`).
    pub face_corner_numbers: [Dword; 3],
    /// +0x220 (0x63989C): not named.
    pub unknown_220: [Dword; 40],
    /// +0x2c0 (0x63993C): `dword_63993C`: set while a mirrored view is drawn (`race::frame_views`, `scene::draw_polygon`, `race::frame_draw`) (a reading).
    pub view_mirrored: Dword,
    /// +0x2c4 (0x639940): unreferenced bytes (0x14 of them).
    pub _gap_2c4: Bytes<20>,
    /// +0x2d8 (0x639954): `g_objectTextureSet`: added to an object's face texture numbers (`scene::draw_scene_object`, `scene::draw_object_faces`).
    pub object_texture_set: Dword,
    /// +0x2dc (0x639958): `dword_639958`: the strength of the lamp light worked out (`scene::add_point_lights`, `race::frame_draw`).
    pub point_light_strength: Dword,
    /// +0x2e0 (0x63995C): unreferenced bytes (0x4 of them).
    pub _gap_2e0: Bytes<4>,
    /// +0x2e4 (0x639960): `dword_639960`: polygons drawn whole this frame (`scene::draw_object_polygon`; zeroed by `race::draw_race_scene`) (a reading).
    pub whole_polygon_count: Dword,
    /// +0x2e8 (0x639964): `dword_639964`: polygons drawn split this frame (`scene::draw_object_polygon`; zeroed by `race::draw_race_scene`) (a reading).
    pub split_polygon_count: Dword,
    /// +0x2ec (0x639968): `dword_639968`: the car whose object is being drawn, or -1; `scene::add_car_lights` skips it (`scene::set_object_light`).
    pub drawn_car_light: Dword,
    /// +0x2f0 (0x63996C): `g_objectFading`: the object being drawn fades (`scene::set_object_fade`, `scene::list_object_face`).
    pub object_fading: Dword,
    /// +0x2f4 (0x639970): `g_objectSeeThrough`: faces of the object are see-through (`scene::draw_object_faces`, `scene::list_object_face`).
    pub object_see_through: Dword,
    /// +0x2f8 (0x639974): `dword_639974`, `dword_639978`: the fade value (0x100 = none) written twice by `scene::set_object_fade` (`scene::list_object_face`).
    pub object_fade: [Dword; 2],
    /// +0x300 (0x63997C): unreferenced bytes (0x2c of them).
    pub _gap_300: Bytes<44>,
    /// +0x32c (0x6399A8): `dword_6399A8`: the normal's number `scene::reflection_coordinates` is asked for (`scene::cached_reflection_coordinates`).
    pub reflection_normal_number: Dword,
    /// +0x330 (0x6399AC): `dword_6399AC`: the point's number it is asked for (`scene::reflection_coordinates`).
    pub reflection_point_number: Dword,
    /// +0x334 (0x6399B0): `dword_6399B0`, `dword_6399B4`: the reflection's two coordinates worked out (`scene::reflection_coordinates`).
    pub reflection_coordinates: [Dword; 2],
    /// +0x33c (0x6399B8): four caches of 1024 by point number, 0x1000 bytes apart: `g_keptReflectionNormals` (cleared to -999), `g_keptReflectionPoints`, `g_keptReflectionU`, `g_keptReflectionV` (`race::frame_draw::start_kept_reflections`, `scene::cached_reflection_coordinates`).
    pub kept_reflections: [Dword; 4096],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece: a static of its own since 2026-10-29 (it left the race block, phase 13:
/// `race::store::LEFT`), with the image's first values.
impl GroundWork {
    pub const fn new() -> Self {
        crate::pointers::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START))
    }
}

/// IDA 0x63967C, the first byte of the piece.
pub const START: usize = 0x63967C;
/// IDA 0x63D9B8, the end of the piece.
pub const END: usize = 0x63D9B8;

const _: () = assert!(END - START == core::mem::size_of::<GroundWork>());

/// Where IDA 0x63967C is now (the piece's static).
#[cfg(test)]
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(std::ptr::from_ref(
        &crate::race::owner::race().physics.ground,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(
            &crate::race::owner::race().physics.ground,
            START,
            END,
        );
    }

    #[test]
    fn piece_has_its_home() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            piece_addr(),
            crate::framework::game_addr(crate::framework::rva(START))
        );
        assert_eq!(offset_of!(GroundWork, frames_drawn), 0x63967C - START);
        assert_eq!(
            offset_of!(GroundWork, corner_x_differences),
            0x639738 - START
        );
        assert_eq!(offset_of!(GroundWork, face_share_scale), 0x639768 - START);
        assert_eq!(offset_of!(GroundWork, ground_x), 0x639824 - START);
        assert_eq!(offset_of!(GroundWork, object_texture_set), 0x639954 - START);
        assert_eq!(offset_of!(GroundWork, object_see_through), 0x639970 - START);
    }
}
