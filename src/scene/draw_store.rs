//! The scene's draw state: the view window, scale and depths (`scene::set_view_window`), the
//! polygon being drawn and its clipping (`scene::clip`, `scene::draw_polygon`), the turns'
//! sines, the light, and the x87 code's shared work dword (`scratch`), held by
//! `scene::owner::Drawing::scene`. Plain cells: a drawing store (`session::DrawShared`),
//! written on the game's thread between frames (mostly the physics' work dwords) and read by
//! the drawing parts through their `scene::workers::DrawReads` (the polygons' alpha, the fade and clip
//! factors, `v3_driver_fix`).

use crate::memory::cells::ZeroCells;
use crate::memory::session::{PlainDword, PlainFloat};

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct SceneDraw {
    /// `scene::clip`'s work; `race::car_shadow` keeps a vertex's pieces in [2], [4], [10],
    /// [11]. Unused: each view has its own in `scene::polygon_work`.
    pub clip_work: [PlainFloat; 12],
    /// The view window's offsets x, y (`scene::set_view_window`; the menus and
    /// `race::frame_views` set them). Unused: each view has its own in `scene::view_window`.
    pub view_offset: [PlainDword; 2],
    /// A work dword (`physics::engine_torque`, `physics::wheel_springs`);
    /// `race::find_present_courses`' four-byte probe.
    pub work_dword: PlainDword,
    /// The motion's length (`physics::limit_motion`).
    pub motion_length: PlainFloat,
    /// Five dwords only the debug wear bars (`debug::`) use; the scene's code has none (a gap).
    pub unknown_40: [PlainDword; 5],
    /// The camera's heading, saved round a step (`physics::camera_avoid_ground`).
    pub camera_heading_saved: PlainDword,
    /// The camera's heading kept from step to step (`physics::camera_avoid_ground`).
    pub camera_heading_kept: PlainDword,
    /// Five words `physics::camera_avoid_ground` keeps.
    pub camera_saved: [PlainDword; 5],
    /// Four dwords; no code uses them (a gap).
    pub unknown_70: [PlainDword; 4],
    /// The fog colour (`race::frame_draw`). Unused: each view has its own in `scene::fog::Fog`.
    pub fog_colour: PlainDword,
    /// One dword between the fog colour and the clip's kept mask; no code uses it (a gap).
    pub unknown_84: PlainDword,
    /// The clip's kept mask (`scene::clip`). Unused: each view has its own in
    /// `scene::polygon_work`.
    pub clip_kept_mask: PlainDword,
    /// The x87 code's shared work dword: every converted helper that stored to it still does.
    pub scratch: PlainDword,
    /// Three dwords after `scratch`; no code uses them (a gap).
    pub unknown_90: [PlainDword; 3],
    /// A work float (`physics::drive_torque_share`, `physics::drive_grip_forces`,
    /// `physics::drive_wheels_step`).
    pub work_float_a: PlainFloat,
    /// A work float (the same); `race::frame_draw`'s far limit as a float.
    pub work_float_b: PlainFloat,
    /// Seven dwords only the debug wear bars (`debug::wear_page`) use; the scene's code
    /// has none (a gap).
    pub unknown_a4: [PlainDword; 7],
    /// A wheel's surface point: x, then y and z less x (`physics::wheel_surfaces`).
    pub surface_point: [PlainDword; 3],
    /// 12 dwords after the wheel's surface point; no code uses them (a gap).
    pub unknown_cc: [PlainDword; 12],
    /// A face's sort key (`scene::face_sort_key`). Unused: each view has its own in
    /// `scene::polygon_work`.
    pub face_sort_key: PlainDword,
    /// The view's zoom, a whole number (0x100; the menus set it), in the scale. Unused: each
    /// view has its own in `scene::view_window`.
    pub view_zoom: PlainDword,
    /// One dword between the zoom and the scale; no code uses it, and the view's copy
    /// (`scene::view_window`) is never read either (a gap).
    pub unknown_104: PlainDword,
    /// The view's scale (`scene::set_view_window`, `scene::project_point`). Unused: each view
    /// has its own in `scene::view_window`.
    pub view_scale: PlainFloat,
    /// The view window's width and height. Unused: each view has its own in
    /// `scene::view_window`.
    pub view_size: [PlainDword; 2],
    /// The window's middle, x and y (half of each, unsigned). Unused: each view has its own in
    /// `scene::view_window`.
    pub view_middle: [PlainDword; 2],
    /// The middle as floats (`scene::project_point`'s centre; `draw_polygon`'s mirror). Unused:
    /// each view has its own in `scene::view_window`.
    pub view_centre: [PlainFloat; 2],
    /// The next free record of the polygon's pool. Unused: each view has its own in
    /// `scene::polygon_work`.
    pub polygon_next_record: PlainDword,
    /// The light's direction. Unused: each view has its own in `scene::object_light`.
    pub light_direction: [PlainFloat; 3],
    /// Two dwords after the light direction; no code uses them (a gap).
    pub unknown_134: [PlainDword; 2],
    /// The polygon's two texture words (`scene::draw_polygon`). Unused: each view has its own
    /// in `scene::polygon_work`.
    pub polygon_textures: [PlainDword; 2],
    /// The texture page ("tpage"; `scene::look_up_texture`; the loading bar's switch). Unused:
    /// each view has its own in `scene::polygon_work`.
    pub texture_page: PlainDword,
    /// One dword between the texture page and the polygon colour; no code uses it (a gap).
    pub unknown_148: PlainDword,
    /// The polygon's colour. Unused: each view has its own in `scene::polygon_work`.
    pub polygon_colour: PlainDword,
    /// The polygon's alpha (set by `hud::draw_view_strip` only, read by every view).
    pub polygon_alpha: PlainDword,
    /// The polygon's texture. Unused: each view has its own in `scene::polygon_work`.
    pub polygon_texture: PlainDword,
    /// The face's shine (`scene::draw_object_face`). Unused: each view has its own in
    /// `scene::polygon_work`.
    pub face_shine: PlainDword,
    /// A face's word (`scene::draw_object_face`). Unused: each view has its own in
    /// `scene::polygon_work`.
    pub face_word: PlainDword,
    /// The polygon's two side switches (`scene::draw_polygon`, the quads). Unused: each view
    /// has its own in `scene::polygon_work`.
    pub polygon_sides: [PlainDword; 2],
    /// Kept by `scene::draw_polygon`. Unused: each view has its own in `scene::polygon_work`.
    pub polygon_kept: [PlainDword; 3],
    /// The polygon's count of points outside. Unused: each view has its own in
    /// `scene::polygon_work`.
    pub polygon_outside_count: PlainDword,
    /// The polygon's vertex count. Unused: each view has its own in `scene::polygon_work`.
    pub polygon_vertex_count: PlainDword,
    /// Six dwords before the next entry; no code uses them (a gap).
    pub unknown_17c: [PlainDword; 6],
    /// The next free entry of the work table. Unused: each view has its own in
    /// `scene::polygon_work`.
    pub polygon_next_entry: PlainDword,
    /// The polygon's clip mask. Unused: each view has its own in `scene::polygon_work`.
    pub polygon_clip_mask: PlainDword,
    /// The clip's plane (`scene::clip`). Unused: each view has its own in
    /// `scene::polygon_work`.
    pub clip_plane: PlainDword,
    /// Clipped by the screen ("donesuth"). Unused: each view has its own in
    /// `scene::polygon_work`.
    pub screen_clipped: PlainDword,
    /// Clipped by depth ("donezcut"). Unused: each view has its own in `scene::polygon_work`.
    pub depth_clipped: PlainDword,
    /// Two dwords after the depth-clipped flag; no code uses them (a gap).
    pub unknown_1a8: [PlainDword; 2],
    /// A list entry's third word (`scene::list_object_face`). Unused: each view has its own in
    /// `scene::polygon_work`.
    pub entry_third: PlainDword,
    /// Whether the polygon was drawn. Unused: each view has its own in `scene::polygon_work`.
    pub polygon_drawn: PlainDword,
    /// The rectangle to clear: left, top, right, bottom (`race::frame_views`). Unused: each
    /// view has its own in `scene::polygon_work::PolygonWork`.
    pub cleared_rectangle: [PlainDword; 4],
    /// Sine and cosine of the x turn (`scene::matrix`).
    pub turn_x_work: [PlainFloat; 2],
    /// The same, y.
    pub turn_y_work: [PlainFloat; 2],
    /// The same, z (`scene::view_matrix`).
    pub turn_z_work: [PlainFloat; 2],
    /// 15 dwords after the turns' sines; no code uses them (a gap).
    pub unknown_1e0: [PlainDword; 15],
    /// The amount (alpha) byte, +0x21, of the vertex a clip crossing makes: the two ends' bytes
    /// blended by the crossing's share (`race::car_shadow`). Unused: each view has its own in
    /// `scene::polygon_work::PolygonWork`.
    pub crossing_amount: PlainDword,
    /// Three dwords after the vertex byte; no code uses them (a gap).
    pub unknown_220: [PlainDword; 3],
    /// `scene::clip`'s second output run (five words written back from its work);
    /// `race::car_shadow` keeps the new crossing vertex's fog byte (+0x20, blended like the
    /// amount) in [0]. Unused: each view has its own in `scene::polygon_work`.
    pub clip_out_words: [PlainDword; 5],
    /// Two dwords before the farthest depth; no code uses them (a gap).
    pub unknown_240: [PlainDword; 2],
    /// The farthest depth (`scene::set_view_window`). Unused: each view has its own in
    /// `scene::view_window`.
    pub farthest: PlainFloat,
    /// 1 over it. Unused: each view has its own in `scene::view_window`.
    pub one_over_farthest: PlainFloat,
    /// It times `fade_a` over `fade_b` over 65536. Unused: each view has its own in
    /// `scene::view_window`.
    pub farthest_scaled: PlainDword,
    /// The clip rectangle as whole numbers: top, bottom, left, right. Unused: each view has its
    /// own in `scene::view_window`.
    pub clip_edges: [PlainDword; 4],
    /// The same as floats. Unused: each view has its own in `scene::view_window`.
    pub clip_screen: [PlainFloat; 4],
    /// `view_offset` as floats. Unused: each view has its own in `scene::view_window`.
    pub view_offset_float: [PlainFloat; 2],
    /// Two dwords after the offsets as floats; no code uses them (a gap).
    pub unknown_27c: [PlainDword; 2],
    /// (`scene::draw_object_faces`). Unused: each view has its own in `scene::view_window`.
    pub faces_done: PlainDword,
    /// Six dwords before `fade_b`; no code uses them (a gap).
    pub unknown_288: [PlainDword; 6],
    /// (`scene::set_view_window`).
    pub fade_b: PlainFloat,
    /// The depth divisor: 100 less a held setting, times a constant. Unused: each
    /// view has its own in `scene::view_window`.
    pub depth_divisor: PlainFloat,
    /// (`scene::set_view_window`).
    pub fade_a: PlainFloat,
    /// `clip_farthest` is the farthest times it.
    pub clip_factor: PlainFloat,
    /// The farthest clip depth, rounded. Unused: each view has its own in `scene::view_window`.
    pub clip_farthest: PlainDword,
    /// The mirror picture's texture.
    pub mirror_picture: PlainDword,
    /// The `v3driverfix` setting.
    pub v3_driver_fix: PlainDword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl SceneDraw {
    /// The store with its first values (every field not named here starts at zero).
    pub const fn new() -> Self {
        Self {
            unknown_70: [
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0xFFFFFF),
            ],
            scratch: PlainDword::new(0x10000),
            work_float_b: PlainFloat::new(f32::from_bits(0x00010000)),
            unknown_a4: [
                PlainDword::new(0x10000),
                PlainDword::new(0x10000),
                PlainDword::new(0x10000),
                PlainDword::new(0x10000),
                PlainDword::new(0x10000),
                PlainDword::new(0x10000),
                PlainDword::new(0x10000),
            ],
            surface_point: [
                PlainDword::new(0x10000),
                PlainDword::new(0x10000),
                PlainDword::new(0x10000),
            ],
            polygon_colour: PlainDword::new(0xFFFFFF),
            polygon_alpha: PlainDword::new(0xFF),
            cleared_rectangle: [
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0x31F),
                PlainDword::new(0x257),
            ],
            unknown_288: [
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0x3E000000),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
            ],
            fade_b: PlainFloat::new(1.5),
            depth_divisor: PlainFloat::new(0.91),
            fade_a: PlainFloat::new(0.9),
            clip_factor: PlainFloat::new(0.9),
            ..ZeroCells::ZERO
        }
    }
}
