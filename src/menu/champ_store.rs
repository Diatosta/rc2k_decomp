//! The championship's view: the rows its results and standings pages show, the key clock, the
//! event's mode, the stage shown on the stage pages and the switches the championship's flow
//! sets. Plain fields behind one `RefCell`: the game's thread alone reads and writes them, each
//! borrow lasting one statement.

use crate::memory::session::GameThread;
use std::cell::{RefCell, RefMut};

/// The rows the results and standings pages show.
#[derive(Clone, Copy)]
pub struct ResultRows {
    /// The number of rows (players or drivers) to show.
    pub count: u32,
    /// The first row shown (`menu::stage_rows`).
    pub first_row: u32,
    /// The end of the rows shown.
    pub row_end: u32,
    /// The player being ranked (`screens::draw_class_ranking`); stays 1.
    pub ranked_player: u32,
}

/// The clock values the cursor's blink and the menus' key waits are measured from.
#[derive(Clone, Copy)]
pub struct KeyClock {
    /// The clock at the last key.
    pub last_key_time: u32,
    /// What the cursor's blink counts from; stays 0.
    pub cursor_time: u32,
}

/// The stage whose records the stage pages show.
#[derive(Clone, Copy)]
pub struct ShownStage {
    /// Its group, 0 .. 5 (the record table's index; set by `menu::option_items`).
    pub group: u32,
    /// Its number in its group, 1 .. 6 (`screens::stage_results`).
    pub number: u32,
}

/// The stage pages' two record switches (`OptionSwitch::StageRecordsSet` and `StageRecordsAlt`).
#[derive(Clone, Copy)]
pub struct RecordSwitches {
    /// Which set of records the stage pages show.
    pub records_set: u32,
    /// Whether they show the alternative group.
    pub records_alt: u32,
}

/// The championship's view values.
#[derive(Clone, Copy)]
pub struct ChampValues {
    /// The rows shown.
    pub rows: ResultRows,
    /// The key clock.
    pub key_clock: KeyClock,
    /// The stage the stage pages show.
    pub shown_stage: ShownStage,
    /// The stage pages' record switches.
    pub switches: RecordSwitches,
    /// The event's mode: 2, 3 or 0x13.
    pub event_mode: u32,
    /// 1 while the standings summary is taken (`menu::set_up_front_end`): the race's rivals
    /// dword (`RACE_STATE.settings.rivals`, `RACE_VARS.rivals`) copies it, 1 meaning the summary.
    pub summary: u32,
    /// 2 a championship's stage page, 3 a record's (`menu::stage_choice`).
    pub stage_mode: u32,
    /// 1 resets the players' times at the event's start (`menu::run_championship`;
    /// `saves::load_savegame` clears it).
    pub reset_times: u32,
    /// The services counted in the event (`menu::begin_event` clears it, `menu::run_championship`
    /// counts in it; saved with the options).
    pub services: u32,
}

/// The championship's view.
pub struct ChampView(GameThread<RefCell<ChampValues>>);

impl ChampView {
    pub const fn new() -> Self {
        ChampView(GameThread::new(RefCell::new(ChampValues {
            rows: ResultRows {
                count: 0x0,
                first_row: 0x0,
                row_end: 0x0,
                ranked_player: 0x1,
            },
            key_clock: KeyClock {
                last_key_time: 0x0,
                cursor_time: 0x0,
            },
            shown_stage: ShownStage {
                group: 0x0,
                number: 0x1,
            },
            switches: RecordSwitches {
                records_set: 0x1,
                records_alt: 0x1,
            },
            event_mode: 0x2,
            summary: 0,
            stage_mode: 0x1,
            reset_times: 0x0,
            services: 0x0,
        })))
    }

    /// A copy of the values: no borrow outlives the statement that reads them.
    pub fn read(&self) -> ChampValues {
        *self.0.get().borrow()
    }

    /// The values, to write.
    pub fn write(&self) -> RefMut<'_, ChampValues> {
        self.0.get().borrow_mut()
    }
}

/// The best-times tables the stage pages go through (`records::stages_store::RESULT_TABLES`);
/// the tables themselves are typed (`saves::best_times`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimesVariable {
    /// The stage times.
    StageTimes,
    /// The courses' times.
    CourseTimes,
    /// The time trial's times.
    TimeTrialTimes,
    /// The other group's stage times.
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
