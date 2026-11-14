//! Puts a saved state back into the work copy: `clear_place_count` + 1; the three dwords
//! `saved_place` into `position`; the 0x40 bytes at `saved_matrix` into the crash matrix; and
//! `probe_count` dwords from `probe_saved_faces` into `probe_point_face`. What the pieces are (a
//! place, the car's matrix, a list) is a reading.
//!
//! Callers: `physics::find_clear_place`, `physics::crossed_edge`, `physics::wall_contacts` (twice),
//! `physics::face_contact`, `physics::ground_contact`, `physics::corner_hits_car`, under
//! `drive::race_step` (most followed by `physics::feel_ground`). Refuses (before writing) a count
//! above 0x7C (unsigned: the list would run past the work copy's end).

use crate::errors::Refused;
use crate::physics::car_store::Car;
use crate::physics::contact_store::ContactWork;
use crate::physics::tables_store::PhysicsTables;
use crate::session::{Run, read_runs, write_runs};
use rc2k_core::physics::saved_car::{restore_count, restored};

/// The car's last-face list and the fields after it in the record, in order, to its end.
fn faces_and_after(car: &Car) -> [&dyn Run; 28] {
    [
        &car.probe_point_face,
        &car.camera_ground_faces,
        &car.group_mask,
        &car.engine_speed_scaled,
        &car.previous_step_clock,
        &car.wheel_ground_x,
        &car.wheel_ground_height,
        &car.wheel_ground_z,
        &car.skid_grip,
        &car.skid_slots,
        &car.car_event_times,
        &car.car_event_codes,
        &car.car_event_count,
        &car.car_event_limit,
        &car.car_event_position,
        &car.two_car_place,
        &car.view_covered,
        &car.reset_marker,
        &car.value_jump_steps,
        &car.light_hold_steps,
        &car.eased_target,
        &car.eased_value,
        &car.glide_target,
        &car.glide_steps_left,
        &car.kept_throttle_word,
        &car.kept_keys_word,
        &car.kept_steering_word,
        &car.kept_brake_word,
    ]
}

/// Core: `rc2k_core::physics::saved_car::{restore_count, restored}`.
pub fn restore_saved_car(
    car: &Car,
    tables: &PhysicsTables,
    contact: &ContactWork,
) -> Result<(), Refused> {
    let n = restore_count(car.probe_count.get())?;
    tables
        .clear_place_count
        .set(restored(tables.clear_place_count.get()));
    for (to, from) in car.position.iter().zip(&tables.saved_place) {
        to.set(from.get());
    }
    for (to, from) in car.crash_matrix.iter().zip(&tables.saved_matrix) {
        to.set(from.get());
    }
    // Up to 0x7C dwords from the saved faces (0x40), on into the face-flip list after them, into
    // the car's faces (0x40), on through the fields after them to the record's end (0x7C in all).
    let from: [&dyn Run; 2] = [&contact.probe_saved_faces, &contact.probe_face_flip];
    let to = faces_and_after(car);
    for k in 0..n as usize {
        let v = read_runs(&from, k, "the saved faces")?;
        write_runs(&to, k, v, "the saved faces")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The copy of every count up to the limit puts the saved faces and then the flips into the
    /// work copy's list, and leaves the rest of the list as it was.
    #[test]
    fn faces_run_on_into_the_flips() {
        for n in 0..=0x7C {
            let new = Car::new();
            let c_new = ContactWork::new();
            for (k, cell) in c_new.probe_saved_faces.iter().enumerate() {
                cell.set(0x100 + k as u32);
            }
            for (k, cell) in c_new.probe_face_flip.iter().enumerate() {
                cell.set(0x200 + k as u32);
            }
            let from: [&dyn Run; 2] = [&c_new.probe_saved_faces, &c_new.probe_face_flip];
            let to = faces_and_after(&new);
            for k in 0..n {
                write_runs(&to, k, read_runs(&from, k, "t").unwrap(), "t").unwrap();
            }
            for k in 0..0x7C {
                let read =
                    crate::physics::car_offsets_1999::read_on_1999(&new, &new.probe_point_face, k);
                let wanted = if k < n {
                    read_runs(&from, k, "t").unwrap()
                } else {
                    0
                };
                assert_eq!(read, wanted, "count {n}, dword {k}");
            }
        }
    }
}
