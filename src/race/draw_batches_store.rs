//! The race frame's draw batches (part R3): the scene's batch (`race::flush_draw_batch`) with its vertices and indices, and the two batches `race::draw_race_scene` flushes (`race::flush_triangle_batch`, the plain one; `race::flush_texture_batch`, the textured one): vertices, indices, counts, first words and the counts kept. The buffers' lengths are readings (up to the next item). Names are readings unless IDA's.
//! It stops before the replay's variables at 0x6EF7CC (`drive::replay_frames_store`).

use crate::session::{Bytes, Dword};

#[repr(C)]
pub struct DrawBatches {
    /// +0x0 (0x6D3CF4): `unk_6D3CF4`: the scene batch's vertices (`race::flush_draw_batch`; a reading of its length).
    pub scene_vertices: Bytes<0x4000>,
    /// +0x4000 (0x6D7CF4): `unk_6D7CF4`: the scene batch's indices (`race::flush_draw_batch`).
    pub scene_indices: Bytes<0x1800>,
    /// +0x5800 (0x6D94F4): `unk_6D94F4`: the plain batch's vertices (`race::flush_triangle_batch`).
    pub plain_vertices: Bytes<0x8000>,
    /// +0xd800 (0x6E14F4): `unk_6E14F4`: the plain batch's indices.
    pub plain_indices: Bytes<0x3000>,
    /// +0x10800 (0x6E44F4): `dword_6E44F4`: the plain batch's triangle count (`race::flush_triangle_batch`, `race::draw_race_scene`).
    pub plain_count: Dword,
    /// +0x10804 (0x6E44F8): `dword_6E44F8`: the plain batch's first word.
    pub plain_word: Dword,
    /// +0x10808 (0x6E44FC): `unk_6E44FC`: the textured batch's vertices (`race::flush_texture_batch`).
    pub textured_vertices: Bytes<0x8200>,
    /// +0x18a08 (0x6EC6FC): `unk_6EC6FC`: the textured batch's indices (`dword_6EDF5C` lies inside, at +0x1860).
    pub textured_indices: Bytes<0x30C0>,
    /// +0x1bac8 (0x6EF7BC): `dword_6EF7BC`: the textured batch's triangle count.
    pub textured_count: Dword,
    /// +0x1bacc (0x6EF7C0): `dword_6EF7C0`: the textured batch's first word.
    pub textured_word: Dword,
    /// +0x1bad0 (0x6EF7C4): `dword_6EF7C4`: the plain batch's count, kept when it is flushed.
    pub plain_kept: Dword,
    /// +0x1bad4 (0x6EF7C8): `dword_6EF7C8`: the textured batch's count, kept when it is flushed.
    pub textured_kept: Dword,
}

/// The piece: the race block's bytes at IDA 0x6D3CF4 seen as `DrawBatches`. They stay there
/// while code reaches them by RVA from `race::store::asm_base()` and by IDA address
/// (`moved::place`): one copy for all three.
pub struct InRaceBlock;

impl core::ops::Deref for InRaceBlock {
    type Target = DrawBatches;
    fn deref(&self) -> &DrawBatches {
        // SAFETY: the race block lives as long as the program and is aligned as the test
        // checks; every field is an atomic or a cell, as the block's bytes were shared.
        unsafe { &*(crate::pointers::host_at(piece_addr()) as *const DrawBatches) }
    }
}

pub static DRAW_BATCHES: InRaceBlock = InRaceBlock;

/// IDA 0x6D3CF4, the first byte of the piece.
pub const START: usize = 0x6D3CF4;
/// IDA 0x6EF7CC, the end of the piece.
pub const END: usize = 0x6EF7CC;

const _: () = assert!(END - START == core::mem::size_of::<DrawBatches>());

/// Where IDA 0x6D3CF4 is now (in the race block).
pub fn piece_addr() -> usize {
    crate::race::store::home(START)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn piece_sits_on_the_race_block() {
        assert_eq!(piece_addr() % core::mem::align_of::<DrawBatches>(), 0);
        assert_eq!(piece_addr(), crate::moved::place(START));
        assert_eq!(
            crate::moved::place(END - 1),
            piece_addr() + (END - START) - 1
        );

        assert_eq!(offset_of!(DrawBatches, scene_vertices), 0x6D3CF4 - START);
        assert_eq!(offset_of!(DrawBatches, scene_indices), 0x6D7CF4 - START);
        assert_eq!(offset_of!(DrawBatches, plain_vertices), 0x6D94F4 - START);
        assert_eq!(offset_of!(DrawBatches, plain_indices), 0x6E14F4 - START);
        assert_eq!(offset_of!(DrawBatches, plain_count), 0x6E44F4 - START);
        assert_eq!(offset_of!(DrawBatches, textured_vertices), 0x6E44FC - START);
        assert_eq!(offset_of!(DrawBatches, textured_indices), 0x6EC6FC - START);
        assert_eq!(offset_of!(DrawBatches, textured_count), 0x6EF7BC - START);
        assert_eq!(offset_of!(DrawBatches, textured_kept), 0x6EF7C8 - START);
    }
}
