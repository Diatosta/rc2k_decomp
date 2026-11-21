//! A view's window, scale and depths, the view's own: the size, offsets, zoom and middle
//! `scene::set_view_window` is given and the scale, clip rectangle, depths and offsets as floats it
//! derives from them. One per view slot (`race::view_scratch::ViewScratch::window`;
//! `ViewScratch::begin` keeps it, as the window is set before a view's scene begins); the menus and
//! the front end's 3D use the front end's.

use crate::race::plain_slot::{
    Merge, PlainSlot, merge_float, merge_floats, merge_run, merge_value,
};

/// A view's window, behind a `RefCell`: borrows are held for one statement.
pub type ViewWindow = PlainSlot<WindowValues>;

/// The window's values; the floats merge by their bits, as the cells they were.
#[derive(Clone, Copy)]
pub struct WindowValues {
    /// The window's offsets x, y (`scene::set_view_window`; the menus and `race::frame_views`
    /// set them).
    pub view_offset: [u32; 2],
    /// The zoom, a whole number (0x100; the menus set it), in the scale.
    pub view_zoom: u32,
    /// The scale (`scene::set_view_window`, `scene::project_point`).
    pub view_scale: f32,
    /// The window's width and height.
    pub view_size: [u32; 2],
    /// The window's middle, x and y (half of each, unsigned).
    pub view_middle: [u32; 2],
    /// The middle as floats (`scene::project_point`'s centre; `draw_polygon`'s mirror).
    pub view_centre: [f32; 2],
    /// The farthest depth (`scene::set_view_window`).
    pub farthest: f32,
    /// 1 over it.
    pub one_over_farthest: f32,
    /// It times `fade_a` over `fade_b` over 65536.
    pub farthest_scaled: u32,
    /// The clip rectangle as whole numbers: top, bottom, left, right.
    pub clip_edges: [u32; 4],
    /// The same as floats.
    pub clip_screen: [f32; 4],
    /// `view_offset` as floats.
    pub view_offset_float: [f32; 2],
    /// The farthest depth times `SceneDraw.clip_factor`, rounded.
    pub clip_farthest: u32,
    /// How far is seen times 0x20, twice (`scene::set_view_window`).
    pub view_depth: [u32; 2],
}

impl WindowValues {
    pub const fn new() -> Self {
        Self {
            view_offset: [0; 2],
            view_zoom: 0,
            view_scale: 0.0,
            view_size: [0; 2],
            view_middle: [0; 2],
            view_centre: [0.0; 2],
            farthest: 0.0,
            one_over_farthest: 0.0,
            farthest_scaled: 0,
            clip_edges: [0; 4],
            clip_screen: [0.0; 4],
            view_offset_float: [0.0; 2],
            clip_farthest: 0,
            view_depth: [0; 2],
        }
    }
}

impl Merge for WindowValues {
    fn merge_changes(&mut self, now: &Self, start: &Self) {
        merge_run(&mut self.view_offset, &now.view_offset, &start.view_offset);
        merge_value(&mut self.view_zoom, &now.view_zoom, &start.view_zoom);
        merge_float(&mut self.view_scale, &now.view_scale, &start.view_scale);
        merge_run(&mut self.view_size, &now.view_size, &start.view_size);
        merge_run(&mut self.view_middle, &now.view_middle, &start.view_middle);
        merge_floats(&mut self.view_centre, &now.view_centre, &start.view_centre);
        merge_float(&mut self.farthest, &now.farthest, &start.farthest);
        merge_float(
            &mut self.one_over_farthest,
            &now.one_over_farthest,
            &start.one_over_farthest,
        );
        merge_value(
            &mut self.farthest_scaled,
            &now.farthest_scaled,
            &start.farthest_scaled,
        );
        merge_run(&mut self.clip_edges, &now.clip_edges, &start.clip_edges);
        merge_floats(&mut self.clip_screen, &now.clip_screen, &start.clip_screen);
        merge_floats(
            &mut self.view_offset_float,
            &now.view_offset_float,
            &start.view_offset_float,
        );
        merge_value(
            &mut self.clip_farthest,
            &now.clip_farthest,
            &start.clip_farthest,
        );
        merge_run(&mut self.view_depth, &now.view_depth, &start.view_depth);
    }
}

/// A window with every value 0.
#[cfg(test)]
pub const fn first_window() -> ViewWindow {
    PlainSlot::new(WindowValues::new())
}
