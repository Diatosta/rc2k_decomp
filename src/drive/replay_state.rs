//! The replay file's starting state, field by field. A replay frame (`ReplayFrame`, 0x2810
//! bytes) begins with the four cars' first 0x684 bytes, the eight loose parts and the computer
//! driver's state; the byte format is the 1999 game's and is never changed. Each piece below
//! names the fields of the live record (`Car`, `RaceSlot`, `RaceWork`) at the file's offsets, so
//! the file does not depend on how those records lie in memory.

use crate::memory::cells::{ByteView, load_cells, store_cells};
use crate::physics::car_store::Car;
use crate::race::work_store::{RaceSlot, RaceWork};
use core::mem::size_of;
use rc2k_core::drive::replay_frames::{CAR_DWORDS, ReplayFrame, SLOT_DWORDS, START_DWORDS};
use rc2k_core::race::replay_v2::{CarState, HEAD_CARS, STATE_AT, TAIL_CARS};

const CAR_BYTES: usize = 4 * CAR_DWORDS;
/// A loose part's bytes in the file.
const SLOT_BYTES: usize = 0x78;
/// Where the loose parts begin in the starting state.
const SLOTS_AT: usize = 4 * CAR_BYTES;
/// Where the driver's state begins in the starting state.
const EXTRA_AT: usize = SLOTS_AT + 4 * SLOT_DWORDS;

/// The car fields at their file offsets that the replay viewer or the sound listener sets
/// itself: `drive::replay_check` leaves them out when it compares a played frame with the
/// recorded one.
pub(super) const FRAME_HEAD_AT: usize = 0x0;
pub(super) const LAST_PACE_CALL_AT: usize = 0x9c;
pub(super) const OPTION_BITS_AT: usize = 0x220;
pub(super) const GEAR_CHANGE_PENDING_AT: usize = 0x23c;

/// The bytes of a car the file carries that no code reads (file offset, length). The 1999
/// game wrote zeros there; so does every frame saved from here.
const CAR_UNUSED: [(usize, usize); 3] = [(0xf4, 64), (0x190, 4), (0x1b0, 8)];
/// The same in the driver's state (offset from the state's start).
const EXTRA_UNUSED: [(usize, usize); 4] = [(0x80, 4), (0x12c, 12), (0x2c4, 16), (0x304, 16)];
/// The byte the unused places of a frame hold.
const UNUSED_BYTE: u8 = 0;

/// Moves one field between a record and the file's bytes.
trait Move {
    /// The field at byte `at` of the piece being moved.
    fn field<T: ByteView>(&mut self, at: usize, field: &T);

    /// `len` bytes at `at` that no record holds: written as `UNUSED_BYTE`, ignored when read.
    fn unused(&mut self, at: usize, len: usize);
}

/// Reads the fields into the starting state's bytes, from `base`.
struct Gather<'a> {
    bytes: &'a mut [u8],
    base: usize,
}

impl Move for Gather<'_> {
    fn field<T: ByteView>(&mut self, at: usize, field: &T) {
        let from = self.base + at;
        load_cells(field, 0, &mut self.bytes[from..from + size_of::<T>()])
            .expect("a field inside the file's bytes");
    }

    fn unused(&mut self, at: usize, len: usize) {
        let from = self.base + at;
        self.bytes[from..from + len].fill(UNUSED_BYTE);
    }
}

/// Writes the fields from the starting state's bytes, from `base`.
struct Scatter<'a> {
    bytes: &'a [u8],
    base: usize,
}

impl Move for Scatter<'_> {
    fn field<T: ByteView>(&mut self, at: usize, field: &T) {
        let from = self.base + at;
        store_cells(field, 0, &self.bytes[from..from + size_of::<T>()])
            .expect("a field inside the file's bytes");
    }

    fn unused(&mut self, _at: usize, _len: usize) {}
}

