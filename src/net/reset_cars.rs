//! Makes the eight cars' blocks ready for a network race (car number, view, a few settings),
//! clearing each with `net::clear_car_state`.

use crate::net::clear_car_state::clear_car_state;
use crate::physics::car_store::Car;
use crate::race::owner::Race;
use rc2k_core::net::cars;

/// A field of a car's block and the value it is set to.
type Reset = (fn(&mut Car) -> &mut u32, u32);

/// A car's block's dwords set before it is cleared, in order: the car's number + 1 in
/// `start_slot_value`, `camera_view` 4, `gear_ratio_setting` 2, then 0x80 in `suspension_stiffness`,
/// `ride_height`, `brake_balance` and `steering_sensitivity`.
fn reset_fields(car: u32) -> [Reset; 7] {
    [
        (|c| &mut c.start_slot_value, car + 1),
        (|c| &mut c.camera_view, 4),
        (|c| &mut c.gear_ratio_setting, 2),
        (|c| &mut c.suspension_stiffness, 0x80),
        (|c| &mut c.ride_height, 0x80),
        (|c| &mut c.brake_balance, 0x80),
        (|c| &mut c.steering_sensitivity, 0x80),
    ]
}
pub(super) fn reset_cars(race: &Race) -> Result<(), crate::errors::BadInput> {
    for car in 0..cars::CARS {
        let mut block = race.shared.race_car_mut(car as usize)?;
        for (field, v) in reset_fields(car) {
            *field(&mut block) = v;
        }
        clear_car_state(&mut block)?;
    }
    race.shared.vars.get().race_level.set(0);
    race.shared.vars.get().race_stage.set(0);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each field sits where the core's table had its position in the car's block.
    #[test]
    fn the_fields_sit_where_the_offsets_were() {
        let offsets: Vec<u32> = reset_fields(3)
            .iter()
            .map(|(field, _)| {
                crate::physics::car_offsets_1999::offset_changed_by(|c| *field(c) = 1) as u32
            })
            .collect();
        assert_eq!(offsets, [0xAA8, 0x684, 0x28, 0x34, 0x30, 0x20, 0x3C]);
        assert_eq!(reset_fields(3)[0].1, 4);
    }
}
