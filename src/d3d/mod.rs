//! The game's Direct3D layer (0x418400 .. 0x41AE40): platform, so not ported, but the
//! place where what the 3D code above it draws can be seen. The hooks here hand their
//! call on to the original and note it for whoever is recording (`recorder.rs`), as
//! `gfx::blit_glyph` does for the 2D code; a recording that swallows keeps the call
//! from Direct3D altogether, so that a self-test can run original drawing code.
//!
//! Direct3D is used as a rasteriser only: every vertex comes transformed and lit
//! (`Vertex`, the `D3DTLVERTEX` of DirectX 5). In the standalone program a renderer of
//! ours will stand where the originals are; until then nothing is drawn there.

use crate::framework::GameHook;

pub mod copy_screen_to_texture;
pub mod copy_to_device_texture;
pub mod draw_fan;
pub mod draw_fan_unclipped;
pub mod draw_hud_batch;
pub mod draw_indexed;
pub mod draw_marks_hexagons;
pub mod draw_vertex_strip;
pub mod lock_texture;
pub mod recorder;
pub mod set_render_state;
pub mod set_texture;
pub mod texture_lock;
pub mod texture_surface_info;
pub mod unlock_texture;

/// A vertex as the game hands it over: x, y, z, 1/w (floats), colour, specular, u, v
/// (floats), as their bits.
pub type Vertex = [u32; 8];
pub const VERTEX_LEN: usize = 0x20;
/// The texture number that means "none".
pub const NO_TEXTURE: u32 = 0xFFFF_FC43;

pub static HOOKS: &[&dyn GameHook] = &[
    &set_render_state::HOOK,
    &set_texture::HOOK,
    &draw_fan_unclipped::HOOK,
    &draw_fan::HOOK,
    &draw_indexed::HOOK,
    &draw_hud_batch::HOOK,
    &copy_screen_to_texture::HOOK,
    &copy_to_device_texture::HOOK,
    &draw_vertex_strip::HOOK,
    &draw_marks_hexagons::HOOK,
    &texture_surface_info::HOOK,
];
