//! The marks' settings: the record the race hands the effects (`effects::init_marks`, flagged in
//! `marks_settings`, `SETTINGS`) and the effects read every frame: a head the race's views and
//! `drive::set_listener` write, the eight cars' records (`effects::init_mark_step` copies them),
//! and a tail of the listener's figures and switches. One record, between the replay viewer's panel
//! and the kind tables.

use crate::errors::BadInput;
use crate::memory::cells::ZeroCells;
use crate::memory::first_cells::plain_dwords;
use crate::memory::plain_floats::PlainFloats;
use crate::memory::session::{PlainDword, PlainFloat, PlainInt};
use crate::race::blocks_store::CarBlocks;
use crate::race::car_parts_store::CarParts;
#[cfg(test)]
use crate::test_support::own::race;

/// A car's record in the settings (0x4C8 bytes; `effects::blocks`, `effects::cars_particles`,
/// `effects::morph_object_points`, `effects::tidy_marks` work on it). Names are readings from what
/// writes the record (`drive::share_car_state`, from the car block) and what reads it. The five
/// wheel and body records (0x48 bytes each) are `MarkRecord`s; the tail after them is the lines'
/// and the point morphs' part (`CAR_BLOCK`).
#[cfg_attr(test, repr(C))]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct CarEffects {
    /// Whether the streaks are drawn (the first car's is the one `effects::run_ribbons`
    /// asks).
    pub streaks_on: PlainDword,
    /// The car's model (`effects::cars_particles` refuses 34 or more); the light record's
    /// number.
    pub model: PlainDword,
    /// The engine speed (a float), the strength of the kind 13 particles.
    pub engine_speed: PlainFloat,
    /// The four axles' whole numbers (the light record's `axles`).
    pub axles: [PlainDword; 4],
    /// The car's tag (the light colour; 24 bits of it go on the ring entries).
    pub tag: PlainDword,
    /// Not read.
    pub unread_20: PlainDword,
    /// The car block (handle) whose whole matrix is used; 0 for none.
    pub whole_block: PlainDword,
    /// The car block (handle) whose part matrices are used; 0 for none.
    pub parts_block: PlainDword,
    /// The gear notice (copied from the sound object).
    pub gear_notice: PlainDword,
    /// The countdown after a reset (the emitters wait it out; 8 at the start).
    pub reset_wait: PlainDword,
    /// Not read.
    pub unread_34: [PlainDword; 3],
    /// The car's place (the whole matrix's translation, as of the last frame).
    pub last_place: [PlainFloat; 3],
    /// The car's motion (the mean of the four steps).
    pub motion: [PlainFloat; 3],
    /// The last four steps (this frame's first), three floats each.
    pub steps: [[PlainFloat; 3]; 4],
    /// The car's pace (a whole number after `cars_particles`).
    pub pace: PlainDword,
    /// Not read.
    pub unread_8c: PlainDword,
    /// The rate the emitters' gates add (after `cars_particles`); the body's size.
    pub rate: PlainDword,
    /// The car's inverse matrix (`adjugate_matrix` of the whole one: the turn first, then the
    /// translation).
    pub matrix: [PlainFloat; 15],
    /// Not read.
    pub unread_d0: PlainDword,
    /// The four wheels' records, then the body's.
    pub records: [MarkRecord; RECORDS],
    /// The object the lines and morphs work on.
    pub object: PlainDword,
    /// The second object (the morphs' other end).
    pub second_object: PlainDword,
    /// The morph's start (zeroed by `drive::share_car_state`).
    pub morph_start: PlainDword,
    /// The morph's kept figure (`effects::morph_object_points`).
    pub morph_kept: PlainDword,
    /// Three floats `drive::share_car_state` zeroes.
    pub morph_floats: [PlainFloat; 3],
    /// The face sums of the kinds 1 to 6 (`effects::lines`); the car block's wake counts.
    pub face_sums: [PlainDword; 6],
    /// The shares made from them.
    pub face_shares: [PlainDword; 6],
    /// The list (16 bytes: a `MemoryList`) the block's point table is counted on
    /// (`effects::lines::BLOCK_POINTS`).
    pub points_list: [PlainDword; 4],
    /// Not read.
    pub unread_298: [PlainDword; 12],
    /// The number of the table taken for the block's points (1 + the car's).
    pub point_block: PlainDword,
    /// The eight factors of the morph (the car block's octant marks).
    pub factors: [PlainFloat; 8],
    /// The factors kept by the morph.
    pub kept_factors: [PlainFloat; 8],
    /// The eight slots' kinds (the low byte).
    pub slot_kinds: [PlainDword; 8],
    /// The slots' line sums.
    pub slot_sums: [PlainDword; 8],
    /// The slots' shares.
    pub slot_shares: [PlainDword; 8],
    /// The 0x11 kinds (the low byte).
    pub kinds: [PlainDword; 0x11],
    /// Not read.
    pub unread_3b0: [PlainDword; 0x11],
    /// The kinds' line sums.
    pub kind_sums: [PlainDword; 0x11],
    /// The kinds' shares.
    pub kind_shares: [PlainDword; 0x11],
    /// The kinds' point counts.
    pub kind_counts: [PlainDword; 0x11],
    /// The two levels `drive::share_car_state` keeps.
    pub levels: [PlainDword; 2],
}

