//! The marks' settings (IDA 0x5C2F84 .. 0x5C56EC): the record the race hands the effects
//! (`effects::init_marks`, now flagged in `g_marksSettings`, `SETTINGS`)
//! and the effects read every frame: a head the race's views and `drive::set_listener` write,
//! the eight cars' records (`effects::init_mark_step` copies them), and a tail of the
//! listener's figures and switches. One record in the original, between the replay viewer's
//! panel and the kind tables (`VIEWER_AND_KINDS`); typed here as one
//! (2026-11-02) after the stores had split it in three. Names are readings unless IDA's.

use crate::race::owner::race;
use crate::session::{Dword, Float};

/// A car's record in the settings (0x4C8 bytes; `effects::blocks`, `effects::cars_particles`,
/// `effects::morph_object_points`, `effects::tidy_marks` work on it). Names are readings from
/// what writes the record (`drive::share_car_state`, from the car block) and what reads it.
/// The five wheel and body records (0x48 bytes each) are `MarkRecord`s; the tail from +0x23C
/// is the lines' and the point morphs' part (`CAR_BLOCK`).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct MarkCar {
    /// +0x0: whether the streaks are drawn (the first car's is the one `effects::run_ribbons` asks).
    pub streaks_on: Dword,
    /// +0x4: the car's model (`effects::cars_particles` refuses 34 or more); the light
    /// record's number.
    pub model: Dword,
    /// +0x8: the engine speed (a float), the strength of the kind 13 particles.
    pub engine_speed: Float,
    /// +0xc: the four axles' whole numbers (the light record's `axles`).
    pub axles: [Dword; 4],
    /// +0x1c: the car's tag (the light colour; 24 bits of it go on the ring entries).
    pub tag: Dword,
    /// +0x20: not read.
    pub unread_20: Dword,
    /// +0x24: the car block (handle) whose whole matrix is used; 0 for none.
    pub whole_block: Dword,
    /// +0x28: the car block (handle) whose part matrices are used; 0 for none.
    pub parts_block: Dword,
    /// +0x2c: the gear notice (copied from the sound object).
    pub gear_notice: Dword,
    /// +0x30: the countdown after a reset (the emitters wait it out; 8 at the start).
    pub reset_wait: Dword,
    /// +0x34: not read.
    pub unread_34: [Dword; 3],
    /// +0x40: the car's place (the whole matrix's translation, as of the last frame).
    pub last_place: [Float; 3],
    /// +0x4c: the car's motion (the mean of the four steps).
    pub motion: [Float; 3],
    /// +0x58: the last four steps (this frame's first), three floats each.
    pub steps: [[Float; 3]; 4],
    /// +0x88: the car's pace (a whole number after `cars_particles`).
    pub pace: Dword,
    /// +0x8c: not read.
    pub unread_8c: Dword,
    /// +0x90: the rate the emitters' gates add (after `cars_particles`); the body's size.
    pub rate: Dword,
    /// +0x94: the car's inverse matrix (`adjugate_matrix` of the whole one: the turn at
    /// +0x94 .. +0xb4, the translation at +0xc4 .. +0xcc).
    pub matrix: [Float; 15],
    /// +0xd0: not read.
    pub unread_d0: Dword,
    /// +0xd4: the four wheels' records, then the body's (+0x1f4).
    pub records: [MarkRecord; RECORDS],
    /// +0x23c: the object the lines and morphs work on.
    pub object: Dword,
    /// +0x240: the second object (the morphs' other end).
    pub second_object: Dword,
    /// +0x244: the morph's start (zeroed by `drive::share_car_state`).
    pub morph_start: Dword,
    /// +0x248: the morph's kept figure (`effects::morph_object_points`).
    pub morph_kept: Dword,
    /// +0x24c: three floats `drive::share_car_state` zeroes.
    pub morph_floats: [Float; 3],
    /// +0x258: the face sums of the kinds 1 to 6 (`effects::lines`); the car block's wake counts.
    pub face_sums: [Dword; 6],
    /// +0x270: the shares made from them.
    pub face_shares: [Dword; 6],
    /// +0x288: the list (16 bytes: a `MemoryList`) the block's point table is counted on
    /// (`effects::lines::BLOCK_POINTS`).
    pub points_list: [Dword; 4],
    /// +0x298: not read.
    pub unread_298: [Dword; 12],
    /// +0x2c8: the number of the table taken for the block's points (1 + the car's).
    pub point_block: Dword,
    /// +0x2cc: the eight factors of the morph (`g_marksBlockStart`; the car block's octant marks).
    pub factors: [Float; 8],
    /// +0x2ec: the factors kept by the morph.
    pub kept_factors: [Float; 8],
    /// +0x30c: the eight slots' kinds (the low byte).
    pub slot_kinds: [Dword; 8],
    /// +0x32c: the slots' line sums.
    pub slot_sums: [Dword; 8],
    /// +0x34c: the slots' shares.
    pub slot_shares: [Dword; 8],
    /// +0x36c: the 0x11 kinds (the low byte).
    pub kinds: [Dword; 0x11],
    /// +0x3b0: not read.
    pub unread_3b0: [Dword; 0x11],
    /// +0x3f4: the kinds' line sums.
    pub kind_sums: [Dword; 0x11],
    /// +0x438: the kinds' shares.
    pub kind_shares: [Dword; 0x11],
    /// +0x47c: the kinds' point counts.
    pub kind_counts: [Dword; 0x11],
    /// +0x4c0: the two levels `drive::share_car_state` keeps.
    pub levels: [Dword; 2],
}

