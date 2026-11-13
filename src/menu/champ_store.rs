//! The championship pages' rows and the event's tables: the rows shown, the key clock, the
//! event's mode and the places of the best-times and standings tables. One static; the tables'
//! places are the game's allocations (`saves::allocate_best_times`).

use crate::session::PlainDword;

/// The rows, the key clock and the tables' places.
#[derive(rc2k_derive::Cells)]
pub struct ChampView {
    /// The rows (players or drivers) the results and standings pages show.
    pub count: PlainDword,
    /// The first row shown (`menu::stage_rows`).
    pub first_row: PlainDword,
    /// The end of the rows shown.
    pub row_end: PlainDword,
    /// The player being ranked (`screens::draw_class_ranking`).
    pub ranked_player: PlainDword,
    /// The clock at the last key.
    pub last_key_time: PlainDword,
    /// What the cursor's blink counts from.
    pub cursor_time: PlainDword,
    /// The event's mode: 2, 3 or 0x13.
    pub event_mode: PlainDword,
    /// Unused: the best scores are typed (`saves::best_times::SCORES`); stays 0.
    pub best_scores: PlainDword,
    /// Unused: the stages' best times are typed (`saves::best_times::STAGE_TIMES`); stays 0.
    pub stage_best_times: PlainDword,
    /// Unused: the arcade levels' best scores are typed (`saves::best_times::LEVEL_SCORES`);
    /// stays 0.
    pub level_best_scores: PlainDword,
    /// Unused: the courses' best times are typed (`saves::best_times::COURSE_TIMES`); stays 0.
    pub course_best_times: PlainDword,
    /// Unused: the other group's stage times are typed (`saves::best_times::STAGE_TIMES_ALT`);
    /// stays 0.
    pub stage_best_times_alt: PlainDword,
    /// Unused: the time trial's best times are typed (`saves::best_times::TIME_TRIAL_TIMES`);
    /// stays 0.
    pub time_trial_best_times: PlainDword,
    /// Unused: standings table A is typed (`saves::best_times::RANKED_TIMES`); stays 0.
    pub ranked_times: PlainDword,
    /// Unused: standings table B is typed (`saves::best_times::CHAMP_HISTORY`); stays 0.
    pub champ_history: PlainDword,
    /// The standings summary is typed (`saves::best_times::SUMMARY`); this is 1 while it is
    /// taken (`menu::set_up_front_end`): the race's rivals dword
    /// (`RACE_STATE.settings.rivals`, `RACE_VARS.rivals`) copies it, 1 meaning the summary.
    pub summary: PlainDword,
    /// A stage page switch (`OptionSwitch::StageRecordsSet`).
    pub switch_f8: PlainDword,
    /// A stage page switch (`OptionSwitch::StageRecordsAlt`).
    pub switch_fc: PlainDword,
    /// 2 a championship's, 3 a record's (`menu::stage_choice`).
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
/// through, `records::stages_store::RESULT_TABLES`, names them; the image's list held their
/// addresses, `image_name`); the tables are typed now (`saves::best_times`).
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
    /// image's stage tables' list held it) stands for (the tests' oracle of `image_name`).
    #[cfg(test)]
    pub const fn named(name: u32) -> Option<TimesVariable> {
        match name {
            0x7556D8 => Some(TimesVariable::StageTimes),
            0x7556E0 => Some(TimesVariable::CourseTimes),
            0x7556E8 => Some(TimesVariable::TimeTrialTimes),
            0x7556E4 => Some(TimesVariable::StageTimesAlt),
            _ => None,
        }
    }

    /// Its name in the game's data: the address the image's stage tables' list held for it,
    /// what a read on through the tables sees (`records::stages_store::Piece`).
    pub const fn image_name(self) -> u32 {
        match self {
            TimesVariable::StageTimes => 0x7556D8,
            TimesVariable::CourseTimes => 0x7556E0,
            TimesVariable::TimeTrialTimes => 0x7556E8,
            TimesVariable::StageTimesAlt => 0x7556E4,
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
        for v in [
            TimesVariable::StageTimes,
            TimesVariable::CourseTimes,
            TimesVariable::TimeTrialTimes,
            TimesVariable::StageTimesAlt,
        ] {
            assert_eq!(TimesVariable::named(v.image_name()), Some(v));
        }
    }
}
