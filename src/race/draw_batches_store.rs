//! The race frame's draw batches (part R3): the scene's batch (the original's `flush_draw_batch`) with its vertices and indices, and the two batches `race::draw_race_scene` flushes (`race::flush_triangle_batch`, the plain one; `race::flush_texture_batch`, the textured one): vertices, indices, counts, first words and the counts kept. The buffers' lengths are the spans up to the next item: no code checks a capacity (IDA: the scene batch is appended to only by `sub_45FCBA`, which copies a polygon's vertices as 0x20-byte records and adds 16-bit triangle-fan indices, and is reached from no function, so in play the scene batch stays empty and `flush_draw_batch` draws nothing). Names are by use.
//! It stops before the replay's variables at 0x6EF7CC (`drive::replay_frames_store`).

#[cfg(test)]
use crate::race::owner::race;
use crate::session::{PlainBytes, PlainDword};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct DrawBatches {
    /// +0x0 (0x6D3CF4): `unk_6D3CF4`: the scene batch's vertices (the original's `flush_draw_batch`; 0x20-byte vertex records appended by the unreached `sub_45FCBA`; the length is the span to the next item).
    pub scene_vertices: PlainBytes<0x4000>,
    /// +0x4000 (0x6D7CF4): `unk_6D7CF4`: the scene batch's indices (the original's `flush_draw_batch`).
    pub scene_indices: PlainBytes<0x1800>,
    /// +0x5800 (0x6D94F4): `unk_6D94F4`: the plain batch's vertices (`race::flush_triangle_batch`).
    pub plain_vertices: PlainBytes<0x8000>,
    /// +0xd800 (0x6E14F4): `unk_6E14F4`: the plain batch's indices.
    pub plain_indices: PlainBytes<0x3000>,
    /// +0x10800 (0x6E44F4): `dword_6E44F4`: the plain batch's triangle count (`race::flush_triangle_batch`, `race::draw_race_scene`).
    pub plain_count: PlainDword,
    /// +0x10804 (0x6E44F8): `dword_6E44F8`: the plain batch's first word.
    pub plain_word: PlainDword,
    /// +0x10808 (0x6E44FC): `unk_6E44FC`: the textured batch's vertices (`race::flush_texture_batch`).
    pub textured_vertices: PlainBytes<0x8200>,
    /// +0x18a08 (0x6EC6FC): `unk_6EC6FC`: the textured batch's indices (`dword_6EDF5C` lies inside, at +0x1860).
    pub textured_indices: PlainBytes<0x30C0>,
    /// +0x1bac8 (0x6EF7BC): `dword_6EF7BC`: the textured batch's triangle count.
    pub textured_count: PlainDword,
    /// +0x1bacc (0x6EF7C0): `dword_6EF7C0`: the textured batch's first word.
    pub textured_word: PlainDword,
    /// +0x1bad0 (0x6EF7C4): `dword_6EF7C4`: the plain batch's count, kept when it is flushed.
    pub plain_kept: PlainDword,
    /// +0x1bad4 (0x6EF7C8): `dword_6EF7C8`: the textured batch's count, kept when it is flushed.
    pub textured_kept: PlainDword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece, with the image's first values.
impl DrawBatches {
    pub const fn new() -> Self {
        crate::cells::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START))
    }
}

/// IDA 0x6D3CF4, the first byte of the piece.
pub const START: usize = 0x6D3CF4;
/// IDA 0x6EF7CC, the end of the piece.
pub const END: usize = 0x6EF7CC;

const _: () = assert!(END - START == core::mem::size_of::<DrawBatches>());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            race().frame.draw_batches(),
        );
    }
}