/// A wheel's or the body's record inside a car's (0x48 bytes): what the emitters keep per
/// wheel. Names are readings.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct MarkRecord {
    /// +0x0: a flag the body's emitter flips each time (bit 0 the side).
    pub flip: Dword,
    /// +0x4: the pace counter the emitters' rate gates keep (an integer count, or a float
    /// remainder after `emit_kind_1`).
    pub pace: Dword,
    /// +0x8: not named.
    pub unknown_8: Dword,
    /// +0xc: a speed figure (`emit_kind_1`, the bursts, kind 6, kind 21); the body's is cleared.
    pub speed_c: Float,
    /// +0x10: a speed figure; the gates test it for 0.
    pub speed_10: Float,
    /// +0x14: not named.
    pub unknown_14: Dword,
    /// +0x18: a speed figure; the gates test it for 0.
    pub speed_18: Float,
    /// +0x1c: the surface kind the emitters go by (copied from +0x28).
    pub kind: Dword,
    /// +0x20: the surface flags (copied from +0x2c); bits 6 and up count entries.
    pub flags: Dword,
    /// +0x24: the second flags word (copied from +0x30); bits 6 and up count entries.
    pub flags2: Dword,
    /// +0x28: the kind of the surface under the wheel.
    pub under_kind: Dword,
    /// +0x2c: the flags of the surface under the wheel.
    pub under_flags: Dword,
    /// +0x30: the second flags of the surface under the wheel.
    pub under_flags2: Dword,
    /// +0x34: the chance (a float) the surface under the wheel becomes the current one.
    pub chance: Float,
    /// +0x38: the body's size figure (`emit_particle_18`).
    pub size: Float,
    /// +0x3c: the wheel's place (three floats, turned by `emit_car_particles`).
    pub place: [Float; 3],
}

/// A wheel's or body's record length.
pub const RECORD_LEN: usize = 0x48;
/// Where the car's records begin.
pub const RECORDS_AT: usize = 0xD4;
/// The records a car has: four wheels, then the body.
pub const RECORDS: usize = 5;

