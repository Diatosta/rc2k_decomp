//! The Direct3D layer: the place where what the 3D code above it draws can be seen.
//!
//! Direct3D is used as a rasteriser only: every vertex comes transformed and lit (`Vertex`).
//! The standalone program draws them with the renderers in `render/`; the recorders here note
//! what the 3D code asks for.

mod cards_store;
pub(crate) mod copy_screen_to_texture;
pub(crate) mod copy_to_device_texture;
pub(crate) mod device;
pub(crate) mod draw_fan;
pub(crate) mod draw_fan_unclipped;
pub(crate) mod draw_hud_batch;
pub(crate) mod draw_marks_windscreen_drops;
pub(crate) mod draw_vertex_strip;
pub(crate) mod lock_texture;
pub(crate) mod owner;
mod recorder;
pub(crate) mod render_cache;
pub(crate) mod run_states;
pub(crate) mod set_render_state;
pub(crate) mod set_texture;
mod shared_states;
mod texture_lock;
mod texture_surface_info;
pub(crate) mod unlock_texture;
pub(crate) mod vertices_store;
pub(crate) mod with_locked_texture;

/// A vertex as the game hands it over: x, y, z, 1/w (floats), colour, specular, u, v
/// (floats), as their bits.
pub type Vertex = [u32; 8];
const VERTEX_LEN: usize = 0x20;

/// A vertex in the form the drawing layer takes: the floats turned to their bits here, the
/// one place.
fn wire_vertex(position: [f32; 4], colour: u32, specular: u32, texture: [f32; 2]) -> Vertex {
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
