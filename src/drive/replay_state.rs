//! The replay file's starting state, field by field. A replay frame (`ReplayFrame`, 0x2810
//! bytes) begins with the four cars' first 0x684 bytes, the eight loose parts and the computer
//! driver's state; the byte format is the 1999 game's and is never changed. Each piece below
//! names the fields of the live record (`Car`, `RaceSlot`, `RaceWork`) at the file's offsets, so
//! the file does not depend on how those records lie in memory.

use crate::physics::car_store::Car;
use crate::race::work_store::{RaceSlot, RaceWork};
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

/// A plain value (or array of them) read from and written to the file's little-endian bytes.
trait LeView {
    /// The bytes the value takes in the file.
    const BYTES: usize;
    /// The value's bytes into `out` (`BYTES` long).
    fn load(&self, out: &mut [u8]);
    /// The value from `bytes` (`BYTES` long).
    fn store(&mut self, bytes: &[u8]);
}

macro_rules! le_view {
    ($($t:ty),*) => {$(
        impl LeView for $t {
            const BYTES: usize = 4;
            fn load(&self, out: &mut [u8]) {
                out.copy_from_slice(&self.to_le_bytes());
            }
            fn store(&mut self, bytes: &[u8]) {
                *self = <$t>::from_le_bytes(bytes.try_into().expect("a word"));
            }
        }
    )*};
}
le_view!(u32, i32, f32);

impl<T: LeView, const N: usize> LeView for [T; N] {
    const BYTES: usize = N * T::BYTES;
    fn load(&self, out: &mut [u8]) {
        for (value, bytes) in self.iter().zip(out.chunks_exact_mut(T::BYTES)) {
            value.load(bytes);
        }
    }
    fn store(&mut self, bytes: &[u8]) {
        for (value, bytes) in self.iter_mut().zip(bytes.chunks_exact(T::BYTES)) {
            value.store(bytes);
        }
    }
}

/// Moves one field between a record and the file's bytes.
trait Move {
    /// The plain field at byte `at` of the piece being moved.
    fn plain<T: LeView>(&mut self, at: usize, field: &mut T);

    /// `len` bytes at `at` that no record holds: written as `UNUSED_BYTE`, ignored when read.
    fn unused(&mut self, at: usize, len: usize);
}

/// Reads the fields into the starting state's bytes, from `base`.
struct Gather<'a> {
    bytes: &'a mut [u8],
    base: usize,
}

impl Move for Gather<'_> {
    fn plain<T: LeView>(&mut self, at: usize, field: &mut T) {
        let from = self.base + at;
        field.load(&mut self.bytes[from..from + T::BYTES]);
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
    fn plain<T: LeView>(&mut self, at: usize, field: &mut T) {
        let from = self.base + at;
        field.store(&self.bytes[from..from + T::BYTES]);
    }

    fn unused(&mut self, _at: usize, _len: usize) {}
}