const _: () = assert!(std::mem::size_of::<MarkRecord>() == RECORD_LEN);
const _: () = assert!(std::mem::offset_of!(MarkRecord, pace) == 0x4);
const _: () = assert!(std::mem::offset_of!(MarkRecord, speed_c) == 0xC);
const _: () = assert!(std::mem::offset_of!(MarkRecord, speed_18) == 0x18);
const _: () = assert!(std::mem::offset_of!(MarkRecord, kind) == 0x1C);
const _: () = assert!(std::mem::offset_of!(MarkRecord, under_kind) == 0x28);
const _: () = assert!(std::mem::offset_of!(MarkRecord, chance) == 0x34);
const _: () = assert!(std::mem::offset_of!(MarkRecord, place) == 0x3C);
const _: () = assert!(std::mem::size_of::<MarkCar>() == CAR_LEN);
const _: () = assert!(std::mem::offset_of!(MarkCar, records) == RECORDS_AT);
const _: () = {
    use std::mem::offset_of as at;
    assert!(at!(MarkCar, model) == 0x4);
    assert!(at!(MarkCar, engine_speed) == 0x8);
    assert!(at!(MarkCar, axles) == 0xC);
    assert!(at!(MarkCar, tag) == 0x1C);
    assert!(at!(MarkCar, whole_block) == 0x24);
    assert!(at!(MarkCar, parts_block) == 0x28);
    assert!(at!(MarkCar, gear_notice) == 0x2C);
    assert!(at!(MarkCar, reset_wait) == 0x30);
    assert!(at!(MarkCar, last_place) == 0x40);
    assert!(at!(MarkCar, motion) == 0x4C);
    assert!(at!(MarkCar, steps) == 0x58);
    assert!(at!(MarkCar, pace) == 0x88);
    assert!(at!(MarkCar, rate) == 0x90);
    assert!(at!(MarkCar, matrix) == 0x94);
    assert!(at!(MarkCar, unread_d0) == 0xD0);
    assert!(at!(MarkCar, object) == 0x23C);
    assert!(at!(MarkCar, second_object) == 0x240);
    assert!(at!(MarkCar, morph_start) == 0x244);
    assert!(at!(MarkCar, morph_kept) == 0x248);
    assert!(at!(MarkCar, morph_floats) == 0x24C);
    assert!(at!(MarkCar, face_sums) == 0x258);
    assert!(at!(MarkCar, face_shares) == 0x270);
    assert!(at!(MarkCar, points_list) == 0x288);
    assert!(at!(MarkCar, unread_298) == 0x298);
    assert!(at!(MarkCar, point_block) == 0x2C8);
    assert!(at!(MarkCar, factors) == 0x2CC);
    assert!(at!(MarkCar, kept_factors) == 0x2EC);
    assert!(at!(MarkCar, slot_kinds) == 0x30C);
    assert!(at!(MarkCar, slot_sums) == 0x32C);
    assert!(at!(MarkCar, slot_shares) == 0x34C);
    assert!(at!(MarkCar, kinds) == 0x36C);
    assert!(at!(MarkCar, unread_3b0) == 0x3B0);
    assert!(at!(MarkCar, kind_sums) == 0x3F4);
    assert!(at!(MarkCar, kind_shares) == 0x438);
    assert!(at!(MarkCar, kind_counts) == 0x47C);
    assert!(at!(MarkCar, levels) == 0x4C0);
};

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
#[derive(rc2k_derive::Cells)]
pub struct MarksSettings {
    /// +0x0 (0x5C2F84): `asc_5C2F84`: a 2-byte tab text, a pointer taken in place by `effects::init_mark_step` (a reading).
    pub settings_text: Dword,
    /// +0x4 (0x5C2F88): `dword_5C2F88`: set by `drive::finish_timing`, cleared twice by `drive::finish_race` (a reading).
    pub race_done: Dword,
    /// +0x8 (0x5C2F8C): `dword_5C2F8C`: the view's marks layout (`race::frame_views`).
    pub marks_layout: Dword,
    /// +0xc (0x5C2F90): `dword_5C2F90`: this view's marks show (`race::frame_views`).
    pub own_marks_shown: Dword,
    /// +0x10 (0x5C2F94): `dword_5C2F94`: the other view's marks show; `drive::race_init` sets it (`race::frame_views`).
    pub other_marks_shown: Dword,
    /// +0x14 (0x5C2F98): not named.
    pub unknown_67c: [Dword; 2],
    /// +0x1c (0x5C2FA0): `dword_5C2FA0` ..: the view's place negated, three whole numbers (`drive::set_listener`).
    pub listener_place: [Dword; 3],
    /// +0x28 (0x5C2FAC): `dword_5C2FAC` ..: the view's turn matrix, nine floats, popped from the x87 (image value identity) (`drive::set_listener`).
    pub listener_turn: [Float; 9],
    /// +0x4c (0x5C2FD0): `dword_5C2FD0`: `dword_7558DC` copied after the loading picture (`race::single_rally`).
    pub after_loading_copy: Dword,
    /// +0x50 (0x5C2FD4): `dword_5C2FD4`: the viewed car's +0xABC >= 0x10; image value 1 (`drive::set_listener`).
    pub listener_near: Dword,
    /// +0x54 (0x5C2FD8): not named.
    pub unknown_6bc: [Dword; 2],
    /// +0x5c (0x5C2FE0): `dword_5C2FE0`: a copy of the race's cars (`effects::init_mark_step`).
    pub cars_copy: Dword,
    /// +0x60 (0x5C2FE4): the cars' records.
    pub cars: [MarkCar; CARS],
    /// +0x26a0 (0x5C5624): `flt_5C5624`: a float copied from `dword_70F5C4` (`drive::set_listener`; a reading).
    pub listener_float_a: Float,
    /// +0x26a4 (0x5C5628): `flt_5C5628`: a float copied from `dword_7102F4` (`drive::set_listener`; a reading).
    pub listener_float_b: Float,
    /// +0x26a8 (0x5C562C): `dword_5C562C`: the work car's file number (`effects::init_mark_step`). The effects' marks kind (`effects::init_marks` refuses 0x22 or more).
    pub mark_car_file_number: Dword,
    /// +0x26ac (0x5C5630): `flt_5C5630` ..: the viewed car's three whole numbers +0x134 .. +0x13C as floats (`drive::set_listener`).
    pub listener_car_place: [Float; 3],
    /// +0x26b8 (0x5C563C): `dword_5C563C`: the count of the settings' objects (1 in the mark set-up, `effects::init_mark_step`; the effects' splines and shapes read that many from +0x26BC, on through the fields after it for a count past 1).
    pub object_count: Dword,
    /// +0x26bc (0x5C5640): `dword_5C5640`: the settings' first object (0x50 in the mark set-up, `effects::init_mark_step`).
    pub first_object: Dword,
    /// +0x26c0 (0x5C5644): not named.
    pub unknown_20: [Dword; 7],
    /// +0x26dc (0x5C5660): `flt_5C5660`: `dword_70FA3C` / 2^29 + `flt_637644` (`drive::set_listener`; a reading). The effects: the splines' size (`effects::place_spline`).
    pub listener_scale: Float,
    /// +0x26e0 (0x5C5664): not named.
    pub unknown_40: [Dword; 7],
    /// +0x26fc (0x5C5680): `dword_5C5680`: the old `dword_70F628`, 0 with `dword_601024` (`drive::set_listener`; a reading). The effects: a spline switch cleared when used (`effects::run_splines`).
    pub listener_gear_changed: Dword,
    /// +0x2700 (0x5C5684): `dword_5C5684`: `dword_70FA2C` != 0, 0 with `dword_601024` (`drive::set_listener`; a reading). The effects: a spline switch (`effects::run_splines`).
    pub listener_flag: Dword,
    /// +0x2704 (0x5C5688): `dword_5C5688`: 1 after the race sound objects are shared, else 0 (`drive::share_all_cars`). The effects: the mask switch (`effects::swap_mask`).
    pub sound_objects_shown: Dword,
    /// +0x2708 (0x5C568C): `dword_5C568C` ..: the first light point turned, x negated, as whole numbers (`drive::set_listener`). Written with the next three groups as 12 dwords.
    pub listener_point_a_negated: [Dword; 3],
    /// +0x2714 (0x5C5698): `dword_5C5698` ..: the first light point turned (`drive::set_listener`).
    pub listener_point_a: [Dword; 3],
    /// +0x2720 (0x5C56A4): `dword_5C56A4` ..: the second light point turned, x negated (`drive::set_listener`).
    pub listener_point_b_negated: [Dword; 3],
    /// +0x272c (0x5C56B0): `dword_5C56B0` ..: the second light point turned (`drive::set_listener`).
    pub listener_point_b: [Dword; 3],
    /// +0x2738 (0x5C56BC): `dword_5C56BC`: a switch by `dword_6415E0`, `dword_70FCEC`, `dword_70FCF0` (`drive::set_listener`; a reading).
    pub listener_switch_0: Dword,
    /// +0x273c (0x5C56C0): `dword_5C56C0`: a switch (`drive::set_listener`; a reading). The effects: the windscreen drops wanted (`effects::windscreen_drops_frame`).
    pub listener_switch_2: Dword,
    /// +0x2740 (0x5C56C4): `dword_5C56C4`: a switch (`drive::set_listener`; a reading).
    pub listener_switch_3: Dword,
    /// +0x2744 (0x5C56C8): `dword_5C56C8`: a switch (`drive::set_listener`; a reading). The effects: the windscreen drops and flakes on (`effects::windscreen_drops_top`, `effects::windscreen_flakes_frame`).
    pub listener_switch_6: Dword,
    /// +0x2748 (0x5C56CC): `dword_5C56CC`: a switch (`drive::set_listener`; a reading).
    pub listener_switch_7: Dword,
    /// +0x274c (0x5C56D0): `dword_5C56D0`: a switch (`drive::set_listener`; a reading).
    pub listener_switch_5: Dword,
    /// +0x2750 (0x5C56D4): not named. The effects: a number `effects::scale_setting` scales.
    pub unknown_b0: Dword,
    /// +0x2754 (0x5C56D8): `dword_5C56D8` ..: the first figure and `dword_6415D0`, `dword_6415D4` (`drive::set_listener`). The effects: the lens flare's strength, then its place on the screen (`effects::draw_lens_flare`).
    pub listener_figures: [Dword; 3],
    /// +0x2760 (0x5C56E4): `dword_5C56E4`: a switch (`drive::set_listener`; a reading). The effects: the snow's flakes (`effects::run_snow`).
    pub listener_switch_4: Dword,
    /// +0x2764 (0x5C56E8): `dword_5C56E8`: a switch (`drive::set_listener`; a reading). The effects: the rain's drops (`effects::run_rain`).
    pub listener_switch_1: Dword,
}

