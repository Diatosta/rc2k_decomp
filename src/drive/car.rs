//! The current car's work copy: the physics and the drawing work on `g_car` (0x719FD8,
//! 0xFE0 bytes), copied in from the car's block (`g_carBlocks[g_currentCar & 7]`) and back
//! out to it (`drive::load_car`, `drive::store_car`). Fields of the copy met so far are
//! named in IDA by their addresses (`g_carGear` 0x71A1AC, ...; `physics/gearing.rs`).

use crate::errors::Refused;
use crate::physics::car_store::Car;
use crate::race::owner_shared::RaceShared;

pub const CAR_LEN: usize = 0xFE0;
const _: () = assert!(core::mem::size_of::<Car>() == CAR_LEN);

/// The current car's block (`g_carBlocks[g_currentCar & 7]`, by its handle), refused when
/// null (the original's `rep movsd` would fault there).
pub fn current_block(shared: &RaceShared) -> Result<&'static Car, Refused> {
    use rc2k_core::drive::work_car::{car_block, car_slot};
    let slot = car_slot(shared.work.current_car.get());
    car_block(u32::from(shared.work.has_car_block(slot as usize)))?;

    Ok(shared.work.car(slot as usize)?)
}

/// Copies `CAR_LEN` bytes, dword by dword upwards as `rep movsd` does (the two never
/// overlap in the game; the order is kept anyway).
pub fn copy_car(from: &Car, to: &Car) {
    use crate::cells::Cells;
    to.copy_from(from);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cells::{Cells, cells_of_bytes};

    /// A record's bytes, read through its runs.
    fn bytes(car: &Car) -> Vec<u8> {
        let mut out = vec![0; CAR_LEN];
        let mut at = 0;
        while at < CAR_LEN {
            let (start, run) = car.cell_run(at);
            let n = (run.len() - (at - start)).min(CAR_LEN - at);
            run.load(at - start, &mut out[at..at + n]);
            at += n;
        }
        out
    }

    #[test]
    fn the_typed_copy_writes_what_the_walked_copy_writes() {
        let mut seed = 0x1234_5678u32;
        let mut random = || {
            let mut b = [0u8; CAR_LEN];
            for x in &mut b {
                seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
                *x = (seed >> 16) as u8;
            }
            b
        };
        for _ in 0..20 {
            let from: Box<Car> = Box::new(cells_of_bytes(random()));
            let (walked, typed): (Box<Car>, Box<Car>) = (
                Box::new(cells_of_bytes(random())),
                Box::new(cells_of_bytes(random())),
            );
            crate::cells::copy_cells((&*from, 0), (&*walked, 0), CAR_LEN);
            copy_car(&from, &typed);
            assert_eq!(bytes(&walked), bytes(&typed));
            assert_eq!(bytes(&from), bytes(&typed));
        }
    }
}