/// A car's first 0x684 bytes (everything before its camera mode), in the car's order.
fn car_fields(car: &Car, m: &mut impl Move) {
    m.field(FRAME_HEAD_AT, &car.frame_head);
    m.field(0x8, &car.step_clock);
    m.field(0xc, &car.noise_word);
    m.field(0x10, &car.car_flags);
    m.field(0x14, &car.run_ticks);
    m.field(0x18, &car.start_in_gear);
    m.field(0x1c, &car.false_start_count);
    m.field(0x20, &car.brake_balance);
    m.field(0x24, &car.car_kind);
    m.field(0x28, &car.gear_ratio_setting);
    m.field(0x2c, &car.tyre_kind);
    m.field(0x30, &car.ride_height);
    m.field(0x34, &car.suspension_stiffness);
    m.field(0x38, &car.gearbox_kind);
    m.field(0x3c, &car.steering_sensitivity);
    m.field(0x40, &car.drive_kind);
    m.field(0x44, &car.reserved_44);
    m.field(0x48, &car.part_woken);
    m.field(0x60, &car.ground_bump_scale);
    m.field(0x64, &car.impact_octant_marks);
    m.field(0x84, &car.lights_on);
    m.field(0x88, &car.stage_time);
    m.field(0x8c, &car.time_left);
    m.field(0x90, &car.splits_passed);
    m.field(0x94, &car.reset_checkpoint);
    m.field(0x98, &car.checkpoint);
    m.field(LAST_PACE_CALL_AT, &car.last_pace_call);
    m.field(0xa0, &car.started_stand_timer);
    m.field(0xa4, &car.go_stand_timer);
    m.field(0xa8, &car.finish_flag);
    m.field(0xac, &car.splits_recorded);
    m.field(0xb0, &car.stop_state);
    m.field(0xb4, &car.crash_matrix);
    m.field(0x134, &car.position);
    m.field(0x140, &car.split_times);
    m.field(0x180, &car.drive_slip_speed);
    m.field(0x184, &car.spin);
    m.field(0x194, &car.spin_length);
    m.field(0x198, &car.velocity);
    m.field(0x1a4, &car.local_velocity);
    m.field(0x1b8, &car.drive_speed_a);
    m.field(0x1bc, &car.drive_speed_b);
    m.field(0x1c0, &car.mean_drive_speed);
    m.field(0x1c4, &car.speed);
    m.field(0x1c8, &car.side_speed_held);
    m.field(0x1cc, &car.gear_change_hold);
    m.field(0x1d0, &car.drive_cut_steps);
    m.field(0x1d4, &car.gear);
    m.field(0x1d8, &car.engine_speed);
    m.field(0x1dc, &car.engine_speed_limit);
    m.field(0x1e0, &car.spec_scale_copy);
    m.field(0x1e4, &car.engine_sound_revs);
    m.field(0x1e8, &car.stuck_count);
    m.field(0x1ec, &car.position_message_word);
    m.field(0x1f0, &car.rev_cut_count);
    m.field(0x1f4, &car.wall_hit_timer);
    m.field(0x1f8, &car.driven_wheel_turns);
    m.field(0x200, &car.eased_yaw_spin);
    m.field(0x204, &car.grip_force_sum);
    m.field(0x208, &car.steer_trim);
    m.field(0x20c, &car.reserved_20c);
    m.field(0x210, &car.grip_forces);
    m.field(OPTION_BITS_AT, &car.option_bits);
    m.field(0x224, &car.contact_point);
    m.field(0x230, &car.contact_depth);
    m.field(0x234, &car.contact_flag);
    m.field(0x238, &car.gear_changed);
    m.field(GEAR_CHANGE_PENDING_AT, &car.gear_change_pending);
    m.field(0x240, &car.light_colour);
    m.field(0x244, &car.light_level);
    m.field(0x248, &car.front_wheels_off);
    m.field(0x24c, &car.rear_wheels_off);
    m.field(0x250, &car.driven_wheels_off);
    m.field(0x254, &car.all_wheels_off);
    m.field(0x258, &car.hard_ground_count);
    m.field(0x25c, &car.driver_kind);
    m.field(0x260, &car.probe_count);
    m.field(0x264, &car.reset_key_count);
    m.field(0x268, &car.stopped_steps);
    m.field(0x26c, &car.tyre_change_state);
    m.field(0x270, &car.off_line_steps);
    m.field(0x274, &car.turned_steps);
    m.field(0x278, &car.off_line_cue_time);
    m.field(0x27c, &car.turned_cue_time);
    m.field(0x280, &car.surface_blend);
    m.field(0x284, &car.parts_state);
    m.field(0x288, &car.slip_speed);
    m.field(0x28c, &car.sound_slip_share);
    m.field(0x290, &car.sound_level);
    m.field(0x294, &car.sound_speed);
    m.field(0x298, &car.parts_present);
    m.field(0x29c, &car.damage_failed);
    m.field(0x2a0, &car.brake_fade);
    m.field(0x2a4, &car.damper_wear);
    m.field(0x2ac, &car.brake_heat);
    m.field(0x2b4, &car.engine_heat);
    m.field(0x2b8, &car.engine_damage);
    m.field(0x2bc, &car.exhaust_damage);
    m.field(0x2c0, &car.turbo_damage);
    m.field(0x2c4, &car.cooling_damage);
    m.field(0x2c8, &car.clutch_damage);
    m.field(0x2cc, &car.differential_damage);
    m.field(0x2d0, &car.gear_wear);
    m.field(0x2f4, &car.gearbox_damage);
    m.field(0x2f8, &car.wheel_hit_damage);
    m.field(0x308, &car.body_hit_damage);
    m.field(0x30c, &car.body_damage);
    m.field(0x310, &car.electrics_damage);
    m.field(0x314, &car.light_damage);
    m.field(0x32c, &car.lights_damage);
    m.field(0x330, &car.steering_damage);
    m.field(0x334, &car.suspension_damage);
    m.field(0x344, &car.overall_suspension_damage);
    m.field(0x348, &car.brake_wear_a);
    m.field(0x34c, &car.brake_wear_b);
    m.field(0x350, &car.brake_damage);
    m.field(0x354, &car.tyre_wears);
    m.field(0x364, &car.tyre_wear);
    m.field(0x368, &car.damage_levels);
    m.field(0x3f4, &car.engine_power_factor);
    m.field(0x3f8, &car.brake_efficiency);
    m.field(0x400, &car.drive_efficiency);
    m.field(0x408, &car.steering_efficiency);
    m.field(0x40c, &car.electrical_failures);
    m.field(0x428, &car.handbrake_failed);
    m.field(0x42c, &car.gear_change_blocked);
    m.field(0x430, &car.forced_neutral);
    m.field(0x434, &car.lost_gear_bits);
    m.field(0x438, &car.wheel_ride_height);
    m.field(0x448, &car.wheel_stiffness);
    m.field(0x458, &car.stiffness_mean);
    m.field(0x45c, &car.ride_height_mean);
    m.field(0x460, &car.hit_cooldown);
    m.field(0x464, &car.input_throttle);
    m.field(0x468, &car.input_keys);
    m.field(0x46c, &car.input_steering);
    m.field(0x470, &car.input_brake);
    m.field(0x474, &car.keys_pressed);
    m.field(0x478, &car.notice_count);
    m.field(0x47c, &car.notice_ids);
    m.field(0x4fc, &car.notice_values);
    m.field(0x57c, &car.notice_ranks);
    m.field(0x5fc, &car.top_speed);
    m.field(0x600, &car.wheel_squeeze);
    m.field(0x610, &car.wheel_squeeze_last);
    m.field(0x620, &car.checkpoint_offset);
    m.field(0x628, &car.pace_gaps);
    m.field(0x630, &car.pace_gaps_eased);
    m.field(0x638, &car.throttle);
    m.field(0x63c, &car.brake);
    m.field(0x640, &car.handbrake);
    m.field(0x644, &car.steering);
    m.field(0x648, &car.steering_raw);
    m.field(0x64c, &car.steer_angle);
    m.field(0x650, &car.steer_angle_visual);
    m.field(0x654, &car.mode2_place);
    m.field(0x658, &car.mode2_behind);
    m.field(0x65c, &car.pace_cars);
    m.field(0x664, &car.pace_starts);
    m.field(0x66c, &car.pace_placed);
    m.field(0x670, &car.pace_apart);
    m.field(0x674, &car.pace_push);
    m.field(0x678, &car.start_share);
    m.field(0x67c, &car.start_blend_share);
    m.field(0x680, &car.start_blend);
    for (at, len) in CAR_UNUSED {
        m.unused(at, len);
    }
}

