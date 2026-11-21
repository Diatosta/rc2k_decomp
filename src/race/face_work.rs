//! The course face being drawn, a view's own working values: the corners, the two textures with
//! their orders and backs, the colours, lights and see-through by corner, the polygon mode, the
//! object face's number, kind and unlit byte, and the fade of the car being drawn. One per view
//! slot (`race::view_scratch::ViewScratch::face`): the corner values (`face_turned` ..
//! `polygon_mode`), `face_number`, `mapping_a` and `mapping_b`, `face_kind` and `face_unlit`, the
//! fade (`fade_alpha`, `faded_bits`, `faded`), the current course face and the texture
//! coordinates by corner (`corner_uv`, `corner_uv_b`). A face without a second texture copies the
//! second set into its entries unwritten: the last such face's in the same view (none drawn:
//! zeros). Nothing reads them on such a face.

use crate::race::plain_slot::{Merge, PlainSlot, merge_run, merge_value};

/// A view's face values, plain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FaceValues {
    /// The face's halves turned to the view (`race::course_faces`).
    pub turned: [u32; 2],
    /// The face's corners (the fourth -1 for a face of three points).
    pub corners: [u32; 4],
    /// The first texture, its corner order and its back.
    pub texture_a: u32,
    pub order_a: u32,
    pub back_a: u32,
    /// The second texture, its corner order and its back.
    pub texture_b: u32,
    pub order_b: u32,
    pub back_b: u32,
    /// The colour, the light and the see-through (a byte) by corner.
    pub corner_colours: [u32; 4],
    pub corner_lights: [u32; 4],
    pub corner_alphas: [u32; 4],
    /// How `scene::draw_polygon` draws.
    pub polygon_mode: u32,
    /// The two textures' mapping modes (the first dword the mode; the second is not used).
    pub mapping_a: [u32; 2],
    pub mapping_b: [u32; 2],
    /// The object face being listed or drawn.
    pub face_number: u32,
    /// The object face's kind (its head's byte 6) and unlit byte (byte 0xB), set by
    /// `scene::draw_object_faces` for `scene::draw_object_face`.
    pub face_kind: u32,
    pub face_unlit: u32,
    /// The fade of the car being drawn: its alpha (`race::car_draw`; `fade_alpha_moved`), the faded
    /// bits of its model (-1 outside a car's drawing: `faded_bits`) and whether the face being
    /// drawn is faded (`scene::draw_object_face` for `scene::draw_polygon`: `faded`).
    pub fade_alpha: u32,
    pub faded_bits: u32,
    pub faded: u32,
    /// The course face being drawn or listed (`current_course_face`), named by the bad polygon's
    /// log; the objects of two shapes would keep their moving state in the face's normals' place
    /// (`scene::set_object_material`), but no kind in the game's files has that material (way 4:
    /// none in objsinfo, carsinfo, frntinfo or extrinfo).
    pub course_face: u32,
    /// The two textures' coordinates by corner, x then y (`corner_uv`, and
    /// `corner_uv_b`): set for a face's textured sets (`race::face_texture_coordinates`), read into
    /// its polygon entries.
    pub uv_a: [u32; 8],
    pub uv_b: [u32; 8],
    /// `face_depth_key`: the face's depth key, each view's own.
    pub depth_key: u32,
    /// `reflections_kept`: each view's own.
    pub reflections_kept: u32,
    /// `face_reflects`: each view's own.
    pub reflects: u32,
    /// `no_far_clip`: the sky is drawn uncut, each view's own.
    pub no_far_clip: u32,
    /// The faces hidden from the shown view, set
    /// from the face under the car (`race::face_hide_mask`), each view's own.
    pub face_hide_mask: u32,
    /// `face_any_shine`: whether a vertex of the object face
    /// being drawn shines (`scene::draw_object_face`), each view's own.
    pub face_any_shine: u32,
    /// `depth_test` (`RaceView.depth_test`): the face turned the depth test off
    /// (`race::face_depth_test_off`), each view's own.
    pub depth_test: u32,
}

