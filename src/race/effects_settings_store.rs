//! The marks' settings (IDA 0x5C2F84 .. 0x5C56EC): the record the race hands the effects
//! (`effects::init_marks`, now flagged in `g_marksSettings`, `SETTINGS`)
//! and the effects read every frame: a head the race's views and `drive::set_listener` write,
//! the eight cars' records (`effects::init_mark_step` copies them), and a tail of the
//! listener's figures and switches. One record in the original, between the replay viewer's
//! panel and the kind tables (`VIEWER_AND_KINDS`); typed here as one
//! (2026-11-02) after the stores had split it in three. Names are readings unless IDA's.

use crate::cells::ZeroCells;
use crate::errors::BadInput;
use crate::race::blocks_store::CarBlocks;
use crate::race::first_cells::plain_dwords;
#[cfg(test)]
use crate::race::owner::race;
use crate::session::{PlainDword, PlainFloat, PlainInt};

/// A car's record in the settings (0x4C8 bytes; `effects::blocks`, `effects::cars_particles`,
/// `effects::morph_object_points`, `effects::tidy_marks` work on it). Names are readings from
/// what writes the record (`drive::share_car_state`, from the car block) and what reads it.
/// The five wheel and body records (0x48 bytes each) are `MarkRecord`s; the tail from +0x23C
/// is the lines' and the point morphs' part (`CAR_BLOCK`).
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct CarEffects {
    /// +0x0: whether the streaks are drawn (the first car's is the one `effects::run_ribbons` asks).
    pub streaks_on: PlainDword,
    /// +0x4: the car's model (`effects::cars_particles` refuses 34 or more); the light
    /// record's number.
    pub model: PlainDword,
    /// +0x8: the engine speed (a float), the strength of the kind 13 particles.
    pub engine_speed: PlainFloat,
    /// +0xc: the four axles' whole numbers (the light record's `axles`).
    pub axles: [PlainDword; 4],
    /// +0x1c: the car's tag (the light colour; 24 bits of it go on the ring entries).
    pub tag: PlainDword,
    /// +0x20: not read.
    pub unread_20: PlainDword,
    /// +0x24: the car block (handle) whose whole matrix is used; 0 for none.
    pub whole_block: PlainDword,
    /// +0x28: the car block (handle) whose part matrices are used; 0 for none.
    pub parts_block: PlainDword,
    /// +0x2c: the gear notice (copied from the sound object).
    pub gear_notice: PlainDword,
    /// +0x30: the countdown after a reset (the emitters wait it out; 8 at the start).
    pub reset_wait: PlainDword,
    /// +0x34: not read.
    pub unread_34: [PlainDword; 3],
    /// +0x40: the car's place (the whole matrix's translation, as of the last frame).
    pub last_place: [PlainFloat; 3],
    /// +0x4c: the car's motion (the mean of the four steps).
    pub motion: [PlainFloat; 3],
    /// +0x58: the last four steps (this frame's first), three floats each.
    pub steps: [[PlainFloat; 3]; 4],
    /// +0x88: the car's pace (a whole number after `cars_particles`).
    pub pace: PlainDword,
    /// +0x8c: not read.
    pub unread_8c: PlainDword,
    /// +0x90: the rate the emitters' gates add (after `cars_particles`); the body's size.
    pub rate: PlainDword,
    /// +0x94: the car's inverse matrix (`adjugate_matrix` of the whole one: the turn at
    /// +0x94 .. +0xb4, the translation at +0xc4 .. +0xcc).
    pub matrix: [PlainFloat; 15],
    /// +0xd0: not read.
    pub unread_d0: PlainDword,
    /// +0xd4: the four wheels' records, then the body's (+0x1f4).
    pub records: [MarkRecord; RECORDS],
    /// +0x23c: the object the lines and morphs work on.
    pub object: PlainDword,
    /// +0x240: the second object (the morphs' other end).
    pub second_object: PlainDword,
    /// +0x244: the morph's start (zeroed by `drive::share_car_state`).
    pub morph_start: PlainDword,
    /// +0x248: the morph's kept figure (`effects::morph_object_points`).
    pub morph_kept: PlainDword,
    /// +0x24c: three floats `drive::share_car_state` zeroes.
    pub morph_floats: [PlainFloat; 3],
    /// +0x258: the face sums of the kinds 1 to 6 (`effects::lines`); the car block's wake counts.
    pub face_sums: [PlainDword; 6],
    /// +0x270: the shares made from them.
    pub face_shares: [PlainDword; 6],
    /// +0x288: the list (16 bytes: a `MemoryList`) the block's point table is counted on
    /// (`effects::lines::BLOCK_POINTS`).
    pub points_list: crate::memory::ListCells,
    /// +0x298: not read.
    pub unread_298: [PlainDword; 12],
    /// +0x2c8: the number of the table taken for the block's points (1 + the car's).
    pub point_block: PlainDword,
    /// +0x2cc: the eight factors of the morph (`g_marksBlockStart`; the car block's octant marks).
    pub factors: [PlainFloat; 8],
    /// +0x2ec: the factors kept by the morph.
    pub kept_factors: [PlainFloat; 8],
    /// +0x30c: the eight slots' kinds (the low byte).
    pub slot_kinds: [PlainDword; 8],
    /// +0x32c: the slots' line sums.
    pub slot_sums: [PlainDword; 8],
    /// +0x34c: the slots' shares.
    pub slot_shares: [PlainDword; 8],
    /// +0x36c: the 0x11 kinds (the low byte).
    pub kinds: [PlainDword; 0x11],
    /// +0x3b0: not read.
    pub unread_3b0: [PlainDword; 0x11],
    /// +0x3f4: the kinds' line sums.
    pub kind_sums: [PlainDword; 0x11],
    /// +0x438: the kinds' shares.
    pub kind_shares: [PlainDword; 0x11],
    /// +0x47c: the kinds' point counts.
    pub kind_counts: [PlainDword; 0x11],
    /// +0x4c0: the two levels `drive::share_car_state` keeps.
    pub levels: [PlainDword; 2],
}

