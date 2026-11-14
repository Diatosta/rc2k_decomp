//! The 1999 byte layout of the work car record, for the tests whose subject is where a field sat
//! in the original (`(field, offset, length)`; places no code read are not in it). The record
//! itself keeps no layout: the offsets of the original's code, the replay file's and the recorded
//! vectors' are mapped to fields here, and nowhere else.

use crate::physics::car_store::Car;
use crate::test_support::paths::{bytes, first_addr, get_u32, set_bytes, set_u32};

/// The size of the 1999 record.
pub const SIZE_1999: usize = 0xFE0;

const FIELDS: [(&str, usize, usize); 263] = [
    ("frame_head", 0x0, 8),
    ("step_clock", 0x8, 4),
    ("noise_word", 0xc, 4),
    ("car_flags", 0x10, 4),
    ("run_ticks", 0x14, 4),
    ("start_in_gear", 0x18, 4),
    ("false_start_count", 0x1c, 4),
    ("brake_balance", 0x20, 4),
    ("car_kind", 0x24, 4),
    ("gear_ratio_setting", 0x28, 4),
    ("tyre_kind", 0x2c, 4),
    ("ride_height", 0x30, 4),
    ("suspension_stiffness", 0x34, 4),
    ("gearbox_kind", 0x38, 4),
    ("steering_sensitivity", 0x3c, 4),
    ("drive_kind", 0x40, 4),
    ("unused_44", 0x44, 4),
    ("part_woken", 0x48, 24),
    ("ground_bump_scale", 0x60, 4),
    ("impact_octant_marks", 0x64, 32),
    ("lights_on", 0x84, 4),
    ("stage_time", 0x88, 4),
    ("time_left", 0x8c, 4),
    ("splits_passed", 0x90, 4),
    ("reset_checkpoint", 0x94, 4),
    ("checkpoint", 0x98, 4),
    ("last_pace_call", 0x9c, 4),
    ("started_stand_timer", 0xa0, 4),
    ("go_stand_timer", 0xa4, 4),
    ("finish_flag", 0xa8, 4),
    ("splits_recorded", 0xac, 4),
    ("stop_state", 0xb0, 4),
    ("crash_matrix", 0xb4, 64),
    ("position", 0x134, 12),
    ("split_times", 0x140, 64),
    ("drive_slip_speed", 0x180, 4),
    ("spin", 0x184, 12),
    ("spin_length", 0x194, 4),
    ("velocity", 0x198, 12),
    ("local_velocity", 0x1a4, 12),
    ("drive_speed_a", 0x1b8, 4),
    ("drive_speed_b", 0x1bc, 4),
    ("mean_drive_speed", 0x1c0, 4),
    ("speed", 0x1c4, 4),
    ("side_speed_held", 0x1c8, 4),
    ("gear_change_hold", 0x1cc, 4),
    ("drive_cut_steps", 0x1d0, 4),
    ("gear", 0x1d4, 4),
    ("engine_speed", 0x1d8, 4),
    ("engine_speed_limit", 0x1dc, 4),
    ("spec_scale_copy", 0x1e0, 4),
    ("engine_sound_revs", 0x1e4, 4),
    ("stuck_count", 0x1e8, 4),
    ("position_message_word", 0x1ec, 4),
    ("rev_cut_count", 0x1f0, 4),
    ("wall_hit_timer", 0x1f4, 4),
    ("driven_wheel_turns", 0x1f8, 8),
    ("eased_yaw_spin", 0x200, 4),
    ("grip_force_sum", 0x204, 4),
    ("steer_trim", 0x208, 4),
    ("unused_20c", 0x20c, 4),
    ("grip_forces", 0x210, 16),
    ("option_bits", 0x220, 4),
    ("contact_point", 0x224, 12),
    ("contact_depth", 0x230, 4),
    ("contact_flag", 0x234, 4),
    ("gear_changed", 0x238, 4),
    ("gear_change_pending", 0x23c, 4),
    ("light_colour", 0x240, 4),
    ("light_level", 0x244, 4),
    ("front_wheels_off", 0x248, 4),
    ("rear_wheels_off", 0x24c, 4),
    ("driven_wheels_off", 0x250, 4),
    ("all_wheels_off", 0x254, 4),
    ("hard_ground_count", 0x258, 4),
    ("driver_kind", 0x25c, 4),
    ("probe_count", 0x260, 4),
    ("reset_key_count", 0x264, 4),
    ("stopped_steps", 0x268, 4),
    ("tyre_change_state", 0x26c, 4),
    ("off_line_steps", 0x270, 4),
    ("turned_steps", 0x274, 4),
    ("off_line_cue_time", 0x278, 4),
    ("turned_cue_time", 0x27c, 4),
    ("surface_blend", 0x280, 4),
    ("parts_state", 0x284, 4),
    ("slip_speed", 0x288, 4),
    ("sound_slip_share", 0x28c, 4),
    ("sound_level", 0x290, 4),
    ("sound_speed", 0x294, 4),
    ("parts_present", 0x298, 4),
    ("damage_failed", 0x29c, 4),
    ("brake_fade", 0x2a0, 4),
    ("damper_wear", 0x2a4, 8),
    ("brake_heat", 0x2ac, 8),
    ("engine_heat", 0x2b4, 4),
    ("engine_damage", 0x2b8, 4),
    ("exhaust_damage", 0x2bc, 4),
    ("turbo_damage", 0x2c0, 4),
    ("cooling_damage", 0x2c4, 4),
    ("clutch_damage", 0x2c8, 4),
    ("differential_damage", 0x2cc, 4),
    ("gear_wear", 0x2d0, 36),
    ("gearbox_damage", 0x2f4, 4),
    ("wheel_hit_damage", 0x2f8, 16),
    ("body_hit_damage", 0x308, 4),
    ("body_damage", 0x30c, 4),
    ("electrics_damage", 0x310, 4),
    ("light_damage", 0x314, 24),
    ("lights_damage", 0x32c, 4),
    ("steering_damage", 0x330, 4),
    ("suspension_damage", 0x334, 16),
    ("overall_suspension_damage", 0x344, 4),
    ("brake_wear_a", 0x348, 4),
    ("brake_wear_b", 0x34c, 4),
    ("brake_damage", 0x350, 4),
    ("tyre_wears", 0x354, 16),
    ("tyre_wear", 0x364, 4),
    ("damage_levels", 0x368, 140),
    ("engine_power_factor", 0x3f4, 4),
    ("brake_efficiency", 0x3f8, 8),
    ("drive_efficiency", 0x400, 8),
    ("steering_efficiency", 0x408, 4),
    ("electrical_failures", 0x40c, 28),
    ("handbrake_failed", 0x428, 4),
    ("gear_change_blocked", 0x42c, 4),
    ("forced_neutral", 0x430, 4),
    ("lost_gear_bits", 0x434, 4),
    ("wheel_ride_height", 0x438, 16),
    ("wheel_stiffness", 0x448, 16),
    ("stiffness_mean", 0x458, 4),
    ("ride_height_mean", 0x45c, 4),
    ("hit_cooldown", 0x460, 4),
    ("input_throttle", 0x464, 4),
    ("input_keys", 0x468, 4),
    ("input_steering", 0x46c, 4),
    ("input_brake", 0x470, 4),
    ("keys_pressed", 0x474, 4),
    ("notice_count", 0x478, 4),
    ("notice_ids", 0x47c, 128),
    ("notice_values", 0x4fc, 128),
    ("notice_ranks", 0x57c, 128),
    ("top_speed", 0x5fc, 4),
    ("wheel_squeeze", 0x600, 16),
    ("wheel_squeeze_last", 0x610, 16),
    ("checkpoint_offset", 0x620, 8),
    ("pace_gaps", 0x628, 8),
    ("pace_gaps_eased", 0x630, 8),
    ("throttle", 0x638, 4),
    ("brake", 0x63c, 4),
    ("handbrake", 0x640, 4),
    ("steering", 0x644, 4),
    ("steering_raw", 0x648, 4),
    ("steer_angle", 0x64c, 4),
    ("steer_angle_visual", 0x650, 4),
    ("mode2_place", 0x654, 4),
    ("mode2_behind", 0x658, 4),
    ("pace_cars", 0x65c, 8),
    ("pace_starts", 0x664, 8),
    ("pace_placed", 0x66c, 4),
    ("pace_apart", 0x670, 4),
    ("pace_push", 0x674, 4),
    ("start_share", 0x678, 4),
    ("start_blend_share", 0x67c, 4),
    ("start_blend", 0x680, 4),
    ("camera_view", 0x684, 4),
    ("camera_blend_view", 0x688, 4),
    ("camera_view_return", 0x68c, 4),
    ("camera_switch_clock", 0x690, 4),
    ("camera_switch_active", 0x694, 4),
    ("camera_switch_length", 0x698, 4),
    ("camera_switch_share", 0x69c, 4),
    ("camera_roll", 0x6a0, 4),
    ("camera_position", 0x6a4, 12),
    ("camera_heading", 0x6b0, 4),
    ("camera_roll_turn", 0x6b4, 4),
    ("camera_pitch", 0x6b8, 4),
    ("camera_offset_turns", 0x6bc, 12),
    ("camera_zoom", 0x6c8, 4),
    ("kept_focus_point", 0x6cc, 36),
    ("camera_mode_is_2_or_3", 0x6f0, 4),
    ("state_bits", 0x6fc, 4),
    ("part_wake_counts", 0x700, 24),
    ("shown_engine_speed", 0x718, 4),
    ("notice_id", 0x71c, 4),
    ("notice_counter", 0x720, 4),
    ("notice_last", 0x724, 4),
    ("notice_shown", 0x728, 4),
    ("notice_time", 0x72c, 4),
    ("notice_time_b", 0x730, 4),
    ("kept_view_place", 0x734, 12),
    ("part_matrices", 0x740, 256),
    ("body_matrix", 0x840, 64),
    ("place_matrix", 0x880, 64),
    ("whole_matrix", 0x8c0, 64),
    ("surface_noise_a", 0x900, 4),
    ("surface_noise_b", 0x904, 4),
    ("surface_noise_second", 0x908, 4),
    ("vertical_speed_change", 0x90c, 4),
    ("wheel_spring_force", 0x910, 16),
    ("wheel_surface_a", 0x920, 20),
    ("wheel_surface_b", 0x934, 20),
    ("point_wear_factor", 0x948, 20),
    ("wheel_cell_a_byte18", 0x95c, 20),
    ("wheel_cell_b_byte18", 0x970, 20),
    ("wheel_cell_a_byte14", 0x984, 20),
    ("wheel_cell_b_byte14", 0x998, 20),
    ("wheel_surface_blend", 0x9ac, 20),
    ("car_heading", 0x9c0, 4),
    ("car_pitch", 0x9c4, 4),
    ("car_roll", 0x9c8, 4),
    ("car_inverse_matrix", 0x9cc, 64),
    ("split_gaps", 0xa0c, 28),
    ("contact_point_x", 0xa28, 4),
    ("contact_point_ground", 0xa2c, 4),
    ("contact_point_z", 0xa30, 4),
    ("contact_found", 0xa34, 4),
    ("contact_value", 0xa38, 4),
    ("part_damage_limits", 0xa3c, 28),
    ("saved_point_figures", 0xa58, 80),
    ("start_slot_value", 0xaa8, 4),
    ("shown_steering", 0xaac, 4),
    ("engine_sound_speed_int", 0xab0, 4),
    ("marks_on", 0xab4, 4),
    ("mode_on", 0xab8, 4),
    ("marks_count", 0xabc, 4),
    ("part_switches", 0xac0, 17),
    ("_pad_part_switches", 0xad1, 3),
    ("map_corner", 0xad4, 8),
    ("map_redrawn", 0xadc, 4),
    ("map_place", 0xae0, 8),
    ("map_picture_place", 0xae8, 8),
    ("probe_point_x", 0xaf0, 256),
    ("probe_point_y", 0xbf0, 256),
    ("probe_point_z", 0xcf0, 256),
    ("probe_point_face", 0xdf0, 256),
    ("camera_ground_faces", 0xef0, 20),
    ("group_mask", 0xf04, 4),
    ("engine_speed_scaled", 0xf08, 4),
    ("previous_step_clock", 0xf0c, 4),
    ("wheel_ground_x", 0xf10, 16),
    ("wheel_ground_height", 0xf20, 16),
    ("wheel_ground_z", 0xf30, 16),
    ("skid_grip", 0xf40, 8),
    ("skid_slots", 0xf48, 16),
    ("car_event_times", 0xf58, 32),
    ("car_event_codes", 0xf78, 32),
    ("car_event_count", 0xf98, 4),
    ("car_event_limit", 0xf9c, 4),
    ("car_event_position", 0xfa0, 4),
    ("two_car_place", 0xfa4, 4),
    ("view_covered", 0xfa8, 4),
    ("reset_marker", 0xfac, 4),
    ("value_jump_steps", 0xfb0, 4),
    ("light_hold_steps", 0xfb4, 4),
    ("eased_target", 0xfb8, 4),
    ("eased_value", 0xfbc, 4),
    ("glide_target", 0xfc0, 12),
    ("glide_steps_left", 0xfcc, 4),
    ("kept_throttle_word", 0xfd0, 4),
    ("kept_keys_word", 0xfd4, 4),
    ("kept_steering_word", 0xfd8, 4),
    ("kept_brake_word", 0xfdc, 4),
];

