//! A view's window, scale and depths, the view's own (frame plan F7): the size, offsets, zoom
//! and middle `scene::set_view_window` is given and the scale, clip rectangle, depths and
//! offsets as floats it derives from them. One per view slot
//! (`race::view_scratch::ViewScratch::window`; `ViewScratch::begin` keeps it, as the window
//! is set before a view's scene begins); the menus and the front end's 3D use the front end's.
//! In the original these were `SceneDraw`'s `view_*`, `farthest`, `clip_*`, `faces_done` and
//! `depth_divisor` and `RaceView`'s `view_depth`, written by every view in turn.

use crate::session::{PlainDword, PlainFloat};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ViewWindow {
    /// The window's offsets x, y (`scene::set_view_window`; the menus and `race::frame_views` set them).
    pub view_offset: [PlainDword; 2],
    /// The zoom, a whole number (0x100; the menus set it), in the scale.
    pub view_zoom: PlainDword,
    /// Not named (+0x104 of `SceneDraw`).
    pub unknown_104: PlainDword,
    /// The scale (`scene::set_view_window`, `scene::project_point`).
    pub view_scale: PlainFloat,
    /// The window's width and height.
    pub view_size: [PlainDword; 2],
    /// The window's middle, x and y (half of each, unsigned).
    pub view_middle: [PlainDword; 2],
    /// The middle as floats (`scene::project_point`'s centre; `draw_polygon`'s mirror).
    pub view_centre: [PlainFloat; 2],
    /// The farthest depth (`scene::set_view_window`).
    pub farthest: PlainFloat,
    /// 1 over it.
    pub one_over_farthest: PlainFloat,
    /// It times `fade_a` over `fade_b` over 65536.
    pub farthest_scaled: PlainDword,
    /// The clip rectangle as whole numbers: top, bottom, left, right.
    pub clip_edges: [PlainDword; 4],
    /// The same as floats (`g_clipScreenTop` .. `Right`).
    pub clip_screen: [PlainFloat; 4],
    /// `view_offset` as floats.
    pub view_offset_float: [PlainFloat; 2],
    /// (`scene::draw_object_faces`).
    pub faces_done: PlainDword,
    /// (100 less `StagePrefs.nearer`, held) times `Numbers.hundredth_a`.
    pub depth_divisor: PlainFloat,
    /// `g_clipFarthest`, rounded.
    pub clip_farthest: PlainDword,
    /// How far is seen times 0x20, twice (`scene::set_view_window`; the original's
    /// `RACE_VIEW.view_depth`, 0x63DC54).
    pub view_depth: [PlainDword; 2],
}

impl ViewWindow {
    pub const fn new() -> Self {
        Self {
            view_offset: [const { PlainDword::new(0) }; 2],
            view_zoom: PlainDword::new(0),
            unknown_104: PlainDword::new(0),
            view_scale: PlainFloat::new(0.0),
            view_size: [const { PlainDword::new(0) }; 2],
            view_middle: [const { PlainDword::new(0) }; 2],
            view_centre: [const { PlainFloat::new(0.0) }; 2],
            farthest: PlainFloat::new(0.0),
            one_over_farthest: PlainFloat::new(0.0),
            farthest_scaled: PlainDword::new(0),
            clip_edges: [const { PlainDword::new(0) }; 4],
            clip_screen: [const { PlainFloat::new(0.0) }; 4],
            view_offset_float: [const { PlainFloat::new(0.0) }; 2],
            faces_done: PlainDword::new(0),
            depth_divisor: PlainFloat::new(0.0),
            clip_farthest: PlainDword::new(0),
            view_depth: [const { PlainDword::new(0) }; 2],
        }
    }
}
