//! The marks' settings (IDA 0x5C2F84 .. 0x5C56EC): the record the race hands the effects
//! (`effects::init_marks`' first argument, kept in `g_marksSettings`, `effects::setup::SETTINGS`)
//! and the effects read every frame: a head the race's views and `drive::set_listener` write,
//! the eight cars' records (`effects::init_mark_step` copies them), and a tail of the
//! listener's figures and switches. One record in the original, between the replay viewer's
//! panel and the kind tables (`race::groups_store::VIEWER_AND_KINDS`); typed here as one
//! (2026-11-02) after the stores had split it in three. Names are readings unless IDA's.

use crate::session::{Dword, Float};

/// A car's record in the settings (0x4C8 bytes; `effects::blocks`, `effects::cars_particles`,
/// `effects::morph_object_points`, `effects::tidy_marks` read its words by offset).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct MarkCar {
    pub words: [Dword; CAR_LEN / 4],
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

/// The record: a field of `race::groups_store::VIEWER_AND_KINDS`.
pub struct InGroup;

impl core::ops::Deref for InGroup {
    type Target = MarksSettings;
    fn deref(&self) -> &MarksSettings {
        &crate::race::groups_store::VIEWER_AND_KINDS.marks_settings
    }
}

pub static MARKS_SETTINGS: InGroup = InGroup;

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
