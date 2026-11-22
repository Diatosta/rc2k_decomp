//! A word of the tables reached by its field path, for the tests that read and write the places
//! of the recorded blocks (`test_support::race_places`).

use super::{ForceRecord, PhysicsTables, Surface};

/// A 32-bit word of a plain record: its bits, read and set.
pub trait Word {
    fn bits(&self) -> u32;
    fn set_bits(&mut self, bits: u32);
}

impl Word for f32 {
    fn bits(&self) -> u32 {
        self.to_bits()
    }
    fn set_bits(&mut self, bits: u32) {
        *self = f32::from_bits(bits);
    }
}

impl Word for u32 {
    fn bits(&self) -> u32 {
        *self
    }
    fn set_bits(&mut self, bits: u32) {
        *self = bits;
    }
}

impl Word for i32 {
    fn bits(&self) -> u32 {
        *self as u32
    }
    fn set_bits(&mut self, bits: u32) {
        *self = bits as i32;
    }
}

/// `name` and its index in `name[index]`.
fn segment(s: &str) -> Option<(&str, Option<usize>)> {
    match s.split_once('[') {
        None => Some((s, None)),
        Some((name, index)) => Some((name, Some(index.strip_suffix(']')?.parse().ok()?))),
    }
}

/// The record's word at `path` (`name[index]`, then more segments for a record in an array).
fn split(path: &str) -> Option<(&str, Option<usize>, Option<&str>)> {
    let (first, next) = match path.split_once('.') {
        Some((first, next)) => (first, Some(next)),
        None => (path, None),
    };
    let (name, index) = segment(first)?;
    Some((name, index, next))
}

fn scalar<'a, T: Word>(
    word: &'a mut T,
    index: Option<usize>,
    next: Option<&str>,
) -> Option<&'a mut dyn Word> {
    (index.is_none() && next.is_none()).then_some(word as &mut dyn Word)
}

fn in_array<'a, T: Word>(
    words: &'a mut [T],
    index: Option<usize>,
    next: Option<&str>,
) -> Option<&'a mut dyn Word> {
    if next.is_some() {
        return None;
    }
    Some(words.get_mut(index?)? as &mut dyn Word)
}

impl Surface {
    fn word_mut(&mut self, path: &str) -> Option<&mut dyn Word> {
        let s = self;
        let (name, index, next) = split(path)?;
        match name {
            "table" => scalar(&mut s.table, index, next),
            "slide_grip" => scalar(&mut s.slide_grip, index, next),
            "accel_grip" => scalar(&mut s.accel_grip, index, next),
            "brake_grip" => scalar(&mut s.brake_grip, index, next),
            "mud_rates" => in_array(&mut s.mud_rates, index, next),
            "reserved_18" => in_array(&mut s.reserved_18, index, next),
            "drag" => scalar(&mut s.drag, index, next),
            "rolling_drag_min_speed" => scalar(&mut s.rolling_drag_min_speed, index, next),
            "grip" => scalar(&mut s.grip, index, next),
            "wear_factor" => scalar(&mut s.wear_factor, index, next),
            _ => None,
        }
    }
}

impl ForceRecord {
    fn word_mut(&mut self, path: &str) -> Option<&mut dyn Word> {
        let r = self;
        let (name, index, next) = split(path)?;
        match name {
            "force" => in_array(&mut r.force, index, next),
            "point" => in_array(&mut r.point, index, next),
            _ => None,
        }
    }
}

