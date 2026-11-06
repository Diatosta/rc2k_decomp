//! The cars' blocks (`g_carBlockStore`, IDA 0x70F3EC .. 0x7194BC; phase 7, typed): ten car
//! blocks of 0xFE0 with the work car's layout (`physics::car_store::Car`, IDA's
//! `CarBlock`), then 0x210 bytes no ported code names. `g_carBlocks[n]` point at them
//! (blocks 0 .. 7 the race's cars; block 8 the replay's, `drive::load_replay_cars`);
//! `drive::load_car` copies the current car's block into `g_car` and `drive::store_car`
//! back. `#[repr(C)]` with the image's layout, over the race block's bytes (`CAR_BLOCKS`),
//! as `CAR`.

use crate::physics::car_store::Car;
use crate::race::owner::race;
use crate::session::Dword;

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct CarBlocks {
    /// +0x0 (0x70F3EC): the blocks, 0xFE0 apart.
    pub cars: [Car; 10],
    /// +0x9ec0 (0x7192AC): not named.
    pub unknown_tail: [Dword; 0x84],
}

const _: () = assert!(core::mem::size_of::<CarBlocks>() == 0xA0D0);

/// The piece: a field of `CARS_AND_REPLAY` since 2026-10-29 (it left the race
/// block with its group, phase 13).
pub struct InGroup;

impl core::ops::Deref for InGroup {
    type Target = CarBlocks;
    fn deref(&self) -> &CarBlocks {
        &race().replay.cars_and_replay.car_blocks
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
        &race().replay.cars_and_replay.car_blocks,
    ))
}

/// A car block by handle (phase 13): what `g_carBlocks` and `g_viewBlocks` hold where the
/// original kept the block's address. `BLOCK_TAG | n` for block `n`; 0 stays 0.
pub const BLOCK_TAG: u32 = 0xCB00_0000;
/// The blocks.
pub const BLOCK_COUNT: usize = 10;

/// The handle for IDA's `ida` (a block's address, or 0; anything else fails to compile).
pub const fn block_ref(ida: u32) -> u32 {
    if ida == 0 {
        return 0;
    }
    let off = ida as usize - START;
    assert!(
        ida as usize >= START && off.is_multiple_of(BLOCK_LEN) && off / BLOCK_LEN < BLOCK_COUNT
    );
    BLOCK_TAG | (off / BLOCK_LEN) as u32
}

/// `bytes` (a store's first values) with the dwords at `at` .. made `block_ref`s.
pub const fn with_block_refs<const N: usize>(
    mut bytes: [u8; N],
    at: usize,
    count: usize,
) -> [u8; N] {
    let mut k = 0;
    while k < count {
        let o = at + 4 * k;
        let v = u32::from_le_bytes([bytes[o], bytes[o + 1], bytes[o + 2], bytes[o + 3]]);
        let h = block_ref(v).to_le_bytes();
        bytes[o] = h[0];
        bytes[o + 1] = h[1];
        bytes[o + 2] = h[2];
        bytes[o + 3] = h[3];
        k += 1;
    }
    bytes
}

/// Where the block a handle names is (0 for 0). A value that is no handle ends the program
/// (as `Car::at` does an address off the blocks).
pub fn block_of(handle: u32) -> u32 {
    if handle == 0 {
        return 0;
    }
    let n = (handle ^ BLOCK_TAG) as usize;
    if n >= BLOCK_COUNT {
        crate::fatal::raise::end(
            "race::blocks_store::block_of",
            crate::errors::BadInput("a car block's handle that names none"),
        );
    }
    block_addr(n)
}

/// The handle of the block at `addr` (0 for 0); an address that is no block's start ends
/// the program.
pub fn handle_of(addr: u32) -> u32 {
    if addr == 0 {
        return 0;
    }
    let off = addr.wrapping_sub(blocks_addr()) as usize;
    if !off.is_multiple_of(BLOCK_LEN) || off / BLOCK_LEN >= BLOCK_COUNT {
        crate::fatal::raise::end(
            "race::blocks_store::handle_of",
            crate::errors::BadInput("a car block's address off the blocks"),
        );
    }
    BLOCK_TAG | (off / BLOCK_LEN) as u32
}

/// IDA's `g_viewBlocks` (two) and `g_carBlocks` (eight): their places hold handles
/// (`moved::point_into_homes` passes them over).
pub const VIEW_BLOCKS_IDA: usize = 0x63DBB0;
pub const CAR_BLOCKS_IDA: usize = 0x71B3B8;

/// The car this machine plays: the first, but in a network race its own. The original has
/// the first block's address wherever it means the player's car, so on a joined machine the
/// cockpit's effects, the mud and the listener followed the host's car.
pub fn own_car_number() -> usize {
    if crate::net::store::NET.session.get() == 1 {
        let local = crate::race::view_store::RACE_VIEW.local_car.get() as usize;
        if local < 8 { local } else { 0 }
    } else {
        0
    }
}

/// The block a handle names, none for 0; a value naming no block ends the program (as
/// `block_of`).
pub fn car_of(handle: u32) -> Option<&'static Car> {
    (block_of(handle) != 0).then(|| &CAR_BLOCKS.cars[(handle ^ BLOCK_TAG) as usize])
}

/// The block at `address` (what `block_of` or `RACE_WORK.car_block` gave), or the refusal
/// `what` for 0 (the original's read at a small offset from 0 fails).
pub fn block_at(address: u32, what: &'static str) -> Result<&'static Car, crate::errors::BadInput> {
    if address == 0 {
        return Err(crate::errors::BadInput(what));
    }
    Ok(Car::at(address))
}

/// The replay's starting state (`unk_7172EC`): its dwords, from car block 8 on.
pub const REPLAY_STATE_DWORDS: usize = 0xA04;
/// Where the replay's starting state begins in `CARS_AND_REPLAY`: car block 8.
pub const REPLAY_STATE_OFFSET: usize = 8 * BLOCK_LEN;
const _: () = assert!(
    8 * BLOCK_LEN + 4 * REPLAY_STATE_DWORDS
        <= core::mem::size_of::<crate::race::groups_store::CarsAndReplay>()
);

/// The replay's starting state: 0xA04 dwords from car block 8, running past the car blocks
/// into the replay work (their group holds both), as a pointer bounded by that length.
pub fn replay_state() -> crate::pointers::GamePtr {
    crate::pointers::GamePtr::to(&race().replay.cars_and_replay)
        .offset((8 * BLOCK_LEN) as u32)
        .limited((4 * REPLAY_STATE_DWORDS) as u32)
}

/// `own_car_number`'s block.
pub fn own_car() -> &'static Car {
    &CAR_BLOCKS.cars[own_car_number()]
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
            crate::pointers::game_address(&raw const CAR_BLOCKS.cars[1].gear),
            blocks_addr() + 0xFE0 + 0x1D4
        );
        assert_eq!(
            block_addr(8),
            crate::framework::game_addr(crate::framework::rva(START + 8 * 0xFE0))
        );
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host(blocks_addr()).cast_const(),
                END - START,
            )
        };
        assert_eq!(mine, &image_bytes(START, END - START)[..]);
    }
}