/// IDA 0x5C2F84, the record's first byte.
pub const START: usize = 0x5C2F84;
/// IDA 0x5C56EC, its end.
pub const END: usize = 0x5C56EC;

const _: () = assert!(END - START == core::mem::size_of::<MarksSettings>());

/// The record: a field of `VIEWER_AND_KINDS`.
pub struct InGroup;

impl core::ops::Deref for InGroup {
    type Target = MarksSettings;
    fn deref(&self) -> &MarksSettings {
        &race().replay.viewer_and_kinds.marks_settings
    }
}

pub static MARKS_SETTINGS: InGroup = InGroup;

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

/// The car a handle names. A value that names no car ends the program (as an address off
/// the records would have read elsewhere in the original).
pub fn car(handle: u32) -> &'static MarkCar {
    let n = (handle ^ CAR_TAG) as usize;
    match MARKS_SETTINGS.cars.get(n) {
        Some(car) if handle & !0xFF == CAR_TAG => car,
        _ => crate::fatal::raise::end(
            "race::marks_settings_store::car",
            crate::errors::BadInput("a car record's handle that names none"),
        ),
    }
}

/// The car a handle names, none for 0 (see `car`).
pub fn car_or_none(handle: u32) -> Option<&'static MarkCar> {
    (handle != 0).then(|| car(handle))
}