/// A car's first 0x684 bytes (everything before its camera mode), in the car's order.
fn car_fields(car: &mut Car, m: &mut impl Move) {
    m.plain(FRAME_HEAD_AT, &mut car.frame_head);
    m.plain(0x8, &mut car.step_clock);
    m.plain(0xc, &mut car.noise_word);
    m.plain(0x10, &mut car.car_flags);
    m.plain(0x14, &mut car.run_ticks);
    m.plain(0x18, &mut car.start_in_gear);
    m.plain(0x1c, &mut car.false_start_count);
    m.plain(0x20, &mut car.brake_balance);
    m.plain(0x24, &mut car.car_kind);
    m.plain(0x28, &mut car.gear_ratio_setting);
    m.plain(0x2c, &mut car.tyre_kind);
    m.plain(0x30, &mut car.ride_height);
    m.plain(0x34, &mut car.suspension_stiffness);
    m.plain(0x38, &mut car.gearbox_kind);
    m.plain(0x3c, &mut car.steering_sensitivity);
    m.plain(0x40, &mut car.drive_kind);
    m.plain(0x44, &mut car.reserved_44);
    m.plain(0x48, &mut car.part_woken);
    m.plain(0x60, &mut car.ground_bump_scale);
    m.plain(0x64, &mut car.impact_octant_marks);
    m.plain(0x84, &mut car.lights_on);
    m.plain(0x88, &mut car.stage_time);
    m.plain(0x8c, &mut car.time_left);
    m.plain(0x90, &mut car.splits_passed);
    m.plain(0x94, &mut car.reset_checkpoint);
    m.plain(0x98, &mut car.checkpoint);
    m.plain(LAST_PACE_CALL_AT, &mut car.last_pace_call);
    m.plain(0xa0, &mut car.started_stand_timer);
    m.plain(0xa4, &mut car.go_stand_timer);
    m.plain(0xa8, &mut car.finish_flag);
    m.plain(0xac, &mut car.splits_recorded);
    m.plain(0xb0, &mut car.stop_state);
    m.plain(0xb4, &mut car.crash_matrix);
    m.plain(0x134, &mut car.position);
    m.plain(0x140, &mut car.split_times);
    m.plain(0x180, &mut car.drive_slip_speed);
    m.plain(0x184, &mut car.spin);
    m.plain(0x194, &mut car.spin_length);
    m.plain(0x198, &mut car.velocity);
    m.plain(0x1a4, &mut car.local_velocity);
    m.plain(0x1b8, &mut car.drive_speed_a);
    m.plain(0x1bc, &mut car.drive_speed_b);
    m.plain(0x1c0, &mut car.mean_drive_speed);
    m.plain(0x1c4, &mut car.speed);
    m.plain(0x1c8, &mut car.side_speed_held);
    m.plain(0x1cc, &mut car.gear_change_hold);
    m.plain(0x1d0, &mut car.drive_cut_steps);
    m.plain(0x1d4, &mut car.gear);
    m.plain(0x1d8, &mut car.engine_speed);
    m.plain(0x1dc, &mut car.engine_speed_limit);
    m.plain(0x1e0, &mut car.spec_scale_copy);
    m.plain(0x1e4, &mut car.engine_sound_revs);
    m.plain(0x1e8, &mut car.stuck_count);
    m.plain(0x1ec, &mut car.position_message_word);
    m.plain(0x1f0, &mut car.rev_cut_count);
    m.plain(0x1f4, &mut car.wall_hit_timer);
    m.plain(0x1f8, &mut car.driven_wheel_turns);
    m.plain(0x200, &mut car.eased_yaw_spin);
    m.plain(0x204, &mut car.grip_force_sum);
    m.plain(0x208, &mut car.steer_trim);
    m.plain(0x20c, &mut car.reserved_20c);
    m.plain(0x210, &mut car.grip_forces);
    m.plain(OPTION_BITS_AT, &mut car.option_bits);
    m.plain(0x224, &mut car.contact_point);
    m.plain(0x230, &mut car.contact_depth);
    m.plain(0x234, &mut car.contact_flag);
    m.plain(0x238, &mut car.gear_changed);
    m.plain(GEAR_CHANGE_PENDING_AT, &mut car.gear_change_pending);
    m.plain(0x240, &mut car.light_colour);
    m.plain(0x244, &mut car.light_level);
    m.plain(0x248, &mut car.front_wheels_off);
    m.plain(0x24c, &mut car.rear_wheels_off);
    m.plain(0x250, &mut car.driven_wheels_off);
    m.plain(0x254, &mut car.all_wheels_off);
    m.plain(0x258, &mut car.hard_ground_count);
    m.plain(0x25c, &mut car.driver_kind);
    m.plain(0x260, &mut car.probe_count);
    m.plain(0x264, &mut car.reset_key_count);
    m.plain(0x268, &mut car.stopped_steps);
    m.plain(0x26c, &mut car.tyre_change_state);
    m.plain(0x270, &mut car.off_line_steps);
    m.plain(0x274, &mut car.turned_steps);
    m.plain(0x278, &mut car.off_line_cue_time);
    m.plain(0x27c, &mut car.turned_cue_time);
    m.plain(0x280, &mut car.surface_blend);
    m.plain(0x284, &mut car.parts_state);
    m.plain(0x288, &mut car.slip_speed);
    m.plain(0x28c, &mut car.sound_slip_share);
    m.plain(0x290, &mut car.sound_level);
    m.plain(0x294, &mut car.sound_speed);
    m.plain(0x298, &mut car.parts_present);
    m.plain(0x29c, &mut car.damage_failed);
    m.plain(0x2a0, &mut car.brake_fade);
    m.plain(0x2a4, &mut car.damper_wear);
    m.plain(0x2ac, &mut car.brake_heat);
    m.plain(0x2b4, &mut car.engine_heat);
    m.plain(0x2b8, &mut car.engine_damage);
    m.plain(0x2bc, &mut car.exhaust_damage);
    m.plain(0x2c0, &mut car.turbo_damage);
    m.plain(0x2c4, &mut car.cooling_damage);
    m.plain(0x2c8, &mut car.clutch_damage);
    m.plain(0x2cc, &mut car.differential_damage);
    m.plain(0x2d0, &mut car.gear_wear);
    m.plain(0x2f4, &mut car.gearbox_damage);
    m.plain(0x2f8, &mut car.wheel_hit_damage);
    m.plain(0x308, &mut car.body_hit_damage);
    m.plain(0x30c, &mut car.body_damage);
    m.plain(0x310, &mut car.electrics_damage);
    m.plain(0x314, &mut car.light_damage);
    m.plain(0x32c, &mut car.lights_damage);
    m.plain(0x330, &mut car.steering_damage);
    m.plain(0x334, &mut car.suspension_damage);
    m.plain(0x344, &mut car.overall_suspension_damage);
    m.plain(0x348, &mut car.brake_wear_a);
    m.plain(0x34c, &mut car.brake_wear_b);
    m.plain(0x350, &mut car.brake_damage);
    m.plain(0x354, &mut car.tyre_wears);
    m.plain(0x364, &mut car.tyre_wear);
    m.plain(0x368, &mut car.damage_levels);
    m.plain(0x3f4, &mut car.engine_power_factor);
    m.plain(0x3f8, &mut car.brake_efficiency);
    m.plain(0x400, &mut car.drive_efficiency);
    m.plain(0x408, &mut car.steering_efficiency);
    m.plain(0x40c, &mut car.electrical_failures);
    m.plain(0x428, &mut car.handbrake_failed);
    m.plain(0x42c, &mut car.gear_change_blocked);
    m.plain(0x430, &mut car.forced_neutral);
    m.plain(0x434, &mut car.lost_gear_bits);
    m.plain(0x438, &mut car.wheel_ride_height);
    m.plain(0x448, &mut car.wheel_stiffness);
    m.plain(0x458, &mut car.stiffness_mean);
    m.plain(0x45c, &mut car.ride_height_mean);
    m.plain(0x460, &mut car.hit_cooldown);
    m.plain(0x464, &mut car.input_throttle);
    m.plain(0x468, &mut car.input_keys);
    m.plain(0x46c, &mut car.input_steering);
    m.plain(0x470, &mut car.input_brake);
    m.plain(0x474, &mut car.keys_pressed);
    m.plain(0x478, &mut car.notice_count);
    m.plain(0x47c, &mut car.notice_ids);
    m.plain(0x4fc, &mut car.notice_values);
    m.plain(0x57c, &mut car.notice_ranks);
    m.plain(0x5fc, &mut car.top_speed);
    m.plain(0x600, &mut car.wheel_squeeze);
    m.plain(0x610, &mut car.wheel_squeeze_last);
    m.plain(0x620, &mut car.checkpoint_offset);
    m.plain(0x628, &mut car.pace_gaps);
    m.plain(0x630, &mut car.pace_gaps_eased);
    m.plain(0x638, &mut car.throttle);
    m.plain(0x63c, &mut car.brake);
    m.plain(0x640, &mut car.handbrake);
    m.plain(0x644, &mut car.steering);
    m.plain(0x648, &mut car.steering_raw);
    m.plain(0x64c, &mut car.steer_angle);
    m.plain(0x650, &mut car.steer_angle_visual);
    m.plain(0x654, &mut car.mode2_place);
    m.plain(0x658, &mut car.mode2_behind);
    m.plain(0x65c, &mut car.pace_cars);
    m.plain(0x664, &mut car.pace_starts);
    m.plain(0x66c, &mut car.pace_placed);
    m.plain(0x670, &mut car.pace_apart);
    m.plain(0x674, &mut car.pace_push);
    m.plain(0x678, &mut car.start_share);
    m.plain(0x67c, &mut car.start_blend_share);
    m.plain(0x680, &mut car.start_blend);
    for (at, len) in CAR_UNUSED {
        m.unused(at, len);
    }
}

