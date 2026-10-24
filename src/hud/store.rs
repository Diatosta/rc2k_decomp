//! The HUD's variables, out of RAL.EXE's image (phase 5, `notes/conversion.md`): IDA
//! 0x75B940 .. 0x764F2C, the whole of it the HUD's: the rectangle table (`g_hudRects`), the
//! quad record, the layouts' set-up values and floats, the dials' and needles' work
//! variables, the batch's vertices (`unk_75C324`, 0x400 of them), indices (`unk_764324`,
//! 0x600) and the two counts (`g_hudVertexCount`, `g_hudIndexCount`). Only `hud` and
//! `d3d::draw_hud_batch` (which asks `vertices()` and `indices()`) read or write it. The
//! image keeps the two pointers (0x764F2C, 0x764F30) the original's batch drawer followed;
//! nothing reads them now.
//!
//! One block with the image's layout (`gfx::block`): an address `X` of IDA is at
//! `base() + (X - START)`; a `Slot` is such an address, checked against the block.

pub use crate::gfx::block::{game_addr, game_read, game_write};

crate::region!(Hud, STORE, 0x75B940, 0x764F2C, crate::hud::store_data::RUNS);

/// The float at `place` (a block's or the image's).
pub fn read_f32(place: impl crate::gfx::block::Where) -> f32 {
    f32::from_bits(unsafe { game_read::<u32>(place) })
}

pub const START: usize = 0x75B940;

pub type Slot = crate::gfx::block::Slot<Hud>;

/// The block's address for inline assembly (`[hud + offset]`, `offset` IDA's address less
/// `START`).
pub fn base() -> u32 {
    crate::gfx::block::base::<Hud>()
}

/// The batch's vertices (`unk_75C324`).
pub fn vertices() -> usize {
    Slot::at(0x75C324).address()
}

/// The batch's indices (`unk_764324`).
pub fn indices() -> usize {
    Slot::at(0x764324).address()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::block::first_values_are_the_games::<Hud>(&STORE);
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(crate::moved::place(0x75C060), Slot::at(0x75C060).address());
        assert_eq!(crate::moved::place(START), STORE.addr());
        assert_eq!(vertices(), STORE.addr() + 0x75C324 - START);
        assert_eq!(indices(), STORE.addr() + 0x764324 - START);
    }
}
