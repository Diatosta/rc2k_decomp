//! The course face being drawn, a view's own working values: the corners, the two textures with
//! their orders and backs, the colours, lights and see-through by corner, the polygon mode, the
//! object face's number, kind and unlit byte, and the fade of the car being drawn. One per view
//! slot (`race::view_scratch::ViewScratch::face`): the corner values (`face_turned` ..
//! `polygon_mode`), `face_number`, `mapping_a` and `mapping_b`, `face_kind` and `face_unlit`, the
//! fade (`fade_alpha`, `faded_bits`, `faded`), the current course face and the texture
//! coordinates by corner (`corner_uv`, `corner_uv_b`). A face without a second texture copies the
//! second set into its entries unwritten: the last such face's in the same view (none drawn:
//! zeros). Nothing reads them on such a face.

use crate::memory::session::PlainDword;

#[derive(rc2k_derive::Cells)]
pub struct FaceWork {
    /// The face's halves turned to the view (`race::course_faces`).
    pub turned: [PlainDword; 2],
    /// The face's corners (the fourth -1 for a face of three points).
    pub corners: [PlainDword; 4],
    /// The first texture, its corner order and its back.
    pub texture_a: PlainDword,
    pub order_a: PlainDword,
    pub back_a: PlainDword,
    /// The second texture, its corner order and its back.
    pub texture_b: PlainDword,
    pub order_b: PlainDword,
    pub back_b: PlainDword,
    /// The colour, the light and the see-through (a byte) by corner.
    pub corner_colours: [PlainDword; 4],
    pub corner_lights: [PlainDword; 4],
    pub corner_alphas: [PlainDword; 4],
    /// How `scene::draw_polygon` draws.
    pub polygon_mode: PlainDword,
    /// The two textures' mapping modes (the first dword the mode; the second is not used).
    pub mapping_a: [PlainDword; 2],
    pub mapping_b: [PlainDword; 2],
    /// The object face being listed or drawn.
    pub face_number: PlainDword,
    /// The object face's kind (its head's byte 6) and unlit byte (byte 0xB), set by
    /// `scene::draw_object_faces` for `scene::draw_object_face`.
    pub face_kind: PlainDword,
    pub face_unlit: PlainDword,
    /// The fade of the car being drawn: its alpha (`race::car_draw`; `fade_alpha_moved`), the faded
    /// bits of its model (-1 outside a car's drawing: `faded_bits`) and whether the face being
    /// drawn is faded (`scene::draw_object_face` for `scene::draw_polygon`: `faded`).
    pub fade_alpha: PlainDword,
    pub faded_bits: PlainDword,
    pub faded: PlainDword,
    /// The course face being drawn or listed (`current_course_face`), named by the bad polygon's
    /// log; the objects of two shapes would keep their moving state in the face's normals' place
    /// (`scene::set_object_material`), but no kind in the game's files has that material (way 4:
    /// none in objsinfo, carsinfo, frntinfo or extrinfo).
    pub course_face: PlainDword,
    /// The two textures' coordinates by corner, x then y (`corner_uv`, and
    /// `corner_uv_b`): set for a face's textured sets (`race::face_texture_coordinates`), read into
    /// its polygon entries.
    pub uv_a: [PlainDword; 8],
    pub uv_b: [PlainDword; 8],
    /// `face_depth_key`: the face's depth key, each view's own.
    pub depth_key: PlainDword,
    /// `reflections_kept`: each view's own.
    pub reflections_kept: PlainDword,
    /// `face_reflects`: each view's own.
    pub reflects: PlainDword,
    /// `no_far_clip`: the sky is drawn uncut, each view's own.
    pub no_far_clip: PlainDword,
    /// The faces hidden from the shown view, set
    /// from the face under the car (`race::face_hide_mask`), each view's own.
    pub face_hide_mask: PlainDword,
    /// `face_any_shine`: whether a vertex of the object face
    /// being drawn shines (`scene::draw_object_face`), each view's own.
    pub face_any_shine: PlainDword,
    /// `depth_test` (`RaceView.depth_test`): the face turned the depth test off
    /// (`race::face_depth_test_off`), each view's own.
    pub depth_test: PlainDword,
}

impl FaceWork {
    pub const fn new() -> Self {
        Self {
            turned: [const { PlainDword::new(0) }; 2],
            corners: [const { PlainDword::new(0) }; 4],
            texture_a: PlainDword::new(0),
            order_a: PlainDword::new(0),
            back_a: PlainDword::new(0),
            texture_b: PlainDword::new(0),
            order_b: PlainDword::new(0),
            back_b: PlainDword::new(0),
            corner_colours: [const { PlainDword::new(0) }; 4],
            corner_lights: [const { PlainDword::new(0) }; 4],
            corner_alphas: [const { PlainDword::new(0) }; 4],
            polygon_mode: PlainDword::new(0),
            mapping_a: [const { PlainDword::new(0) }; 2],
            mapping_b: [const { PlainDword::new(0) }; 2],
            face_number: PlainDword::new(0),
            face_kind: PlainDword::new(0),
            face_unlit: PlainDword::new(0),
            fade_alpha: PlainDword::new(0),
            faded_bits: PlainDword::new(u32::MAX),
            faded: PlainDword::new(0),
            course_face: PlainDword::new(0),
            uv_a: [const { PlainDword::new(0) }; 8],
            uv_b: [const { PlainDword::new(0) }; 8],
            depth_key: PlainDword::new(0),
            reflections_kept: PlainDword::new(0),
            reflects: PlainDword::new(0),
            no_far_clip: PlainDword::new(0),
            face_hide_mask: PlainDword::new(0),
            face_any_shine: PlainDword::new(0),
            depth_test: PlainDword::new(0),
        }
    }
}

/// The fourth corner of the last face drawn by any view (-1 for a face of three points), read by
/// `race::norms` at a stage's start-up as `RaceView.corners[3]` was (it should be the face's own: a
/// slip kept as is); before any face was drawn it is the image's first value.
pub(super) fn last_fourth_corner(
    shared: &crate::race::owner_shared::RaceShared,
    scratch: &crate::scene::workers::DrawingScratch,
) -> u32 {
    match scratch.last_face_view() {
        None => shared.view.get().corners[3].get(),
        Some(slot) => scratch.slot(slot).face.corners[3].get(),
    }
}
