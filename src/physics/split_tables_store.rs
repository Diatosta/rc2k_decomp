//! The race's split tables (IDA 0x60101C .. 0x60230C; phase 7, typed): the arcade kind, the
//! replay still flag, the timer factor, the checkpoint list of the splits, the stage's car
//! numbers, the split count and the dwords after it, the stage's car total and extra time,
//! the kind factor, the kept split times and the 90 rivals' tables (names, times, cars, times
//! scaled to a split). `#[repr(C)]` with the image's layout, over the race block's bytes
//! (`SPLIT_TABLES`). Names are readings unless IDA's.

#[cfg(test)]
use crate::race::owner::race;
use crate::session::{Dword, Float};

#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct SplitTables {
    /// +0x0 (0x60101C): `dword_60101C`: the arcade kind (1 or 2: `race::race_end`, `race::credits_race`, `race::race_cars`).
    pub arcade_kind: Dword,
    /// +0x4 (0x601020): no instruction of the game refers to it (IDA, 2026-11-10); unused.
    pub unused_4: Dword,
    /// +0x8 (0x601024): `dword_601024`: the replay still / marks flag (`race::show_replay_still`, `race::frame_views`, `physics::camera_frame`, `drive::set_listener`).
    pub still: Dword,
    /// +0xc (0x601028): `flt_601028`: the timer's factor once finished (`physics::race_timers`).
    pub timer_factor: Float,
    /// +0x10 (0x60102C): `dword_60102C`: the splits' checkpoint list; [0] is the start's checkpoint, [1 + split] the split's (`physics::passed_split_point`, `physics::rank_split_time`, `race::level_init`; `dword_601030` is [1]).
    pub checkpoints: [Dword; 257],
    /// +0x414 (0x601430): `g_stageCarNumbers` (and `dword_601434` ..): the stage's numbers from the file (`race::race_cars`, `physics::take_split_counted`).
    pub stage_numbers: [Dword; 256],
    /// +0x814 (0x601830): `g_splitColourCount`: the splits' count ("numcheckposits"; `physics::splits`, `race::frame_draw`, `race::level_init`).
    pub split_colour_count: Dword,
    /// +0x818 (0x601834): no instruction of the game refers to it (IDA, 2026-11-10); unused.
    pub unused_818: [Dword; 262],
    /// +0xc30 (0x601C4C): `g_stageCarTotal`: the sum of the stage's numbers (`race::race_cars`, `race::race_shell`).
    pub stage_car_total: Dword,
    /// +0xc34 (0x601C50): `dword_601C50`: the extra time (`race::race_cars::set_car_times`).
    pub extra_time: Dword,
    /// +0xc38 (0x601C54): no instruction of the game refers to it (IDA, 2026-11-10); unused.
    pub unused_c38: [Dword; 4],
    /// +0xc48 (0x601C64): `flt_601C64`: the kind's time factor (`race::race_cars::set_car_times`).
    pub kind_factor: Float,
    /// +0xc4c (0x601C68): `dword_601C68`: the kept split times, 64 of them (IDA's `int[62]` is short: `physics::keep_split_time` and `race::race_cars` write 0x40, over `dword_601D60` and `dword_601D64`).
    pub kept_times: [Dword; 64],
    /// +0xd4c (0x601D68): `id` (`dword_601D68`): the 90 rivals' name ids (`physics::fill_rival_table`, `physics::rank_split_time`).
    pub rival_names: [Dword; 90],
    /// +0xeb4 (0x601ED0): `dword_601ED0`: the rivals' times.
    pub rival_times: [Dword; 90],
    /// +0x101c (0x602038): `dword_602038`: the rivals' cars.
    pub rival_cars: [Dword; 90],
    /// +0x1184 (0x6021A0): `dword_6021A0`: the rivals' times scaled to the split.
    pub rival_scaled: [Dword; 90],
    /// +0x12ec (0x602308): `dword_602308`: the car's time at the split (`physics::rank_split_time`).
    pub car_time: Dword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece, with the image's first values.
impl SplitTables {
    pub const fn new() -> Self {
        use crate::cells::ZeroCells;
        use crate::session::{Dword, Float};
        Self {
            timer_factor: Float::new(2.5),
            unused_818: {
                let mut v: [Dword; 262] = ZeroCells::ZERO;
                v[256] = Dword::new(0x656C6966);
                v[257] = Dword::new(0x5C735C73);
                v[258] = Dword::new(0x656D6974);
                v[259] = Dword::new(0x6D69745C);
                v[260] = Dword::new(0x302E7365);
                v
            },
            kind_factor: Float::new(1.0),
            ..ZeroCells::ZERO
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            race().physics.split_tables(),
        );
    }
}
