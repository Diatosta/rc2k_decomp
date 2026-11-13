//! The game's Direct3D layer (0x418400 .. 0x41AE40): platform, so not ported, but the
//! place where what the 3D code above it draws can be seen.
//!
//! Direct3D is used as a rasteriser only: every vertex comes transformed and lit
//! (`Vertex`, the `D3DTLVERTEX` of DirectX 5). The standalone program draws them with the
//! renderers in `render/`; the recorders here note what the 3D code asks for.

pub mod cards_store;
pub mod copy_screen_to_texture;
pub mod copy_to_device_texture;
pub mod draw_fan;
pub mod draw_fan_unclipped;
pub mod draw_hud_batch;
pub mod draw_marks_windscreen_drops;
pub mod draw_vertex_strip;
pub mod lock_texture;
pub mod owner;
pub mod recorder;
pub mod render_cache;
pub mod run_states;
pub mod set_render_state;
pub mod set_texture;
pub mod texture_lock;
pub mod texture_surface_info;
pub mod unlock_texture;
pub mod vertices_store;

/// A vertex as the game hands it over: x, y, z, 1/w (floats), colour, specular, u, v
/// (floats), as their bits.
pub type Vertex = [u32; 8];
pub const VERTEX_LEN: usize = 0x20;

/// A vertex in the form the drawing layer takes: the floats turned to their bits here, the
/// one place.
pub fn wire_vertex(position: [f32; 4], colour: u32, specular: u32, texture: [f32; 2]) -> Vertex {
    [
        position[0].to_bits(),
        position[1].to_bits(),
        position[2].to_bits(),
        position[3].to_bits(),
        colour,
        specular,
        texture[0].to_bits(),
        texture[1].to_bits(),
    ]
}
/// The texture number that means "none".
pub const NO_TEXTURE: u32 = 0xFFFF_FC43;