/// A loose part (0x78 bytes).
fn slot_fields(slot: &RaceSlot, m: &mut impl Move) {
    m.field(0x0, &slot.part);
    m.field(0x4, &slot.place);
    m.field(0x10, &slot.matrix);
    m.field(0x50, &slot.motion);
    m.field(0x5c, &slot.draw_height_offset);
    m.field(0x60, &slot.car_word);
    m.field(0x64, &slot.age);
    m.field(0x68, &slot.spin);
    m.field(0x74, &slot.spin_length);
}

/// The computer driver's state: the 0x400 bytes of the race work from `replay_extra`.
fn extra_fields(work: &RaceWork, m: &mut impl Move) {
    m.field(0x0, &work.replay_extra);
    m.field(0x4, &work.driver_steer);
    m.field(0x8, &work.driver_throttle);
    m.field(0xc, &work.driver_brake);
    m.field(0x10, &work.driver_flags);
    m.field(0x20, &work.driver_start_blend);
    m.field(0x24, &work.driver_steps);
    m.field(0x4c, &work.driver_reverse_left);
    m.field(0x74, &work.driver_moved);
    m.field(0x78, &work.driver_turn);
    m.field(0x7c, &work.driver_bend_index);
    m.field(0x84, &work.driver_wanted_speed);
    m.field(0x88, &work.driver_speed);
    m.field(0x8c, &work.driver_bend_to);
    m.field(0xb4, &work.driver_bend_noted);
    m.field(0xdc, &work.driver_reverse_tries);
    m.field(0x104, &work.driver_bend_turn);
    m.field(0x138, &work.driver_flag_bc);
    m.field(0x13c, &work.driver_step_x);
    m.field(0x140, &work.driver_step_z);
    m.field(0x144, &work.driver_speed_scale);
    m.field(0x148, &work.driver_slot);
    m.field(0x14c, &work.driver_lookahead);
    m.field(0x150, &work.driver_bend_a);
    m.field(0x154, &work.driver_bend_b);
    m.field(0x158, &work.driver_point_now);
    m.field(0x15c, &work.driver_point_then);
    m.field(0x160, &work.driver_aim);
    m.field(0x164, &work.driver_points_now);
    m.field(0x204, &work.driver_points_then);
    m.field(0x2a4, &work.driver_from);
    m.field(0x2b4, &work.driver_to);
    m.field(0x2d4, &work.driver_way);
    m.field(0x2e4, &work.driver_way_unit);
    m.field(0x2f4, &work.driver_ahead);
    m.field(0x314, &work.driver_flat);
    m.field(0x324, &work.driver_flat_unit);
    m.field(0x334, &work.driver_side);
    m.field(0x344, &work.driver_work);
    for (at, len) in EXTRA_UNUSED {
        m.unused(at, len);
    }
}

