//! The cars' blocks (`g_carBlockStore`, IDA 0x70F3EC .. 0x7194BC; phase 7, typed): ten car
//! blocks of 0xFE0 with the work car's layout (`physics::car_store::Car`, IDA's
//! `CarBlock`), then 0x210 bytes no ported code names. `g_carBlocks[n]` point at them
//! (blocks 0 .. 7 the race's cars; block 8 the replay's, `drive::load_replay_cars`);
//! `drive::load_car` copies the current car's block into `g_car` and `drive::store_car`
//! back. `#[repr(C)]` with the image's layout, over the race block's bytes (`CAR_BLOCKS`),
//! as `CAR`.

use crate::physics::car_store::Car;
#[cfg(test)]
use crate::race::owner::race;
use crate::session::PlainDword;

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct CarBlocks {
    /// +0x0 (0x70F3EC): the blocks, 0xFE0 apart.
    pub cars: [Car; 10],
    /// +0x9ec0 (0x7192AC): no code reads or writes it by name (no IDA xref to any of its bytes, 2026-11-10); it is carried in the replay's starting state (`REPLAY_STATE_DWORDS`), which runs over it.
    pub unused_tail: [PlainDword; 0x84],
}

const _: () = assert!(core::mem::size_of::<CarBlocks>() == 0xA0D0);

/// The piece: a field of `CARS_AND_REPLAY` since 2026-10-29 (it left the race block with
/// its group, phase 13). One copy: no drawing part writes it (each view draws its own car,
/// `ViewScratch::car`; the view's value for its block is stored by the game's thread,
/// `ViewOutput::store_view_value`; the write watch's carried census, frame plan F7).
#[cfg(test)]
pub struct InGroup;

#[cfg(test)]
impl core::ops::Deref for InGroup {
    type Target = CarBlocks;
    #[inline]
    fn deref(&self) -> &CarBlocks {
        &race().replay.cars_and_replay.get().car_blocks
    }
}

#[cfg(test)]
pub static CAR_BLOCKS: InGroup = InGroup;

/// IDA 0x70F3EC, the first byte of the piece.
pub const START: usize = 0x70F3EC;
/// IDA 0x7194BC, the end of the piece.
pub const END: usize = 0x7194BC;
/// A block's length.
pub const BLOCK_LEN: usize = core::mem::size_of::<Car>();

const _: () = assert!(END - START == core::mem::size_of::<CarBlocks>());

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

/// The number of the block a handle names, none for 0. A value that is no handle ends the
/// program.
pub fn block_number(handle: u32) -> Option<usize> {
    if handle == 0 {
        return None;
    }
    match known_block(handle) {
        Some(n) => Some(n),
        None => crate::fatal::raise::end(
            "race::blocks_store::block_of",
            crate::errors::BadInput("a car block's handle that names none"),
        ),
    }
}

/// The number of the block a handle names; none for 0 or any value that is no handle.
pub fn known_block(handle: u32) -> Option<usize> {
    let n = (handle ^ BLOCK_TAG) as usize;
    (n < BLOCK_COUNT).then_some(n)
}

/// IDA's `g_viewBlocks` (two) and `g_carBlocks` (eight): their places hold handles.
pub const VIEW_BLOCKS_IDA: usize = 0x63DBB0;
pub const CAR_BLOCKS_IDA: usize = 0x71B3B8;

/// The number of the block a handle names; refused for 0 (the address version found no car
/// block's place there) and the end of the program for a value naming none (as
/// `block_number`).
pub fn number_of(handle: u32) -> Result<usize, crate::errors::BadInput> {
    block_number(handle).ok_or(crate::errors::BadInput("a car block at no car's place"))
}

/// The replay's starting state (`CarsAndReplay.car_blocks.cars[8].frame_head[0]`): its dwords, from car block 8 on.
/// The original copies it as one run of 0xA04 dwords; the run is whole fields: car blocks 8
/// and 9, the blocks' unused tail, and the replay work's `replay_tape` (it ends there, as
/// the assertion below checks).
pub const REPLAY_STATE_DWORDS: usize = 0xA04;
/// Where the replay's starting state begins in `CARS_AND_REPLAY`: car block 8.
pub const REPLAY_STATE_OFFSET: usize = 8 * BLOCK_LEN;
const _: () = assert!(
    REPLAY_STATE_OFFSET + 4 * REPLAY_STATE_DWORDS
        == core::mem::offset_of!(crate::race::groups_store::CarsAndReplay, replay_work)
            + core::mem::offset_of!(crate::race::replay_work_store::ReplayWork, replay_tape)
            + core::mem::size_of::<[PlainDword; 400]>()
);

impl CarBlocks {
    /// The block a handle names (a `g_carBlocks` entry, a view's block, or a dword the
    /// original reads on to past the eight); any other value is refused.
    pub fn named(&self, handle: u32) -> Result<&Car, crate::errors::BadInput> {
        match known_block(handle) {
            Some(n) => Ok(&self.cars[n]),
            None => {
                crate::framework::log(&format!("trap: {handle:#x} is no car block's place"));
                Err(crate::errors::BadInput("a car block at no car's place"))
            }
        }
    }

    /// The handle of the block `car`; a car that is none of the blocks ends the program.
    pub fn handle_of(&self, car: &Car) -> u32 {
        match self.cars.iter().position(|c| core::ptr::eq(c, car)) {
            Some(n) => BLOCK_TAG | n as u32,
            None => crate::fatal::raise::end(
                "race::blocks_store::handle_of_car",
                crate::errors::BadInput("a car that is none of the blocks"),
            ),
        }
    }

    /// The block a handle names, none for 0; a value naming no block ends the program (as
    /// `block_number`).
    pub fn of(&self, handle: u32) -> Option<&Car> {
        block_number(handle).map(|n| &self.cars[n])
    }

    /// The block a handle names (what `RaceWork::car_block` gave), or the refusal `what` for
    /// 0 (the original's read at a small offset from 0 fails); a value naming no block ends
    /// the program.
    pub fn at(&self, handle: u32, what: &'static str) -> Result<&Car, crate::errors::BadInput> {
        if handle == 0 {
            return Err(crate::errors::BadInput(what));
        }
        Ok(self
            .named(handle)
            .unwrap_or_else(|e| crate::fatal::raise::end("physics::Car::at", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn blocks_start_as_the_games() {
        assert_eq!(offset_of!(CarBlocks, unused_tail), 0x9EC0);
        assert_eq!(BLOCK_LEN, 0xFE0);
        assert_eq!(offset_of!(Car, gear), 0x1D4);
        for n in 0..BLOCK_COUNT {
            assert_eq!(known_block(BLOCK_TAG | n as u32), Some(n));
            assert!(core::ptr::eq(
                race().car_blocks().named(BLOCK_TAG | n as u32).unwrap(),
                &raw const race().car_blocks().cars[n]
            ));
        }
        assert_eq!(known_block(BLOCK_TAG | BLOCK_COUNT as u32), None);
        assert_eq!(known_block(0), None);
        assert_eq!(block_number(0), None);
        // The first values: the store `race.owner.RACE.replay.cars_and_replay` they begin
        // (`every_store_starts_as_the_games`).
    }
}
