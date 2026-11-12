//! The start and tables work (IDA 0x60463C .. 0x60526C; phase 7, typed): the stage time factors, the arcade
//! mode's tables by rally and stage (cars, places, car types and numbers), the cars data text read in
//! place, the noise seed, the start and pace cars' values, the car kind order, and the camera's share and
//! ground-turn variables. `#[repr(C)]` with the image's layout, over the race block's bytes. Names are readings unless IDA's.

#[cfg(test)]
use crate::race::owner::race;
use crate::session::{Dword, Float};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct RaceTables {
    /// +0x0 (0x60463C): `flt_60463C`: the factor by section for the last three record times (`menu::scale_stage_times`).
    pub stage_time_factors_last: [Float; 6],
    /// +0x18 (0x604654): `flt_604654`: the factor by section for the first record time (`menu::scale_stage_times`).
    pub stage_time_factors_first: [Float; 6],
    /// +0x30 (0x60466C): `dword_60466C`: the arcade cars count by rally (`race::set_up_mode_2_cars`).
    pub arcade_cars_by_rally: [Dword; 6],
    /// +0x48 (0x604684): `dword_604684`: the best place wanted by arcade stage (six stages); `race::set_up_mode_2_cars` copies the stage's entry to `best_place`.
    pub arcade_best_place_by_stage: [Dword; 6],
    /// +0x60 (0x60469C): `dword_60469C`: the share a car gets on the last start place by arcade rally (six rallies); `race::set_up_mode_2_cars` copies the rally's entry to `start_share_last`.
    pub start_share_last_by_rally: [Dword; 6],
    /// +0x78 (0x6046B4): `dword_6046B4`: the share on the first start place by arcade rally; copied to `start_share_first` by `race::set_up_mode_2_cars`.
    pub start_share_first_by_rally: [Dword; 6],
    /// +0x90 (0x6046CC): `dword_6046CC`: the time percentage by difficulty level (`race::race_cars` prints it); the IDA item holds 36 entries.
    pub time_percentages: [Dword; 36],
    /// +0x120 (0x60475C): `flt_60475C`: 0x17 floats by car kind (`race::race_cars`).
    pub kind_time_factors: [Float; 23],
    /// +0x17c (0x6047B8): `g_carDataText` and the two items after it (`byte_6047B9`, `byte_6047BA`): the cars data text, 0x708 bytes, read in place (`race::read_stage_car_numbers`, `parse_3_digits`, `parse_2_digits`; `race::race_cars`).
    pub car_data_text: [Dword; 450],
    /// +0x884 (0x604EC0): `flt_604EC0`: the step of the start blend (`drive::start_blend`).
    pub start_blend_step: Float,
    /// +0x888 (0x604EC4): `flt_604EC4`: scale of the push apart of the pace cars (`drive::start_cars_behind`).
    pub pace_push_scale: Float,
    /// +0x88c (0x604EC8): `g_carByPlace`'s first word, the float 30.0, never written: the speed
    /// `drive::drive_computer_car` eases the computer's wanted speed toward (`int_toward_to_int`).
    /// It is also slot 0 of the cars-by-place table that runs on into `arcade_car_kinds`
    /// (`race::panel_neighbours` reads `[place]` and `[place + 1]`): only a place of 0 would read
    /// the 30.0 as a car (refused there).
    pub computer_ease_speed: Dword,
    /// +0x890 (0x604ECC): `dword_604ECC`: each car's type 1 .. 3, 89 rooms; also the cars by start (`race::set_up_mode_2_cars`, `drive::start_cars_behind`, `race::panel_neighbours`).
    pub arcade_car_kinds: [Dword; 89],
    /// +0x9f4 (0x605030): `dword_605030`: read as the number at place -1 by `race::panel_neighbours`; `arcade_car_numbers` follows it.
    pub number_before_first: Dword,
    /// +0x9f8 (0x605034): `dword_605034`: each car's number by start (`race::set_up_mode_2_cars`, `drive::start_cars_behind`, `drive::start_car_on_line`, `race::panel_neighbours`).
    pub arcade_car_numbers: [Dword; 90],
    /// +0xb60 (0x60519C): `dword_60519C`: the noise seed (`race::set_up_mode_2_cars`, `race::race_end`, `race::credits_race`).
    pub noise_seed: Dword,
    /// +0xb64 (0x6051A0): `dword_6051A0`: the start in use (`drive::start_car_on_line`, `drive::start_cars_behind`).
    pub start_slot: Dword,
    /// +0xb68 (0x6051A4): `dword_6051A4`: the share of the last start place (`lo` of `rc2k_core::drive::start_places::start_share`, which blends it to `start_share_first` by the start's place); set by `race::set_up_mode_2_cars`.
    pub start_share_last: Float,
    /// +0xb6c (0x6051A8): `dword_6051A8`: the share of the first start place (`hi` of `start_share`); set by `race::set_up_mode_2_cars`.
    pub start_share_first: Float,
    /// +0xb70 (0x6051AC): `flt_6051AC`: the start's share (`drive::start_car_on_line`).
    pub start_share: Float,
    /// +0xb74 (0x6051B0): `dword_6051B0`: the start speed, loaded as a float (`drive::start_car_on_line`, `drive::start_cars_behind`).
    pub start_speed: Float,
    /// +0xb78 (0x6051B4): `dword_6051B4`: "NUM arcadecars", the starts count (`race::set_up_mode_2_cars`, `race::panel_neighbours`, `drive::start_cars_behind`).
    pub arcade_cars: Dword,
    /// +0xb7c (0x6051B8): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_b7c: Dword,
    /// +0xb80 (0x6051BC): `dword_6051BC`: "arc start pos (1-n)" (`race::set_up_mode_2_cars`, `race::settings_copy`, `race::race_shell`).
    pub start_place: Dword,
    /// +0xb84 (0x6051C0): `dword_6051C0`: "bestpos" (`race::set_up_mode_2_cars`).
    pub best_place: Dword,
    /// +0xb88 (0x6051C4): `dword_6051C4`: the first pace car's start (`drive::start_cars_behind`).
    pub pace_start_a: Dword,
    /// +0xb8c (0x6051C8): `dword_6051C8`: the second pace car's start (`drive::start_cars_behind`). Written with the three before it as a run.
    pub pace_start_b: Dword,
    /// +0xb90 (0x6051CC): `dword_6051CC`: the first pace car's distance (`drive::start_cars_behind`).
    pub pace_distance_a: Dword,
    /// +0xb94 (0x6051D0): `dword_6051D0`: the second pace car's distance (`drive::start_cars_behind`).
    pub pace_distance_b: Dword,
    /// +0xb98 (0x6051D4): `dword_6051D4`: the gap at which the lead has passed a pace car (`drive::start_cars_behind`).
    pub pace_pass_gap: Dword,
    /// +0xb9c (0x6051D8): `dword_6051D8`: the gap a pace car is moved by (`drive::start_cars_behind`).
    pub pace_back_gap: Dword,
    /// +0xba0 (0x6051DC): `dword_6051DC`: the sum of the two gaps (`drive::start_cars_behind`).
    pub pace_gap_sum: Dword,
    /// +0xba4 (0x6051E0): `dword_6051E0`: the order of the car kinds, 0x22 entries (the last two overlap `view_share` and `camera_turn_step`); `race::index_of_kind` searches it.
    pub kind_order: [Dword; 32],
    /// +0xc24 (0x605260): `flt_605260`: the share of a view change done (`physics::ease_view_angles`, `ease_view_blend`, `physics::camera_view`).
    pub view_share: Float,
    /// +0xc28 (0x605264): `dword_605264`: the camera's turn step, -1 or 0x4000000 and down (`physics::camera_avoid_ground`).
    pub camera_turn_step: Dword,
    /// +0xc2c (0x605268): `dword_605268`: 0 / -1 flag set by the camera's ground check (`physics::camera_avoid_ground`).
    pub camera_ground_flag: Dword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl RaceTables {
    /// The piece, with the image's first values.
    pub const fn new() -> Self {
        crate::cells::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START))
    }
}

/// IDA 0x60463C, the first byte of the piece.
pub const START: usize = 0x60463C;
/// IDA 0x60526C, the end of the piece.
pub const END: usize = 0x60526C;

const _: () = assert!(END - START == core::mem::size_of::<RaceTables>());

/// The piece's host address (the alignment checks).
#[cfg(test)]
pub fn piece_addr() -> usize {
    core::ptr::from_ref(&race().shared.tables).addr()
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(&race().shared.tables);
    }

    #[test]
    fn fields_are_at_the_games_offsets() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(offset_of!(RaceTables, stage_time_factors_last), 0x0);
        assert_eq!(offset_of!(RaceTables, time_percentages), 0x90);
        assert_eq!(offset_of!(RaceTables, number_before_first), 0x9f4);
        assert_eq!(offset_of!(RaceTables, start_share), 0xb70);
        assert_eq!(offset_of!(RaceTables, pace_start_b), 0xb8c);
        assert_eq!(offset_of!(RaceTables, view_share), 0xc24);
    }
}
