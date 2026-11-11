//! The championship pages' rows and the event's tables, out of RAL.EXE's image (IDA 0x7556B4 ..
//! 0x755784; phase 7, typed): the rows shown, the key clock, the event's mode and the addresses of
//! the best-times and standings tables. A `#[repr(C)]` struct with the image's layout, one static;
//! `moved::MOVED` points the range at it. The tables' addresses are the game's allocations
//! (`saves::allocate_best_times`). (Split 2026-10-25 from the front end's block `menu::store`.)

use crate::session::{Bytes, Dword, with_runs};

/// The rows, the key clock and the tables' addresses (IDA 0x7556B4 .. 0x755784).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ChampView {
    /// `dword_7556B4`: The rows (players or drivers) the results and standings pages show.
    pub count: Dword,
    /// `g_champFirstRow`: The first row shown (`menu::stage_rows`).
    pub first_row: Dword,
    /// `g_champRowEnd`: The end of the rows shown.
    pub row_end: Dword,
    /// `dword_7556C0`: The player being ranked (`screens::draw_class_ranking`).
    pub ranked_player: Dword,
    /// `dword_7556C4`: The clock at the last key.
    pub last_key_time: Dword,
    /// `dword_7556C8`: What the cursor's blink counts from.
    pub cursor_time: Dword,
    /// Unreferenced bytes (IDA 0x7556CC .. 0x7556D0).
    pub _gap_7556cc: Bytes<4>,
    /// `g_eventMode`: The event's mode: 2, 3 or 0x13.
    pub event_mode: Dword,
    /// `g_bestScores`: held the best scores' table; the table is typed now
    /// (`saves::best_times::SCORES`) and this stays 0.
    pub best_scores: Dword,
    /// `g_stageBestTimes`: held the stages' best times; typed now
    /// (`saves::best_times::STAGE_TIMES`), this stays 0 (`off_7354FE` names it).
    pub stage_best_times: Dword,
    /// `g_levelBestScores`: held the arcade levels' best scores; the table is typed now
    /// (`saves::best_times::LEVEL_SCORES`) and this stays 0.
    pub level_best_scores: Dword,
    /// `g_courseBestTimes`: held the courses' best times; typed now
    /// (`saves::best_times::COURSE_TIMES`), this stays 0 (`off_7354FE` names it).
    pub course_best_times: Dword,
    /// `g_stageBestTimesAlt`: held the other group's stage times; typed now
    /// (`saves::best_times::STAGE_TIMES_ALT`), this stays 0 (`off_7354FE` names it).
    pub stage_best_times_alt: Dword,
    /// `g_timeTrialBestTimes`: held the time trial's best times; typed now
    /// (`saves::best_times::TIME_TRIAL_TIMES`), this stays 0 (`off_7354FE` names it).
    pub time_trial_best_times: Dword,
    /// `g_rankedTimes`: held standings table A; typed now (`saves::best_times::RANKED_TIMES`),
    /// this stays 0.
    pub ranked_times: Dword,
    /// `g_champHistory`: held standings table B; typed now
    /// (`saves::best_times::CHAMP_HISTORY`), this stays 0.
    pub champ_history: Dword,
    /// `dword_7556F4`: held the standings summary; typed now (`saves::best_times::SUMMARY`),
    /// this is 1 while it is taken (`menu::set_up_front_end`): the race's rivals dword
    /// (`RACE_STATE.settings.rivals`, `RACE_VARS.rivals`) copies it, 1 meaning the summary.
    pub summary: Dword,
    /// `dword_7556F8`: A stage page switch (`OptionSwitch::StageRecordsSet`).
    pub switch_f8: Dword,
    /// `dword_7556FC`: A stage page switch (`OptionSwitch::StageRecordsAlt`).
    pub switch_fc: Dword,
    /// `dword_755700`: 2 a championship's, 3 a record's (`menu::stage_choice`).
    pub stage_mode: Dword,
    /// Unreferenced bytes (IDA 0x755704 .. 0x755784).
    pub _gap_755704: Bytes<128>,
}

/// First values the image's (`tests::champ_view_starts_as_the_games`).
impl ChampView {
    pub const fn new() -> Self {
        ChampView {
            count: Dword::new(0x0),
            first_row: Dword::new(0x0),
            row_end: Dword::new(0x0),
            ranked_player: Dword::new(0x1),
            last_key_time: Dword::new(0x0),
            cursor_time: Dword::new(0x0),
            _gap_7556cc: Bytes::new(with_runs::<4>(&[(0x0, &[0x01])])),
            event_mode: Dword::new(0x2),
            best_scores: Dword::new(0),
            stage_best_times: Dword::new(0),
            level_best_scores: Dword::new(0),
            course_best_times: Dword::new(0),
            stage_best_times_alt: Dword::new(0),
            time_trial_best_times: Dword::new(0),
            ranked_times: Dword::new(0),
            champ_history: Dword::new(0),
            summary: Dword::new(0),
            switch_f8: Dword::new(0x1),
            switch_fc: Dword::new(0x1),
            stage_mode: Dword::new(0x1),
            _gap_755704: Bytes::new([0; 128]),
        }
    }
}

/// IDA 0x7556B4, the first byte of the piece.
pub const START: usize = 0x7556B4;
/// IDA 0x755784, the end of the piece.
pub const END: usize = 0x755784;

/// Where IDA 0x7556B4 is now (`moved::MOVED`'s home).
#[cfg(test)]
pub fn champ_view_addr() -> u32 {
    crate::pointers::game_address(&raw const crate::menu::owner::MENUS.play.champ_view)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moved::image_bytes;
    use core::mem::{offset_of, size_of};

    #[test]
    fn champ_view_starts_as_the_games() {
        assert!(
            END - START <= size_of::<ChampView>()
                && size_of::<ChampView>() <= (END - START).next_multiple_of(4)
        );
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host(champ_view_addr()).cast_const(),
                END - START,
            )
        };
        assert_eq!(mine, &image_bytes(START, END - START)[..]);
        let at = |field: usize, ida: usize| assert_eq!(field, ida - START, "{ida:#X}");
        at(offset_of!(ChampView, count), 0x7556B4);
        at(offset_of!(ChampView, first_row), 0x7556B8);
        at(offset_of!(ChampView, row_end), 0x7556BC);
        at(offset_of!(ChampView, ranked_player), 0x7556C0);
        at(offset_of!(ChampView, last_key_time), 0x7556C4);
        at(offset_of!(ChampView, cursor_time), 0x7556C8);
        at(offset_of!(ChampView, event_mode), 0x7556D0);
        at(offset_of!(ChampView, best_scores), 0x7556D4);
        at(offset_of!(ChampView, stage_best_times), 0x7556D8);
        at(offset_of!(ChampView, level_best_scores), 0x7556DC);
        at(offset_of!(ChampView, course_best_times), 0x7556E0);
        at(offset_of!(ChampView, stage_best_times_alt), 0x7556E4);
        at(offset_of!(ChampView, time_trial_best_times), 0x7556E8);
        at(offset_of!(ChampView, ranked_times), 0x7556EC);
        at(offset_of!(ChampView, champ_history), 0x7556F0);
        at(offset_of!(ChampView, summary), 0x7556F4);
        at(offset_of!(ChampView, switch_f8), 0x7556F8);
        at(offset_of!(ChampView, switch_fc), 0x7556FC);
        at(offset_of!(ChampView, stage_mode), 0x755700);
    }
}
