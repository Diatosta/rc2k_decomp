//! The race's split times and timers under `physics::car_physics_step`: `physics::race_timers` by
//! the race mode (`run_kind`) runs the start's counters and, at each new checkpoint split,
//! `physics::take_split` or for mode 2 `physics::take_split_counted`; those ask
//! `physics::split_by_mode` for the split's figure: mode 2 the car's place, mode 1
//! `physics::rank_split_time` (against the 90 rival times of `physics::fill_rival_table`), mode 3
//! `physics::compare_split_times` (against ten times kept per split). This file holds the shared
//! names and the test.

use crate::physics::car_store::Car;

/// Split `split`'s time into the car. The splits run to 0x3F (`split_due` refuses a later one),
/// past the sixteen figures of `split_times`: they go into the car's next fields, named here in the
/// car's order (a split past the 64 is refused).
pub fn put_split_time(car: &Car, split: usize, v: u32) -> Result<(), crate::errors::BadInput> {
    if let Some(time) = car.split_times.get(split) {
        time.set(v);
        return Ok(());
    }
    crate::memory::session::write_runs(
        &[
            &car.split_times,
            &car.drive_slip_speed,
            &car.spin,
            &car.spin_length,
            &car.velocity,
            &car.local_velocity,
            &car.drive_speed_a,
            &car.drive_speed_b,
            &car.mean_drive_speed,
            &car.speed,
            &car.side_speed_held,
            &car.gear_change_hold,
            &car.drive_cut_steps,
            &car.gear,
            &car.engine_speed,
            &car.engine_speed_limit,
            &car.spec_scale_copy,
            &car.engine_sound_revs,
            &car.stuck_count,
            &car.position_message_word,
            &car.rev_cut_count,
            &car.wall_hit_timer,
            &car.driven_wheel_turns,
            &car.eased_yaw_spin,
            &car.grip_force_sum,
            &car.steer_trim,
            &car.unused_20c,
            &car.grip_forces,
            &car.option_bits,
            &car.contact_point,
            &car.contact_depth,
            &car.contact_flag,
            &car.gear_changed,
            &car.gear_change_pending,
            &car.light_colour,
            &car.light_level,
            &car.front_wheels_off,
        ],
        split,
        v,
        "a split's time",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each split to the last one takes writes the dword the old byte-offset write did.
    #[test]
    fn split_times_run_on_as_the_old_write() {
        let (new, old) = (Car::new(), Car::new());
        for n in 0..0x40 {
            let v = 0x3000_0000 + n as u32;
            put_split_time(&new, n, v).unwrap();
            crate::physics::car_offsets_1999::write_on_1999(&old, &old.split_times, n, v);
        }
        for n in 0..0x40 + 8 {
            let read =
                |c: &Car| crate::physics::car_offsets_1999::read_on_1999(c, &c.split_times, n);
            assert_eq!(read(&new), read(&old), "dword {n}");
        }
        assert!(put_split_time(&new, 0x40, 1).is_err());
    }
}
