//! The current car's work copy: the physics and the drawing work on `g_car` (0x719FD8,
//! 0xFE0 bytes), copied in from the car's block (`g_carBlocks[g_currentCar & 7]`) and back
//! out to it (`drive::load_car`, `drive::store_car`). Fields of the copy met so far are
//! named in IDA by their addresses (`g_carGear` 0x71A1AC, ...; `physics/gearing.rs`).

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, rva};

pub const CAR_RVA: usize = rva(0x719FD8); // g_car: the current car's work copy
pub const CAR_LEN: usize = 0xFE0;
pub const CURRENT_CAR_RVA: usize = rva(0x71B3F8); // g_currentCar: its number (& 7)
pub use crate::race::objects::CAR_BLOCKS_RVA; // g_carBlocks: eight block pointers

/// Where a copy of the car functions finds its variables: the game's, or a test's.
#[derive(Clone, Copy)]
pub struct CarPlaces {
    pub current: u32,
    pub blocks: u32,
    pub work: u32,
}

impl CarPlaces {
    pub fn game() -> Self {
        Self {
            current: game_addr(CURRENT_CAR_RVA) as u32,
            blocks: game_addr(CAR_BLOCKS_RVA) as u32,
            work: game_addr(CAR_RVA) as u32,
        }
    }

    /// The current car's slot (0 .. 7) and its block, refused when the block is null
    /// (the original's `rep movsd` would fault there).
    pub unsafe fn block(&self) -> Result<(u32, u32), Refused> {
        unsafe {
            let slot = (self.current as *const u32).read_unaligned() & 7;
            let block = ((self.blocks + 4 * slot) as *const u32).read_unaligned();
            if block == 0 {
                return Err(BadInput("the current car's block is null").into());
            }
            Ok((slot, block))
        }
    }
}

/// What the two copies leave: eax the slot, ecx 0, esi and edi past the source and the
/// destination.
pub fn left(slot: u32, from: u32, to: u32) -> crate::screens::harness::Left {
    use crate::framework::{EAX, ECX, EDI, ESI};
    crate::screens::harness::Left::nothing()
        .with(EAX, slot)
        .with(ECX, 0)
        .with(ESI, from + CAR_LEN as u32)
        .with(EDI, to + CAR_LEN as u32)
}

/// Copies `CAR_LEN` bytes, dword by dword upwards as `rep movsd` does (the two never
/// overlap in the game; the order is kept anyway).
pub unsafe fn copy_dwords(from: u32, to: u32) {
    unsafe {
        for k in 0..(CAR_LEN as u32 / 4) {
            let v = ((from + 4 * k) as *const u32).read_unaligned();
            ((to + 4 * k) as *mut u32).write_unaligned(v);
        }
    }
}

/// For the two self-tests: a slot number, a table of eight blocks, a work copy and the
/// eight blocks of the test's, all reached through their addresses only.
pub struct TestCars {
    _current: Box<u32>,
    _blocks: Box<[u32; 8]>,
    _work: Vec<u8>,
    _cars: Vec<Vec<u8>>,
    places: CarPlaces,
    cars: [u32; 8],
}

impl TestCars {
    pub fn new() -> Self {
        let mut cars: Vec<Vec<u8>> = (0..8).map(|_| vec![0; CAR_LEN]).collect();
        let at: [u32; 8] = std::array::from_fn(|n| cars[n].as_mut_ptr() as u32);
        let mut current = Box::new(0u32);
        let mut blocks = Box::new(at);
        let mut work = vec![0u8; CAR_LEN];
        let places = CarPlaces {
            current: &mut *current as *mut u32 as u32,
            blocks: blocks.as_mut_ptr() as u32,
            work: work.as_mut_ptr() as u32,
        };
        Self {
            _current: current,
            _blocks: blocks,
            _work: work,
            _cars: cars,
            places,
            cars: at,
        }
    }

    pub fn places(&self) -> CarPlaces {
        self.places
    }

    /// Sets the slot number and block `n` of the table (0 for a null block).
    pub fn set(&self, current: u32, n: usize, block: Option<u32>) {
        unsafe {
            (self.places.current as *mut u32).write(current);
            if let Some(b) = block {
                (self.places.blocks as *mut u32).add(n).write(b);
            }
        }
    }

    /// Fills the blocks by `seed`, the work copy with 0xAA.
    pub fn fill(&self, seed: u32) {
        unsafe {
            std::ptr::write_bytes(self.places.work as *mut u8, 0xAA, CAR_LEN);
            for (n, &c) in self.cars.iter().enumerate() {
                let at = c as *mut u8;
                let mut s = seed.wrapping_add(n as u32 * 0x9E37_79B9);
                for k in 0..CAR_LEN {
                    s = s.wrapping_mul(1_103_515_245).wrapping_add(12345);
                    at.add(k).write((s >> 16) as u8);
                }
            }
        }
    }

    /// The work copy and every block, for comparing.
    pub fn regions(&self) -> Vec<crate::service::harness::Region> {
        let mut r = vec![(self.places.work, CAR_LEN)];
        r.extend(self.cars.iter().map(|&c| (c, CAR_LEN)));
        r
    }
}

impl Default for TestCars {
    fn default() -> Self {
        Self::new()
    }
}
