//! The course face being drawn, a view's own working values (frame plan F4): the corners,
//! the two textures with their orders and backs, the colours, lights and see-through by
//! corner, the polygon mode, the object face's number, kind and unlit byte, and the fade of
//! the car being drawn. One per view slot (`race::view_scratch::ViewScratch::face`). In the
//! original these were `RACE_VIEW`'s fields (`face_turned` .. `polygon_mode`), `RaceVars`'s
//! `face_number`, `mapping_a` and `mapping_b`, the sound spot block's `face_kind` and
//! `face_unlit`, and `dword_606284`, `dword_608CAC`, `dword_63660C` (the fade), written and
//! read by every view in turn. Not here: the texture coordinates by corner (`RACE_VIEW.corner_uv`, `corner_uv_b`: the faces without a texture read the
//! last face's) and `current_course_face` (`scene::set_object_material` in the menus), read
//! before a view's own write.

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
    /// The object face's kind (its head's byte 6) and unlit byte (byte 0xB), set by
    /// `scene::draw_object_faces` for `scene::draw_object_face` (the original's `g_faceKind`,
    /// `dword_6D18D0` in the sound spot block).
    pub face_kind: Dword,
    pub face_unlit: Dword,
    /// The fade of the car being drawn: its alpha (`race::car_draw`; `dword_606284`), the
    /// faded bits of its model (record +0x10, -1 outside a car's drawing: `dword_608CAC`) and
    /// whether the face being drawn is faded (`scene::draw_object_face` for
    /// `scene::draw_polygon`: `dword_63660C`).
    pub fade_alpha: Dword,
    pub faded_bits: Dword,
    pub faded: Dword,
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
            face_kind: Dword::new(0),
            face_unlit: Dword::new(0),
            fade_alpha: Dword::new(0),
            faded_bits: Dword::new(u32::MAX),
            faded: Dword::new(0),
        }
    }
}
