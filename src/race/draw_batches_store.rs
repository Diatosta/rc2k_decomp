//! The race frame's draw batches: the scene's batch with its vertices and indices, and the two
//! batches `race::draw_race_scene` flushes (`race::frame_flush::plain`, the plain one;
//! `race::frame_flush::textured`, the textured one): vertices, indices, counts, first words and the
//! counts kept. The buffers' lengths are the spans up to the next item: no code checks a capacity.
//! It stops before the replay's variables (`drive::replay_frames_store`).

use crate::memory::cells::ZeroCells;

use crate::memory::session::{PlainBytes, PlainDword};

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct DrawBatches {
    /// The scene batch's vertices.
    pub scene_vertices: PlainBytes<0x4000>,
    /// The scene batch's indices.
    pub scene_indices: PlainBytes<0x1800>,
    /// The plain batch's vertices (`race::frame_flush::plain`).
    pub plain_vertices: PlainBytes<0x8000>,
    /// The plain batch's indices.
    pub plain_indices: PlainBytes<0x3000>,
    /// The plain batch's triangle count (`race::frame_flush::plain`,
    /// `race::draw_race_scene`).
    pub plain_count: PlainDword,
    /// The plain batch's first word.
    pub plain_word: PlainDword,
    /// The textured batch's vertices (`race::frame_flush::textured`).
    pub textured_vertices: PlainBytes<0x8200>,
    /// The textured batch's indices (lies inside, at +0x1860).
    pub textured_indices: PlainBytes<0x30C0>,
    /// The textured batch's triangle count.
    pub textured_count: PlainDword,
    /// The textured batch's first word.
    pub textured_word: PlainDword,
    /// The plain batch's count, kept when it is flushed.
    pub plain_kept: PlainDword,
    /// The textured batch's count, kept when it is flushed.
    pub textured_kept: PlainDword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece, with the image's first values.
impl DrawBatches {
    pub const fn new() -> Self {
        Self::ZERO
    }
}