fn pieces(cars: &[Car], work: &RaceWork, m: &mut dyn FnMut(usize, Piece)) {
    for (k, car) in cars.iter().take(4).enumerate() {
        m(k * CAR_BYTES, Piece::Car(car));
    }
    for (i, slot) in work.race_slots.iter().enumerate() {
        m(SLOTS_AT + i * SLOT_BYTES, Piece::Slot(slot));
    }
    m(EXTRA_AT, Piece::Extra(work));
}

enum Piece<'a> {
    Car(&'a Car),
    Slot(&'a RaceSlot),
    Extra(&'a RaceWork),
}

fn move_piece(piece: &Piece, m: &mut impl Move) {
    match piece {
        Piece::Car(car) => car_fields(car, m),
        Piece::Slot(slot) => slot_fields(slot, m),
        Piece::Extra(work) => extra_fields(work, m),
    }
}

/// The starting state of the first four cars, the loose parts and the driver's state (the
/// tape zero), as a replay frame.
pub fn gather(cars: &[Car], work: &RaceWork) -> ReplayFrame {
    let mut bytes = vec![0u8; 4 * START_DWORDS];
    pieces(cars, work, &mut |base, piece| {
        move_piece(
            &piece,
            &mut Gather {
                bytes: &mut bytes,
                base,
            },
        );
    });
    ReplayFrame::from_start_bytes(&bytes).expect("the starting state's size")
}

/// A replay frame's starting state put into the first four cars, the loose parts and the
/// driver's state.
pub(super) fn scatter(frame: &ReplayFrame, cars: &[Car], work: &RaceWork) {
    let bytes = frame.start_bytes();
    pieces(cars, work, &mut |base, piece| {
        move_piece(
            &piece,
            &mut Scatter {
                bytes: &bytes,
                base,
            },
        );
    });
}

/// A car's first `CAR_DWORDS` dwords, field by field at the file's offsets.
pub(super) fn gather_car(car: &Car) -> [u32; CAR_DWORDS] {
    let mut bytes = vec![0u8; CAR_BYTES];
    car_fields(
        car,
        &mut Gather {
            bytes: &mut bytes,
            base: 0,
        },
    );
    std::array::from_fn(|n| u32::from_le_bytes(bytes[4 * n..4 * n + 4].try_into().expect("4")))
}

/// A car's fields put from its dwords in the file's layout.
fn scatter_car(words: &[u32; CAR_DWORDS], car: &Car) {
    let bytes: Vec<u8> = words.iter().flat_map(|w| w.to_le_bytes()).collect();
    car_fields(
        car,
        &mut Scatter {
            bytes: &bytes,
            base: 0,
        },
    );
}

/// Writes only the fields that start at a recorded state word (`STATE_AT`).
struct ScatterState<'a> {
    bytes: &'a [u8],
}

impl Move for ScatterState<'_> {
    fn field<T: ByteView>(&mut self, at: usize, field: &T) {
        if STATE_AT.contains(&(at / 4)) {
            store_cells(field, 0, &self.bytes[at..at + size_of::<T>()])
                .expect("a field inside the file's bytes");
        }
    }

    fn unused(&mut self, _at: usize, _len: usize) {}
}

