//! The current car's work copy: the physics and the drawing work on it (0xFE0 bytes), copied in
//! from the car's block and back out to it (`drive::load_car`, `drive::store_car`).

use crate::errors::Refused;
use crate::physics::car_store::Car;
use crate::race::owner_shared::RaceShared;

/// The current car's block (by its handle), refused when null.
pub fn current_block(shared: &RaceShared) -> Result<&Car, Refused> {
    use rc2k_core::drive::work_car::{car_block, car_slot};
    let slot = car_slot(shared.work.get().current_car.get());
    car_block(u32::from(shared.work.get().has_car_block(slot as usize)))?;

    Ok(shared.race_car(slot as usize)?)
}

/// Copies the car record, dword by dword upwards (the two never overlap in the game; the order
/// is kept anyway).
pub fn copy_car(from: &Car, to: &Car) {
    use crate::memory::cells::Cells;
    to.copy_from(from);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::paths::{field_bytes, set_field_range};

    #[test]
    fn the_typed_copy_writes_every_field_of_the_source() {
        let mut seed = 0x1234_5678u32;
        let flat_len = field_bytes(&Car::new(), "").len();
        let mut random = || {
            let mut b = vec![0u8; flat_len];
            for x in &mut b {
                seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
                *x = (seed >> 16) as u8;
            }
            b
        };
        for _ in 0..20 {
            let (from, typed): (Box<Car>, Box<Car>) = (Box::new(Car::new()), Box::new(Car::new()));
            assert!(set_field_range(&*from, "", &random()));
            assert!(set_field_range(&*typed, "", &random()));
            copy_car(&from, &typed);
            assert_eq!(field_bytes(&*from, ""), field_bytes(&*typed, ""));
        }
    }
}