impl MarkCar {
    /// A record of zeros (for the tests' stand-in cars).
    #[cfg(all(test, target_arch = "x86"))]
    pub fn zeroed() -> Self {
        // SAFETY: the record is plain atomics, all zero a valid value.
        unsafe { std::mem::zeroed() }
    }

    /// The car block a handle names (the original kept the block's whole matrix's and first
    /// part matrix's addresses; a block handle here, set by `drive::share_car_state`), none
    /// when 0.
    fn block_of(handle: &Dword) -> Option<&'static crate::physics::car_store::Car> {
        crate::race::blocks_store::car_of(handle.get())
    }

    /// The car's whole matrix (the block's +0x8C0, the original's +0x24); refused when no
    /// block is set (the original reads from address 0).
    pub fn whole(&self) -> Result<&'static [crate::session::Float; 16], crate::errors::BadInput> {
        Self::block_of(&self.whole_block)
            .map(|b| &b.whole_matrix)
            .ok_or(crate::errors::BadInput("a car record without its block"))
    }

    /// The car's four part matrices (the block's +0x740, the original's +0x28); refused when no
    /// block is set.
    pub fn parts(&self) -> Result<&'static [crate::session::Float; 64], crate::errors::BadInput> {
        Self::block_of(&self.parts_block)
            .map(|b| &b.part_matrices)
            .ok_or(crate::errors::BadInput("a car record without its block"))
    }

    /// Whether the whole matrix's block is set.
    pub fn has_whole(&self) -> bool {
        Self::block_of(&self.whole_block).is_some()
    }

    /// Whether the part matrices' block is set.
    pub fn has_parts(&self) -> bool {
        Self::block_of(&self.parts_block).is_some()
    }

    /// Names block `handle` for `whole` and `parts`.
    pub fn set_block(&self, handle: u32) {
        self.whole_block.set(handle);
        self.parts_block.set(handle);
    }
}

