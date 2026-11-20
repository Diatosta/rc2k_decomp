//! The race's split tables: the arcade kind, the replay still flag, the timer factor, the
//! checkpoint list of the splits, the stage's car numbers, the split count and the dwords after it,
//! the stage's car total and extra time, the kind factor, the kept split times and the 90 rivals'
//! tables (names, times, cars, times scaled to a split). Names are readings unless taken from the
//! game's own.

use crate::memory::session::{PlainDword, PlainFloat};

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct SplitTables {
    /// The arcade kind (1 or 2: `race::race_end`, `race::credits_race`, `race::race_cars`).
    pub arcade_kind: PlainDword,
    /// Unused.
    pub unused_4: PlainDword,
    /// The replay still / marks flag (`race::show_replay_still`, `race::frame_views`,
    /// `physics::camera_frame`, `drive::set_listener`).
    pub still: PlainDword,
    /// The timer's factor once finished (`physics::race_timers`).
    pub timer_factor: PlainFloat,
    /// The splits' checkpoint list; [0] is the start's checkpoint, [1 + split] the split's
    /// (`physics::passed_split_point`, `physics::rank_split_time`, `race::level_init`).
    pub checkpoints: [PlainDword; 257],
    /// The stage's car numbers from the file (`race::race_cars`, `physics::take_split_counted`).
    pub stage_numbers: [PlainDword; 256],
    /// The splits' count ("numcheckposits"; `physics::splits`, `race::frame_draw`,
    /// `race::level_init`).
    pub split_colour_count: PlainDword,
    /// Unused.
    pub unused_818: [PlainDword; 262],
    /// The sum of the stage's numbers (`race::race_cars`, `race::race_shell`).
    pub stage_car_total: PlainDword,
    /// The extra time (`race::race_cars::set_car_times`).
    pub extra_time: PlainDword,
    /// Unused.
    pub unused_c38: [PlainDword; 4],
    /// The kind's time factor (`race::race_cars::set_car_times`).
    pub kind_factor: PlainFloat,
    /// The kept split times, 64 of them.
    pub kept_times: [PlainDword; 64],
    /// The 90 rivals' names (`physics::fill_rival_table`, `physics::rank_split_time`).
    pub rival_names: [crate::saves::name_cell::NameIdCell; 90],
    /// The rivals' times.
    pub rival_times: [PlainDword; 90],
    /// The rivals' cars.
    pub rival_cars: [PlainDword; 90],
    /// The rivals' times scaled to the split.
    pub rival_scaled: [PlainDword; 90],
    /// The car's time at the split (`physics::rank_split_time`).
    pub car_time: PlainDword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece, with the image's first values.
impl SplitTables {
    pub const fn new() -> Self {
        use crate::memory::cells::ZeroCells;
        use crate::memory::session::{PlainDword, PlainFloat};
        Self {
            timer_factor: PlainFloat::new(2.5),
            unused_818: {
                let mut v: [PlainDword; 262] = ZeroCells::ZERO;
                v[256] = PlainDword::new(0x656C6966);
                v[257] = PlainDword::new(0x5C735C73);
                v[258] = PlainDword::new(0x656D6974);
                v[259] = PlainDword::new(0x6D69745C);
                v[260] = PlainDword::new(0x302E7365);
                v
            },
            kind_factor: PlainFloat::new(1.0),
            ..ZeroCells::ZERO
        }
    }
}