/// A wheel's or the body's record inside a car's (0x48 bytes): what the emitters keep per wheel.
/// Names are readings.
#[cfg_attr(test, repr(C))]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct MarkRecord {
    /// A flag the body's emitter flips each time (bit 0 the side).
    pub flip: PlainDword,
    /// The pace counter the emitters' rate gates keep (an integer count, or a float remainder
    /// after `emit_rolling_dust`).
    pub pace: PlainDword,
    /// Read and written by no code.
    pub reserved_8: PlainDword,
    /// The driven wheels' speed (the block's `drive_speed_a` / `_b` for the front / rear pair
    /// of wheel records; `emit_rolling_dust`, the bursts, kind 6, kind 21); the body's is
    /// cleared.
    pub driven_speed: PlainFloat,
    /// The car's local velocity x, sideways (the body's: `velocity[0]`), copied by
    /// `drive::share_car_state`; the emitters' gates test it for 0.
    pub local_velocity_x: PlainFloat,
    /// The local velocity y, always written 0 by `drive::share_car_state`; read by nothing.
    pub local_velocity_y: PlainDword,
    /// The car's local velocity z, forward (the body's: `velocity[2]`), copied by
    /// `drive::share_car_state`; the emitters' gates test it for 0.
    pub local_velocity_z: PlainFloat,
    /// The surface kind the emitters go by (copied by `drive::share_car_state`).
    pub kind: PlainDword,
    /// The surface flags (copied by `drive::share_car_state`); bits 6 and up count entries.
    pub flags: PlainDword,
    /// The second flags word (copied by `drive::share_car_state`); bits 6 and up count entries.
    pub flags2: PlainDword,
    /// The kind of the surface under the wheel.
    pub under_kind: PlainDword,
    pub under_flags: PlainDword,
    /// The second flags of the surface under the wheel.
    pub under_flags2: PlainDword,
    /// The chance (a float) the surface under the wheel becomes the current one.
    pub chance: PlainFloat,
    /// The body's size figure (`emit_body_contact_particles`).
    pub size: PlainFloat,
    /// The wheel's place (three floats, turned by `emit_car_particles`).
    pub place: [PlainFloat; 3],
}

/// Where the car's records begin.
#[cfg(test)]
const RECORDS_AT: usize = 0xD4;
/// The records a car has: four wheels, then the body.
pub const RECORDS: usize = 5;

#[cfg(test)]
const _: () = assert!(std::mem::size_of::<MarkRecord>() == 0x48);
#[cfg(test)]
const _: () = assert!(std::mem::size_of::<CarEffects>() == CAR_LEN);
#[cfg(test)]
const _: () = assert!(std::mem::offset_of!(CarEffects, records) == RECORDS_AT);

/// The floats' cells.
const fn plain_floats<const N: usize>(values: [f32; N]) -> [PlainFloat; N] {
    let mut out: [PlainFloat; N] = [const { PlainFloat::new(0.0) }; N];
    let mut k = 0;
    while k < N {
        out[k] = PlainFloat::new(values[k]);
        k += 1;
    }
    out
}

impl MarkRecord {
    /// The surface under the wheel becomes the current one (`under_kind`, `under_flags`,
    /// `under_flags2` into `kind`, `flags`, `flags2`).
    pub fn take_surface(&self) {
        self.kind.set(self.under_kind.get());
        self.flags.set(self.under_flags.get());
        self.flags2.set(self.under_flags2.get());
    }
}

