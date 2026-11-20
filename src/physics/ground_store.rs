//! The ground and face work: the frame count, the normal work of a face, the object drawing
//! state (texture set, fade, see-through, polygon counts, lamp strength) and the reflection work
//! with its first kept table (normals; the point and coordinate tables follow at 0x1000 each).
//! Names are readings unless taken from the game's own.

use crate::memory::session::PlainDword;

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct GroundWork {
    /// A count kept up by one for each frame drawn (`race::draw_race_scene`).
    pub frames_drawn: PlainDword,
    /// `face_normal`'s work: the three points (9 dwords) and the six differences (`race::norms`,
    /// `scene::face_faces_away`); the drawing's is each view's (`ctx.view.object.normal_work`);
    /// this one is written by `race::face_normal` and read by none.
    pub normal_work: [PlainDword; 15],
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
