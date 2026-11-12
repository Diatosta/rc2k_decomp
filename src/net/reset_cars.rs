//! `net::reset_cars` (0x463E52): the eight cars' blocks (`g_carBlocks`) made ready for a
//! network race: +0xAA8 the car's number + 1, +0x684 4, +0x28 2, +0x34, +0x30, +0x20,
//! +0x3C 0x80, then `net::clear_car_state`; two more race variables 0.
//!
//! Caller: `net::set_up_race`.

use crate::net::clear_car_state::clear_car_state;
use crate::physics::car_store::Car;
use crate::race::owner::Race;
use crate::session::Dword;
use rc2k_core::net::cars;

/// A field of a car's block and the value it is set to.
type Reset = (fn(&Car) -> &Dword, u32);

/// A car's block's dwords set before it is cleared, in order: the car's number + 1 in
/// +0xAA8, +0x684 4, +0x28 2, +0x34, +0x30, +0x20, +0x3C 0x80.
fn reset_fields(car: u32) -> [Reset; 7] {
    [
        (|c| &c.start_slot_value, car + 1),
        (|c| &c.camera_view, 4),
        (|c| &c.gear_ratio_setting, 2),
        (|c| &c.suspension_stiffness, 0x80),
        (|c| &c.ride_height, 0x80),
        (|c| &c.brake_balance, 0x80),
        (|c| &c.steering_sensitivity, 0x80),
    ]
}
pub fn reset_cars(race: &Race) -> Result<(), crate::errors::BadInput> {
    for car in 0..cars::CARS {
        let block = race.shared.race_car(car as usize)?;
        for (field, v) in reset_fields(car) {
            field(block).set(v);
        }
        clear_car_state(block)?;
    }
    race.shared.vars.race_level.set(0);
    race.shared.vars.race_stage.set(0);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::race::owner::race;

    /// Each field sits where the core's table had its offset in the car's block.
    #[test]
    fn the_fields_sit_where_the_offsets_were() {
        let car = &race().shared.car;
        let base = core::ptr::from_ref(car).addr();
        let offsets: Vec<u32> = reset_fields(3)
            .iter()
            .map(|(field, _)| (core::ptr::from_ref(field(car)).addr() - base) as u32)
            .collect();
        assert_eq!(offsets, [0xAA8, 0x684, 0x28, 0x34, 0x30, 0x20, 0x3C]);
        assert_eq!(reset_fields(3)[0].1, 4);
    }
}
