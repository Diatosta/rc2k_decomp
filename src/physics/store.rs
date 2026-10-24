//! The variables of IDA 0x7594AA .. 0x75B940, out of RAL.EXE's image (phase 5,
//! `notes/conversion.md`): the car lists the menus walk, the carousel's set-ups, the race's
//! settings (stage, direction, car, HUD bits, the setup values), the championship's and the
//! race's results, and the damage model's figures, tables and work floats. One block with
//! the image's layout (`crate::store`), first values the image's (`store_data.rs`,
//! generated); the code names them by IDA's address (`framework::game_addr` takes such an
//! address to the block). Up to the gfx block at 0x75B940; from the last draw list
//! (`dword_7594AA`, `menu::player4_buttons_loop`'s), which IDA sizes 25 entries, so that it
//! is read whole from here (the menus' lists block ends before it).
//!
//! The damage model's inline asm reads these as `[{m} + RVA]` with `m = asm_base()`, the
//! image's `[{b} + RVA]` moved to the block.

use crate::block;

block!(pub mod vars {
    start: 0x7594AA,
    end: 0x75B940,
    runs: crate::physics::store_data::RUNS,
});

/// What an RVA of the block is added to in inline asm (`[{m} + RVA]`): the block's address
/// less its first byte's RVA.
pub fn asm_base() -> u32 {
    (vars::base() as u32).wrapping_sub(crate::framework::rva(vars::START) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asm_base_reaches_the_block() {
        let rva = crate::framework::rva(0x75B35C) as u32;
        assert_eq!(
            asm_base().wrapping_add(rva) as usize,
            crate::framework::game_addr(rva as usize)
        );
    }
}