/// A loose part (0x78 bytes).
fn slot_fields(slot: &mut RaceSlot, m: &mut impl Move) {
    m.plain(0x0, &mut slot.part);
    m.plain(0x4, &mut slot.place);
    m.plain(0x10, &mut slot.matrix);
    m.plain(0x50, &mut slot.motion);
    m.plain(0x5c, &mut slot.draw_height_offset);
    m.plain(0x60, &mut slot.car_word);
    m.plain(0x64, &mut slot.age);
    m.plain(0x68, &mut slot.spin);
    m.plain(0x74, &mut slot.spin_length);
}

/// The computer driver's state: the 0x400 bytes of the race work from `replay_extra`.
fn extra_fields(work: &mut RaceWork, m: &mut impl Move) {
    m.plain(0x0, &mut work.replay_extra);
    m.plain(0x4, &mut work.driver_steer);
    m.plain(0x8, &mut work.driver_throttle);
    m.plain(0xc, &mut work.driver_brake);
    m.plain(0x10, &mut work.driver_flags);
    m.plain(0x20, &mut work.driver_start_blend);
    m.plain(0x24, &mut work.driver_steps);
    m.plain(0x4c, &mut work.driver_reverse_left);
    m.plain(0x74, &mut work.driver_moved);
    m.plain(0x78, &mut work.driver_turn);
    m.plain(0x7c, &mut work.driver_bend_index);
    m.plain(0x84, &mut work.driver_wanted_speed);
    m.plain(0x88, &mut work.driver_speed);
    m.plain(0x8c, &mut work.driver_bend_to);
    m.plain(0xb4, &mut work.driver_bend_noted);
    m.plain(0xdc, &mut work.driver_reverse_tries);
    m.plain(0x104, &mut work.driver_bend_turn);
    m.plain(0x138, &mut work.driver_flag_bc);
    m.plain(0x13c, &mut work.driver_step_x);
    m.plain(0x140, &mut work.driver_step_z);
    m.plain(0x144, &mut work.driver_speed_scale);
    m.plain(0x148, &mut work.driver_slot);
    m.plain(0x14c, &mut work.driver_lookahead);
    m.plain(0x150, &mut work.driver_bend_a);
    m.plain(0x154, &mut work.driver_bend_b);
    m.plain(0x158, &mut work.driver_point_now);
    m.plain(0x15c, &mut work.driver_point_then);
    m.plain(0x160, &mut work.driver_aim);
    m.plain(0x164, &mut work.driver_points_now);
    m.plain(0x204, &mut work.driver_points_then);
    m.plain(0x2a4, &mut work.driver_from);
    m.plain(0x2b4, &mut work.driver_to);
    m.plain(0x2d4, &mut work.driver_way);
    m.plain(0x2e4, &mut work.driver_way_unit);
    m.plain(0x2f4, &mut work.driver_ahead);
    m.plain(0x314, &mut work.driver_flat);
    m.plain(0x324, &mut work.driver_flat_unit);
    m.plain(0x334, &mut work.driver_side);
    m.plain(0x344, &mut work.driver_work);
    for (at, len) in EXTRA_UNUSED {
        m.unused(at, len);
    }
}

