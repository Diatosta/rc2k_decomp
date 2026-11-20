//! `scene::submit_polygon`: hands the polygon in the vertex buffer to Direct3D with its texture.
//!
//! The vertices are in the polygon vertex buffer, their number in the polygon's vertex count,
//! the texture's number in `polygon_texture`. With `scene::draw_fan`, the vertex check,
//! `scene::set_texture` and `scene::clear_texture` inlined.
//! The texture: 0xFFFFFC43 means none; a number of 0x400 or more counts as 0; one that is not
//! the current texture is set first. Both go to Direct3D only if there is 3D (the draw switch
//! is not 0: the device is drawing, set by `race::draw_race_scene`). The vertices: each must
//! have x in 0 .. the screen's width, y in 0 .. its height and z in 0 .. 1 (compared on the
//! x87; one that is not a number fails), else "Error" is logged, the buffer shown by
//! `scene::show_bad_polygon` and nothing drawn. Then, with 3D, the fan.

use crate::d3d::draw_fan_unclipped::polygon;
use crate::d3d::vertices_store::PolygonVertices;
use crate::d3d::{NO_TEXTURE, Vertex};
use crate::memory::session::Screen;

// ---- game data ----
const TEXTURES: u32 = 0x400;

const ERROR_TEXT: &std::ffi::CStr = c"Error";

/// What the function asks of the code around it.
pub trait Outside {
    /// The texture set last (`d3d::render_cache`).
    fn texture(&self) -> u32;
    /// `d3d::set_texture`.
    fn set_texture(&mut self, texture: u32);
    /// Draws the fan.
    fn draw_fan(&mut self, vertices: &[Vertex]);
    /// A vertex is off the screen: "Error" to the log and `scene::show_bad_polygon`.
    fn bad_vertex(&mut self, buffer: &PolygonVertices, screen: &Screen);
}

/// `scene::check_polygon_vertices`: every vertex on the screen and between the planes.
pub fn vertices_fit(vertices: &[Vertex], width: u32, height: u32) -> bool {
    // As the x87 compares: below 0 or not a number fails, above the limit fails.
    let fits = |bits: u32, limit: f64| {
        let value = f32::from_bits(bits) as f64;
        value >= 0.0 && value <= limit
    };
    vertices
        .iter()
        .all(|v| fits(v[0], width as f64) && fits(v[1], height as f64) && fits(v[2], 1.0))
}

/// `device_on`: the draw switch (`d3d_drawing_on`) is set; `screen` is the screen's size.
pub fn submit_polygon(
    poly: &crate::scene::polygon_work_store::PolygonWork,
    buffer: &PolygonVertices,
    outside: &mut dyn Outside,
    device_on: bool,
    screen: &Screen,
) {
    let texture: u32 = poly.polygon_texture.get();
    if texture == NO_TEXTURE {
        if device_on {
            outside.set_texture(NO_TEXTURE);
        }
    } else {
        let texture = if texture >= TEXTURES { 0 } else { texture };
        if texture != outside.texture() && device_on {
            outside.set_texture(texture);
        }
    }
    let vertices = polygon(poly, buffer);
    if !vertices_fit(&vertices, screen.width.get(), screen.height.get()) {
        outside.bad_vertex(buffer, screen);
        return;
    }
    if device_on {
        outside.draw_fan(&vertices);
    }
}

/// The game's Direct3D layer and log, for drawing part `.0` and its view `.1` (a bad polygon's
/// log names the view's object and face).
pub struct Game<'a>(
    pub &'a crate::scene::scratch_store::PartScratch,
    pub &'a crate::race::view_scratch::ViewScratch,
);

impl Outside for Game<'_> {
    fn texture(&self) -> u32 {
        crate::d3d::render_cache::texture(self.0)
    }
    fn set_texture(&mut self, texture: u32) {
        crate::d3d::set_texture::set_texture(self.0, texture);
    }
    fn draw_fan(&mut self, vertices: &[Vertex]) {
        crate::d3d::device::draw_fan(self.0, vertices);
    }
    fn bad_vertex(&mut self, buffer: &PolygonVertices, screen: &Screen) {
        crate::logfile::log_write::log_nul(&ERROR_TEXT, true, 1);
        if let Err(e) = crate::scene::show_bad_polygon::show_bad_polygon(self.1, buffer, screen) {
            crate::fatal::raise::end("scene::show_bad_polygon", e);
        }
    }
}
