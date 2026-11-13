//! The scene's draw state (IDA 0x623F80 .. 0x62423C): the view window, scale and
//! depths (`scene::set_view_window`), the polygon being drawn and its clipping (`scene::clip`,
//! `scene::draw_polygon`), the turns' sines, the light, and the x87 code's shared work dword
//! (`scratch`, 0x62400C), held by `scene::owner::Drawing::scene`. Plain cells: a drawing store (`session::DrawShared`),
//! written on the game's thread between frames (mostly the physics' work dwords) and read by
//! the drawing parts through their `workers::DrawReads` (the polygons' alpha, the fade and
//! clip factors, `v3_driver_fix`). Names are readings unless IDA's.

use crate::cells::ZeroCells;
use crate::session::{PlainDword, PlainFloat};

#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct SceneDraw {
    /// (0x623F80): `scene::clip`'s work (`flt_623F80` ..); `race::car_shadow` keeps a vertex's pieces in [2], [4], [10], [11]; now each view's (`scene::polygon_work`); unused.
    pub clip_work: [PlainFloat; 12],
    /// (0x623FB0): the view window's offsets x, y (`scene::set_view_window`; the menus and `race::frame_views` set them); now each view's (`scene::view_window`); unused.
    pub view_offset: [PlainDword; 2],
    /// (0x623FB8): a work dword (`physics::engine_torque`, `physics::wheel_springs`); `race::find_present_courses`' four-byte probe.
    pub work_dword: PlainDword,
    /// (0x623FBC): the motion's length (`physics::limit_motion`).
    pub motion_length: PlainFloat,
    /// (0x623FC0): five dwords; the original's only reference is the debug wear-bar code at 0x429936 (`sub_4298BD`, now `debug::`), the scene's code has none (a gap).
    pub unknown_40: [PlainDword; 5],
    /// (0x623FD4): the camera's heading, saved round a step (`physics::camera_avoid_ground`).
    pub camera_heading_saved: PlainDword,
    /// (0x623FD8): the camera's heading kept from step to step (`physics::camera_avoid_ground`).
    pub camera_heading_kept: PlainDword,
    /// (0x623FDC): five words kept from 0x71AEC8 (`physics::camera_avoid_ground`).
    pub camera_saved: [PlainDword; 5],
    /// (0x623FF0): four dwords; no code references them (a gap).
    pub unknown_70: [PlainDword; 4],
    /// (0x624000): `g_fogColour` (`race::frame_draw`); now each view's (`scene::fog::Fog`); unused.
    pub fog_colour: PlainDword,
    /// (0x624004): one dword between the fog colour and the clip's kept mask; no code references it (a gap).
    pub unknown_84: PlainDword,
    /// (0x624008): the clip's kept mask (`scene::clip`); now each view's (`scene::polygon_work`); unused.
    pub clip_kept_mask: PlainDword,
    /// (0x62400C): the x87 code's shared work dword (IDA `g_scratch`): every converted helper that stored to it still does.
    pub scratch: PlainDword,
    /// (0x624010): three dwords after `scratch`; no code references them (a gap).
    pub unknown_90: [PlainDword; 3],
    /// (0x62401C): a work float (`physics::drive_torque_share`, `physics::drive_grip_forces`, `physics::drive_wheels_step`).
    pub work_float_a: PlainFloat,
    /// (0x624020): a work float (the same); `race::frame_draw`'s far limit (`dword_63F17C` as a float).
    pub work_float_b: PlainFloat,
    /// (0x624024): seven dwords; the original's only reference is the debug wear bars (`debug::draw_wear_bars` at 0x62403C), the scene's code has none (a gap).
    pub unknown_a4: [PlainDword; 7],
    /// (0x624040): a wheel's surface point: x, then y and z less x (`physics::wheel_surfaces`).
    pub surface_point: [PlainDword; 3],
    /// (0x62404C): 12 dwords after the wheel's surface point; no code references them (a gap).
    pub unknown_cc: [PlainDword; 12],
    /// (0x62407C): `g_faceSortKey` (`scene::face_sort_key`); now each view's (`scene::polygon_work`); unused.
    pub face_sort_key: PlainDword,
    /// (0x624080): the view's zoom, a whole number (0x100; the menus set it), in the scale; now each view's (`scene::view_window`); unused.
    pub view_zoom: PlainDword,
    /// (0x624084): one dword between the zoom and the scale; no code of the original or the port references it (a gap); the view's copy (`scene::view_window`) is never read either.
    pub unknown_104: PlainDword,
    /// (0x624088): the view's scale (`scene::set_view_window`, `scene::project_point`); now each view's (`scene::view_window`); unused.
    pub view_scale: PlainFloat,
    /// (0x62408C): the view window's width and height; now each view's (`scene::view_window`); unused.
    pub view_size: [PlainDword; 2],
    /// (0x624094): the window's middle, x and y (half of each, unsigned); now each view's (`scene::view_window`); unused.
    pub view_middle: [PlainDword; 2],
    /// (0x62409C): the middle as floats (`scene::project_point`'s centre; `draw_polygon`'s mirror); now each view's (`scene::view_window`); unused.
    pub view_centre: [PlainFloat; 2],
    /// (0x6240A4): `g_polygonNextRecord`; now each view's (`scene::polygon_work`); unused.
    pub polygon_next_record: PlainDword,
    /// (0x6240A8): `g_lightDirection`; now each view's (`scene::object_light`), this
    /// one not used.
    pub light_direction: [PlainFloat; 3],
    /// (0x6240B4): two dwords after the light direction; no code references them (a gap).
    pub unknown_134: [PlainDword; 2],
    /// (0x6240BC): the polygon's two texture words (`scene::draw_polygon`); now each view's (`scene::polygon_work`); unused.
    pub polygon_textures: [PlainDword; 2],
    /// (0x6240C4): the texture page ("tpage"; `scene::look_up_texture`; the loading bar's switch); now each view's (`scene::polygon_work`); unused.
    pub texture_page: PlainDword,
    /// (0x6240C8): one dword between the texture page and the polygon colour; no code references it (a gap).
    pub unknown_148: PlainDword,
    /// (0x6240CC): `g_polygonColour`; now each view's (`scene::polygon_work`); unused.
    pub polygon_colour: PlainDword,
    /// (0x6240D0): `g_polygonAlpha` (set by `hud::draw_view_strip` only, read by every view).
    pub polygon_alpha: PlainDword,
    /// (0x6240D4): `g_polygonTexture`; now each view's (`scene::polygon_work`); unused.
    pub polygon_texture: PlainDword,
    /// (0x6240D8): `g_faceShine` (`scene::draw_object_face`); now each view's (`scene::polygon_work`); unused.
    pub face_shine: PlainDword,
    /// (0x6240DC): a face's word (`scene::draw_object_face`); now each view's (`scene::polygon_work`); unused.
    pub face_word: PlainDword,
    /// (0x6240E0): the polygon's two side switches (`scene::draw_polygon`, the quads); now each view's (`scene::polygon_work`); unused.
    pub polygon_sides: [PlainDword; 2],
    /// (0x6240E8): kept by `scene::draw_polygon`; now each view's (`scene::polygon_work`); unused.
    pub polygon_kept: [PlainDword; 3],
    /// (0x6240F4): `g_polygonOutsideCount`; now each view's (`scene::polygon_work`); unused.
    pub polygon_outside_count: PlainDword,
    /// (0x6240F8): `g_polygonVertexCount`; now each view's (`scene::polygon_work`); unused.
    pub polygon_vertex_count: PlainDword,
    /// (0x6240FC): six dwords before the next entry; no code references them (a gap).
    pub unknown_17c: [PlainDword; 6],
    /// (0x624114): `g_polygonNextEntry`; now each view's (`scene::polygon_work`); unused.
    pub polygon_next_entry: PlainDword,
    /// (0x624118): `g_polygonClipMask` ("cuttest"); now each view's (`scene::polygon_work`); unused.
    pub polygon_clip_mask: PlainDword,
    /// (0x62411C): the clip's plane (`scene::clip`); now each view's (`scene::polygon_work`); unused.
    pub clip_plane: PlainDword,
    /// (0x624120): clipped by the screen ("donesuth"); now each view's (`scene::polygon_work`); unused.
    pub screen_clipped: PlainDword,
    /// (0x624124): clipped by depth ("donezcut"); now each view's (`scene::polygon_work`); unused.
    pub depth_clipped: PlainDword,
    /// (0x624128): two dwords after the depth-clipped flag; no code references them (a gap).
    pub unknown_1a8: [PlainDword; 2],
    /// (0x624130): a list entry's third word (`scene::list_object_face`); now each view's (`scene::polygon_work`); unused.
    pub entry_third: PlainDword,
    /// (0x624134): `g_polygonDrawn`; now each view's (`scene::polygon_work`); unused.
    pub polygon_drawn: PlainDword,
    /// (0x624138): the rectangle to clear: left, top, right, bottom (`race::frame_views`); now each view's (`scene::polygon_work::PolygonWork`); unused.
    pub cleared_rectangle: [PlainDword; 4],
    /// (0x624148): sine and cosine of the x turn (`scene::matrix`).
    pub turn_x_work: [PlainFloat; 2],
    /// (0x624150): the same, y.
    pub turn_y_work: [PlainFloat; 2],
    /// (0x624158): the same, z (`scene::view_matrix`).
    pub turn_z_work: [PlainFloat; 2],
    /// (0x624160): 15 dwords after the turns' sines; no code references them (a gap).
    pub unknown_1e0: [PlainDword; 15],
    /// (0x62419C): the amount (alpha) byte, +0x21, of the vertex a clip crossing makes: the two ends' bytes blended by the crossing's share (`race::car_shadow`); now each view's (`scene::polygon_work::PolygonWork`); unused.
    pub crossing_amount: PlainDword,
    /// (0x6241A0): three dwords after the vertex byte; no code references them (a gap).
    pub unknown_220: [PlainDword; 3],
    /// (0x6241AC): `scene::clip`'s second output run (five words written back from its work); `race::car_shadow` keeps the new crossing vertex's fog byte (+0x20, blended like the amount) in [0]; now each view's (`scene::polygon_work`); unused.
    pub clip_out_words: [PlainDword; 5],
    /// (0x6241C0): two dwords before the farthest depth; no code references them (a gap).
    pub unknown_240: [PlainDword; 2],
    /// (0x6241C8): the farthest depth (`scene::set_view_window`); now each view's (`scene::view_window`); unused.
    pub farthest: PlainFloat,
    /// (0x6241CC): 1 over it; now each view's (`scene::view_window`); unused.
    pub one_over_farthest: PlainFloat,
    /// (0x6241D0): it times `fade_a` over `fade_b` over 65536; now each view's (`scene::view_window`); unused.
    pub farthest_scaled: PlainDword,
    /// (0x6241D4): the clip rectangle as whole numbers: top, bottom, left, right; now each view's (`scene::view_window`); unused.
    pub clip_edges: [PlainDword; 4],
    /// (0x6241E4): the same as floats (`g_clipScreenTop` .. `Right`); now each view's (`scene::view_window`); unused.
    pub clip_screen: [PlainFloat; 4],
    /// (0x6241F4): `view_offset` as floats; now each view's (`scene::view_window`); unused.
    pub view_offset_float: [PlainFloat; 2],
    /// (0x6241FC): two dwords after the offsets as floats; no code references them (a gap).
    pub unknown_27c: [PlainDword; 2],
    /// (0x624204): (`scene::draw_object_faces`); now each view's (`scene::view_window`); unused.
    pub faces_done: PlainDword,
    /// (0x624208): six dwords before `fade_b`; no code references them (a gap).
    pub unknown_288: [PlainDword; 6],
    /// (0x624220): (`scene::set_view_window`).
    pub fade_b: PlainFloat,
    /// (0x624224): (100 less `dword_60AC80`, held) times `flt_5C5CFC`; now each view's (`scene::view_window`); unused.
    pub depth_divisor: PlainFloat,
    /// (0x624228): (`scene::set_view_window`).
    pub fade_a: PlainFloat,
    /// (0x62422C): `clip_farthest` is the farthest times it.
    pub clip_factor: PlainFloat,
    /// (0x624230): `g_clipFarthest`, rounded; now each view's (`scene::view_window`); unused.
    pub clip_farthest: PlainDword,
    /// (0x624234): `g_mirrorPicture`: the mirror picture's texture.
    pub mirror_picture: PlainDword,
    /// (0x624238): the `v3driverfix` setting.
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
