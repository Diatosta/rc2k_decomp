//! The cars' blocks: the eight race cars' blocks, laid out as the work car's
//! (`physics::car_store::Car`). `car_blocks[n]` point at them; `drive::load_car` copies the current
//! car's block into `car` and `drive::store_car` copies it back.

use crate::physics::car_store::Car;
#[cfg(test)]
use crate::test_support::own::race;

#[derive(rc2k_derive::Cells)]
pub struct CarBlocks {
    /// The blocks.
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

/// The piece: a field of the race's replay group (`race::owner_replay::Replay::car_blocks`). One
/// copy: no drawing part writes it (each view draws its own car, `ViewScratch::car`, and the view's
/// value for its block is stored by the game's thread, `ViewOutput::store_view_value`).
#[cfg(test)]
pub struct InGroup;

#[cfg(test)]
impl core::ops::Deref for InGroup {
    type Target = CarBlocks;
    #[inline]
    fn deref(&self) -> &CarBlocks {
        race().shared.car_blocks.get()
    }
}

#[cfg(test)]
pub static CAR_BLOCKS: InGroup = InGroup;

/// A car block by handle, as `car_blocks` and `view_blocks` hold it: `BLOCK_TAG | n` for block
/// `n`, 0 for none.
pub const BLOCK_TAG: u32 = 0xCB00_0000;
/// The blocks.
pub const BLOCK_COUNT: usize = 8;

/// The handle of block `n` (a number past the blocks fails to compile).
pub(super) const fn block_handle(n: usize) -> u32 {
    assert!(n < BLOCK_COUNT);
    BLOCK_TAG | n as u32
}

/// The number of the block a handle names, none for 0. A value that is no handle ends the program.
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
pub(super) fn known_block(handle: u32) -> Option<usize> {
    let n = (handle ^ BLOCK_TAG) as usize;
    (n < BLOCK_COUNT).then_some(n)
}

/// The number of the block a handle names; refused for 0 (no car block's place there) and the end
/// of the program for a value naming none (as `block_number`).
pub fn number_of(handle: u32) -> Result<usize, crate::errors::BadInput> {
    block_number(handle).ok_or(crate::errors::BadInput("a car block at no car's place"))
}

impl CarBlocks {
    /// The block a handle names (a `car_blocks` entry, a view's block, or a dword read on past the
    /// eight); any other value is refused.
    pub fn named(&self, handle: u32) -> Result<&Car, crate::errors::BadInput> {
        match known_block(handle) {
            Some(n) => Ok(&self.cars[n]),
            None => {
                crate::program_log::log(&format!("trap: {handle:#x} is no car block's place"));
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

    /// The block a handle names (what `RaceWork::car_block` gave), or the refusal `what` for 0; a
    /// value naming no block ends the program.
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
        assert_eq!(block_handle(1), BLOCK_TAG | 1);
        for n in 0..BLOCK_COUNT {
            assert_eq!(known_block(BLOCK_TAG | n as u32), Some(n));
            assert!(core::ptr::eq(
                crate::test_support::own::race()
                    .car_blocks()
                    .named(BLOCK_TAG | n as u32)
                    .unwrap(),
                &raw const crate::test_support::own::race().car_blocks().cars[n]
            ));
        }
        assert_eq!(known_block(BLOCK_TAG | BLOCK_COUNT as u32), None);
        assert_eq!(known_block(0), None);
        assert_eq!(block_number(0), None);
    }
}
