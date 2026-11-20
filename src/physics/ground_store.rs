//! The ground and face work: the frame count, the normal work of a face, the face under a point
//! (the point, the three corners, the differences, the shares and the height used by
//! `physics::ground`, `plane_height`, the car shadow and the lighting share), the object drawing
//! state (texture set, fade, see-through, polygon counts, lamp strength) and the reflection work
//! with its first kept table (normals; the point and coordinate tables follow at 0x1000 each).
//! Names are readings unless taken from the game's own.

use crate::memory::session::{PlainDword, PlainFloat};

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct GroundWork {
    /// A count kept up by one for each frame drawn (`race::draw_race_scene`).
    pub frames_drawn: PlainDword,
    /// `face_normal`'s work: the three points (9 dwords) and the six differences (`race::norms`,
    /// `scene::face_faces_away`); the drawing's is each view's (`ctx.view.object.normal_work`);
    /// this one is written by `race::face_normal` and read by none.
    pub normal_work: [PlainDword; 15],
    /// The point's x, copied from `ground_x` by `physics::face_shares` (`physics::ground`).
    pub face_point_x: PlainDword,
    /// The point's z, copied from `ground_z` by `physics::face_shares` (`physics::ground`).
    pub face_point_z: PlainDword,
    /// The face's three corners (x, y, z each) copied by `physics::face_shares`; the y's are
    /// `physics::plane_height`'s inputs (`physics::ground`).
    pub face_corners: [PlainDword; 9],
    /// Point x - corner 0 x, corner 1 x - corner 0 x, corner 2 x - corner 0 x (`physics::ground`).
    pub corner_x_differences: [PlainDword; 3],
    /// Corner 1 y - corner 0 y, corner 2 y - corner 0 y (`physics::ground`).
    pub corner_y_differences: [PlainDword; 2],
    /// Point z - corner 0 z, corner 1 z - corner 0 z, corner 2 z - corner 0 z (`physics::ground`).
    pub corner_z_differences: [PlainDword; 3],
    /// A float `physics::face_shares` leaves (a reading, output 1) (`physics::ground`).
    pub face_share_b: PlainFloat,
    /// A float `physics::face_shares` leaves (a reading, output 2) (`physics::ground`).
    pub face_share_c: PlainFloat,
    /// A float `physics::face_shares` leaves (a reading, output 0) (`physics::ground`).
    pub face_share_a: PlainFloat,
    /// A share as a whole number from `physics::face_shares`; read by `physics::car_lighting`,
    /// copied on.
    pub share_s: PlainDword,
    /// The second share as a whole number from `physics::face_shares`; read by
    /// `physics::car_lighting`, copied on.
    pub share_t: PlainDword,
    /// A float from `physics::face_shares`, a `physics::plane_height` factor; `race::car_shadow`
    /// writes it; `physics::probe_ground` copies it on.
    pub plane_f: PlainFloat,
    /// A float from `physics::face_shares`, a `physics::plane_height` factor;
    /// `physics::probe_ground` copies it on.
    pub plane_g: PlainFloat,
    /// Set to 1 by `physics::face_under_point` (`physics::ground`, a reading).
    pub face_under_point_done: PlainDword,
    /// The x of the place whose ground is asked for (`physics::ground`, `physics::probe_ground`,
    /// `physics::car_lighting`, `race::car_shadow`, `race::start_place`).
    pub ground_x: PlainDword,
    /// The ground's height `physics::plane_height` works out (`physics::camera_clear_of_ground`,
    /// `drive::simple_car_step`).
    pub ground_height: PlainDword,
    /// The z of the place whose ground is asked for (as `ground_x`).
    pub ground_z: PlainDword,
    /// 1 when `physics::ground_search` finds a face, else 0 (`physics::count_ground_probe`).
    pub ground_found: PlainDword,
    /// The three corner point numbers of the face now looked at (`physics::face_shares`,
    /// `race::car_shadow::raise_onto_face`).
    pub face_corner_numbers: [PlainDword; 3],
    /// Set while a mirrored view is drawn (`race::frame_views`, `scene::draw_polygon`,
    /// `race::frame_draw`) (a reading); now each view's (`PolygonWork::view_mirrored`); unused.
    pub view_mirrored: PlainDword,
    /// Added to an object's face texture numbers (`scene::draw_scene_object`,
    /// `scene::draw_object_faces`); now each view's (`ObjectPlace::texture_set`); unused.
    pub object_texture_set: PlainDword,
    /// Polygons drawn whole this frame (`scene::draw_object_polygon`; zeroed by
    /// `race::draw_race_scene`) (a reading); now each view's
    /// (`ctx.view.object.whole_polygon_count`); unused.
    pub whole_polygon_count: PlainDword,
    /// Polygons drawn split this frame (`scene::draw_object_polygon`; zeroed by
    /// `race::draw_race_scene`) (a reading); now each view's
    /// (`ctx.view.object.split_polygon_count`); unused.
    pub split_polygon_count: PlainDword,
    /// The car whose object is being drawn, or -1; `scene::add_car_lights` skips it
    /// (`scene::set_object_light`); now each view's (`ObjectLight::drawn_car_light`); unused.
    pub drawn_car_light: PlainDword,
    /// The object being drawn fades (`scene::set_object_fade`, `scene::list_object_face`); now each
    /// view's (`ctx.view.object.fading`); unused.
    pub object_fading: PlainDword,
    /// Faces of the object are see-through (`scene::draw_object_faces`, `scene::list_object_face`);
    /// now each view's (`ctx.view.object.see_through`); unused.
    pub object_see_through: PlainDword,
    /// The fade value (0x100 = none) written twice by `scene::set_object_fade`
    /// (`scene::list_object_face`); now each view's (`ctx.view.object.fade`); unused.
    pub object_fade: [PlainDword; 2],
    /// The normal's number `scene::reflection_coordinates` is asked for
    /// (`scene::cached_reflection_coordinates`); now each view's
    /// (`ctx.view.object.reflection_normal_number`); unused.
    pub reflection_normal_number: PlainDword,
    /// The point's number it is asked for (`scene::reflection_coordinates`); now each view's
    /// (`ctx.view.object.reflection_point_number`); unused.
    pub reflection_point_number: PlainDword,
    /// The reflection's two coordinates worked out (`scene::reflection_coordinates`); now each
    /// view's (`ctx.view.object.reflection_coordinates`); unused.
    pub reflection_coordinates: [PlainDword; 2],
    /// Four caches of 1024 by point number, 0x1000 bytes apart: the kept reflection normals
    /// (cleared to -999), points, U and V (`race::frame_draw::start_kept_reflections`,
    /// `scene::cached_reflection_coordinates`); now each view's
    /// (`ctx.view.object.kept_reflections`); unused.
    pub kept_reflections: [PlainDword; 4096],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece, with the image's first values.
impl GroundWork {
    pub const fn new() -> Self {
        use crate::memory::cells::ZeroCells;
        use crate::memory::session::PlainDword;
        Self {
            drawn_car_light: PlainDword::new(0xFFFFFFFF),
            ..ZeroCells::ZERO
        }
    }
}
