//! The commands as a GPU wants them: triangles in one list, and runs of them drawn alike. What a
//! state means is decided here, the same for every GPU backend; nothing here touches one.

use super::{Command, CommandList, STATES, TextureNote};

/// A vertex as the game makes it: x, y, z, 1/w, colour, specular, u, v.
pub type Vertex = crate::render::rasteriser::Vertex;
mod build;
mod keys;

pub use self::build::*;
pub use self::keys::*;

/// A second pass drawn over a batch's own in the same draw: the face drawn again at once
/// without depth writes, blended by its alpha (the ground's second texture). Its vertices' colour,
/// specular and texture place are in `Batches::seconds`.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Second {
    pub values: PixelValues,
    pub texture: Option<u32>,
    pub linear: bool,
    pub clamp: bool,
    /// Read from the texture's smaller levels as the face shrinks (the game's mip filter).
    pub mips: bool,
}

/// A run of triangles drawn alike.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Batch {
    pub first_vertex: u32,
    pub vertices: u32,
    pub pipeline: PipelineKey,
    pub values: PixelValues,
    /// Which of `Batches::textures`.
    pub texture: Option<u32>,
    pub linear: bool,
    pub clamp: bool,
    /// Read from the texture's smaller levels as the face shrinks (the game's mip filter).
    pub mips: bool,
    /// Scissor rectangle for a depth clear: left, top, right, bottom.
    pub scissor: Option<[u32; 4]>,
    pub second: Option<Second>,
    /// The fragment shader may leave out its alpha test: it cannot discard a pixel of this
    /// batch (`fan_fast_path`). A GPU that draws such a batch with a shader with no `discard`
    /// can reject hidden pixels before running it. Only set by `of_layered` (the handheld's).
    pub discard_free: bool,
}

#[derive(Default)]
pub struct Batches {
    pub vertices: Vec<Vertex>,
    pub batches: Vec<Batch>,
    pub textures: Vec<TextureNote>,
    /// With second passes joined (`of`), each vertex's second pass's colour, specular and
    /// texture place (vertices of batches without one: zeros).
    pub seconds: Vec<[u32; 4]>,
    /// Layered (`of_layered`): each vertex's first and second pass's texture, by number in
    /// `textures` (`NO_TEXTURE` for none).
    pub vertex_textures: Vec<[u16; 2]>,
    /// Layered: whether two textures can be read in one draw.
    layered: Option<fn(&TextureNote, &TextureNote) -> bool>,
    /// Fans are given their `fan_fast_path` (`of_layered` with `fast_path`).
    fast_path: bool,
    /// The target's width and height (`apart`).
    size: (f64, f64),
    /// The pixels each batch can touch, while they are being gathered.
    areas: Vec<Area>,
    /// The last fan, while it can still take a second pass.
    last_fan: Option<LastFan>,
    /// Each of `textures`' number, by its id; the last one numbered (`texture_number`).
    texture_numbers: super::TextureMap<u32>,
    last_texture: Option<(super::TextureId, u32)>,
}

/// A pass without a texture, in `Batches::vertex_textures`.
pub const NO_TEXTURE: u16 = u16::MAX;

impl Batches {
    /// Everything taken out, the vectors' room kept.
    fn clear(&mut self) {
        let Batches {
            vertices,
            batches,
            textures,
            seconds,
            vertex_textures,
            layered,
            fast_path,
            size,
            areas,
            last_fan,
            texture_numbers,
            last_texture,
        } = self;
        vertices.clear();
        batches.clear();
        textures.clear();
        seconds.clear();
        vertex_textures.clear();
        *layered = None;
        *fast_path = false;
        *size = (0.0, 0.0);
        areas.clear();
        *last_fan = None;
        texture_numbers.clear();
        *last_texture = None;
    }
}

/// Batches drawn and handed back (`give_back_batches`), their room kept for the next flushes:
/// the vectors grown to a frame's fans are most of the batching's time.
static SPARE_BATCHES: std::sync::Mutex<Vec<Batches>> = std::sync::Mutex::new(Vec::new());

/// Batches kept at most: the ones in flight between the batching thread and the GL's, and the
/// one being drawn.
const SPARE_BATCHES_KEPT: usize = 4;

/// Empty batches: drawn ones handed back, else new.
fn spare_batches() -> Batches {
    SPARE_BATCHES.lock().unwrap().pop().unwrap_or_default()
}

/// Batches drawn: emptied and kept for the next flushes.
pub fn give_back_batches(mut batches: Batches) {
    batches.clear();
    let mut spare = SPARE_BATCHES.lock().unwrap();
    if spare.len() < SPARE_BATCHES_KEPT {
        spare.push(batches);
    }
}

#[cfg(test)]
mod pipeline_change_tests;
#[cfg(test)]
mod shader_kind_tests;
#[cfg(test)]
mod tests;
