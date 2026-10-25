//! `scene::submit_polygon` (0x44CCC0): hands the polygon in the vertex buffer to
//! Direct3D with its texture.
//!
//! In:  nothing: the vertices are at `g_polygonVertices` (0x9605A0), their number in
//!      `g_polygonVertexCount` (0x6240F8), the texture's number in `g_polygonTexture`
//!      (0x6240D4)
//! Out: the original leaves eax, ecx, edx and esi changed; none of its fourteen call
//!      sites (all in `sub_44BA2C`) reads them, and the Rust version keeps every
//!      register
//! With `scene::draw_polygon_fan` (0x44EA36), `scene::check_polygon_vertices`
//! (0x44EA6A), `scene::set_texture` (0x44E77E) and `scene::clear_texture` (0x44E958)
//! inlined, each of which has this as its only caller on the way here.
//! The texture: 0xFFFFFC43 means none; a number of 0x400 or more counts as 0; one that
//! is not `g_currentTexture` is set first. Both go to Direct3D only if there is 3D
//! (`dword_5CDA8C` not 0; a reading).
//! The vertices: each must have x in 0 .. the screen's width, y in 0 .. its height
//! and z in 0 .. 1 (compared on the x87; one that is not a number fails), else "Error"
//! is logged, the buffer shown by `sub_443AEC` and nothing drawn. Then, with 3D, the
//! fan; a result other than 0 is logged as "drawtri" and the number.

use crate::d3d::cards_store::CARDS_3D;
use crate::d3d::draw_fan_unclipped::polygon;
use crate::d3d::{NO_TEXTURE, Vertex};
use crate::logfile::log_hex8::log_hex8;
use crate::logfile::log_write::log_write;
use crate::scene::draw_store::DRAW;
use crate::scene::object_tables_store::OBJECT_TABLES;
use crate::session::SCREEN;

// ---- game data ----
const TEXTURES: u32 = 0x400;

const SHOW_VERTICES: usize = 0x443AEC; // sub_443AEC
const ERROR_TEXT: &std::ffi::CStr = c"Error"; // "Error"
const DRAWTRI_TEXT: &std::ffi::CStr = c"drawtri"; // "drawtri"

// ---- Rust version ----

/// What the function asks of the code around it.
pub trait Outside {
    /// `d3d::set_texture`.
    fn set_texture(&mut self, texture: u32);
    /// `d3d::draw_fan_unclipped`: Direct3D's result.
    fn draw_fan(&mut self) -> u32;
    /// A vertex is off the screen: "Error" to the log and `sub_443AEC`.
    fn bad_vertex(&mut self);
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

pub unsafe fn submit_polygon(outside: &mut dyn Outside) {
    let has_3d = OBJECT_TABLES.has_3d.get() != 0;
    let texture: u32 = DRAW.polygon_texture.bits();
    if texture == NO_TEXTURE {
        if has_3d {
            outside.set_texture(NO_TEXTURE);
        }
    } else {
        let texture = if texture >= TEXTURES { 0 } else { texture };
        if texture != CARDS_3D.current_texture.get() && has_3d {
            outside.set_texture(texture);
        }
    }
    let vertices = polygon();
    if !vertices.is_empty() {
        // The check clears this before it looks at a vertex.
        DRAW.scratch.set_bits(0u32);
    }
    if !vertices_fit(&vertices, SCREEN.width.get(), SCREEN.height.get()) {
        outside.bad_vertex();
        return;
    }
    if has_3d {
        let result = outside.draw_fan();
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
    fn draw_fan(&mut self) -> u32 {
        // `d3d::draw_fan_unclipped`: its result is always 0.
        crate::platform_3d::draw_fan(&polygon());
        0
    }
    fn bad_vertex(&mut self) {
        unsafe { log_write(ERROR_TEXT.as_ptr().cast(), true, 1) };
        if let Err(e) = crate::scene::show_bad_polygon::show_bad_polygon() {
            crate::standalone::finish(
                &format!(
                    "scene::show_bad_polygon ({SHOW_VERTICES:#x}) refused: {}",
                    e.0
                ),
                1,
            );
        }
    }
    fn not_drawn(&mut self, result: u32) {
        unsafe { log_write(DRAWTRI_TEXT.as_ptr().cast(), false, 1) };
        log_hex8(result, false, 1);
    }
}