/// A car record's length.
pub const CAR_LEN: usize = 0x4C8;
/// The cars the record has room for.
pub const CARS: usize = 8;

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct EffectsSettings {
    /// A 2-byte tab text that `effects::init_mark_step` takes a pointer to in place (probably
    /// the settings' text path start).
    pub settings_text: PlainDword,
    /// Set by `drive::finish_timing` when the race ends, cleared twice by
    /// `drive::finish_race`; also read by `effects::run_particles`.
    pub race_done: PlainDword,
    /// The view's marks layout (`race::frame_views`).
    pub marks_layout: PlainDword,
    /// This view's marks show (`race::frame_views`).
    pub own_marks_shown: PlainDword,
    /// The other view's marks show; `drive::race_init` sets it (`race::frame_views`).
    pub other_marks_shown: PlainDword,
    /// [0] read and written by no code; [1] the marks were just set up:
    /// `effects::init_marks` sets it to 1, `effects::frame_marks` copies it into the clock's
    /// `settings_copies[0]` and clears it after the frame's view step.
    pub fresh_marks_flags: [PlainDword; 2],
    /// ..: the view's place negated, three whole numbers (`drive::set_listener`).
    pub listener_place: [PlainDword; 3],
    /// ..: the view's turn matrix, nine floats (`drive::set_listener`).
    pub listener_turn: [PlainFloat; 9],
    /// Copied after the loading picture (`race::single_rally`).
    pub after_loading_copy: PlainDword,
    /// The viewed car's `marks_count` is 0x10 or more; starts as 1 (`drive::set_listener`).
    pub listener_near: PlainDword,
    /// [0] read and written by no code; [1] copied into the snow light's `kept` word
    /// by `effects::run_snow` (never written: it keeps its first value).
    pub snow_light_words: [PlainDword; 2],
    /// A copy of the race's cars (`effects::init_mark_step`).
    pub cars_copy: PlainDword,
    /// The cars' records.
    pub cars: [CarEffects; CARS],
    /// The viewed car's `engine_speed` (float), copied by `drive::set_listener`.
    pub engine_speed_copy: PlainFloat,
    /// The viewed car's `engine_speed_scaled` (float), copied by `drive::set_listener`.
    pub engine_speed_scaled_copy: PlainFloat,
    /// The work car's file number (`effects::init_mark_step`). The effects' marks kind
    /// (`effects::init_marks` refuses 0x22 or more).
    pub mark_car_file_number: PlainDword,
    /// ..: the viewed car's `position` (three whole numbers) as floats
    /// (`drive::set_listener`).
    pub listener_car_place: [PlainFloat; 3],
    /// The count of the settings' objects (1 in the mark set-up, `effects::init_mark_step`; the
    /// effects' splines and shapes read that many objects from here on, through the fields after it
    /// for a count past 1).
    pub object_count: PlainInt,
    /// The settings' first object (0x50 in the mark set-up, `effects::init_mark_step`).
    pub first_object: PlainDword,
    /// Unused; it only keeps the image's layout.
    pub reserved_20: [PlainDword; 7],
    /// The car's visual steering angle / 2^29 plus the listener camera's value
    /// (`drive::set_listener`). The effects: the splines' size (`effects::place_spline`).
    pub listener_scale: PlainFloat,
    /// Unused; it only keeps the image's layout.
    pub reserved_40: [PlainDword; 7],
    /// The car's gear-changed flag kept when set (the car's copy is cleared), 0 while the
    /// replay still frame is shown (`drive::set_listener`). The effects: a spline switch cleared
    /// when used (`effects::run_splines`).
    pub spline_gear_changed: PlainDword,
    /// 1 while the car's handbrake is on, 0 while the replay still frame is shown
    /// (`drive::set_listener`). The effects: a spline switch (`effects::run_splines`).
    pub spline_handbrake: PlainDword,
    /// 1 after the race sound objects are shared, else 0 (`drive::share_all_cars`). The
    /// effects: the mask switch (`effects::swap_mask`).
    pub sound_objects_shown: PlainDword,
    /// ..: the first light point turned, x negated, as whole numbers
    /// (`drive::set_listener`). Written with the next three groups as 12 dwords.
    pub light_point_a_negated: [PlainDword; 3],
    /// ..: the first light point turned (`drive::set_listener`).
    pub light_point_a: [PlainDword; 3],
    /// ..: the second light point turned, x negated (`drive::set_listener`).
    pub light_point_b_negated: [PlainDword; 3],
    /// ..: the second light point turned (`drive::set_listener`).
    pub light_point_b: [PlainDword; 3],
    /// Surface noise a << 2 while the mirror is wanted, else 0 (`drive::set_listener`);
    /// the windscreen drops' rate factor (`effects::windscreen_drops_top`).
    pub windscreen_drops_rate_factor: PlainDword,
    /// Twice the car's surface noise a (`drive::set_listener`). The effects: the
    /// windscreen drops wanted (`effects::windscreen_drops_frame`).
    pub windscreen_drops_wanted: PlainDword,
    /// Surface noise b >> 3 while the mirror is wanted, else 0 (`drive::set_listener`);
    /// the windscreen flakes' rate factor (`effects::windscreen_flakes_frame`,
    /// `effects::draw_plane`).
    pub windscreen_flakes_rate_factor: PlainDword,
    /// Always 1 (`drive::set_listener`). The effects: the windscreen drops and flakes on
    /// (`effects::windscreen_drops_top`, `effects::windscreen_flakes_frame`).
    pub windscreen_wet: PlainDword,
    /// Always 0 (`drive::set_listener`); the mask number `effects::run_shapes` picks with.
    pub mask_number: PlainDword,
    /// 0 while a surface noise sounds and the mirror is wanted, else 1
    /// (`drive::set_listener`); `effects::run_shapes` tests it for 1.
    pub mask_rate_switch: PlainDword,
    /// An unnamed setting; the effects: a number `effects::scale_setting` scales.
    pub setting_to_scale: PlainDword,
    /// ..: the listener's first figures (`drive::set_listener`). The effects: the lens
    /// flare's strength, then its place on the screen (`effects::draw_lens_flare`).
    pub listener_figures: [PlainDword; 3],
    /// The car's surface noise b times 8 (`drive::set_listener`), the snow flakes' count.
    /// The effects: the snow's flakes (`effects::run_snow`).
    pub snow_flakes: PlainDword,
    /// Twice the car's surface noise a (`drive::set_listener`), the rain drops' count. The
    /// effects: the rain's drops (`effects::run_rain`).
    pub rain_drops: PlainDword,
}