/// A wheel's or the body's record inside a car's (0x48 bytes): what the emitters keep per
/// wheel. Names are readings.
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct MarkRecord {
    /// +0x0: a flag the body's emitter flips each time (bit 0 the side).
    pub flip: PlainDword,
    /// +0x4: the pace counter the emitters' rate gates keep (an integer count, or a float
    /// remainder after `emit_rolling_particles`).
    pub pace: PlainDword,
    /// +0x8: read and written by no code.
    pub unused_8: PlainDword,
    /// +0xc: the driven wheels' speed (the block's `drive_speed_a` / `_b` for the front / rear pair of wheel records; `emit_rolling_particles`, the bursts, kind 6, kind 21); the body's is cleared.
    pub driven_speed: PlainFloat,
    /// +0x10: the car's local velocity x, sideways (the body's: `velocity[0]`), copied by `drive::share_car_state`; the emitters' gates test it for 0.
    pub local_velocity_x: PlainFloat,
    /// +0x14: the local velocity y, always written 0 by `drive::share_car_state`; read by nothing.
    pub local_velocity_y: PlainDword,
    /// +0x18: the car's local velocity z, forward (the body's: `velocity[2]`), copied by `drive::share_car_state`; the emitters' gates test it for 0.
    pub local_velocity_z: PlainFloat,
    /// +0x1c: the surface kind the emitters go by (copied from +0x28).
    pub kind: PlainDword,
    /// +0x20: the surface flags (copied from +0x2c); bits 6 and up count entries.
    pub flags: PlainDword,
    /// +0x24: the second flags word (copied from +0x30); bits 6 and up count entries.
    pub flags2: PlainDword,
    /// +0x28: the kind of the surface under the wheel.
    pub under_kind: PlainDword,
    pub under_flags: PlainDword,
    /// +0x30: the second flags of the surface under the wheel.
    pub under_flags2: PlainDword,
    /// +0x34: the chance (a float) the surface under the wheel becomes the current one.
    pub chance: PlainFloat,
    /// +0x38: the body's size figure (`emit_body_contact_particles`).
    pub size: PlainFloat,
    /// +0x3c: the wheel's place (three floats, turned by `emit_car_particles`).
    pub place: [PlainFloat; 3],
}

