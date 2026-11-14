//! The race frame's draw batches: the scene's batch with its vertices and indices, and the two
//! batches `race::draw_race_scene` flushes (`race::frame_flush::plain`, the plain one;
//! `race::frame_flush::textured`, the textured one): vertices, indices, counts, first words and the
//! counts kept. The buffers' lengths are the spans up to the next item: no code checks a capacity.
//! It stops before the replay's variables (`drive::replay_frames_store`).

use crate::cells::ZeroCells;

use crate::session::{PlainBytes, PlainDword};

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct DrawBatches {
    /// +0x0: the scene batch's vertices.
    pub scene_vertices: PlainBytes<0x4000>,
    /// +0x4000: the scene batch's indices.
    pub scene_indices: PlainBytes<0x1800>,
    /// +0x5800: the plain batch's vertices (`race::frame_flush::plain`).
    pub plain_vertices: PlainBytes<0x8000>,
    /// +0xd800: the plain batch's indices.
    pub plain_indices: PlainBytes<0x3000>,
    /// +0x10800: the plain batch's triangle count (`race::frame_flush::plain`,
    /// `race::draw_race_scene`).
    pub plain_count: PlainDword,
    /// +0x10804: the plain batch's first word.
    pub plain_word: PlainDword,
    /// +0x10808: the textured batch's vertices (`race::frame_flush::textured`).
    pub textured_vertices: PlainBytes<0x8200>,
    /// +0x18a08: the textured batch's indices (lies inside, at +0x1860).
    pub textured_indices: PlainBytes<0x30C0>,
    /// +0x1bac8: the textured batch's triangle count.
    pub textured_count: PlainDword,
    /// +0x1bacc: the textured batch's first word.
    pub textured_word: PlainDword,
    /// +0x1bad0: the plain batch's count, kept when it is flushed.
    pub plain_kept: PlainDword,
    /// +0x1bad4: the textured batch's count, kept when it is flushed.
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
