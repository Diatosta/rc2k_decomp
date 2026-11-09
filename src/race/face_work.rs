//! The course face being drawn, a view's own working values (frame plan F4): the corners,
//! the two textures with their orders and backs, the colours, lights and see-through by
//! corner, the polygon mode and the object face's number. One per view slot
//! (`race::view_scratch::ViewScratch::face`). In the original these were `RACE_VIEW`'s
//! fields (`face_turned` .. `polygon_mode`) and `RaceVars`'s `face_number`, `mapping_a` and
//! `mapping_b`, written and read by every view in turn. Not here: the texture coordinates
//! by corner (`RACE_VIEW.corner_uv`, `corner_uv_b`: the faces without a texture read the
//! last face's), `current_course_face` (`scene::set_object_material` in the menus) and
//! `polygon_flags` (`scene::draw_marked_polygon` in the menus), read before a view's own
//! write.

use crate::session::Dword;

pub struct FaceWork {
    /// The face's halves turned to the view (`race::course_faces`).
    pub turned: [Dword; 2],
    /// The face's corners (the fourth -1 for a face of three points).
    pub corners: [Dword; 4],
    /// The first texture, its corner order and its back.
    pub texture_a: Dword,
    pub order_a: Dword,
    pub back_a: Dword,
    /// The second texture, its corner order and its back.
    pub texture_b: Dword,
    pub order_b: Dword,
    pub back_b: Dword,
    /// The colour, the light and the see-through (a byte) by corner.
    pub corner_colours: [Dword; 4],
    pub corner_lights: [Dword; 4],
    pub corner_alphas: [Dword; 4],
    /// `g_polygonMode`: how `scene::draw_polygon` draws.
    pub polygon_mode: Dword,
    /// The two textures' mapping modes (+0 the mode; the original's pair, +4 not used).
    pub mapping_a: [Dword; 2],
    pub mapping_b: [Dword; 2],
    /// `g_faceNumber`: the object face being listed or drawn.
    pub face_number: Dword,
}

impl FaceWork {
    pub const fn new() -> Self {
        Self {
            turned: [const { Dword::new(0) }; 2],
            corners: [const { Dword::new(0) }; 4],
            texture_a: Dword::new(0),
            order_a: Dword::new(0),
            back_a: Dword::new(0),
            texture_b: Dword::new(0),
            order_b: Dword::new(0),
            back_b: Dword::new(0),
            corner_colours: [const { Dword::new(0) }; 4],
            corner_lights: [const { Dword::new(0) }; 4],
            corner_alphas: [const { Dword::new(0) }; 4],
            polygon_mode: Dword::new(0),
            mapping_a: [const { Dword::new(0) }; 2],
            mapping_b: [const { Dword::new(0) }; 2],
            face_number: Dword::new(0),
        }
    }
}