impl EffectsSettings {
    /// The marks' settings as the game starts: the listener's figures, the cars' streak switches
    /// and the windscreen defaults.
    pub const fn new() -> Self {
        EffectsSettings {
            settings_text: PlainDword::new(9),
            listener_turn: plain_floats([1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]),
            listener_near: PlainDword::new(1),
            snow_light_words: [const { PlainDword::new(1) }; 2],
            cars_copy: PlainDword::new(1),
            cars: [const {
                CarEffects {
                    streaks_on: PlainDword::new(1),
                    ..ZeroCells::ZERO
                }
            }; 8],
            light_point_a_negated: plain_dwords([0xFFFF0000, 0x8000, 0x20000]),
            light_point_a: plain_dwords([0x10000, 0x8000, 0x20000]),
            light_point_b_negated: plain_dwords([0x10000, 0xFFFF8000, 0x20000]),
            light_point_b: plain_dwords([0xFFFF0000, 0xFFFF8000, 0x20000]),
            windscreen_drops_rate_factor: PlainDword::new(0x3C),
            windscreen_drops_wanted: PlainDword::new(0x64),
            windscreen_wet: PlainDword::new(1),
            mask_number: PlainDword::new(4),
            setting_to_scale: PlainDword::new(1),
            listener_figures: [const { PlainDword::new(1) }; 3],
            ..ZeroCells::ZERO
        }
    }
}

/// The record's place among the stores that follow the viewer's panel.
#[cfg(test)]
pub struct InGroup;

#[cfg(test)]
impl core::ops::Deref for InGroup {
    type Target = EffectsSettings;
    fn deref(&self) -> &EffectsSettings {
        &race().replay.viewer_and_kinds.marks_settings
    }
}

#[cfg(test)]
pub static EFFECTS_SETTINGS: InGroup = InGroup;

/// The part of each car record the lines, the point morphs and the tidy-up work on: its offset in
/// the car's record, which starts after the settings' leading fields.
pub const CAR_BLOCK: u32 = 0x29C - 0x60;