/// A wheel's or body's record length.
pub const RECORD_LEN: usize = 0x48;
/// Where the car's records begin.
pub const RECORDS_AT: usize = 0xD4;
/// The records a car has: four wheels, then the body.
pub const RECORDS: usize = 5;

const _: () = assert!(std::mem::size_of::<MarkRecord>() == RECORD_LEN);
const _: () = assert!(std::mem::size_of::<CarEffects>() == CAR_LEN);
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
    /// The surface under the wheel becomes the current one (+0x28 .. +0x30 to +0x1C .. +0x24).
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

#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct EffectsSettings {
    /// +0x0 (0x5C2F84): `asc_5C2F84`: a 2-byte tab text, a pointer taken in place by `effects::init_mark_step` (probably the settings' text path start).
    pub settings_text: PlainDword,
    /// +0x4 (0x5C2F88): `dword_5C2F88`: set by `drive::finish_timing` when the race ends, cleared twice by `drive::finish_race`; also read by `effects::run_particles`.
    pub race_done: PlainDword,
    /// +0x8 (0x5C2F8C): `dword_5C2F8C`: the view's marks layout (`race::frame_views`).
    pub marks_layout: PlainDword,
    /// +0xc (0x5C2F90): `dword_5C2F90`: this view's marks show (`race::frame_views`).
    pub own_marks_shown: PlainDword,
    /// +0x10 (0x5C2F94): `dword_5C2F94`: the other view's marks show; `drive::race_init` sets it (`race::frame_views`).
    pub other_marks_shown: PlainDword,
    /// +0x14 (0x5C2F98): [0] read and written by no code; [1] (+0x18, 0x5C2F9C) the marks were just set up: `effects::init_marks` sets it to 1, `effects::frame_marks` copies it into the clock's `settings_copies[0]` and clears it after the frame's view step.
    pub fresh_marks_flags: [PlainDword; 2],
    /// +0x1c (0x5C2FA0): `dword_5C2FA0` ..: the view's place negated, three whole numbers (`drive::set_listener`).
    pub listener_place: [PlainDword; 3],
    /// +0x28 (0x5C2FAC): `dword_5C2FAC` ..: the view's turn matrix, nine floats, popped from the x87 (image value identity) (`drive::set_listener`).
    pub listener_turn: [PlainFloat; 9],
    /// +0x4c (0x5C2FD0): `dword_5C2FD0`: `dword_7558DC` copied after the loading picture (`race::single_rally`).
    pub after_loading_copy: PlainDword,
    /// +0x50 (0x5C2FD4): `dword_5C2FD4`: the viewed car's +0xABC >= 0x10; image value 1 (`drive::set_listener`).
    pub listener_near: PlainDword,
    /// +0x54 (0x5C2FD8): [0] read and written by no code; [1] (+0x58, 0x5C2FDC) copied into the snow light's `kept` word by `effects::run_snow` (never written by ported code: the image's value).
    pub snow_light_words: [PlainDword; 2],
    /// +0x5c (0x5C2FE0): `dword_5C2FE0`: a copy of the race's cars (`effects::init_mark_step`).
    pub cars_copy: PlainDword,
    /// +0x60 (0x5C2FE4): the cars' records.
    pub cars: [CarEffects; CARS],
    /// +0x26a0 (0x5C5624): `flt_5C5624`: the viewed car's `engine_speed` (float), copied by `drive::set_listener`.
    pub engine_speed_copy: PlainFloat,
    /// +0x26a4 (0x5C5628): `flt_5C5628`: the viewed car's `engine_speed_scaled` (float), copied by `drive::set_listener`.
    pub engine_speed_scaled_copy: PlainFloat,
    /// +0x26a8 (0x5C562C): `dword_5C562C`: the work car's file number (`effects::init_mark_step`). The effects' marks kind (`effects::init_marks` refuses 0x22 or more).
    pub mark_car_file_number: PlainDword,
    /// +0x26ac (0x5C5630): `flt_5C5630` ..: the viewed car's three whole numbers +0x134 .. +0x13C as floats (`drive::set_listener`).
    pub listener_car_place: [PlainFloat; 3],
    /// +0x26b8 (0x5C563C): `dword_5C563C`: the count of the settings' objects (1 in the mark set-up, `effects::init_mark_step`; the effects' splines and shapes read that many from +0x26BC, on through the fields after it for a count past 1).
    pub object_count: PlainInt,
    /// +0x26bc (0x5C5640): `dword_5C5640`: the settings' first object (0x50 in the mark set-up, `effects::init_mark_step`).
    pub first_object: PlainDword,
    /// +0x26c0 (0x5C5644): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_20: [PlainDword; 7],
    /// +0x26dc (0x5C5660): `flt_5C5660`: the car's visual steering angle / 2^29 plus the listener camera's value (`drive::set_listener`). The effects: the splines' size (`effects::place_spline`).
    pub listener_scale: PlainFloat,
    /// +0x26e0 (0x5C5664): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_40: [PlainDword; 7],
    /// +0x26fc (0x5C5680): `dword_5C5680`: the car's `gear_changed_2` kept when set (the car's copy is cleared), 0 while the replay still frame is shown (`drive::set_listener`). The effects: a spline switch cleared when used (`effects::run_splines`).
    pub spline_gear_changed: PlainDword,
    /// +0x2700 (0x5C5684): `dword_5C5684`: 1 while the car's handbrake is on, 0 while the replay still frame is shown (`drive::set_listener`). The effects: a spline switch (`effects::run_splines`).
    pub spline_handbrake: PlainDword,
    /// +0x2704 (0x5C5688): `dword_5C5688`: 1 after the race sound objects are shared, else 0 (`drive::share_all_cars`). The effects: the mask switch (`effects::swap_mask`).
    pub sound_objects_shown: PlainDword,
    /// +0x2708 (0x5C568C): `dword_5C568C` ..: the first light point turned, x negated, as whole numbers (`drive::set_listener`). Written with the next three groups as 12 dwords.
    pub light_point_a_negated: [PlainDword; 3],
    /// +0x2714 (0x5C5698): `dword_5C5698` ..: the first light point turned (`drive::set_listener`).
    pub light_point_a: [PlainDword; 3],
    /// +0x2720 (0x5C56A4): `dword_5C56A4` ..: the second light point turned, x negated (`drive::set_listener`).
    pub light_point_b_negated: [PlainDword; 3],
    /// +0x272c (0x5C56B0): `dword_5C56B0` ..: the second light point turned (`drive::set_listener`).
    pub light_point_b: [PlainDword; 3],
    /// +0x2738 (0x5C56BC): `dword_5C56BC`: surface noise a << 2 while the mirror is wanted, else 0 (`drive::set_listener`); the windscreen drops' rate factor (`effects::windscreen_drops_top`).
    pub windscreen_drops_rate_factor: PlainDword,
    /// +0x273c (0x5C56C0): `dword_5C56C0`: twice the car's surface noise a (`drive::set_listener`). The effects: the windscreen drops wanted (`effects::windscreen_drops_frame`).
    pub windscreen_drops_wanted: PlainDword,
    /// +0x2740 (0x5C56C4): `dword_5C56C4`: surface noise b >> 3 while the mirror is wanted, else 0 (`drive::set_listener`); the windscreen flakes' rate factor (`effects::windscreen_flakes_frame`, `effects::draw_plane`).
    pub windscreen_flakes_rate_factor: PlainDword,
    /// +0x2744 (0x5C56C8): `dword_5C56C8`: always 1 (`drive::set_listener`). The effects: the windscreen drops and flakes on (`effects::windscreen_drops_top`, `effects::windscreen_flakes_frame`).
    pub windscreen_wet: PlainDword,
    /// +0x2748 (0x5C56CC): `dword_5C56CC`: always 0 (`drive::set_listener`); the mask number `effects::run_shapes` picks with.
    pub mask_number: PlainDword,
    /// +0x274c (0x5C56D0): `dword_5C56D0`: 0 while a surface noise sounds and the mirror is wanted, else 1 (`drive::set_listener`); `effects::run_shapes` tests it for 1.
    pub mask_rate_switch: PlainDword,
    /// +0x2750 (0x5C56D4): an unnamed setting; the effects: a number `effects::scale_setting` scales.
    pub setting_to_scale: PlainDword,
    /// +0x2754 (0x5C56D8): `dword_5C56D8` ..: the first figure and `dword_6415D0`, `dword_6415D4` (`drive::set_listener`). The effects: the lens flare's strength, then its place on the screen (`effects::draw_lens_flare`).
    pub listener_figures: [PlainDword; 3],
    /// +0x2760 (0x5C56E4): `dword_5C56E4`: the car's surface noise b times 8 (`drive::set_listener`), the snow flakes' count. The effects: the snow's flakes (`effects::run_snow`).
    pub snow_flakes: PlainDword,
    /// +0x2764 (0x5C56E8): `dword_5C56E8`: twice the car's surface noise a (`drive::set_listener`), the rain drops' count. The effects: the rain's drops (`effects::run_rain`).
    pub rain_drops: PlainDword,
}

