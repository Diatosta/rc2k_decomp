//! The game's Direct3D layer (0x418400 .. 0x41AE40): platform, so not ported, but the
//! place where what the 3D code above it draws can be seen.
//!
//! Direct3D is used as a rasteriser only: every vertex comes transformed and lit
//! (`Vertex`, the `D3DTLVERTEX` of DirectX 5). In the standalone program a renderer of
//! ours will stand where the originals are; until then nothing is drawn there.

pub mod copy_screen_to_texture;
pub mod copy_to_device_texture;
pub mod draw_fan;
pub mod draw_fan_unclipped;
pub mod draw_hud_batch;
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
