//! Puts a saved state back into the work copy: `clear_place_count` + 1; the three dwords
//! `saved_place` into `position`; the 0x40 bytes at `saved_matrix` into the crash matrix; and
//! `probe_count` dwords from `probe_saved_faces` into `probe_point_face`. What the pieces are (a
//! place, the car's matrix, a list) is a reading.
//!
//! Refuses (before writing) a count above 0x7C (unsigned: the list would run past the work copy's
//! end).

use crate::errors::Refused;
use crate::memory::session::Run;
use crate::physics::car_store::{Car, put_dwords};
use crate::physics::contact_store::ProbeLists;
use crate::physics::tables_store::PhysicsTables;
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

/// Dword `k` of the saved faces run on into the face flips (0x7C at most, the flips' list ends
/// at 0x80).
fn saved_face_or_flip(probes: &ProbeLists, k: usize) -> u32 {
    if k < 64 {
        probes.probe_saved_faces[k]
    } else {
        probes.probe_face_flip[k - 64]
    }
}

/// Core: `rc2k_core::physics::saved_car::{restore_count, restored}`.
pub(super) fn restore_saved_car(
    car: &Car,
    tables: &mut PhysicsTables,
    probes: &ProbeLists,
) -> Result<(), Refused> {
    let n = restore_count(car.probe_count.get())?;
    tables.clear_place_count = restored(tables.clear_place_count);
    car.position
        .set_bits(std::array::from_fn(|k| tables.saved_place[k]));
    car.crash_matrix
        .set_bits(tables.saved_matrix.map(f32::to_bits));
    // Up to 0x7C dwords from the saved faces (0x40), on into the face-flip list after them, into
    // the car's faces (0x40), on through the fields after them to the record's end (0x7C in all).
    put_dwords(
        &faces_and_after(car),
        0,
        (0..n as usize).map(|k| saved_face_or_flip(probes, k)),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::car_offsets_1999::{bytes_1999, car_with_1999_bytes, write_on_1999};

    /// The copy of every count up to the limit does what the 1999 record's run did over a
    /// randomly filled record: the saved faces and then the flips go into the work copy's list and
    /// on through the fields after it, and every other field is as it was.
    #[test]
    fn faces_run_on_as_the_old_run() {
        let mut seed = 0x1234_5678_u32;
        let mut random = move || {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (seed >> 24) as u8
        };
        let fill: Vec<u8> = (0..crate::physics::car_offsets_1999::SIZE_1999)
            .map(|_| random())
            .collect();
        let mut probes = ProbeLists::new();
        for (k, cell) in probes.probe_saved_faces.iter_mut().enumerate() {
            *cell = 0x100 + k as u32;
        }
        for (k, cell) in probes.probe_face_flip.iter_mut().enumerate() {
            *cell = 0x200 + k as u32;
        }
        for n in 0..=0x7C {
            let new = car_with_1999_bytes(|at| fill[at]);
            let old = car_with_1999_bytes(|at| fill[at]);
            put_dwords(
                &faces_and_after(&new),
                0,
                (0..n).map(|k| saved_face_or_flip(&probes, k)),
            )
            .unwrap();
            for k in 0..n {
                write_on_1999(
                    &old,
                    &old.probe_point_face,
                    k,
                    saved_face_or_flip(&probes, k),
                );
            }
            assert!(bytes_1999(&new) == bytes_1999(&old), "count {n}");
        }
    }
}
