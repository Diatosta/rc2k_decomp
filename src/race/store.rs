//! The race core's variables, out of RAL.EXE's image (phase 5, `notes/conversion.md`): IDA
//! 0x5BEC48 .. 0x747AD8, from the PNG reader's memory list to the menus' tables. The
//! replay's file state and index, the course, the cars' state, physics and sounds, the
//! camera and views, the scene's work, the stage's and the records' tables, the timing.
//!
//! One block with the image's layout (`crate::store`), first values the image's
//! (`store_data.rs`, generated). Six ranges inside it had moved before into blocks of
//! their own (the mouse, `g_raceSoundsOn`, the hexagons' batch, ...): their bytes here are
//! not used, and `crate::moved::MOVED` lists this block as the seven ranges around them.
//! The code names the variables by IDA's address (`framework::game_addr` takes such an
//! address here); the inline asm reads them as `[{m} + RVA]` with `m = asm_base()`.
//!
//! `buffers`: IDA 0x960000 .. 0xA8C180, the image's zeroed tail (the object file and its
//! materials, the faces, the replay viewer's blocks and cameras, the sort and depth lists,
//! the checkpoints' angles, the nearest cameras, the shuffled words, the polygon
//! vertices); its inline asm reads it as `[{bf} + RVA]` with `bf = buffers_asm_base()`.

use crate::block;

block!(pub mod vars {
    start: 0x5BEC48,
    end: 0x747AD8,
    runs: crate::race::store_data::RUNS,
});

block!(pub mod buffers {
    start: 0x960000,
    end: 0xA8C180,
    runs: &[],
});

/// What an RVA of `buffers` is added to in inline asm (`[{bf} + RVA]`).
pub fn buffers_asm_base() -> u32 {
    (buffers::base() as u32).wrapping_sub(crate::framework::rva(buffers::START) as u32)
}

/// The ranges of the block that are its own (IDA start, end): all but the holes.
pub const OWN: [(usize, usize); 7] = [
    (0x5BEC48, 0x5C6924),
    (0x5C6948, 0x60B2CC),
    (0x60B2D0, 0x6218F0),
    (0x6218F4, 0x62423C),
    (0x628A44, 0x6D1870),
    (0x6D18C8, 0x6D1DD4),
    (0x6D1DD8, 0x747AD8),
];

/// Where IDA's address `ida` of the block is.
pub fn home(ida: usize) -> usize {
    vars::base() + (ida - vars::START)
}

/// What an RVA of the block is added to in inline asm (`[{m} + RVA]`): the block's address
/// less its first byte's RVA.
pub fn asm_base() -> u32 {
    (vars::base() as u32).wrapping_sub(crate::framework::rva(vars::START) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_holes_are_the_moved_ranges() {
        for w in OWN.windows(2) {
            let (end, next) = (w[0].1, w[1].0);
            let hole = crate::moved::MOVED.iter().find(|m| m.ida == end);
            assert!(hole.is_some_and(|m| m.ida + m.len == next), "{end:#x}");
        }
    }

    #[test]
    fn asm_base_reaches_the_block() {
        let rva = crate::framework::rva(0x71A30C) as u32;
        assert_eq!(
            asm_base().wrapping_add(rva) as usize,
            crate::framework::game_addr(rva as usize)
        );
    }
}