impl PhysicsTables {
    /// The word at `path` (`.name`, `.name[i]`, `.records[i].name[j]`).
    pub fn word_mut(&mut self, path: &str) -> Option<&mut dyn Word> {
        let t = self;
        let (name, index, next) = split(path.strip_prefix('.')?)?;
        match name {
            "camera_drop" => scalar(&mut t.camera_drop, index, next),
            "contact_steps" => scalar(&mut t.contact_steps, index, next),
            "surfaces" => {
                let record = t.surfaces.get_mut(index?)?;
                record.word_mut(next?)
            }
            "ground_spin_limit" => scalar(&mut t.ground_spin_limit, index, next),
            "spec_default_bc0" => in_array(&mut t.spec_default_bc0, index, next),
            "rev_limits" => in_array(&mut t.rev_limits, index, next),
            "gravity" => scalar(&mut t.gravity, index, next),
            "kept_motion" => in_array(&mut t.kept_motion, index, next),
            "stall_figure" => scalar(&mut t.stall_figure, index, next),
            "drive_shares" => in_array(&mut t.drive_shares, index, next),
            "gear_ratios" => in_array(&mut t.gear_ratios, index, next),
            "idle_engine_speed" => scalar(&mut t.idle_engine_speed, index, next),
            "stall_engine_speed" => scalar(&mut t.stall_engine_speed, index, next),
            "top_speed" => scalar(&mut t.top_speed, index, next),
            "rev_limit" => scalar(&mut t.rev_limit, index, next),
            "clear_place_face" => scalar(&mut t.clear_place_face, index, next),
            "surface_scales" => in_array(&mut t.surface_scales, index, next),
            "vertical_speed" => scalar(&mut t.vertical_speed, index, next),
            "start_gear_figures" => in_array(&mut t.start_gear_figures, index, next),
            "contact_step" => scalar(&mut t.contact_step, index, next),
            "hard_ground" => in_array(&mut t.hard_ground, index, next),
            "grip_drive_word" => scalar(&mut t.grip_drive_word, index, next),
            "drive_speed_figure" => scalar(&mut t.drive_speed_figure, index, next),
            "no_auto_neutral" => scalar(&mut t.no_auto_neutral, index, next),
            "weight" => scalar(&mut t.weight, index, next),
            "downforce" => scalar(&mut t.downforce, index, next),
            "wheel_spin_push" => in_array(&mut t.wheel_spin_push, index, next),
            "saved_place" => in_array(&mut t.saved_place, index, next),
            "saved_matrix" => in_array(&mut t.saved_matrix, index, next),
            "wheel_speed_step" => scalar(&mut t.wheel_speed_step, index, next),
            "surface_softness" => scalar(&mut t.surface_softness, index, next),
            "drive_limits" => in_array(&mut t.drive_limits, index, next),
            "grip_limit" => scalar(&mut t.grip_limit, index, next),
            "slip_limit" => scalar(&mut t.slip_limit, index, next),
            "wheel_load" => scalar(&mut t.wheel_load, index, next),
            "settle_steps" => in_array(&mut t.settle_steps, index, next),
            "wheel_loads" => in_array(&mut t.wheel_loads, index, next),
            "wheel_grips" => in_array(&mut t.wheel_grips, index, next),
            "wheel_grips_b" => in_array(&mut t.wheel_grips_b, index, next),
            "point_grips" => in_array(&mut t.point_grips, index, next),
            "surface_blends" => in_array(&mut t.surface_blends, index, next),
            "surface_firsts" => in_array(&mut t.surface_firsts, index, next),
            "surface_seconds" => in_array(&mut t.surface_seconds, index, next),
            "clear_place_count" => scalar(&mut t.clear_place_count, index, next),
            "drive_ratio" => scalar(&mut t.drive_ratio, index, next),
            "engine_torque" => scalar(&mut t.engine_torque, index, next),
            "engine_force" => scalar(&mut t.engine_force, index, next),
            "drive_torque" => scalar(&mut t.drive_torque, index, next),
            "torque_share" => scalar(&mut t.torque_share, index, next),
            "low_speed_share" => scalar(&mut t.low_speed_share, index, next),
            "wheel_drag" => in_array(&mut t.wheel_drag, index, next),
            "lever_share" => scalar(&mut t.lever_share, index, next),
            "downforce_spec" => scalar(&mut t.downforce_spec, index, next),
            "forces" => {
                let record = t.forces.get_mut(index?)?;
                record.word_mut(next?)
            }
            "contacts" => {
                let record = t.contacts.get_mut(index?)?;
                record.word_mut(next?)
            }
            "grip_scale" => scalar(&mut t.grip_scale, index, next),
            "side_grip_scale" => scalar(&mut t.side_grip_scale, index, next),
            "spec_torque" => in_array(&mut t.spec_torque, index, next),
            "spec_grip" => scalar(&mut t.spec_grip, index, next),
            "spec_wheel" => scalar(&mut t.spec_wheel, index, next),
            "spec_drive" => in_array(&mut t.spec_drive, index, next),
            "spec_surface" => in_array(&mut t.spec_surface, index, next),
            "spec_ints" => in_array(&mut t.spec_ints, index, next),
            "kept_vectors" => in_array(&mut t.kept_vectors, index, next),
            "handling" => in_array(&mut t.handling, index, next),
            "gear_count" => scalar(&mut t.gear_count, index, next),
            "ride_height_handling_scale" => scalar(&mut t.ride_height_handling_scale, index, next),
            "axle_slip_limits" => in_array(&mut t.axle_slip_limits, index, next),
            "gear_asked" => scalar(&mut t.gear_asked, index, next),
            "reset_offset" => scalar(&mut t.reset_offset, index, next),
            "reset_candidate" => scalar(&mut t.reset_candidate, index, next),
            "nearest_checkpoint_distance" => {
                scalar(&mut t.nearest_checkpoint_distance, index, next)
            }
            "split_point_distance" => scalar(&mut t.split_point_distance, index, next),
            "pace_slot" => scalar(&mut t.pace_slot, index, next),
            _ => None,
        }
    }
}
