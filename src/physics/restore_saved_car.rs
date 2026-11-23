//! Puts a saved state back into the work copy: `clear_place_count` + 1; the three dwords
//! `saved_place` into `position`; the 0x40 bytes at `saved_matrix` into the crash matrix; and
//! `probe_count` dwords from `probe_saved_faces` into `probe_point_face`. What the pieces are (a
//! place, the car's matrix, a list) is a reading.
//!
//! Refuses (before writing) a count above 0x7C (unsigned: the list would run past the work copy's
//! end).

use crate::errors::Refused;
use crate::memory::plain_run::PlainRun;
use crate::physics::car_store::{Car, put_dwords};
use crate::physics::contact_store::ProbeLists;
use crate::physics::tables_store::PhysicsTables;
use rc2k_core::physics::saved_car::{restore_count, restored};

/// The car's last-face list and the fields after it in the record, in order, to its end.
fn faces_and_after(car: &mut Car) -> [&mut dyn PlainRun; 28] {
    [
        &mut car.probe_point_face,
        &mut car.camera_ground_faces,
        &mut car.group_mask,
        &mut car.engine_speed_scaled,
        &mut car.previous_step_clock,
        &mut car.wheel_ground_x,
        &mut car.wheel_ground_height,
        &mut car.wheel_ground_z,
        &mut car.skid_grip,
        &mut car.skid_slots,
        &mut car.car_event_times,
        &mut car.car_event_codes,
        &mut car.car_event_count,
        &mut car.car_event_limit,
        &mut car.car_event_position,
        &mut car.two_car_place,
        &mut car.view_covered,
        &mut car.reset_marker,
        &mut car.value_jump_steps,
        &mut car.light_hold_steps,
        &mut car.eased_target,
        &mut car.eased_value,
        &mut car.glide_target,
        &mut car.glide_steps_left,
        &mut car.kept_throttle_word,
        &mut car.kept_keys_word,
        &mut car.kept_steering_word,
        &mut car.kept_brake_word,
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
    car: &mut Car,
    tables: &mut PhysicsTables,
    probes: &ProbeLists,
) -> Result<(), Refused> {
    let n = restore_count(car.probe_count)?;
    tables.clear_place_count = restored(tables.clear_place_count);
    car.position = std::array::from_fn(|k| f32::from_bits(tables.saved_place[k]));
    car.crash_matrix = tables.saved_matrix;
    // Up to 0x7C dwords from the saved faces (0x40), on into the face-flip list after them, into
    // the car's faces (0x40), on through the fields after them to the record's end (0x7C in all).
    put_dwords(
        &mut faces_and_after(car),
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
            let mut new = car_with_1999_bytes(|at| fill[at]);
            let mut old = car_with_1999_bytes(|at| fill[at]);
            put_dwords(
                &mut faces_and_after(&mut new),
                0,
                (0..n).map(|k| saved_face_or_flip(&probes, k)),
            )
            .unwrap();
            for k in 0..n {
                write_on_1999(
                    &mut old,
                    "probe_point_face",
                    k,
                    saved_face_or_flip(&probes, k),
                );
            }
            assert!(bytes_1999(&new) == bytes_1999(&old), "count {n}");
        }
    }
}
