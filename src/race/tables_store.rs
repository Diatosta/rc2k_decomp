//! The start and tables work: the stage time factors, the arcade mode's tables by rally and stage
//! (cars, places, car types and numbers), the cars data text read in place, the noise seed, the
//! start and pace cars' values, the car kind order, and the camera's share and ground-turn
//! variables. `#[repr(C)]`, laid over the race block bytes.

use crate::memory::cells::ZeroCells;
use crate::memory::session::{PlainDword, PlainFloat};

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct RaceTables {
    /// The factor by section for the last three record times (`menu::scale_stage_times`).
    pub stage_time_factors_last: [PlainFloat; 6],
    /// The factor by section for the first record time (`menu::scale_stage_times`).
    pub stage_time_factors_first: [PlainFloat; 6],
    /// The arcade cars count by rally (`race::set_up_mode_2_cars`).
    pub arcade_cars_by_rally: [PlainDword; 6],
    /// The best place wanted by arcade stage (six stages); `race::set_up_mode_2_cars` copies
    /// the stage's entry to `best_place`.
    pub arcade_best_place_by_stage: [PlainDword; 6],
    /// The share a car gets on the last start place by arcade rally (six rallies);
    /// `race::set_up_mode_2_cars` copies the rally's entry to `start_share_last`.
    pub start_share_last_by_rally: [PlainDword; 6],
    /// The share on the first start place by arcade rally; copied to `start_share_first` by
    /// `race::set_up_mode_2_cars`.
    pub start_share_first_by_rally: [PlainDword; 6],
    /// The time percentage by difficulty level (`race::race_cars` prints it); 36 entries.
    pub time_percentages: [PlainDword; 36],
    /// 0x17 floats by car kind (`race::race_cars`).
    pub kind_time_factors: [PlainFloat; 23],
    /// `car_data_text` and the two bytes after it: the cars data text, 0x708 bytes, read
    /// in place (`race::read_stage_car_numbers`, `race::race_cars`).
    pub car_data_text: [PlainDword; 450],
    /// The step of the start blend (`drive::start_blend`).
    pub start_blend_step: PlainFloat,
    /// Scale of the push apart of the pace cars (`drive::start_cars_behind`).
    pub pace_push_scale: PlainFloat,
    /// The float 30.0, never written: the speed
    /// `drive::drive_computer_car` eases the computer's wanted speed toward (`int_toward_to_int`).
    /// It is also slot 0 of the cars-by-place table that runs on into `arcade_car_kinds`
    /// (`race::panel_neighbours` reads `[place]` and `[place + 1]`): only a place of 0 would read
    /// the 30.0 as a car (refused there).
    pub computer_ease_speed: PlainDword,
    /// Each car's type 1 .. 3, 89 rooms; also the cars by start
    /// (`race::set_up_mode_2_cars`, `drive::start_cars_behind`, `race::panel_neighbours`).
    pub arcade_car_kinds: [PlainDword; 89],
    /// Read as the number at place -1 by `race::panel_neighbours`; `arcade_car_numbers`
    /// follows it.
    pub number_before_first: PlainDword,
    /// Each car's number by start (`race::set_up_mode_2_cars`, `drive::start_cars_behind`,
    /// `drive::start_car_on_line`, `race::panel_neighbours`).
    pub arcade_car_numbers: [PlainDword; 90],
    /// The noise seed (`race::set_up_mode_2_cars`, `race::race_end`, `race::credits_race`).
    pub noise_seed: PlainDword,
    /// The start in use (`drive::start_car_on_line`, `drive::start_cars_behind`).
    pub start_slot: PlainDword,
    /// The share of the last start place (`lo` of
    /// `rc2k_core::drive::start_places::start_share`, which blends it to `start_share_first` by the
    /// start's place); set by `race::set_up_mode_2_cars`.
    pub start_share_last: PlainFloat,
    /// The share of the first start place (`hi` of `start_share`); set by
    /// `race::set_up_mode_2_cars`.
    pub start_share_first: PlainFloat,
    /// The start's share (`drive::start_car_on_line`).
    pub start_share: PlainFloat,
    /// The start speed, loaded as a float (`drive::start_car_on_line`,
    /// `drive::start_cars_behind`).
    pub start_speed: PlainFloat,
    /// "NUM arcadecars", the starts count (`race::set_up_mode_2_cars`,
    /// `race::panel_neighbours`, `drive::start_cars_behind`).
    pub arcade_cars: PlainDword,
    /// Unused; it only keeps the layout.
    pub reserved_b7c: PlainDword,
    /// "arc start pos (1-n)" (`race::set_up_mode_2_cars`, `race::settings_copy`,
    /// `race::race_shell`).
    pub start_place: PlainDword,
    /// "bestpos" (`race::set_up_mode_2_cars`).
    pub best_place: PlainDword,
    /// The first pace car's start (`drive::start_cars_behind`).
    pub pace_start_a: PlainDword,
    /// The second pace car's start (`drive::start_cars_behind`). Written with the three
    /// before it as a run.
    pub pace_start_b: PlainDword,
    /// The first pace car's distance (`drive::start_cars_behind`).
    pub pace_distance_a: PlainDword,
    /// The second pace car's distance (`drive::start_cars_behind`).
    pub pace_distance_b: PlainDword,
    /// The gap at which the lead has passed a pace car (`drive::start_cars_behind`).
    pub pace_pass_gap: PlainDword,
    /// The gap a pace car is moved by (`drive::start_cars_behind`).
    pub pace_back_gap: PlainDword,
    /// The sum of the two gaps (`drive::start_cars_behind`).
    pub pace_gap_sum: PlainDword,
    /// The order of the car kinds, 0x22 entries (the last two overlap `view_share` and
    /// `camera_turn_step`); `race::index_of_kind` searches it.
    pub kind_order: [PlainDword; 32],
    /// The share of a view change done (`physics::ease_view_angles`, `ease_view_blend`,
    /// `physics::camera_view`).
    pub view_share: PlainFloat,
    /// The camera's turn step, -1 or 0x4000000 and down (`physics::camera_avoid_ground`).
    pub camera_turn_step: PlainDword,
    /// 0 / -1 flag set by the camera's ground check (`physics::camera_avoid_ground`).
    pub camera_ground_flag: PlainDword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The tables the start set-up and the panel read by an index that may run past the table
/// (`RaceTables::dword_from`), numbered by their place among the fields from `arcade_cars_by_rally`
/// on.
#[derive(Clone, Copy)]
pub enum Listed {
    ArcadeCarsByRally = 0,
    BestPlaceByStage = 1,
    StartShareLastByRally = 2,
    StartShareFirstByRally = 3,
    ComputerEaseSpeed = 9,
    ArcadeCarKinds = 10,
    NumberBeforeFirst = 11,
    ArcadeCarNumbers = 12,
    KindOrder = 30,
}

impl RaceTables {
    /// Dword `n` of the table `from`. `n` is not held to the table's end: reading goes on through
    /// the fields after the table in order, to the end of the record (`n` past that is refused).
    pub fn dword_from(&self, from: Listed, n: usize) -> Result<u32, crate::errors::BadInput> {
        let fields: [&dyn crate::memory::session::Run; 34] = [
            &self.arcade_cars_by_rally,
            &self.arcade_best_place_by_stage,
            &self.start_share_last_by_rally,
            &self.start_share_first_by_rally,
            &self.time_percentages,
            &self.kind_time_factors,
            &self.car_data_text,
            &self.start_blend_step,
            &self.pace_push_scale,
            &self.computer_ease_speed,
            &self.arcade_car_kinds,
            &self.number_before_first,
            &self.arcade_car_numbers,
            &self.noise_seed,
            &self.start_slot,
            &self.start_share_last,
            &self.start_share_first,
            &self.start_share,
            &self.start_speed,
            &self.arcade_cars,
            &self.reserved_b7c,
            &self.start_place,
            &self.best_place,
            &self.pace_start_a,
            &self.pace_start_b,
            &self.pace_distance_a,
            &self.pace_distance_b,
            &self.pace_pass_gap,
            &self.pace_back_gap,
            &self.pace_gap_sum,
            &self.kind_order,
            &self.view_share,
            &self.camera_turn_step,
            &self.camera_ground_flag,
        ];
        crate::memory::session::read_runs(&fields[from as usize..], n, "the race tables")
    }
}

/// The starting values of `stage_time_factors_last`.
#[rustfmt::skip]
const STAGE_TIME_FACTORS_LAST: [f32; 6] = [
    1.07, 1.03, 1.01, 0.99, 1.0, 0.99,
];

/// The starting values of `stage_time_factors_first`.
#[rustfmt::skip]
const STAGE_TIME_FACTORS_FIRST: [f32; 6] = [
    1.05, 1.02, 0.99, 0.98, 0.99, 0.99,
];

/// The starting values of `arcade_cars_by_rally`.
#[rustfmt::skip]
const ARCADE_CARS_BY_RALLY: [u32; 6] = [
    50, 55, 30, 70, 85, 85,
];

/// The starting values of `arcade_best_place_by_stage`.
#[rustfmt::skip]
const ARCADE_BEST_PLACE_BY_STAGE: [u32; 6] = [
    15, 10, 7, 5, 3, 1,
];

/// The starting values of `start_share_last_by_rally`.
#[rustfmt::skip]
const START_SHARE_LAST_BY_RALLY: [u32; 6] = [
    0x3F19999A, 0x3F23D70A, 0x3F2E147B, 0x3F3851EC, 0x3F428F5C, 0x3F4CCCCD,
];

/// The starting values of `start_share_first_by_rally`.
#[rustfmt::skip]
const START_SHARE_FIRST_BY_RALLY: [u32; 6] = [
    0x3F4CCCCD, 0x3F570A3D, 0x3F6147AE, 0x3F6B851F, 0x3F75C28F, 0x3F800000,
];

/// The starting values of `time_percentages`.
#[rustfmt::skip]
const TIME_PERCENTAGES: [u32; 36] = [
    25, 23, 21, 19, 17, 15, 22, 20, 18, 16, 14, 12, 19, 17, 15, 13, 11, 9, 16, 14, 12, 10, 8, 6, 13,
    11, 9, 7, 5, 3, 10, 8, 6, 4, 2, 0,
];

/// The starting values of `kind_time_factors`.
#[rustfmt::skip]
const KIND_TIME_FACTORS: [f32; 23] = [
    1.07, 1.08, 1.04, 1.09, 1.02, 1.13, 0.99, 1.0, 1.09, 1.09, 1.05, 1.12, 1.12, 1.15, 1.01, 1.1,
    1.1, 1.09, 1.13, 1.14, 1.1, 1.15, 1.03,
];

/// The starting values of `car_data_text`.
#[rustfmt::skip]
const CAR_DATA_TEXT: [u32; 450] = [
    0x2031324C, 0x20323330, 0x20373230, 0x20373230, 0x20323330, 0x20373330, 0x20303430, 0x20363330,
    0x20393230, 0xD333230, 0x32324C0A, 0x31343020, 0x31343020, 0x32333020, 0x35333020, 0x37333020,
    0x31343020, 0x36333020, 0x35333020, 0x39333020, 0x35333020, 0x36333020, 0x324C0A0D, 0x34302033,
    0x35302030, 0x34302037, 0x35302036, 0x34302031, 0x34302031, 0x35302034, 0x34302036, 0x34302039,
    0x34302031, 0x34302036, 0x34302030, 0x4C0A0D34, 0x30203432, 0x30203732, 0x30203632, 0x30203333,
    0x30203232, 0x30203731, 0x30203732, 0x30203332, 0xA0D3033, 0x2035324C, 0x20333430, 0x20383430,
    0x20373430, 0x20353430, 0x20303530, 0x20373430, 0x20353430, 0x20333430, 0x20313530, 0x20353330,
    0x20313430, 0xD363330, 0x36324C0A, 0x38333020, 0x32343020, 0x34343020, 0x34343020, 0x33343020,
    0x32343020, 0x31343020, 0x30353020, 0x39333020, 0x38333020, 0x38333020, 0x34343020, 0x33343020,
    0x334C0A0D, 0x34302031, 0x34302035, 0x34302035, 0x34302035, 0x34302032, 0x33302035, 0x33302036,
    0x34302033, 0x34302038, 0x34302034, 0x33302032, 0x4C0A0D37, 0x30203233, 0x30203533, 0x30203433,
    0x30203733, 0x30203433, 0x30203633, 0x30203433, 0x30203533, 0xA0D3333, 0x2033334C, 0x20383330,
    0x20343330, 0x20363330, 0x20373330, 0x20353230, 0x20333330, 0x20373330, 0x20323330, 0xD323330,
    0x34334C0A, 0x31343020, 0x36333020, 0x34333020, 0x34333020, 0x39333020, 0x33343020, 0x30333020,
    0x39323020, 0x36333020, 0x37333020, 0x334C0A0D, 0x36302035, 0x36302036, 0x36302035, 0x36302035,
    0x35302032, 0x36302035, 0x36302039, 0x35302031, 0x37302039, 0x36302030, 0x36302035, 0x36302036,
    0x36302037, 0x36302039, 0x4C0A0D30, 0x31203633, 0x30203330, 0x30203536, 0x30203336, 0x30203536,
    0x30203636, 0x30203137, 0x30203436, 0x30203336, 0x30203836, 0x30203037, 0x30203836, 0x30203936,
    0x30203237, 0xA0D3836, 0x2031344C, 0x20333530, 0x20373430, 0x20313530, 0x20383430, 0x20313530,
    0x20333530, 0x20393330, 0x20303530, 0x20373430, 0x20393530, 0xD393430, 0x32344C0A, 0x37343020,
    0x33343020, 0x39343020, 0x33343020, 0x32343020, 0x37333020, 0x35343020, 0x34343020, 0x34343020,
    0x31343020, 0x30343020, 0x344C0A0D, 0x34302033, 0x35302035, 0x35302034, 0x35302035, 0x35302034,
    0x35302031, 0x35302039, 0x36302035, 0x35302031, 0x35302035, 0x35302037, 0x35302035, 0x35302039,
    0x4C0A0D35, 0x30203434, 0x30203635, 0x30203435, 0x30203335, 0x30203235, 0x30203834, 0x30203335,
    0x30203234, 0x30203634, 0x30203334, 0x30203634, 0x30203934, 0x30203535, 0x30203235, 0xA0D3234,
    0x2035344C, 0x20393330, 0x20363530, 0x20383530, 0x20363430, 0x20393530, 0x20373630, 0x20343730,
    0x20343630, 0x20363530, 0x20363530, 0x20383530, 0x20383630, 0xD373630, 0x36344C0A, 0x31373020,
    0x36353020, 0x32353020, 0x33353020, 0x36353020, 0x35353020, 0x30353020, 0x38343020, 0x36353020,
    0x34353020, 0x30363020, 0x35343020, 0x354C0A0D, 0x34302031, 0x34302037, 0x33302032, 0x33302038,
    0x34302037, 0x33302033, 0x34302032, 0x33302034, 0x34302038, 0x33302031, 0x33302039, 0x33302037,
    0x4C0A0D39, 0x30203235, 0x30203834, 0x30203434, 0x30203933, 0x30203235, 0x30203934, 0x30203234,
    0x30203534, 0x30203235, 0x30203934, 0x30203235, 0x30203634, 0xA0D3734, 0x2033354C, 0x20373330,
    0x20373430, 0x20303530, 0x20353430, 0x20393430, 0x20323530, 0x20313430, 0x20323530, 0x20363430,
    0x20383430, 0x20373430, 0xD393430, 0x34354C0A, 0x35353020, 0x30353020, 0x33353020, 0x38343020,
    0x33353020, 0x36343020, 0x37353020, 0x39353020, 0x32343020, 0x34353020, 0x32343020, 0x354C0A0D,
    0x34302035, 0x33302039, 0x33302039, 0x34302037, 0x34302032, 0x34302036, 0x35302030, 0x34302031,
    0x33302036, 0x34302039, 0x4C0A0D31, 0x30203635, 0x30203833, 0x30203833, 0x30203733, 0x30203034,
    0x30203234, 0x30203433, 0x30203633, 0x30203633, 0x30203833, 0x30203034, 0xA0D3333, 0x2031364C,
    0x20303630, 0x20343630, 0x20313830, 0x20343630, 0x20393530, 0x20383630, 0x20333730, 0x20363630,
    0x20303830, 0x20353730, 0x20373730, 0x20393730, 0x20303830, 0x20373630, 0x20333630, 0xD393630,
    0x32364C0A, 0x37363020, 0x33373020, 0x37363020, 0x36353020, 0x31363020, 0x32373020, 0x35363020,
    0x31363020, 0x36363020, 0x32363020, 0x39353020, 0x32363020, 0x30363020, 0x36363020, 0x364C0A0D,
    0x35302033, 0x36302038, 0x36302034, 0x36302032, 0x35302031, 0x35302034, 0x36302032, 0x35302037,
    0x35302034, 0x36302039, 0x37302032, 0x34302030, 0x34302037, 0x35302038, 0x4C0A0D35, 0x30203436,
    0x30203238, 0x30203335, 0x30203635, 0x30203336, 0x30203936, 0x30203236, 0x30203136, 0x30203936,
    0x30203138, 0x30203137, 0x30203436, 0x30203135, 0x30203836, 0x30203637, 0xA0D3636, 0x2035364C,
    0x20323430, 0x20373330, 0x20303430, 0x20363330, 0x20373330, 0x20313430, 0x20363430, 0x20393430,
    0x20323430, 0xD333330, 0x36364C0A, 0x34373020, 0x36353020, 0x30363020, 0x34353020, 0x30353020,
    0x34343020, 0x38353020, 0x36343020, 0x31353020, 0x34363020, 0x30363020, 0x39353020, 0x38363020,
    0x38363020, 0x374C0A0D, 0x33302031, 0x32302032, 0x32302031, 0x32302033, 0x32302035, 0x32302033,
    0x32302039, 0x32302033, 0x4C0A0D34, 0x30203237, 0x30203034, 0x30203133, 0x30203433, 0x30203233,
    0x30203633, 0x30203233, 0x30203134, 0x30203633, 0xA0D3433, 0x2033374C, 0x20303430, 0x20393230,
    0x20393230, 0x20303430, 0x20363230, 0x20323330, 0x20363230, 0x20303330, 0xD343330, 0x34374C0A,
    0x35333020, 0x32333020, 0x39333020, 0x33333020, 0x36333020, 0x37323020, 0x31333020, 0x35333020,
    0x33333020, 0x374C0A0D, 0x33302035, 0x32302033, 0x32302033, 0x31302036, 0x4C0A0D37, 0x30203637,
    0x30203933, 0x30203632, 0x30203933, 0x30203632, 0x30203034, 0x30203533, 0x30203132, 0x30203633,
    0x30203333, 0xA0D3932,
];

/// The starting values of `kind_order`.
#[rustfmt::skip]
const KIND_ORDER: [u32; 32] = [
    7, 14, 6, 4, 22, 2, 10, 0, 1, 3, 8, 9, 17, 20, 16, 15, 12, 11, 5, 18, 19, 21, 13, 0xFFFFFFFF,
    0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF,
];

impl RaceTables {
    pub const fn new() -> Self {
        Self {
            stage_time_factors_last: PlainFloat::array(STAGE_TIME_FACTORS_LAST),
            stage_time_factors_first: PlainFloat::array(STAGE_TIME_FACTORS_FIRST),
            arcade_cars_by_rally: PlainDword::array(ARCADE_CARS_BY_RALLY),
            arcade_best_place_by_stage: PlainDword::array(ARCADE_BEST_PLACE_BY_STAGE),
            start_share_last_by_rally: PlainDword::array(START_SHARE_LAST_BY_RALLY),
            start_share_first_by_rally: PlainDword::array(START_SHARE_FIRST_BY_RALLY),
            time_percentages: PlainDword::array(TIME_PERCENTAGES),
            kind_time_factors: PlainFloat::array(KIND_TIME_FACTORS),
            car_data_text: PlainDword::array(CAR_DATA_TEXT),
            start_blend_step: PlainFloat::new(-0.02),
            pace_push_scale: PlainFloat::new(0.05),
            computer_ease_speed: PlainDword::new(0x41F00000),
            kind_order: PlainDword::array(KIND_ORDER),
            ..ZeroCells::ZERO
        }
    }
}

#[cfg(test)]
mod run_on_tests {
    use super::*;
    /// The record's whole bytes, as a checksum: a wrong dword in a row shows here.
    #[test]
    fn the_race_tables_keep_the_bytes_of_the_image() {
        let bytes = crate::memory::cells::flatten(&RaceTables::new());
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        for byte in &bytes {
            hash = (hash ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3);
        }
        assert_eq!(bytes.len(), 3120);
        assert_eq!(hash, 0xbad7_6d84_2ed4_3bd4);
    }

    /// A dword from each listed table is read by index inside the table, and an index far past the
    /// record is refused.
    #[test]
    fn dword_from_reads_the_tables_and_refuses_far_past_them() {
        let t = Box::new(RaceTables::new());
        let tables: [(Listed, &[PlainDword]); 4] = [
            (Listed::ArcadeCarsByRally, &t.arcade_cars_by_rally),
            (Listed::BestPlaceByStage, &t.arcade_best_place_by_stage),
            (Listed::StartShareLastByRally, &t.start_share_last_by_rally),
            (
                Listed::StartShareFirstByRally,
                &t.start_share_first_by_rally,
            ),
        ];
        for (listed, table) in tables {
            for (n, cell) in table.iter().enumerate() {
                cell.set(0x8000 + n as u32);
                assert_eq!(t.dword_from(listed, n).ok(), Some(0x8000 + n as u32), "{n}");
            }
            assert!(t.dword_from(listed, 1 << 20).is_err());
        }
    }

    /// The kind order is read by index from its first entry.
    #[test]
    fn kind_order_reads_the_entries() {
        let t = Box::new(RaceTables::new());
        for (k, cell) in t.kind_order.iter().enumerate() {
            cell.set(0x300 + k as u32);
            assert_eq!(
                t.dword_from(Listed::KindOrder, k).ok(),
                Some(0x300 + k as u32)
            );
        }
    }
}