/// The 1999 offset of the byte at address `addr` of `car`, which lies in one of its fields.
pub fn offset_1999(car: &Car, addr: usize) -> usize {
    for (name, start, len) in FIELDS {
        let first = first_addr(car, &format!(".{name}")).expect("a field of the car");
        if (first..first + len).contains(&addr) {
            return start + addr - first;
        }
    }
    panic!("{addr:#x} is in no field of the car");
}

/// The path of the dword cell the 1999 offset `offset` names.
fn cell_path(offset: usize) -> String {
    for (name, start, len) in FIELDS {
        if (start..start + len).contains(&offset) {
            assert!(
                (offset - start).is_multiple_of(4),
                "{offset:#x} is inside a dword"
            );
            return if len == 4 {
                format!(".{name}")
            } else {
                format!(".{name}[{}]", (offset - start) / 4)
            };
        }
    }
    panic!("{offset:#x} was an unused place");
}

/// The dword at the 1999 offset `offset`.
pub fn load_1999(car: &Car, offset: usize) -> u32 {
    get_u32(car, &cell_path(offset)).expect("a dword cell")
}

/// Sets the dword at the 1999 offset `offset`.
pub fn store_1999(car: &Car, offset: usize, value: u32) {
    assert!(set_u32(car, &cell_path(offset), value));
}