impl EffectsSettings {
    /// The marks' settings as the game starts: the listener's figures, the cars' streak switches and the windscreen defaults.
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

/// The record: a field of `VIEWER_AND_KINDS`.
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

/// The part of each car record the lines, the point morphs and the tidy-up work on (the
/// original's settings + 0x29C + n * 0x4C8, `effects::lines`' "settings block"): its offset
/// in the car's record, which starts at the settings' +0x60.
pub const CAR_BLOCK: u32 = 0x29C - 0x60;

/// A car record by handle, where the original kept its address (`effects::emitters`'
/// `EMITTER.car`, a ring entry's +0x68): `CAR_TAG | n` for car `n`; 0 is none.
pub const CAR_TAG: u32 = 0xCC00_0000;

/// Car `n`'s handle.
pub const fn car_handle(n: usize) -> u32 {
    assert!(n < CARS);
    CAR_TAG | n as u32
}

impl EffectsSettings {
    /// The car a handle names. A value that names no car ends the program (as an address off
    /// the records would have read elsewhere in the original).
    pub fn car(&self, handle: u32) -> &CarEffects {
        let n = (handle ^ CAR_TAG) as usize;
        match self.cars.get(n) {
            Some(car) if handle & !0xFF == CAR_TAG => car,
            _ => crate::fatal::raise::end(
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

    /// The car's whole matrix (the block's +0x8C0, the original's +0x24; the original kept
    /// the matrix's address, a block handle here, set by `drive::share_car_state`); refused
    /// when no block is set (the original reads from address 0).
    pub fn whole<'a>(&self, blocks: &'a CarBlocks) -> Result<&'a [PlainFloat; 16], BadInput> {
        blocks
            .of(self.whole_block.get())
            .map(|b| &b.whole_matrix)
            .ok_or(BadInput("a car record without its block"))
    }

    /// The car's four part matrices (the block's +0x740, the original's +0x28); refused when no
    /// block is set.
    pub fn parts<'a>(&self, blocks: &'a CarBlocks) -> Result<&'a [PlainFloat; 64], BadInput> {
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
    /// The settings' object number `k`: the list starts at `first_object` and, for a count
    /// past 1, runs on through the fields after it (the original's layout), as far as 19
    /// objects (`sound_objects_shown`); further is refused.
    pub fn object(&self, k: usize) -> Result<u32, crate::errors::BadInput> {
        match k {
            0 => Ok(self.first_object.get()),
            1..=7 => Ok(self.unused_20[k - 1].get()),
            8 => Ok(self.listener_scale.bits()),
            9..=15 => Ok(self.unused_40[k - 9].get()),
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
        assert_eq!(offset_of!(EffectsSettings, unused_20), first + 4);
        assert_eq!(offset_of!(EffectsSettings, listener_scale), first + 4 * 8);
        assert_eq!(offset_of!(EffectsSettings, unused_40), first + 4 * 9);
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
