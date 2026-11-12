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
#[derive(rc2k_derive::Cells)]
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

/// The piece: a static of its own since 2026-10-29 (it left the race block, phase 13:
/// `race::store::LEFT`), with the image's first values.
impl SplitTables {
    pub const fn new() -> Self {
        crate::pointers::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START))
    }
}

/// IDA 0x60101C, the first byte of the piece.
pub const START: usize = 0x60101C;
/// IDA 0x60230C, the end of the piece.
pub const END: usize = 0x60230C;

const _: () = assert!(END - START == core::mem::size_of::<SplitTables>());

/// Where IDA 0x60101C is now.
#[cfg(test)]
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(core::ptr::from_ref(&race().physics.split_tables))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            &race().physics.split_tables,
        );
    }

    #[test]
    fn fields_are_at_the_games_offsets() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(offset_of!(SplitTables, still), 0x8);
        assert_eq!(offset_of!(SplitTables, checkpoints), 0x10);
        assert_eq!(offset_of!(SplitTables, stage_numbers), 0x414);
        assert_eq!(offset_of!(SplitTables, split_colour_count), 0x814);
        assert_eq!(offset_of!(SplitTables, stage_car_total), 0xc30);
        assert_eq!(offset_of!(SplitTables, kind_factor), 0xc48);
        assert_eq!(offset_of!(SplitTables, kept_times), 0xc4c);
        assert_eq!(offset_of!(SplitTables, rival_names), 0xd4c);
        assert_eq!(offset_of!(SplitTables, rival_scaled), 0x1184);
        assert_eq!(offset_of!(SplitTables, car_time), 0x12ec);
    }
}
