//! The cars' blocks (`g_carBlockStore`, IDA 0x70F3EC .. 0x7194BC; phase 7, typed): ten car
//! blocks of 0xFE0 with the work car's layout (`physics::car_store::Car`, IDA's
//! `CarBlock`), then 0x210 bytes no ported code names. `g_carBlocks[n]` point at them
//! (blocks 0 .. 7 the race's cars; block 8 the replay's, `drive::load_replay_cars`);
//! `drive::load_car` copies the current car's block into `g_car` and `drive::store_car`
//! back. `#[repr(C)]` with the image's layout, over the race block's bytes (`CAR_BLOCKS`),
//! as `CAR`.

use crate::physics::car_store::Car;
use crate::session::Dword;

#[repr(C)]
pub struct CarBlocks {
    /// +0x0 (0x70F3EC): the blocks, 0xFE0 apart.
    pub cars: [Car; 10],
    /// +0x9ec0 (0x7192AC): not named.
    pub unknown_tail: [Dword; 0x84],
}

const _: () = assert!(core::mem::size_of::<CarBlocks>() == 0xA0D0);

/// The piece: a field of `race::groups_store::CARS_AND_REPLAY` since 2026-10-29 (it left the race
/// block with its group, phase 13).
pub struct InGroup;

impl core::ops::Deref for InGroup {
    type Target = CarBlocks;
    fn deref(&self) -> &CarBlocks {
        &crate::race::groups_store::CARS_AND_REPLAY.car_blocks
    }
}

pub static CAR_BLOCKS: InGroup = InGroup;

/// IDA 0x70F3EC, the first byte of the piece.
pub const START: usize = 0x70F3EC;
/// IDA 0x7194BC, the end of the piece.
pub const END: usize = 0x7194BC;
/// A block's length.
pub const BLOCK_LEN: usize = core::mem::size_of::<Car>();

const _: () = assert!(END - START == core::mem::size_of::<CarBlocks>());

/// Where IDA 0x70F3EC is now (its group's static).
pub fn blocks_addr() -> u32 {
    crate::pointers::game_address(core::ptr::from_ref(
        &crate::race::groups_store::CARS_AND_REPLAY.car_blocks,
    ))
}

/// Where block `n` is now; `n` unchecked, as the game's `0x70F3EC + 0xFE0 n` (an address
/// only: reading it is the caller's business).
pub fn block_addr(n: usize) -> u32 {
    blocks_addr() + (BLOCK_LEN * n) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moved::image_bytes;
    use core::mem::offset_of;

    #[test]
    fn blocks_start_as_the_games() {
        assert_eq!(offset_of!(CarBlocks, unknown_tail), 0x9EC0);
        assert_eq!(blocks_addr() % 4, 0);
        assert_eq!(
            blocks_addr(),
            crate::framework::game_addr(crate::framework::rva(START))
        );
        assert_eq!(
            crate::pointers::game_address(&CAR_BLOCKS.cars[1].gear as *const _),
            blocks_addr() + 0xFE0 + 0x1D4
        );
        assert_eq!(
            block_addr(8),
            crate::framework::game_addr(crate::framework::rva(START + 8 * 0xFE0))
        );
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host(blocks_addr()) as *const u8,
                END - START,
            )
        };
        assert_eq!(mine, &image_bytes(START, END - START)[..]);
    }
}