/// A car's recorded state put into it, the car's other fields left as they are.
pub(super) fn scatter_state(state: &CarState, car: &Car) {
    let mut words = [0u32; CAR_DWORDS];
    state.put(&mut words);
    let bytes: Vec<u8> = words.iter().flat_map(|w| w.to_le_bytes()).collect();
    car_fields(car, &mut ScatterState { bytes: &bytes });
}

/// The state of cars 4 .. 7 (the tail's keyframes); a car past the slice is left zero.
pub(super) fn gather_tail(cars: &[Car]) -> [[u32; CAR_DWORDS]; TAIL_CARS] {
    std::array::from_fn(|k| cars.get(HEAD_CARS + k).map_or([0; CAR_DWORDS], gather_car))
}

/// The tail's keyframes put into cars 4 .. 7, as far as the race has cars (`count` cars).
pub(super) fn scatter_tail(keys: &[[u32; CAR_DWORDS]; TAIL_CARS], cars: &[Car], count: usize) {
    for (k, words) in keys.iter().enumerate() {
        if HEAD_CARS + k < count
            && let Some(car) = cars.get(HEAD_CARS + k)
        {
            scatter_car(words, car);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::cells::cells_of_bytes;
    use rc2k_core::drive::replay_frames::FRAME_BYTES;

    const SLOTS_AT: usize = core::mem::offset_of!(RaceWork, race_slots);
    const EXTRA_AT: usize = core::mem::offset_of!(RaceWork, replay_extra);

    /// A car, the race work and their bytes in pieces: `pattern(n)` for byte n.
    fn pattern(n: usize) -> u8 {
        (n.wrapping_mul(2654435761) >> 11) as u8 | 1
    }

    /// A car whose byte at the 1999 (file) offset `n` is `fill(n)`.
    fn car_of(fill: impl Fn(usize) -> u8) -> Car {
        crate::physics::car_offsets_1999::car_with_1999_bytes(fill)
    }

    fn work_of(fill: impl Fn(usize) -> u8) -> Box<RaceWork> {
        Box::new(cells_of_bytes::<RaceWork, { size_of::<RaceWork>() }>(
            core::array::from_fn(fill),
        ))
    }

    fn cars_of(fill: impl Fn(usize) -> u8 + Copy) -> [Car; 4] {
        core::array::from_fn(|_| car_of(fill))
    }

    /// The file's starting state with the places no record holds set to `UNUSED_BYTE`.
    fn with_unused_places(mut bytes: Vec<u8>) -> Vec<u8> {
        for k in 0..4 {
            for (at, len) in CAR_UNUSED {
                let from = k * CAR_BYTES + at;
                bytes[from..from + len].fill(UNUSED_BYTE);
            }
        }
        for (at, len) in EXTRA_UNUSED {
            let from = super::EXTRA_AT + at;
            bytes[from..from + len].fill(UNUSED_BYTE);
        }
        bytes
    }

    fn bytes_of<T: ByteView>(record: &T, from: usize, len: usize) -> Vec<u8> {
        let mut bytes = vec![0; len];
        load_cells(record, from, &mut bytes).unwrap();
        bytes
    }

    /// Where the file's byte `at` of a piece lies in the record, when the record keeps it:
    /// `unused` lists the file's places no record holds (the records closed up over them).
    fn in_record(at: usize, unused: &[(usize, usize)]) -> Option<usize> {
        let mut gone = 0;
        for &(from, len) in unused {
            if (from..from + len).contains(&at) {
                return None;
            }
            if at >= from + len {
                gone += len;
            }
        }
        Some(at - gone)
    }

    /// `len` bytes of the file's piece `from` the record at `base`, as the game's raw copy had
    /// them: the places no record holds are `UNUSED_BYTE`.
    fn raw_bytes<T: ByteView>(
        record: &T,
        base: usize,
        len: usize,
        unused: &[(usize, usize)],
    ) -> Vec<u8> {
        (0..len)
            .map(|at| match in_record(at, unused) {
                Some(n) => bytes_of(record, base + n, 1)[0],
                None => UNUSED_BYTE,
            })
            .collect()
    }

    /// The frame a gather makes puts every field at the offset the game's raw copy had:
    /// the cars' first 0x684 bytes, the loose parts (`race_slots` on), the driver's state (`replay_extra` on).
    #[test]
    fn gather_makes_the_files_bytes() {
        let cars = cars_of(pattern);
        let work = work_of(pattern);
        let frame = gather(&cars, &work);
        let mut expected = Vec::new();
        for car in &cars {
            let mut file = crate::physics::car_offsets_1999::bytes_1999(car);
            file.truncate(CAR_BYTES);
            for (at, len) in CAR_UNUSED {
                file[at..at + len].fill(UNUSED_BYTE);
            }
            expected.extend(file);
        }
        expected.extend(bytes_of(&*work, SLOTS_AT, 8 * SLOT_BYTES));
        expected.extend(raw_bytes(&*work, EXTRA_AT, 0x400, &EXTRA_UNUSED));
        assert_eq!(frame.start_bytes(), expected);
        assert_eq!(frame.tape, [0; 400]);
    }

    #[test]
    fn scatter_writes_only_the_starting_state() {
        let frame = gather(&cars_of(pattern), &work_of(pattern));
        let cars = cars_of(|_| 0);
        let work = work_of(|_| 0);
        scatter(&frame, &cars, &work);
        assert_eq!(gather(&cars, &work), frame);
        for car in &cars {
            let file = crate::physics::car_offsets_1999::bytes_1999(car);
            assert_eq!(file[CAR_BYTES..], vec![0; file.len() - CAR_BYTES]);
        }
        assert_eq!(bytes_of(&*work, 0, SLOTS_AT), vec![0; SLOTS_AT]);
        assert_eq!(
            bytes_of(&*work, SLOTS_AT + 0x3C0, EXTRA_AT - SLOTS_AT - 0x3C0),
            vec![0; EXTRA_AT - SLOTS_AT - 0x3C0]
        );
    }

    /// A synthetic file frame goes through the cars and back byte for byte.
    #[test]
    fn a_synthetic_frame_round_trips() {
        let bytes = with_unused_places((0..FRAME_BYTES).map(|n| pattern(n + 7)).collect());
        let frame = ReplayFrame::from_bytes(&bytes).unwrap();
        let (cars, work) = (cars_of(|_| 0), work_of(|_| 0));
        scatter(&frame, &cars, &work);
        let back = gather(&cars, &work);
        assert_eq!(back.start_bytes(), bytes[..4 * START_DWORDS]);
    }

    /// The replays saved in the install, when there are some: every frame of each file goes
    /// through the cars and back unchanged. (Skipped without the folder; no file is copied.)
    #[test]
    fn saved_replays_round_trip() {
        let Some(home) = std::env::var_os("HOME") else {
            return;
        };
        let dir =
            std::path::Path::new(&home).join(".local/share/rally-championship/savedata/replay");
        let Ok(files) = std::fs::read_dir(dir) else {
            return;
        };
        let (cars, work) = (cars_of(|_| 0), work_of(|_| 0));
        let mut frames = 0;
        for file in files.flatten() {
            let Ok(data) = std::fs::read(file.path()) else {
                continue;
            };
            for block in file_blocks(&data) {
                let frame = ReplayFrame::from_bytes(&block).expect("a whole frame");
                assert_eq!(frame.to_bytes(), block);
                scatter(&frame, &cars, &work);
                assert_eq!(
                    gather(&cars, &work).start_bytes(),
                    block[..4 * START_DWORDS],
                    "{:?}",
                    file.path()
                );
                frames += 1;
            }
        }
        eprintln!("{frames} saved frames round-tripped");
    }

    /// The blocks of a replay file: the index ("RCIB", 0x3850 bytes) ends the file; it lists
    /// each block's offset and size, negative when run-length packed.
    fn file_blocks(data: &[u8]) -> Vec<Vec<u8>> {
        const INDEX: usize = 0x3850;
        let dword = |at: usize| u32::from_le_bytes(data[at..at + 4].try_into().unwrap());
        let Some(index) = data.len().checked_sub(INDEX) else {
            return Vec::new();
        };
        if data[index..index + 4] != *b"RCIB" {
            return Vec::new();
        }
        let blocks = dword(index + 8) as usize;
        (0..blocks)
            .filter_map(|k| {
                let place = dword(index + 16 + 8 * k) as usize;
                let size = dword(index + 20 + 8 * k) as i32;
                if size < 0 {
                    let packed = data.get(place..place + size.unsigned_abs() as usize)?;
                    let mut out = vec![0u8; FRAME_BYTES];
                    let n = rc2k_core::drive::rle::unpack_rle(&packed, |at, b| {
                        if let Some(slot) = out.get_mut(at as usize) {
                            *slot = b;
                        }
                    })
                    .ok()?;
                    (n as usize == FRAME_BYTES).then_some(out)
                } else {
                    data.get(place..place + size as usize).map(<[u8]>::to_vec)
                }
            })
            .collect()
    }
}
