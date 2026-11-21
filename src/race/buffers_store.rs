//! The image's zeroed tail past the drawing parts' scratch (the polygon vertices and the polygon
//! work, each part's own: `scene::scratch_store::PartScratch`): the read tables and the room the
//! object files are read into.

use crate::math::engine_tables::{EngineTables, EngineTablesRef};
use crate::memory::plain_buffer::PlainBuffer;
use crate::memory::session::{DrawArc, GameThread};
use crate::scene::faces_store::{Faces, FacesRef};
use crate::scene::objects::FILE_ROOM;
use crate::scene::polygon_store::{PolygonTables, PolygonTablesRef};

/// The buffers: the tables the parts only read, written on the game's thread outside a drawing
/// scope and read by every part inside one (lent as `scene::workers::DrawReads`), and the object
/// file's room, which only the game's thread reaches.
pub struct Buffers {
    /// `scene::polygon_store`'s tables.
    pub polygon_tables: DrawArc<PolygonTables>,
    /// `scene::faces_store`.
    pub faces: DrawArc<Faces>,
    /// `math::engine_tables`.
    pub engine_tables: DrawArc<EngineTables>,
    /// The room the object file is read into, 0x40000 bytes (+4 points, +8 faces, +0xC normals,
    /// +0x2C the points: `scene::objects`, `scene::load_object`, `scene::add_object_shape`).
    pub object_file: GameThread<PlainBuffer<FILE_ROOM>>,
}

impl Buffers {
    /// The tables zeroed.
    pub const fn new() -> Self {
        Buffers {
            polygon_tables: DrawArc::new(PolygonTables::new),
            faces: DrawArc::new(Faces::new),
            engine_tables: DrawArc::new(EngineTables::new),
            object_file: GameThread::new(PlainBuffer::new([0; FILE_ROOM])),
        }
    }

    /// The polygon tables, read on the game's thread.
    pub fn polygon_tables(&self) -> PolygonTablesRef<'_> {
        PolygonTablesRef::Store(&self.polygon_tables)
    }

    /// The faces, read on the game's thread.
    pub fn faces(&self) -> FacesRef<'_> {
        FacesRef::Store(&self.faces)
    }

    /// The engine tables, read on the game's thread.
    pub fn engine_tables(&self) -> EngineTablesRef<'_> {
        EngineTablesRef::Store(&self.engine_tables)
    }
}
