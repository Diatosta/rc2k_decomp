//! The race's split tables: the arcade kind, the replay still flag, the timer factor, the
//! checkpoint list of the splits, the stage's car numbers, the split count and the dwords after it,
//! the stage's car total and extra time, the kind factor, the kept split times and the 90 rivals'
//! tables (names, times, cars, times scaled to a split). Names are readings unless taken from the
//! game's own.

use rc2k_core::saves::name_id::NameId;

pub struct SplitTables {
    /// The arcade kind (1 or 2: `race::race_end`, `race::credits_race`, `race::race_cars`).
    pub arcade_kind: u32,
    /// The replay still / marks flag (`race::show_replay_still`, `race::frame_views`,
    /// `physics::camera_frame`, `drive::set_listener`).
    pub still: u32,
    /// The timer's factor once finished (`physics::race_timers`).
    pub timer_factor: f32,
    /// The splits' checkpoint list; [0] is the start's checkpoint, [1 + split] the split's
    /// (`physics::passed_split_point`, `physics::rank_split_time`, `race::level_init`).
    pub checkpoints: [u32; 257],
    /// The stage's car numbers from the file (`race::race_cars`, `physics::take_split_counted`).
    pub stage_numbers: [u32; 256],
    /// The splits' count ("numcheckposits"; `physics::splits`, `race::frame_draw`,
    /// `race::level_init`).
    pub split_colour_count: u32,
    /// The sum of the stage's numbers (`race::race_cars`, `race::race_shell`).
    pub stage_car_total: u32,
    /// The extra time (`race::race_cars::set_car_times`).
    pub extra_time: u32,
    /// The kind's time factor (`race::race_cars::set_car_times`).
    pub kind_factor: f32,
    /// The 90 rivals' names (`physics::fill_rival_table`, `physics::rank_split_time`).
    pub rival_names: [NameId; 90],
    /// The rivals' times.
    pub rival_times: [u32; 90],
    /// The rivals' cars.
    pub rival_cars: [u32; 90],
    /// The rivals' times scaled to the split.
    pub rival_scaled: [u32; 90],
    /// The car's time at the split (`physics::rank_split_time`).
    pub car_time: u32,
}

/// The piece, with the first values.
impl SplitTables {
    pub const fn new() -> Self {
        Self {
            arcade_kind: 0,
            still: 0,
            timer_factor: 2.5,
            checkpoints: [0; 257],
            stage_numbers: [0; 256],
            split_colour_count: 0,
            stage_car_total: 0,
            extra_time: 0,
            kind_factor: 1.0,
            rival_names: [NameId::String(0); 90],
            rival_times: [0; 90],
            rival_cars: [0; 90],
            rival_scaled: [0; 90],
            car_time: 0,
        }
    }
}
