//! The cars' blocks (`g_carBlockStore`, IDA 0x70F3EC .. 0x7172EC; phase 7, typed): the
//! eight race cars' blocks with the work car's layout (`physics::car_store::Car`,
//! IDA's `CarBlock`). `g_carBlocks[n]` point at them; `drive::load_car` copies the current
//! car's block into `g_car` and `drive::store_car` back.

use crate::physics::car_store::Car;
#[cfg(test)]
use crate::race::owner::race;

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct CarBlocks {
    /// +0x0 (0x70F3EC): the blocks.
    pub cars: [Car; BLOCK_COUNT],
}

impl CarBlocks {
    /// The blocks as a race starts.
    pub const fn new() -> Self {
        CarBlocks {
            cars: [const { Car::new() }; BLOCK_COUNT],
        }
    }
}

/// The piece: a field of the race's replay group (`race::owner_replay::Replay::car_blocks`).
/// One copy: no drawing part writes it (each view draws its own car,
/// `ViewScratch::car`; the view's value for its block is stored by the game's thread,
/// `ViewOutput::store_view_value`; the write watch's carried census, frame plan F7).
#[cfg(test)]
pub struct InGroup;

#[cfg(test)]
impl core::ops::Deref for InGroup {
    type Target = CarBlocks;
    #[inline]
    fn deref(&self) -> &CarBlocks {
        race().replay.car_blocks.get()
    }
}

#[cfg(test)]
pub static CAR_BLOCKS: InGroup = InGroup;

/// IDA 0x70F3EC, where the blocks began in the game's memory.
const IDA_START: usize = 0x70F3EC;
/// The distance between two blocks' addresses in the game's memory.
const IDA_BLOCK_LEN: usize = 0xFE0;

/// A car block by handle (phase 13): what `g_carBlocks` and `g_viewBlocks` hold where the
/// original kept the block's address. `BLOCK_TAG | n` for block `n`; 0 stays 0.
pub const BLOCK_TAG: u32 = 0xCB00_0000;
/// The blocks.
pub const BLOCK_COUNT: usize = 8;

/// The handle for IDA's `ida` (a block's address, or 0; anything else fails to compile).
pub const fn block_ref(ida: u32) -> u32 {
    if ida == 0 {
        return 0;
    }
    let off = ida as usize - IDA_START;
    assert!(
        ida as usize >= IDA_START
            && off.is_multiple_of(IDA_BLOCK_LEN)
            && off / IDA_BLOCK_LEN < BLOCK_COUNT
    );
    BLOCK_TAG | (off / IDA_BLOCK_LEN) as u32
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

/// The number of the block a handle names; refused for 0 (the address version found no car
/// block's place there) and the end of the program for a value naming none (as
/// `block_number`).
pub fn number_of(handle: u32) -> Result<usize, crate::errors::BadInput> {
    block_number(handle).ok_or(crate::errors::BadInput("a car block at no car's place"))
}

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
    use core::mem::size_of;

    #[test]
    fn blocks_start_as_the_games() {
        assert_eq!(size_of::<CarBlocks>(), BLOCK_COUNT * size_of::<Car>());
        assert_eq!(block_ref(0x70F3EC + 0xFE0), BLOCK_TAG | 1);
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
        // The first values: the store `race.owner.RACE.replay.car_blocks`
        // (`every_store_starts_as_the_games`).
    }
}
