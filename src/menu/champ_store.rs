//! The championship pages' rows and the event's tables, out of RAL.EXE's image (IDA 0x7556B4 ..
//! 0x755784): the rows shown, the key clock, the event's mode and the addresses of
//! the best-times and standings tables. A `#[repr(C)]` struct with the image's layout, one static;
//! `moved::MOVED` points the range at it. The tables' addresses are the game's allocations
//! (`saves::allocate_best_times`).

use crate::session::{PlainBytes, PlainDword, with_runs};

/// The rows, the key clock and the tables' addresses (IDA 0x7556B4 .. 0x755784).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ChampView {
    /// `dword_7556B4`: The rows (players or drivers) the results and standings pages show.
    pub count: PlainDword,
    /// `g_champFirstRow`: The first row shown (`menu::stage_rows`).
    pub first_row: PlainDword,
    /// `g_champRowEnd`: The end of the rows shown.
    pub row_end: PlainDword,
    /// `dword_7556C0`: The player being ranked (`screens::draw_class_ranking`).
    pub ranked_player: PlainDword,
    /// `dword_7556C4`: The clock at the last key.
    pub last_key_time: PlainDword,
    /// `dword_7556C8`: What the cursor's blink counts from.
    pub cursor_time: PlainDword,
    /// Unreferenced bytes (IDA 0x7556CC .. 0x7556D0).
    pub _gap_7556cc: PlainBytes<4>,
    /// `g_eventMode`: The event's mode: 2, 3 or 0x13.
    pub event_mode: PlainDword,
    /// `g_bestScores`: held the best scores' table; the table is typed now
    /// (`saves::best_times::SCORES`) and this stays 0.
    pub best_scores: PlainDword,
    /// `g_stageBestTimes`: held the stages' best times; typed now
    /// (`saves::best_times::STAGE_TIMES`), this stays 0 (`off_7354FE` names it).
    pub stage_best_times: PlainDword,
    /// `g_levelBestScores`: held the arcade levels' best scores; the table is typed now
    /// (`saves::best_times::LEVEL_SCORES`) and this stays 0.
    pub level_best_scores: PlainDword,
    /// `g_courseBestTimes`: held the courses' best times; typed now
    /// (`saves::best_times::COURSE_TIMES`), this stays 0 (`off_7354FE` names it).
    pub course_best_times: PlainDword,
    /// `g_stageBestTimesAlt`: held the other group's stage times; typed now
    /// (`saves::best_times::STAGE_TIMES_ALT`), this stays 0 (`off_7354FE` names it).
    pub stage_best_times_alt: PlainDword,
    /// `g_timeTrialBestTimes`: held the time trial's best times; typed now
    /// (`saves::best_times::TIME_TRIAL_TIMES`), this stays 0 (`off_7354FE` names it).
    pub time_trial_best_times: PlainDword,
    /// `g_rankedTimes`: held standings table A; typed now (`saves::best_times::RANKED_TIMES`),
    /// this stays 0.
    pub ranked_times: PlainDword,
    /// `g_champHistory`: held standings table B; typed now
    /// (`saves::best_times::CHAMP_HISTORY`), this stays 0.
    pub champ_history: PlainDword,
    /// `dword_7556F4`: held the standings summary; typed now (`saves::best_times::SUMMARY`),
    /// this is 1 while it is taken (`menu::set_up_front_end`): the race's rivals dword
    /// (`RACE_STATE.settings.rivals`, `RACE_VARS.rivals`) copies it, 1 meaning the summary.
    pub summary: PlainDword,
    /// `dword_7556F8`: A stage page switch (`OptionSwitch::StageRecordsSet`).
    pub switch_f8: PlainDword,
    /// `dword_7556FC`: A stage page switch (`OptionSwitch::StageRecordsAlt`).
    pub switch_fc: PlainDword,
    /// `dword_755700`: 2 a championship's, 3 a record's (`menu::stage_choice`).
    pub stage_mode: PlainDword,
    /// Unreferenced bytes (IDA 0x755704 .. 0x755784).
    pub _gap_755704: PlainBytes<128>,
}

/// First values the image's (`tests::champ_view_starts_as_the_games`).
impl ChampView {
    pub const fn new() -> Self {
        ChampView {
            count: PlainDword::new(0x0),
            first_row: PlainDword::new(0x0),
            row_end: PlainDword::new(0x0),
            ranked_player: PlainDword::new(0x1),
            last_key_time: PlainDword::new(0x0),
            cursor_time: PlainDword::new(0x0),
            _gap_7556cc: PlainBytes::new(with_runs::<4>(&[(0x0, &[0x01])])),
            event_mode: PlainDword::new(0x2),
            best_scores: PlainDword::new(0),
            stage_best_times: PlainDword::new(0),
            level_best_scores: PlainDword::new(0),
            course_best_times: PlainDword::new(0),
            stage_best_times_alt: PlainDword::new(0),
            time_trial_best_times: PlainDword::new(0),
            ranked_times: PlainDword::new(0),
            champ_history: PlainDword::new(0),
            summary: PlainDword::new(0),
            switch_f8: PlainDword::new(0x1),
            switch_fc: PlainDword::new(0x1),
            stage_mode: PlainDword::new(0x1),
            _gap_755704: PlainBytes::new([0; 128]),
        }
    }
}

/// IDA 0x7556B4, the first byte of the piece.
pub const START: usize = 0x7556B4;
/// IDA 0x755784, the end of the piece.
pub const END: usize = 0x755784;

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn champ_view_starts_as_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            &crate::menu::owner::menus().play.champ_view,
        );
        assert_eq!(offset_of!(ChampView, count), 0x0);
        assert_eq!(offset_of!(ChampView, first_row), 0x4);
        assert_eq!(offset_of!(ChampView, row_end), 0x8);
        assert_eq!(offset_of!(ChampView, ranked_player), 0xc);
        assert_eq!(offset_of!(ChampView, last_key_time), 0x10);
        assert_eq!(offset_of!(ChampView, cursor_time), 0x14);
        assert_eq!(offset_of!(ChampView, event_mode), 0x1c);
        assert_eq!(offset_of!(ChampView, best_scores), 0x20);
        assert_eq!(offset_of!(ChampView, stage_best_times), 0x24);
        assert_eq!(offset_of!(ChampView, level_best_scores), 0x28);
        assert_eq!(offset_of!(ChampView, course_best_times), 0x2c);
        assert_eq!(offset_of!(ChampView, stage_best_times_alt), 0x30);
        assert_eq!(offset_of!(ChampView, time_trial_best_times), 0x34);
        assert_eq!(offset_of!(ChampView, ranked_times), 0x38);
        assert_eq!(offset_of!(ChampView, champ_history), 0x3c);
        assert_eq!(offset_of!(ChampView, summary), 0x40);
        assert_eq!(offset_of!(ChampView, switch_f8), 0x44);
        assert_eq!(offset_of!(ChampView, switch_fc), 0x48);
        assert_eq!(offset_of!(ChampView, stage_mode), 0x4c);
    }
}
