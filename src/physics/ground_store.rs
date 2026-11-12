//! The ground and face work (IDA 0x63967C .. 0x63A9B8; phase 7, typed): the frame count, the normal work of a
//! face, the face under a point (the point, the three corners, the differences, the shares and the height
//! physics::ground, plane_height, the car shadow and the lighting share), the object drawing state (texture set,
//! fade, see-through, polygon counts, lamp strength) and the reflection work with its first kept table (normals;
//! the point and coordinate tables follow at +0x1000 each). #[repr(C)]: the derive reads the fields in order.
//! Names are readings unless IDA's.

#[cfg(test)]
use crate::race::owner::race;
use crate::session::{PlainDword, PlainFloat};

#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct GroundWork {
    /// (0x63967C): `dword_63967C`: a count kept up by one for each frame drawn (`race::draw_race_scene`).
    pub frames_drawn: PlainDword,
    /// (0x639680): no instruction of the game refers to it (IDA, 2026-11-10); unused.
    pub unused_4: PlainDword,
    /// (0x639684): `dword_639684`: set to 0 at the end of loading a stage (`race::stage_loading`, a reading: 1 while one loads).
    pub stage_loading: PlainDword,
    /// (0x639688): no instruction of the game refers to it (IDA, 2026-11-10); unused.
    pub unused_c: [PlainDword; 6],
    /// (0x6396A0): `face_normal`'s work: the three points (9 dwords) and the six differences (`race::norms`, `scene::face_faces_away`); the drawing's is each view's (`ctx.view.object.normal_work`); this one is written by `race::face_normal` and read by none.
    pub normal_work: [PlainDword; 15],
    /// (0x6396DC): no instruction of the game refers to it (IDA, 2026-11-10); unused.
    pub unused_60: [PlainDword; 8],
    /// (0x6396FC): the point's x, copied from `ground_x` by `physics::face_shares` (`physics::ground`).
    pub face_point_x: PlainDword,
    /// (0x639700): between the point's x and z: the slot of its y, which no instruction of the game uses (IDA, 2026-11-10).
    pub unused_84: PlainDword,
    /// (0x639704): the point's z, copied from `ground_z` by `physics::face_shares` (`physics::ground`).
    pub face_point_z: PlainDword,
    /// (0x639708): the face's three corners (x, y, z each) copied by `physics::face_shares`; the y's are `physics::plane_height`'s inputs (`physics::ground`).
    pub face_corners: [PlainDword; 9],
    /// (0x63972C): no instruction of the game refers to it (IDA, 2026-11-10); unused.
    pub unused_b0: [PlainDword; 3],
    /// (0x639738): point x - corner 0 x, corner 1 x - corner 0 x, corner 2 x - corner 0 x (`physics::ground`).
    pub corner_x_differences: [PlainDword; 3],
    /// (0x639744): between the x and y differences: the slot of the point's y less corner 0's y, which no instruction of the game uses (IDA, 2026-11-10).
    pub unused_c8: PlainDword,
    /// (0x639748): corner 1 y - corner 0 y, corner 2 y - corner 0 y (`physics::ground`).
    pub corner_y_differences: [PlainDword; 2],
    /// (0x639750): point z - corner 0 z, corner 1 z - corner 0 z, corner 2 z - corner 0 z (`physics::ground`).
    pub corner_z_differences: [PlainDword; 3],
    /// (0x63975C): `flt_63975C`: a float `physics::face_shares` leaves (a reading, output 1) (`physics::ground`).
    pub face_share_b: PlainFloat,
    /// (0x639760): `flt_639760`: a float `physics::face_shares` leaves (a reading, output 2) (`physics::ground`).
    pub face_share_c: PlainFloat,
    /// (0x639764): `flt_639764`: a float `physics::face_shares` leaves (a reading, output 0) (`physics::ground`).
    pub face_share_a: PlainFloat,
    /// (0x639768): `dword_639768`: written by `physics::face_shares` only with the SSE switch (a reading, the reciprocal) (`physics::ground`).
    pub face_share_scale: PlainFloat,
    /// (0x639800): `dword_639800`: a share as a whole number (`fistp`) from `physics::face_shares`; read by `physics::car_lighting`, copied to `dword_62DB08`.
    pub share_s: PlainDword,
    /// (0x639804): `dword_639804`: the second share as a whole number (`fistp`) from `physics::face_shares`; read by `physics::car_lighting`, copied to `dword_62DB0C`.
    pub share_t: PlainDword,
    /// (0x639810): `flt_639810`: a float from `physics::face_shares`, a `physics::plane_height` factor; `race::car_shadow` writes it; `physics::probe_ground` copies it on.
    pub plane_f: PlainFloat,
    /// (0x639814): `flt_639814`: a float from `physics::face_shares`, a `physics::plane_height` factor; `physics::probe_ground` copies it on.
    pub plane_g: PlainFloat,
    /// (0x639820): `dword_639820`: set to 1 by `physics::face_under_point` (`physics::ground`, a reading).
    pub face_under_point_done: PlainDword,
    /// (0x639824): `dword_639824`: the x of the place whose ground is asked for (`physics::ground`, `physics::probe_ground`, `physics::car_lighting`, `race::car_shadow`, `race::start_place`).
    pub ground_x: PlainDword,
    /// (0x639828): `dword_639828`: the ground's height `physics::plane_height` works out (`physics::camera_clear_of_ground`, `drive::simple_car_step`).
    pub ground_height: PlainDword,
    /// (0x63982C): `dword_63982C`: the z of the place whose ground is asked for (as `ground_x`).
    pub ground_z: PlainDword,
    /// (0x639830): `g_groundFound`: 1 when `physics::ground_search` finds a face, else 0 (`physics::count_ground_probe`).
    pub ground_found: PlainDword,
    /// (0x639834): no instruction of the game refers to it (IDA, 2026-11-10); unused.
    pub unused_1b8: [PlainDword; 23],
    /// (0x639890): the three corner point numbers of the face now looked at (`physics::face_shares`, `race::car_shadow::raise_onto_face`).
    pub face_corner_numbers: [PlainDword; 3],
    /// (0x63989C): no instruction of the game refers to it (IDA, 2026-11-10); unused.
    pub unused_220: [PlainDword; 40],
    /// (0x63993C): `dword_63993C`: set while a mirrored view is drawn (`race::frame_views`, `scene::draw_polygon`, `race::frame_draw`) (a reading); now each view's (`PolygonWork::view_mirrored`); unused.
    pub view_mirrored: PlainDword,
    /// (0x639954): `g_objectTextureSet`: added to an object's face texture numbers (`scene::draw_scene_object`, `scene::draw_object_faces`); now each view's (`ObjectPlace::texture_set`); unused.
    pub object_texture_set: PlainDword,
    /// (0x639960): `dword_639960`: polygons drawn whole this frame (`scene::draw_object_polygon`; zeroed by `race::draw_race_scene`) (a reading); now each view's (`ctx.view.object.whole_polygon_count`); unused.
    pub whole_polygon_count: PlainDword,
    /// (0x639964): `dword_639964`: polygons drawn split this frame (`scene::draw_object_polygon`; zeroed by `race::draw_race_scene`) (a reading); now each view's (`ctx.view.object.split_polygon_count`); unused.
    pub split_polygon_count: PlainDword,
    /// (0x639968): `dword_639968`: the car whose object is being drawn, or -1; `scene::add_car_lights` skips it (`scene::set_object_light`); now each view's (`ObjectLight::drawn_car_light`); unused.
    pub drawn_car_light: PlainDword,
    /// (0x63996C): `g_objectFading`: the object being drawn fades (`scene::set_object_fade`, `scene::list_object_face`); now each view's (`ctx.view.object.fading`); unused.
    pub object_fading: PlainDword,
    /// (0x639970): `g_objectSeeThrough`: faces of the object are see-through (`scene::draw_object_faces`, `scene::list_object_face`); now each view's (`ctx.view.object.see_through`); unused.
    pub object_see_through: PlainDword,
    /// (0x639974): `dword_639974`, `dword_639978`: the fade value (0x100 = none) written twice by `scene::set_object_fade` (`scene::list_object_face`); now each view's (`ctx.view.object.fade`); unused.
    pub object_fade: [PlainDword; 2],
    /// (0x6399A8): `dword_6399A8`: the normal's number `scene::reflection_coordinates` is asked for (`scene::cached_reflection_coordinates`); now each view's (`ctx.view.object.reflection_normal_number`); unused.
    pub reflection_normal_number: PlainDword,
    /// (0x6399AC): `dword_6399AC`: the point's number it is asked for (`scene::reflection_coordinates`); now each view's (`ctx.view.object.reflection_point_number`); unused.
    pub reflection_point_number: PlainDword,
    /// (0x6399B0): `dword_6399B0`, `dword_6399B4`: the reflection's two coordinates worked out (`scene::reflection_coordinates`); now each view's (`ctx.view.object.reflection_coordinates`); unused.
    pub reflection_coordinates: [PlainDword; 2],
    /// (0x6399B8): four caches of 1024 by point number, 0x1000 bytes apart: `g_keptReflectionNormals` (cleared to -999), `g_keptReflectionPoints`, `g_keptReflectionU`, `g_keptReflectionV` (`race::frame_draw::start_kept_reflections`, `scene::cached_reflection_coordinates`); now each view's (`ctx.view.object.kept_reflections`); unused.
    pub kept_reflections: [PlainDword; 4096],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece, with the image's first values.
impl GroundWork {
    pub const fn new() -> Self {
        use crate::cells::ZeroCells;
        use crate::session::PlainDword;
        Self {
            drawn_car_light: PlainDword::new(0xFFFFFFFF),
            ..ZeroCells::ZERO
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            race().physics.ground(),
        );
    }
}
