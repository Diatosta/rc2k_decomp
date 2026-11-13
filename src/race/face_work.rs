//! The course face being drawn, a view's own working values (frame plan F4): the corners,
//! the two textures with their orders and backs, the colours, lights and see-through by
//! corner, the polygon mode, the object face's number, kind and unlit byte, and the fade of
//! the car being drawn. One per view slot (`race::view_scratch::ViewScratch::face`). In the
//! original these were `RACE_VIEW`'s fields (`face_turned` .. `polygon_mode`), `RaceVars`'s
//! `face_number`, `mapping_a` and `mapping_b`, the sound spot block's `face_kind` and
//! `face_unlit`, and `fade_alpha_moved`, `faded_bits`, `faded` (the fade), written and
//! read by every view in turn; the current course face (`g_currentCourseFace`); and the
//! texture coordinates by corner (`RACE_VIEW.corner_uv`, `corner_uv_b`). A face without a second
//! texture copies the second set into its entries unwritten, the last such face's in the
//! original, whichever view drew it; now the same view's last (none drawn: zeros). Nothing
//! reads them on such a face: the fans' hashes are the same.

use crate::session::PlainDword;
use std::sync::atomic::{AtomicUsize, Ordering};

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
    /// `g_polygonMode`: how `scene::draw_polygon` draws.
    pub polygon_mode: PlainDword,
    /// The two textures' mapping modes (+0 the mode; the original's pair, +4 not used).
    pub mapping_a: [PlainDword; 2],
    pub mapping_b: [PlainDword; 2],
    /// `g_faceNumber`: the object face being listed or drawn.
    pub face_number: PlainDword,
    /// The object face's kind (its head's byte 6) and unlit byte (byte 0xB), set by
    /// `scene::draw_object_faces` for `scene::draw_object_face` (the original's `g_faceKind`,
    /// `face_unlit` in the sound spot block).
    pub face_kind: PlainDword,
    pub face_unlit: PlainDword,
    /// The fade of the car being drawn: its alpha (`race::car_draw`; `fade_alpha_moved`), the
    /// faded bits of its model (record +0x10, -1 outside a car's drawing: `faded_bits`) and
    /// whether the face being drawn is faded (`scene::draw_object_face` for
    /// `scene::draw_polygon`: `faded`).
    pub fade_alpha: PlainDword,
    pub faded_bits: PlainDword,
    pub faded: PlainDword,
    /// The course face being drawn or listed (`g_currentCourseFace`, 0x5CCCD0), named by the
    /// bad polygon's log; the objects of two shapes would keep their moving state in its
    /// +0x30 .. (`scene::set_object_material`), but no kind in the game's files has that
    /// material (way 4: none in objsinfo, carsinfo, frntinfo or extrinfo).
    pub course_face: PlainDword,
    /// The two textures' coordinates by corner, x then y (`RACE_VIEW.corner_uv`, 0x63DA0C,
    /// and `corner_uv_b`, 0x63DAB8): set for a face's textured sets
    /// (`race::face_texture_coordinates`), read into its polygon entries.
    pub uv_a: [PlainDword; 8],
    pub uv_b: [PlainDword; 8],
    /// `g_faceDepthKey` (`RACE_VIEW.face_depth_key`): the face's depth key, each view's own.
    pub depth_key: PlainDword,
    /// `g_reflectionsKept` (`RACE_VIEW.reflections_kept`): each view's own.
    pub reflections_kept: PlainDword,
    /// `g_faceReflects` (`FRAME_SWITCHES.face_reflects`): each view's own.
    pub reflects: PlainDword,
    /// `no_far_clip` (`FRAME_SWITCHES.no_far_clip`): the sky is drawn uncut, each view's own.
    pub no_far_clip: PlainDword,
    /// The faces hidden from the shown view (`FRAME_WORK.face_hide_mask`, `g_faceHideMask`,
    /// 0x64167C): set from the face under the car (`race::face_hide_mask`), each view's own.
    pub face_hide_mask: PlainDword,
    /// `g_faceAnyShine` (`FRAME_SWITCHES.face_any_shine`, 0x621504): whether a vertex of the
    /// object face being drawn shines (`scene::draw_object_face`), each view's own.
    pub face_any_shine: PlainDword,
    /// `g_depthTest` (`RaceView.depth_test`, 0x63D9E0): the face turned the depth test off
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

/// The slot of the view that last drew a course face (`race::face_corners`), none at first.
static LAST_FACE_VIEW: AtomicUsize = AtomicUsize::new(usize::MAX);

/// The view in slot `slot` is drawing a face.
pub fn note_face_drawn(slot: usize) {
    LAST_FACE_VIEW.store(slot, Ordering::Relaxed);
}

/// The fourth corner of the last face drawn by any view (-1 for a face of three points),
/// read by `race::norms` at a stage's start-up as the original read `RaceView.corners[3]` (it
/// should be the face's own: a slip kept, `notes/bugs.md`); before any face was drawn the
/// image's first value.
pub fn last_fourth_corner(shared: &crate::race::owner_shared::RaceShared) -> u32 {
    match LAST_FACE_VIEW.load(Ordering::Relaxed) {
        usize::MAX => shared.view.get().corners[3].get(),
        slot => crate::race::view_scratch::slot(slot).face.corners[3].get(),
    }
}
