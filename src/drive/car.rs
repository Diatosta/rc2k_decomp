//! The current car's work copy: the physics and the drawing work on it (0xFE0 bytes), copied in
//! from the car's block and back out to it (`drive::load_car`, `drive::store_car`).

use crate::errors::Refused;
use crate::physics::car_store::Car;
use crate::race::owner_shared::RaceShared;
use std::cell::{Ref, RefMut};

/// The current car's block (by its handle), refused when null.
pub(super) fn current_block(shared: &RaceShared) -> Result<Ref<'_, Car>, Refused> {
    use rc2k_core::drive::work_car::{car_block, car_slot};
    let slot = car_slot(shared.work().current_car);
    car_block(u32::from(shared.work().has_car_block(slot as usize)))?;

    Ok(shared.race_car(slot as usize)?)
}

/// The current car's block, to write.
pub(super) fn current_block_mut(shared: &RaceShared) -> Result<RefMut<'_, Car>, Refused> {
    use rc2k_core::drive::work_car::{car_block, car_slot};
    let slot = car_slot(shared.work().current_car);
    car_block(u32::from(shared.work().has_car_block(slot as usize)))?;

    Ok(shared.race_car_mut(slot as usize)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::plain_paths::{flat_bytes, set_flat_bytes};

    #[test]
    fn a_copy_writes_every_field_of_the_source() {
        let mut seed = 0x1234_5678u32;
        let flat_len = flat_bytes(&Car::new()).len();
        let mut random = || {
            let mut b = vec![0u8; flat_len];
            for x in &mut b {
                seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
                *x = (seed >> 16) as u8;
            }
            b
        };
        for _ in 0..20 {
            let (mut from, mut copy) = (Car::new(), Car::new());
            set_flat_bytes(&mut from, &random());
            set_flat_bytes(&mut copy, &random());
            copy.clone_from(&from);
            assert_eq!(flat_bytes(&from), flat_bytes(&copy));
        }
    }
}
