//! The scene's draw state (IDA 0x623F80 .. 0x62423C): the view window, scale and
//! depths (`scene::set_view_window`), the polygon being drawn and its clipping (`scene::clip`,
//! `scene::draw_polygon`), the turns' sines, the light, and the x87 code's shared work dword
//! (`scratch`, 0x62400C). `#[repr(C)]` with the image's layout, over the race block's bytes
//! (`DRAW`), as `CAR`. Names are readings unless IDA's.

use crate::session::{Dword, Float};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SceneDraw {
    /// +0x0 (0x623F80): `scene::clip`'s work (`flt_623F80` ..); `race::car_shadow` keeps a vertex's pieces in [2], [4], [10], [11]; now each view's (`scene::polygon_work`); unused.
    pub clip_work: [Float; 12],
    /// +0x30 (0x623FB0): the view window's offsets x, y (`scene::set_view_window`; the menus and `race::frame_views` set them); now each view's (`scene::view_window`); unused.
    pub view_offset: [Dword; 2],
    /// +0x38 (0x623FB8): a work dword (`physics::engine_torque`, `physics::wheel_springs`); `race::find_present_courses`' four-byte probe.
    pub work_dword: Dword,
    /// +0x3c (0x623FBC): the motion's length (`physics::limit_motion`).
    pub motion_length: Float,
    /// +0x40 (0x623FC0): five dwords; the original's only reference is the debug wear-bar code at 0x429936 (`sub_4298BD`, now `debug::`), the scene's code has none (a gap).
    pub unknown_40: [Dword; 5],
    /// +0x54 (0x623FD4): the camera's heading, saved round a step (`physics::camera_avoid_ground`).
    pub camera_heading_saved: Dword,
    /// +0x58 (0x623FD8): the camera's heading kept from step to step (`physics::camera_avoid_ground`).
    pub camera_heading_kept: Dword,
    /// +0x5c (0x623FDC): five words kept from 0x71AEC8 (`physics::camera_avoid_ground`).
    pub camera_saved: [Dword; 5],
    /// +0x70 (0x623FF0): four dwords; no code references them (a gap).
    pub unknown_70: [Dword; 4],
    /// +0x80 (0x624000): `g_fogColour` (`race::frame_draw`); now each view's (`scene::fog::Fog`); unused.
    pub fog_colour: Dword,
    /// +0x84 (0x624004): one dword between the fog colour and the clip's kept mask; no code references it (a gap).
    pub unknown_84: Dword,
    /// +0x88 (0x624008): the clip's kept mask (`scene::clip`); now each view's (`scene::polygon_work`); unused.
    pub clip_kept_mask: Dword,
    /// +0x8c (0x62400C): the x87 code's shared work dword (IDA `g_scratch`): every converted helper that stored to it still does.
    pub scratch: Dword,
    /// +0x90 (0x624010): three dwords after `scratch`; no code references them (a gap).
    pub unknown_90: [Dword; 3],
    /// +0x9c (0x62401C): a work float (`physics::drive_torque_share`, `physics::drive_grip_forces`, `physics::drive_wheels_step`).
    pub work_float_a: Float,
    /// +0xa0 (0x624020): a work float (the same); `race::frame_draw`'s far limit (`dword_63F17C` as a float).
    pub work_float_b: Float,
    /// +0xa4 (0x624024): seven dwords; the original's only reference is the debug wear bars (`debug::draw_wear_bars` at 0x62403C), the scene's code has none (a gap).
    pub unknown_a4: [Dword; 7],
    /// +0xc0 (0x624040): a wheel's surface point: x, then y and z less x (`physics::wheel_surfaces`).
    pub surface_point: [Dword; 3],
    /// +0xcc (0x62404C): 12 dwords after the wheel's surface point; no code references them (a gap).
    pub unknown_cc: [Dword; 12],
    /// +0xfc (0x62407C): `g_faceSortKey` (`scene::face_sort_key`); now each view's (`scene::polygon_work`); unused.
    pub face_sort_key: Dword,
    /// +0x100 (0x624080): the view's zoom, a whole number (0x100; the menus set it), in the scale; now each view's (`scene::view_window`); unused.
    pub view_zoom: Dword,
    /// +0x104 (0x624084): one dword between the zoom and the scale; no code of the original or the port references it (a gap); the view's copy (`scene::view_window`) is never read either.
    pub unknown_104: Dword,
    /// +0x108 (0x624088): the view's scale (`scene::set_view_window`, `scene::project_point`); now each view's (`scene::view_window`); unused.
    pub view_scale: Float,
    /// +0x10c (0x62408C): the view window's width and height; now each view's (`scene::view_window`); unused.
    pub view_size: [Dword; 2],
    /// +0x114 (0x624094): the window's middle, x and y (half of each, unsigned); now each view's (`scene::view_window`); unused.
    pub view_middle: [Dword; 2],
    /// +0x11c (0x62409C): the middle as floats (`scene::project_point`'s centre; `draw_polygon`'s mirror); now each view's (`scene::view_window`); unused.
    pub view_centre: [Float; 2],
    /// +0x124 (0x6240A4): `g_polygonNextRecord`; now each view's (`scene::polygon_work`); unused.
    pub polygon_next_record: Dword,
    /// +0x128 (0x6240A8): `g_lightDirection`; now each view's (`scene::object_light`), this
    /// one not used.
    pub light_direction: [Float; 3],
    /// +0x134 (0x6240B4): two dwords after the light direction; no code references them (a gap).
    pub unknown_134: [Dword; 2],
    /// +0x13c (0x6240BC): the polygon's two texture words (`scene::draw_polygon`); now each view's (`scene::polygon_work`); unused.
    pub polygon_textures: [Dword; 2],
    /// +0x144 (0x6240C4): the texture page ("tpage"; `scene::look_up_texture`; the loading bar's switch); now each view's (`scene::polygon_work`); unused.
    pub texture_page: Dword,
    /// +0x148 (0x6240C8): one dword between the texture page and the polygon colour; no code references it (a gap).
    pub unknown_148: Dword,
    /// +0x14c (0x6240CC): `g_polygonColour`; now each view's (`scene::polygon_work`); unused.
    pub polygon_colour: Dword,
    /// +0x150 (0x6240D0): `g_polygonAlpha` (set by `hud::draw_view_strip` only, read by every view).
    pub polygon_alpha: Dword,
    /// +0x154 (0x6240D4): `g_polygonTexture`; now each view's (`scene::polygon_work`); unused.
    pub polygon_texture: Dword,
    /// +0x158 (0x6240D8): `g_faceShine` (`scene::draw_object_face`); now each view's (`scene::polygon_work`); unused.
    pub face_shine: Dword,
    /// +0x15c (0x6240DC): a face's word (`scene::draw_object_face`); now each view's (`scene::polygon_work`); unused.
    pub face_word: Dword,
    /// +0x160 (0x6240E0): the polygon's two side switches (`scene::draw_polygon`, the quads); now each view's (`scene::polygon_work`); unused.
    pub polygon_sides: [Dword; 2],
    /// +0x168 (0x6240E8): kept by `scene::draw_polygon`; now each view's (`scene::polygon_work`); unused.
    pub polygon_kept: [Dword; 3],
    /// +0x174 (0x6240F4): `g_polygonOutsideCount`; now each view's (`scene::polygon_work`); unused.
    pub polygon_outside_count: Dword,
    /// +0x178 (0x6240F8): `g_polygonVertexCount`; now each view's (`scene::polygon_work`); unused.
    pub polygon_vertex_count: Dword,
    /// +0x17c (0x6240FC): six dwords before the next entry; no code references them (a gap).
    pub unknown_17c: [Dword; 6],
    /// +0x194 (0x624114): `g_polygonNextEntry`; now each view's (`scene::polygon_work`); unused.
    pub polygon_next_entry: Dword,
    /// +0x198 (0x624118): `g_polygonClipMask` ("cuttest"); now each view's (`scene::polygon_work`); unused.
    pub polygon_clip_mask: Dword,
    /// +0x19c (0x62411C): the clip's plane (`scene::clip`); now each view's (`scene::polygon_work`); unused.
    pub clip_plane: Dword,
    /// +0x1a0 (0x624120): clipped by the screen ("donesuth"); now each view's (`scene::polygon_work`); unused.
    pub screen_clipped: Dword,
    /// +0x1a4 (0x624124): clipped by depth ("donezcut"); now each view's (`scene::polygon_work`); unused.
    pub depth_clipped: Dword,
    /// +0x1a8 (0x624128): two dwords after the depth-clipped flag; no code references them (a gap).
    pub unknown_1a8: [Dword; 2],
    /// +0x1b0 (0x624130): a list entry's third word (`scene::list_object_face`); now each view's (`scene::polygon_work`); unused.
    pub entry_third: Dword,
    /// +0x1b4 (0x624134): `g_polygonDrawn`; now each view's (`scene::polygon_work`); unused.
    pub polygon_drawn: Dword,
    /// +0x1b8 (0x624138): the rectangle to clear: left, top, right, bottom (`race::frame_views`); now each view's (`scene::polygon_work::PolygonWork`); unused.
    pub cleared_rectangle: [Dword; 4],
    /// +0x1c8 (0x624148): sine and cosine of the x turn (`scene::matrix`).
    pub turn_x_work: [Float; 2],
    /// +0x1d0 (0x624150): the same, y.
    pub turn_y_work: [Float; 2],
    /// +0x1d8 (0x624158): the same, z (`scene::view_matrix`).
    pub turn_z_work: [Float; 2],
    /// +0x1e0 (0x624160): 15 dwords after the turns' sines; no code references them (a gap).
    pub unknown_1e0: [Dword; 15],
    /// +0x21c (0x62419C): the amount (alpha) byte, +0x21, of the vertex a clip crossing makes: the two ends' bytes blended by the crossing's share (`race::car_shadow`); now each view's (`scene::polygon_work::PolygonWork`); unused.
    pub crossing_amount: Dword,
    /// +0x220 (0x6241A0): three dwords after the vertex byte; no code references them (a gap).
    pub unknown_220: [Dword; 3],
    /// +0x22c (0x6241AC): `scene::clip`'s second output run (five words written back from its work); `race::car_shadow` keeps the new crossing vertex's fog byte (+0x20, blended like the amount) in [0]; now each view's (`scene::polygon_work`); unused.
    pub clip_out_words: [Dword; 5],
    /// +0x240 (0x6241C0): two dwords before the farthest depth; no code references them (a gap).
    pub unknown_240: [Dword; 2],
    /// +0x248 (0x6241C8): the farthest depth (`scene::set_view_window`); now each view's (`scene::view_window`); unused.
    pub farthest: Float,
    /// +0x24c (0x6241CC): 1 over it; now each view's (`scene::view_window`); unused.
    pub one_over_farthest: Float,
    /// +0x250 (0x6241D0): it times `fade_a` over `fade_b` over 65536; now each view's (`scene::view_window`); unused.
    pub farthest_scaled: Dword,
    /// +0x254 (0x6241D4): the clip rectangle as whole numbers: top, bottom, left, right; now each view's (`scene::view_window`); unused.
    pub clip_edges: [Dword; 4],
    /// +0x264 (0x6241E4): the same as floats (`g_clipScreenTop` .. `Right`); now each view's (`scene::view_window`); unused.
    pub clip_screen: [Float; 4],
    /// +0x274 (0x6241F4): `view_offset` as floats; now each view's (`scene::view_window`); unused.
    pub view_offset_float: [Float; 2],
    /// +0x27c (0x6241FC): two dwords after the offsets as floats; no code references them (a gap).
    pub unknown_27c: [Dword; 2],
    /// +0x284 (0x624204): (`scene::draw_object_faces`); now each view's (`scene::view_window`); unused.
    pub faces_done: Dword,
    /// +0x288 (0x624208): six dwords before `fade_b`; no code references them (a gap).
    pub unknown_288: [Dword; 6],
    /// +0x2a0 (0x624220): (`scene::set_view_window`).
    pub fade_b: Float,
    /// +0x2a4 (0x624224): (100 less `dword_60AC80`, held) times `flt_5C5CFC`; now each view's (`scene::view_window`); unused.
    pub depth_divisor: Float,
    /// +0x2a8 (0x624228): (`scene::set_view_window`).
    pub fade_a: Float,
    /// +0x2ac (0x62422C): `clip_farthest` is the farthest times it.
    pub clip_factor: Float,
    /// +0x2b0 (0x624230): `g_clipFarthest`, rounded; now each view's (`scene::view_window`); unused.
    pub clip_farthest: Dword,
    /// +0x2b4 (0x624234): `g_mirrorPicture`: the mirror picture's texture.
    pub mirror_picture: Dword,
    /// +0x2b8 (0x624238): the `v3driverfix` setting.
    pub v3_driver_fix: Dword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl SceneDraw {
    /// The piece with the image's first values.
    pub const fn new() -> Self {
        crate::pointers::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START))
    }
}

/// IDA 0x623F80, the first byte of the piece.
pub const START: usize = 0x623F80;
/// IDA 0x62423C, the end of the piece.
pub const END: usize = 0x62423C;

const _: () = assert!(END - START == core::mem::size_of::<SceneDraw>());

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::owner::scene;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(&scene().drawing.scene);
    }

    #[test]
    fn piece_has_its_home() {
        assert_eq!(offset_of!(SceneDraw, clip_work), 0x0);
        assert_eq!(offset_of!(SceneDraw, camera_saved), 0x5c);
        assert_eq!(offset_of!(SceneDraw, surface_point), 0xc0);
        assert_eq!(offset_of!(SceneDraw, view_centre), 0x11c);
        assert_eq!(offset_of!(SceneDraw, polygon_alpha), 0x150);
        assert_eq!(offset_of!(SceneDraw, polygon_outside_count), 0x174);
        assert_eq!(offset_of!(SceneDraw, depth_clipped), 0x1a4);
        assert_eq!(offset_of!(SceneDraw, turn_z_work), 0x1d8);
        assert_eq!(offset_of!(SceneDraw, clip_edges), 0x254);
        assert_eq!(offset_of!(SceneDraw, fade_a), 0x2a8);
    }
}