impl FaceValues {
    pub const fn new() -> Self {
        Self {
            turned: [0; 2],
            corners: [0; 4],
            texture_a: 0,
            order_a: 0,
            back_a: 0,
            texture_b: 0,
            order_b: 0,
            back_b: 0,
            corner_colours: [0; 4],
            corner_lights: [0; 4],
            corner_alphas: [0; 4],
            polygon_mode: 0,
            mapping_a: [0; 2],
            mapping_b: [0; 2],
            face_number: 0,
            face_kind: 0,
            face_unlit: 0,
            fade_alpha: 0,
            faded_bits: u32::MAX,
            faded: 0,
            course_face: 0,
            uv_a: [0; 8],
            uv_b: [0; 8],
            depth_key: 0,
            reflections_kept: 0,
            reflects: 0,
            no_far_clip: 0,
            face_hide_mask: 0,
            face_any_shine: 0,
            depth_test: 0,
        }
    }
}

impl Merge for FaceValues {
    fn merge_changes(&mut self, now: &Self, start: &Self) {
        merge_run(&mut self.turned, &now.turned, &start.turned);
        merge_run(&mut self.corners, &now.corners, &start.corners);
        merge_value(&mut self.texture_a, &now.texture_a, &start.texture_a);
        merge_value(&mut self.order_a, &now.order_a, &start.order_a);
        merge_value(&mut self.back_a, &now.back_a, &start.back_a);
        merge_value(&mut self.texture_b, &now.texture_b, &start.texture_b);
        merge_value(&mut self.order_b, &now.order_b, &start.order_b);
        merge_value(&mut self.back_b, &now.back_b, &start.back_b);
        merge_run(
            &mut self.corner_colours,
            &now.corner_colours,
            &start.corner_colours,
        );
        merge_run(
            &mut self.corner_lights,
            &now.corner_lights,
            &start.corner_lights,
        );
        merge_run(
            &mut self.corner_alphas,
            &now.corner_alphas,
            &start.corner_alphas,
        );
        merge_value(
            &mut self.polygon_mode,
            &now.polygon_mode,
            &start.polygon_mode,
        );
        merge_run(&mut self.mapping_a, &now.mapping_a, &start.mapping_a);
        merge_run(&mut self.mapping_b, &now.mapping_b, &start.mapping_b);
        merge_value(&mut self.face_number, &now.face_number, &start.face_number);
        merge_value(&mut self.face_kind, &now.face_kind, &start.face_kind);
        merge_value(&mut self.face_unlit, &now.face_unlit, &start.face_unlit);
        merge_value(&mut self.fade_alpha, &now.fade_alpha, &start.fade_alpha);
        merge_value(&mut self.faded_bits, &now.faded_bits, &start.faded_bits);
        merge_value(&mut self.faded, &now.faded, &start.faded);
        merge_value(&mut self.course_face, &now.course_face, &start.course_face);
        merge_run(&mut self.uv_a, &now.uv_a, &start.uv_a);
        merge_run(&mut self.uv_b, &now.uv_b, &start.uv_b);
        merge_value(&mut self.depth_key, &now.depth_key, &start.depth_key);
        merge_value(
            &mut self.reflections_kept,
            &now.reflections_kept,
            &start.reflections_kept,
        );
        merge_value(&mut self.reflects, &now.reflects, &start.reflects);
        merge_value(&mut self.no_far_clip, &now.no_far_clip, &start.no_far_clip);
        merge_value(
            &mut self.face_hide_mask,
            &now.face_hide_mask,
            &start.face_hide_mask,
        );
        merge_value(
            &mut self.face_any_shine,
            &now.face_any_shine,
            &start.face_any_shine,
        );
        merge_value(&mut self.depth_test, &now.depth_test, &start.depth_test);
    }
}

/// A view's face work.
pub type FaceWork = PlainSlot<FaceValues>;

/// The fourth corner of the last face drawn by any view (-1 for a face of three points), read by
/// `race::norms` at a stage's start-up as `RaceView.corners[3]` was (it should be the face's own: a
/// slip kept as is); before any face was drawn it is the image's first value.
pub(super) fn last_fourth_corner(
    shared: &crate::race::owner_shared::RaceShared,
    scratch: &crate::scene::scratch_store::DrawingScratch,
) -> u32 {
    match scratch.last_face_view() {
        None => shared.view.get().corners[3].get(),
        Some(slot) => scratch.slot(slot).face.borrow().corners[3],
    }
}
