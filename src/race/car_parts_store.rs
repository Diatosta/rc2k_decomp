//! What the drawing keeps of each car beside its block: the parts' matrices, the shown engine
//! speed, the marks' switches. `race::car_parts::ready_car_parts` (the game's thread, before any
//! view is drawn) makes them from the car's state; the drawing parts only read them, as the car
//! blocks are read. They are kept apart from the physics' record so that no physics step touches
//! them.

use crate::race::blocks_store::{BLOCK_COUNT, block_number};

/// The parts' switch bytes of a car.
pub const PART_SWITCHES_LEN: usize = 0x11;

/// One car's drawing state.
#[derive(Clone, rc2k_derive::PlainRecord)]
pub struct CarPart {
    /// The engine speed shown (not below the idle speed).
    pub shown_engine_speed: f32,
    /// The four parts' matrices (the wheels, placed and turned about their axles).
    pub part_matrices: [f32; 64],
    /// The body's turn.
    pub body_matrix: [f32; 16],
    /// The car's place.
    pub place_matrix: [f32; 16],
    /// The body's turn times the car's place.
    pub whole_matrix: [f32; 16],
    /// 1 for the cars in a mode that shows the mode's parts.
    pub mode_on: u32,
    /// The marks' count x 0x20 (0 with the lights off or the electrics damaged).
    pub marks_count: u32,
    /// The parts' switch bytes.
    #[plain(bytes)]
    pub part_switches: [u8; PART_SWITCHES_LEN],
}

impl CarPart {
    const fn new() -> Self {
        CarPart {
            shown_engine_speed: 0.0,
            part_matrices: [0.0; 64],
            body_matrix: [0.0; 16],
            place_matrix: [0.0; 16],
            whole_matrix: [0.0; 16],
            mode_on: 0,
            marks_count: 0,
            part_switches: [0; PART_SWITCHES_LEN],
        }
    }
}

/// The eight cars' drawing state, in the order of the car blocks.
#[derive(Clone, rc2k_derive::PlainRecord)]
pub struct CarParts {
    pub cars: [CarPart; BLOCK_COUNT],
}

impl CarParts {
    pub const fn new() -> Self {
        CarParts {
            cars: [const { CarPart::new() }; BLOCK_COUNT],
        }
    }

    /// The state of the car whose block a handle names, none for 0; a value naming no block ends
    /// the program (as `CarBlocks::of`).
    pub fn of(&self, handle: u32) -> Option<&CarPart> {
        block_number(handle).map(|n| &self.cars[n])
    }
}

impl crate::memory::drawn_table::DrawnTable<'_, CarParts> {
    /// A copy of the state of the car in block `n`.
    pub fn car(self, n: usize) -> CarPart {
        self.read(|parts| parts.cars[n].clone())
    }
}
