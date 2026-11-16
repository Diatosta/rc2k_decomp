//! `d3d::draw_vertex_strip`: draws a triangle strip of the given transformed and lit vertices, not
//! clipped, with no lighting and no extents. The strip is only noted (`d3d::recorder`) and the
//! rasteriser draws it (`platform::direct3d::draw_strip`); the result is 0.

use crate::d3d::recorder::{Call, note};
use crate::d3d::{VERTEX_LEN, Vertex};
use crate::errors::BadInput;
use crate::memory::cells::Store;

/// Direct3D takes at most this many vertices in one primitive.
const MOST: u32 = 0x400;

fn vertices(at: &dyn Store, count: u32) -> Result<Vec<Vertex>, BadInput> {
    let mut bytes = vec![0; count.min(MOST) as usize * VERTEX_LEN];
    at.load(0, &mut bytes)?;
    Ok(bytes
        .as_chunks::<VERTEX_LEN>()
        .0
        .iter()
        .map(|vertex| {
            std::array::from_fn(|n| {
                u32::from_le_bytes(vertex[4 * n..4 * n + 4].try_into().unwrap())
            })
        })
        .collect())
}

/// Draws the strip of `count` vertices at `at`.
pub fn draw_vertex_strip(
    part: &crate::scene::workers::PartScratch,
    at: &dyn Store,
    count: u32,
) -> Result<(), BadInput> {
    let vertices = vertices(at, count)?;
    let swallowed = note(|| Call::Fan(vertices.clone()));
    if !swallowed {
        crate::platform::direct3d::draw_strip(part, &vertices);
    }
    Ok(())
}
