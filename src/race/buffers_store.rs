//! The image's zeroed tail (IDA 0x960000 .. 0xA8C180): the seven typed pieces in the image's
//! order, as one static. One static and not seven: the checkpoint tables are read by a number
//! held to the count (`race::checkpoints_store::nth`), and the pieces stay together.

use crate::d3d::vertices_store::PolygonVertices;
use crate::race::checkpoints_store::Checkpoints;
#[cfg(test)]
use crate::race::owner::race;
use crate::race::viewer_buffers_store::ViewerBuffers;
use crate::scene::engine_tables_store::EngineTables;
use crate::scene::faces_store::Faces;
use crate::scene::polygon_store::{PolygonTables, PolygonWork};
use crate::screens::text_buffers_store::TextBuffers;

#[repr(C, align(16))]
#[derive(rc2k_derive::Cells)]
pub struct Buffers {
    /// `d3d::vertices_store`.
    pub vertices: PolygonVertices,
    /// `race::viewer_buffers_store`.
    pub viewer: ViewerBuffers,
    /// `scene::polygon_store`.
    pub polygon: PolygonWork,
    /// `scene::polygon_store`'s tables.
    pub polygon_tables: PolygonTables,
    /// `scene::faces_store`.
    pub faces: Faces,
    /// `race::checkpoints_store`.
    pub checkpoints: Checkpoints,
    /// `scene::engine_tables_store`.
    pub engine_tables: EngineTables,
    /// `screens::text_buffers_store`.
    pub text: TextBuffers,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl Buffers {
    /// The bytes, zero as in the image.
    pub const fn new() -> Self {
        crate::cells::cells_of_bytes([0u8; core::mem::size_of::<Buffers>()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_buffers_have_their_home() {
        assert_eq!(core::ptr::from_ref(&race().shared.buffers).addr() % 16, 0);
    }
}
