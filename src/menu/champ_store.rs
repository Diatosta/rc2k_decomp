//! The championship pages' rows and the event's tables, out of RAL.EXE's image (IDA 0x7556B4 ..
//! 0x755784): the rows shown, the key clock, the event's mode and the addresses of
//! the best-times and standings tables. One static; the tables' addresses are the game's
//! allocations (`saves::allocate_best_times`).

use crate::session::PlainDword;

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
}

impl ChampView {
    pub const fn new() -> Self {
        ChampView {
            count: PlainDword::new(0x0),
            first_row: PlainDword::new(0x0),
            row_end: PlainDword::new(0x0),
            ranked_player: PlainDword::new(0x1),
            last_key_time: PlainDword::new(0x0),
            cursor_time: PlainDword::new(0x0),
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
        }
    }
}

/// The variables of the view that held a best-times table (the list the stage pages go
/// through, `StageTables::result_tables`, names them by their address in the game's data); the
/// tables are typed now (`saves::best_times`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimesVariable {
    /// `ChampView.stage_best_times`.
    StageTimes,
    /// `ChampView.course_best_times`.
    CourseTimes,
    /// `ChampView.time_trial_best_times`.
    TimeTrialTimes,
    /// `ChampView.stage_best_times_alt`.
    StageTimesAlt,
}

impl TimesVariable {
    /// The variable the game's name `name` (the variable's address in the game's data, as the
    /// stage tables' list holds it) stands for.
    pub const fn named(name: u32) -> Option<TimesVariable> {
        match name {
            0x7556D8 => Some(TimesVariable::StageTimes),
            0x7556E0 => Some(TimesVariable::CourseTimes),
            0x7556E8 => Some(TimesVariable::TimeTrialTimes),
            0x7556E4 => Some(TimesVariable::StageTimesAlt),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_times_variables_are_named_by_address() {
        assert_eq!(
            TimesVariable::named(0x7556D8),
            Some(TimesVariable::StageTimes)
        );
        assert_eq!(
            TimesVariable::named(0x7556E4),
            Some(TimesVariable::StageTimesAlt)
        );
        assert_eq!(TimesVariable::named(0x7556DC), None);
    }
}
