//! The image's zeroed tail past the drawing parts' scratch (the polygon vertices and the polygon
//! work, each part's own: `scene::workers::PartScratch`): the read tables (`BufferReads`). One static and
//! not seven: the checkpoint tables are read by a number held to the count
//! (`race::checkpoints_store::nth`), and the pieces of the read tables stay together.

use crate::memory::session::DrawShared;
use crate::race::checkpoints_store::Checkpoints;
use crate::scene::engine_tables_store::EngineTables;
use crate::scene::faces_store::Faces;
use crate::scene::polygon_store::PolygonTables;

/// The buffers: the tables the parts only read (`BufferReads`).
pub struct Buffers {
    /// The tables written on the game's thread outside a drawing scope and read by every
    /// part inside one (lent as `scene::workers::DrawReads::buffers`).
    pub reads: DrawShared<BufferReads>,
}

/// The buffers the drawing parts read and none writes, in the image's order after the polygon
/// work.
#[derive(rc2k_derive::Cells)]
pub struct BufferReads {
    /// `scene::polygon_store`'s tables.
    pub polygon_tables: PolygonTables,
    /// `scene::faces_store`.
    pub faces: Faces,
    /// `race::checkpoints_store`.
    pub checkpoints: Checkpoints,
    /// `scene::engine_tables_store`.
    pub engine_tables: EngineTables,
}

impl BufferReads {
    /// The bytes, zero as in the image.
    pub const fn new() -> Self {
        crate::memory::cells::zeroed_cells()
    }
}

impl Buffers {
    /// The bytes, zero as in the image.
    pub const fn new() -> Self {
        Buffers {
            reads: DrawShared::new(BufferReads::new()),
        }
    }
}
