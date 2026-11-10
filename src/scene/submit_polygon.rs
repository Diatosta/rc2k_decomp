//! `scene::submit_polygon` (0x44CCC0): hands the polygon in the vertex buffer to
//! Direct3D with its texture.
//!
//! In:  nothing: the vertices are at `g_polygonVertices` (0x9605A0), their number in
//!      `g_polygonVertexCount` (0x6240F8), the texture's number in `g_polygonTexture`
//!      (0x6240D4)
//! With `scene::draw_polygon_fan` (0x44EA36), `scene::check_polygon_vertices`
//! (0x44EA6A), `scene::set_texture` (0x44E77E) and `scene::clear_texture` (0x44E958)
//! inlined, each of which has this as its only caller on the way here.
//! The texture: 0xFFFFFC43 means none; a number of 0x400 or more counts as 0; one that
//! is not `g_currentTexture` is set first. Both go to Direct3D only if there is 3D
//! (`dword_5CDA8C` not 0; a reading).
//! The vertices: each must have x in 0 .. the screen's width, y in 0 .. its height
//! and z in 0 .. 1 (compared on the x87; one that is not a number fails), else "Error"
//! is logged, the buffer shown by `scene::show_bad_polygon` and nothing drawn. Then, with 3D, the
//! fan; a result other than 0 is logged as "drawtri" and the number.

use crate::d3d::draw_fan_unclipped::polygon;
use crate::d3d::{NO_TEXTURE, Vertex};
use crate::logfile::log_hex8::log_hex8;
use crate::scene::owner::scene;

// ---- game data ----
const TEXTURES: u32 = 0x400;

const ERROR_TEXT: &std::ffi::CStr = c"Error"; // "Error"
const DRAWTRI_TEXT: &std::ffi::CStr = c"drawtri"; // "drawtri"

// ---- Rust version ----

/// What the function asks of the code around it.
pub trait Outside {
    /// `d3d::set_texture`.
    fn set_texture(&mut self, texture: u32);
    /// `d3d::draw_fan_unclipped`: Direct3D's result.
    fn draw_fan(&mut self, vertices: &[Vertex]) -> u32;
    /// A vertex is off the screen: "Error" to the log and `scene::show_bad_polygon`.
    fn bad_vertex(&mut self, poly: &crate::scene::polygon_work::PolygonWork);
    /// Direct3D did not draw: "drawtri" and its result to the log.
    fn not_drawn(&mut self, result: u32);
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

pub fn submit_polygon(poly: &crate::scene::polygon_work::PolygonWork, outside: &mut dyn Outside) {
    let system = crate::system::owner::system();
    let session = &system.session;
    let scene = scene();
    let has_3d = scene.objects.tables.has_3d.get() != 0;
    let texture: u32 = poly.polygon_texture.bits();
    if texture == NO_TEXTURE {
        if has_3d {
            outside.set_texture(NO_TEXTURE);
        }
    } else {
        let texture = if texture >= TEXTURES { 0 } else { texture };
        if texture != crate::d3d::render_cache::texture() && has_3d {
            outside.set_texture(texture);
        }
    }
    let vertices = polygon(poly);
    if !vertices_fit(
        &vertices,
        session.screen.width.get(),
        session.screen.height.get(),
    ) {
        outside.bad_vertex(poly);
        return;
    }
    if has_3d {
        let result = outside.draw_fan(&vertices);
        if result != 0 {
            outside.not_drawn(result);
        }
    }
}

/// The game's Direct3D layer and log.
pub struct Game;

impl Outside for Game {
    fn set_texture(&mut self, texture: u32) {
        crate::d3d::set_texture::set_texture(texture);
    }
    fn draw_fan(&mut self, vertices: &[Vertex]) -> u32 {
        // `d3d::draw_fan_unclipped`: its result is always 0.
        crate::platform_3d::draw_fan(vertices);
        0
    }
    fn bad_vertex(&mut self, poly: &crate::scene::polygon_work::PolygonWork) {
        crate::logfile::log_write::log_nul(&ERROR_TEXT, true, 1);
        if let Err(e) = crate::scene::show_bad_polygon::show_bad_polygon(poly) {
            crate::fatal::raise::end("scene::show_bad_polygon", e);
        }
    }
    fn not_drawn(&mut self, result: u32) {
        crate::logfile::log_write::log_nul(&DRAWTRI_TEXT, false, 1);
        log_hex8(result, false, 1);
    }
}