fn pieces(cars: &mut [Car], work: &mut RaceWork, m: &mut dyn FnMut(usize, Piece)) {
    for (k, car) in cars.iter_mut().take(4).enumerate() {
        m(k * CAR_BYTES, Piece::Car(car));
    }
    for (i, slot) in work.race_slots.iter_mut().enumerate() {
        m(SLOTS_AT + i * SLOT_BYTES, Piece::Slot(slot));
    }
    m(EXTRA_AT, Piece::Extra(work));
}

enum Piece<'a> {
    Car(&'a mut Car),
    Slot(&'a mut RaceSlot),
    Extra(&'a mut RaceWork),
}

fn move_piece(piece: Piece, m: &mut impl Move) {
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
    // The fields are moved through `&mut`: copies of the cars and the work variables are walked.
    let mut head: Vec<Car> = cars.iter().take(4).cloned().collect();
    let mut work = work.clone();
    pieces(&mut head, &mut work, &mut |base, piece| {
        move_piece(
            piece,
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
pub(super) fn scatter(frame: &ReplayFrame, cars: &mut [Car], work: &mut RaceWork) {
    let bytes = frame.start_bytes();
    pieces(cars, work, &mut |base, piece| {
        move_piece(
            piece,
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
        &mut car.clone(),
        &mut Gather {
            bytes: &mut bytes,
            base: 0,
        },
    );
    std::array::from_fn(|n| u32::from_le_bytes(bytes[4 * n..4 * n + 4].try_into().expect("4")))
}

/// A car's fields put from its dwords in the file's layout.
fn scatter_car(words: &[u32; CAR_DWORDS], car: &mut Car) {
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
    fn plain<T: LeView>(&mut self, at: usize, field: &mut T) {
        if STATE_AT.contains(&(at / 4)) {
            field.store(&self.bytes[at..at + T::BYTES]);
        }
    }

    fn unused(&mut self, _at: usize, _len: usize) {}
}

/// A car's recorded state put into it, the car's other fields left as they are.
pub(super) fn scatter_state(state: &CarState, car: &mut Car) {
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
pub(super) fn scatter_tail(keys: &[[u32; CAR_DWORDS]; TAIL_CARS], cars: &mut [Car], count: usize) {
    for (k, words) in keys.iter().enumerate() {
        if HEAD_CARS + k < count
            && let Some(car) = cars.get_mut(HEAD_CARS + k)
        {
            scatter_car(words, car);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::cells::CellVisitor;
    use crate::memory::plain_record::PlainRecord;
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

    /// The bytes of the record in the order of its fields (the layout the game's records had).
    fn work_bytes(work: &RaceWork) -> Vec<u8> {
        struct Collect(Vec<u8>);
        impl CellVisitor for Collect {
            fn enter(&mut self, _: &'static str) {}
            fn index(&mut self, _: usize) {}
            fn leave(&mut self) {}
            fn leaf(&mut self, bytes: &[u8]) {
                self.0.extend_from_slice(bytes);
            }
        }
        let mut collect = Collect(Vec::new());
        work.visit(&mut collect);
        collect.0
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
    fn raw_bytes(record: &[u8], base: usize, len: usize, unused: &[(usize, usize)]) -> Vec<u8> {
        (0..len)
            .map(|at| match in_record(at, unused) {
                Some(n) => record[base + n],
                None => UNUSED_BYTE,
            })
            .collect()
    }

    /// A frame from the file's bytes put into the cars and the work, which then lie as the game's
    /// raw copy had them: every field at the offset the game's record had (the cars' first 0x684
    /// bytes, the loose parts from `race_slots` on, the driver's state from `replay_extra` on).
    #[test]
    fn scatter_puts_every_field_at_the_files_offset() {
        let bytes = with_unused_places((0..FRAME_BYTES).map(|n| pattern(n + 3)).collect());
        let frame = ReplayFrame::from_bytes(&bytes).unwrap();
        let mut cars = cars_of(|_| 0);
        let mut work = RaceWork::new();
        scatter(&frame, &mut cars, &mut work);
        for (k, car) in cars.iter().enumerate() {
            let mut file = crate::physics::car_offsets_1999::bytes_1999(car);
            file.truncate(CAR_BYTES);
            assert_eq!(file, bytes[k * CAR_BYTES..(k + 1) * CAR_BYTES], "car {k}");
        }
        let memory = work_bytes(&work);
        assert_eq!(
            memory[SLOTS_AT..SLOTS_AT + 8 * SLOT_BYTES],
            bytes[super::SLOTS_AT..super::SLOTS_AT + 8 * SLOT_BYTES]
        );
        assert_eq!(
            raw_bytes(&memory, EXTRA_AT, 0x400, &EXTRA_UNUSED),
            bytes[super::EXTRA_AT..super::EXTRA_AT + 0x400]
        );
    }

    #[test]
    fn scatter_writes_only_the_starting_state() {
        let bytes = with_unused_places((0..FRAME_BYTES).map(|n| pattern(n + 5)).collect());
        let frame = ReplayFrame::from_bytes(&bytes).unwrap();
        let mut cars = cars_of(|_| 0);
        let mut work = RaceWork::new();
        let before = work_bytes(&work);
        scatter(&frame, &mut cars, &mut work);
        assert_eq!(gather(&cars, &work).start_bytes(), frame.start_bytes());
        for car in &cars {
            let file = crate::physics::car_offsets_1999::bytes_1999(car);
            assert_eq!(file[CAR_BYTES..], vec![0; file.len() - CAR_BYTES]);
        }
        // What lies before the slots and between them and the driver's state stays.
        let after = work_bytes(&work);
        assert_eq!(after[..SLOTS_AT], before[..SLOTS_AT]);
        assert_eq!(
            after[SLOTS_AT + 0x3C0..EXTRA_AT],
            before[SLOTS_AT + 0x3C0..EXTRA_AT]
        );
    }

    /// A synthetic file frame goes through the cars and back byte for byte.
    #[test]
    fn a_synthetic_frame_round_trips() {
        let bytes = with_unused_places((0..FRAME_BYTES).map(|n| pattern(n + 7)).collect());
        let frame = ReplayFrame::from_bytes(&bytes).unwrap();
        let mut cars = cars_of(|_| 0);
        let mut work = RaceWork::new();
        scatter(&frame, &mut cars, &mut work);
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
        let mut cars = cars_of(|_| 0);
        let mut work = RaceWork::new();
        let mut frames = 0;
        for file in files.flatten() {
            let Ok(data) = std::fs::read(file.path()) else {
                continue;
            };
            for block in file_blocks(&data) {
                let frame = ReplayFrame::from_bytes(&block).expect("a whole frame");
                assert_eq!(frame.to_bytes(), block);
                scatter(&frame, &mut cars, &mut work);
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
