//! What the drawing keeps of each car beside its block: the parts' matrices, the shown engine
//! speed, the marks' switches. `race::car_parts::ready_car_parts` (the game's thread, before any
//! view is drawn) makes them from the car's state; the drawing parts only read them, as the car
//! blocks are read. They are kept apart from the physics' record so that no physics step touches
//! them.

use crate::memory::plain_floats::PlainFloats;
use crate::memory::session::{Bytes, PlainDword, PlainFloat};
use crate::race::blocks_store::{BLOCK_COUNT, block_number};

/// The parts' switch bytes of a car.
pub const PART_SWITCHES_LEN: usize = 0x11;

/// One car's drawing state.
#[derive(rc2k_derive::Cells)]
pub struct CarPartCells {
    /// The engine speed shown (not below the idle speed).
    pub shown_engine_speed: PlainFloat,
    /// The four parts' matrices (the wheels, placed and turned about their axles).
    pub part_matrices: PlainFloats<64>,
    /// The body's turn.
    pub body_matrix: PlainFloats<16>,
    /// The car's place.
    pub place_matrix: PlainFloats<16>,
    /// The body's turn times the car's place.
    pub whole_matrix: PlainFloats<16>,
    /// 1 for the cars in a mode that shows the mode's parts.
    pub mode_on: PlainDword,
    /// The marks' count x 0x20 (0 with the lights off or the electrics damaged).
    pub marks_count: PlainDword,
    /// The parts' switch bytes.
    pub part_switches: Bytes<PART_SWITCHES_LEN>,
}

impl CarPartCells {
    const fn new() -> Self {
        crate::memory::cells::zeroed_cells()
    }
}

/// The eight cars' drawing state, in the order of the car blocks.
#[derive(rc2k_derive::Cells)]
pub struct CarParts {
    pub cars: [CarPartCells; BLOCK_COUNT],
}

impl CarParts {
    pub const fn new() -> Self {
        CarParts {
            cars: [const { CarPartCells::new() }; BLOCK_COUNT],
        }
    }

    /// The state of the car whose block a handle names, none for 0; a value naming no block ends
    /// the program (as `CarBlocks::of`).
    pub fn of(&self, handle: u32) -> Option<&CarPartCells> {
        block_number(handle).map(|n| &self.cars[n])
    }
}