impl MarksSettings {
    /// The record's place `offset` bytes in, bounded by the record: for the reads the
    /// original makes by offset (a list as long as a count, a field picked by a table).
    pub fn at(&self, offset: u32) -> crate::pointers::GamePtr {
        crate::pointers::GamePtr::to(self).offset(offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn fields_sit_where_the_effects_read_them() {
        assert_eq!(offset_of!(MarksSettings, unknown_67c), 0x14);
        assert_eq!(offset_of!(MarksSettings, listener_place), 0x1C);
        assert_eq!(offset_of!(MarksSettings, listener_turn), 0x28);
        assert_eq!(offset_of!(MarksSettings, cars_copy), 0x5C);
        assert_eq!(offset_of!(MarksSettings, cars), 0x60);
        assert_eq!(offset_of!(MarksSettings, mark_car_file_number), 0x26A8);
        assert_eq!(offset_of!(MarksSettings, listener_scale), 0x26DC);
        assert_eq!(offset_of!(MarksSettings, listener_switch_0), 0x2738);
        assert_eq!(offset_of!(MarksSettings, listener_figures), 0x2754);
        assert_eq!(offset_of!(MarksSettings, listener_switch_1), 0x2764);
        assert_eq!(
            crate::pointers::game_address(core::ptr::from_ref(&*MARKS_SETTINGS)),
            crate::framework::game_addr(crate::framework::rva(START))
        );
    }
}