/// The 1999 offset of cell `n` of the table `cells` of `car`, running on past the table's end
/// into what followed it, as the record's fields do now: the places no code read (not in
/// `FIELDS`) are stepped over, as the closed-up record has no bytes there.
fn run_on(car: &Car, cells: &[impl crate::session::Cell32], n: usize) -> usize {
    let covered = |offset: usize| {
        FIELDS
            .iter()
            .any(|&(_, start, len)| (start..start + len).contains(&offset))
    };
    let mut offset = offset_1999(car, cells.as_ptr().addr());
    for _ in 0..n {
        offset += 4;
        while offset < SIZE_1999 && !covered(offset) {
            offset += 4;
        }
    }
    offset
}

/// Cell `n` of the table `cells` of `car` as the 1999 layout reads it, running on past the
/// table's end into the fields that follow it (`run_on`).
pub fn read_on_1999<C: crate::session::Cell32>(car: &Car, cells: &[C], n: usize) -> u32 {
    load_1999(car, run_on(car, cells, n))
}

/// `read_on_1999`'s write.
pub fn write_on_1999<C: crate::session::Cell32>(car: &Car, cells: &[C], n: usize, value: u32) {
    store_1999(car, run_on(car, cells, n), value);
}

/// The record as the 1999 bytes (`SIZE_1999` of them; the unused places are zero).
pub fn bytes_1999(car: &Car) -> Vec<u8> {
    let mut out = vec![0; SIZE_1999];
    for (name, start, len) in FIELDS {
        let field = bytes(car, &format!(".{name}")).expect("a field of the car");
        assert_eq!(field.len(), len, "{name}");
        out[start..start + len].copy_from_slice(&field);
    }
    out
}

/// A car whose byte at the 1999 offset `n` is `fill(n)` (the unused places have no byte).
pub fn car_with_1999_bytes(fill: impl Fn(usize) -> u8) -> Car {
    let car = Car::new();
    for (name, start, len) in FIELDS {
        let field: Vec<u8> = (start..start + len).map(&fill).collect();
        assert!(set_bytes(&car, &format!(".{name}"), &field), "{name}");
    }
    car
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The table tiles the record apart from the unused places, in the fields' own order.
    #[test]
    fn the_table_covers_the_record_but_for_the_unused_places() {
        let mut covered = vec![false; SIZE_1999];
        for (_, start, len) in FIELDS {
            for b in &mut covered[start..start + len] {
                assert!(!*b);
                *b = true;
            }
        }
        let missing: usize = covered.iter().filter(|&&b| !b).count();
        assert_eq!(missing, 64 + 4 + 8 + 8);
        let flat = crate::test_support::paths::field_bytes(&Car::new(), "").len();
        assert_eq!(flat, SIZE_1999 - missing);
    }
}
