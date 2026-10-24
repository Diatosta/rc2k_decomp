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
//! Shadow mode: the Rust version's calls into the Direct3D layer are worked out
//! without being made, the original's are noted as it makes them (`d3d/recorder.rs`),
//! and the two lists compared.
//! Self-test: a redirected copy of the three functions whose log calls and
//! `sub_443AEC` only note themselves, on polygons of the test's in the game's buffer,
//! with the Direct3D layer's calls swallowed.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::d3d::draw_fan_unclipped::polygon;
use crate::d3d::set_texture::CURRENT_RVA;
use crate::d3d::{NO_TEXTURE, Vertex};
use crate::framework::{game_ptr, game_read, game_write, rva};
use crate::logfile::log_hex8::log_hex8;
use crate::logfile::log_write::log_write;
use crate::session::SCREEN;

// ---- game data ----

const TEXTURE_RVA: usize = rva(0x6240D4); // g_polygonTexture
const HAS_3D_RVA: usize = rva(0x5CDA8C); // dword_5CDA8C
const CHECKED_RVA: usize = rva(0x62400C); // dword_62400C
const TEXTURES: u32 = 0x400;

const SHOW_VERTICES: usize = 0x443AEC; // sub_443AEC
const ERROR_TEXT: usize = 0x4DD9A7; // "Error"
const DRAWTRI_TEXT: usize = 0x4DD99F; // "drawtri"

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
    unsafe {
        let has_3d = game_read::<u32>(HAS_3D_RVA) != 0;
        let texture: u32 = game_read(TEXTURE_RVA);
        if texture == NO_TEXTURE {
            if has_3d {
                outside.set_texture(NO_TEXTURE);
            }
        } else {
            let texture = if texture >= TEXTURES { 0 } else { texture };
            if texture != game_read::<u32>(CURRENT_RVA) && has_3d {
                outside.set_texture(texture);
            }
        }
        let vertices = polygon();
        if !vertices.is_empty() {
            // The check clears this before it looks at a vertex.
            game_write(CHECKED_RVA, 0u32);
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
        unsafe { log_write(game_ptr(rva(ERROR_TEXT)), true, 1) };
        if let Err(e) = crate::scene::show_bad_polygon::show_bad_polygon() {
            // As a refusal of `sub_443AEC` called by its address ended the program.
            crate::standalone::finish(
                &format!(
                    "menu::table_calls: Routed ({SHOW_VERTICES:#x}) refused: {}",
                    e.0
                ),
                1,
            );
        }
    }
    fn not_drawn(&mut self, result: u32) {
        unsafe { log_write(game_ptr(rva(DRAWTRI_TEXT)), false, 1) };
        log_hex8(result, false, 1);
    }
}