/// A car record by handle (`effects::emitters`' `car`, a ring entry's car): `CAR_TAG | n` for car
/// `n`; 0 is none.
pub const CAR_TAG: u32 = 0xCC00_0000;

/// Car `n`'s handle.
pub const fn car_handle(n: usize) -> u32 {
    assert!(n < CARS);
    CAR_TAG | n as u32
}

impl EffectsSettings {
    /// The car a handle names. A value that names no car ends the program.
    pub fn car(&self, handle: u32) -> &CarEffects {
        let n = (handle ^ CAR_TAG) as usize;
        match self.cars.get(n) {
            Some(car) if handle & !0xFF == CAR_TAG => car,
            _ => crate::memory::host_hooks::end(
                "race::effects_settings_store::car",
                crate::errors::BadInput("a car record's handle that names none"),
            ),
        }
    }

    /// The car a handle names, none for 0 (see `car`).
    pub fn car_or_none(&self, handle: u32) -> Option<&CarEffects> {
        (handle != 0).then(|| self.car(handle))
    }
}

impl CarEffects {
    /// A record of zeros (for the tests' stand-in cars).
    #[cfg(test)]
    pub fn zeroed() -> Self {
        // SAFETY: the record is plain atomics, all zero a valid value.
        unsafe { std::mem::zeroed() }
    }

    /// The car's whole matrix (the block's `whole_matrix`; the record holds a block handle, set by
    /// `drive::share_car_state`); refused when no block is set.
    pub fn whole<'a>(&self, blocks: &'a CarParts) -> Result<&'a PlainFloats<16>, BadInput> {
        blocks
            .of(self.whole_block.get())
            .map(|b| &b.whole_matrix)
            .ok_or(BadInput("a car record without its block"))
    }

    /// The car's four part matrices (the block's `part_matrices`); refused when no block is set.
    pub fn parts<'a>(&self, blocks: &'a CarParts) -> Result<&'a PlainFloats<64>, BadInput> {
        blocks
            .of(self.parts_block.get())
            .map(|b| &b.part_matrices)
            .ok_or(BadInput("a car record without its block"))
    }

    /// Whether the whole matrix's block is set.
    pub fn has_whole(&self, blocks: &CarBlocks) -> bool {
        blocks.of(self.whole_block.get()).is_some()
    }

    /// Whether the part matrices' block is set.
    pub fn has_parts(&self, blocks: &CarBlocks) -> bool {
        blocks.of(self.parts_block.get()).is_some()
    }

    /// Names block `handle` for `whole` and `parts`.
    pub fn set_block(&self, handle: u32) {
        self.whole_block.set(handle);
        self.parts_block.set(handle);
    }
}

impl EffectsSettings {
    /// The settings' object number `k`: the list starts at `first_object` and, for a count past 1,
    /// runs on through the fields after it, as far as 19 objects (`sound_objects_shown`); further
    /// is refused.
    pub fn object(&self, k: usize) -> Result<u32, crate::errors::BadInput> {
        match k {
            0 => Ok(self.first_object.get()),
            1..=7 => Ok(self.reserved_20[k - 1].get()),
            8 => Ok(self.listener_scale.bits()),
            9..=15 => Ok(self.reserved_40[k - 9].get()),
            16 => Ok(self.spline_gear_changed.get()),
            17 => Ok(self.spline_handbrake.get()),
            18 => Ok(self.sound_objects_shown.get()),
            _ => Err(crate::errors::BadInput("a settings object past the 19th")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn the_object_list_runs_through_the_fields_after_it() {
        let first = offset_of!(EffectsSettings, first_object);
        assert_eq!(offset_of!(EffectsSettings, reserved_20), first + 4);
        assert_eq!(offset_of!(EffectsSettings, listener_scale), first + 4 * 8);
        assert_eq!(offset_of!(EffectsSettings, reserved_40), first + 4 * 9);
        assert_eq!(
            offset_of!(EffectsSettings, spline_gear_changed),
            first + 4 * 16
        );
        assert_eq!(
            offset_of!(EffectsSettings, spline_handbrake),
            first + 4 * 17
        );
        assert_eq!(
            offset_of!(EffectsSettings, sound_objects_shown),
            first + 4 * 18
        );
        assert!(EFFECTS_SETTINGS.object(18).is_ok());
        assert!(EFFECTS_SETTINGS.object(19).is_err());
    }
}
