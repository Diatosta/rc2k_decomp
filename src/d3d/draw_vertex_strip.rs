//! `d3d::draw_vertex_strip`: draws a triangle strip of the given transformed and lit vertices, not
//! clipped, with no lighting and no extents. The strip is only noted (`d3d::recorder`) and the
//! rasteriser draws it (`d3d::device::draw_strip`); the result is 0.

use crate::d3d::Vertex;
use crate::d3d::recorder::{Call, note};
use crate::errors::BadInput;

/// Direct3D takes at most this many vertices in one primitive.
const MOST: u32 = 0x400;

fn vertices(
    at: &[crate::d3d::vertices_store::Vertex],
    count: u32,
) -> Result<Vec<Vertex>, BadInput> {
    let count = count.min(MOST) as usize;
    let held = at
        .get(..count)
        .ok_or(BadInput("a vertex strip past its buffer"))?;
    Ok(held
        .iter()
        .map(crate::d3d::vertices_store::Vertex::words)
        .collect())
}

/// Draws the strip of `count` vertices at the start of `at`.
pub fn draw_vertex_strip(
    part: &crate::scene::scratch_store::PartScratch,
    at: &[crate::d3d::vertices_store::Vertex],
    count: u32,
) -> Result<(), BadInput> {
    let vertices = vertices(at, count)?;
    let swallowed = note(|| Call::Fan(vertices.clone()));
    if !swallowed {
        crate::d3d::device::draw_strip(part, &vertices);
    }
    Ok(())
}
