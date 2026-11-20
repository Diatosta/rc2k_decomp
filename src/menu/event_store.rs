//! The event's state: its type, kind, section and stage, the players, the flags the flows set,
//! the stage chosen, the service, the time trial's results and what the car pages show. Plain
//! fields behind one `RefCell`, split by what each part is for: the game's thread alone reads and
//! writes them, each borrow lasting one statement.

use crate::memory::session::GameThread;
use std::cell::{RefCell, RefMut};

/// The stage chosen on the stage pages and the pages' own position.
#[derive(Clone, Copy)]
pub struct StageChoice {
    /// The stage's group, 0 or 1 (`menu::stage_choice`).
    pub group: u32,
    /// The stage's number in its group, 1 .. 6.
    pub number: u32,
    /// The stage's sub-group, 0 .. 5.
    pub sub: u32,
    /// The first stage of the page shown.
    pub first_on_page: u32,
    /// The number of stage pages.
    pub pages: u32,
    /// The stage page shown.
    pub page: u32,
}

/// The championship's service and the computer drivers' time spreads.
#[derive(Clone, Copy)]
pub struct ServiceState {
    /// 1 when the service is due.
    pub due: u32,
    /// The service time left.
    pub minutes: u32,
    /// 1 while the next service review is pending.
    pub next: u32,
    /// 1 when the next computer drivers' times start a batch (set by a service record and the
    /// condition-reset step), 0 once `menu::calc_cpu_times` has drawn the first spread: with
    /// `spread_step` the state of `rc2k_core::menu::event::Spread`.
    pub spread_first: u32,
    /// How many spreads of the batch `menu::calc_cpu_times` has drawn after the first: the
    /// multiplier of its second random number (under 1000); 0 while the batch starts.
    pub spread_step: u32,
}

/// The time trial's verdict on the player's time.
#[derive(Clone, Copy)]
pub struct TrialState {
    /// 1 when the time qualifies / beats the target.
    pub qualifies: u32,
    /// 1 when any player's time qualified.
    pub any_qualified: bool,
}

/// The number of car classes.
pub(super) const CLASSES: u32 = 4;

/// What the class and car pages show.
#[derive(Clone, Copy)]
pub struct Showcase {
    /// The class shown on the class pages.
    pub class: u32,
    /// The car shown (an entry of the carousel).
    pub car: u32,
    /// The car page shown.
    pub car_page: u32,
    /// The slideshow's picture, 0 .. 99.
    pub picture: u32,
}

/// The event's values.
#[derive(Clone, Copy)]
pub struct EventValues {
    /// 3 time trial, 4 and 5 the championships (`menu::event_flow`).
    pub event_type: u32,
    /// The event's kind (9 .. 0x18, `menu::run_event_races`).
    pub event_kind: u32,
    /// The event's section.
    pub section: u32,
    /// The event's stage in its section.
    pub stage_in_section: u32,
    /// The stages done in the event.
    pub stages_done: u32,
    /// The players in the event.
    pub player_count: u32,
    /// The player whose turn it is.
    pub current_player: u32,
    /// 1 while the event goes on.
    pub event_goes_on: u32,
    /// 1 when the player is out of the event.
    pub event_out: u32,
    /// 1 when an event is over.
    pub event_over: bool,
    /// The championship is over.
    pub championship_end: u32,
    /// 1 when a saved game goes on.
    pub continuing_save: u32,
    /// The last number given to table A's standings (`menu::number_all`); the stage list's
    /// last row is derived from it.
    pub standings_count: u32,
    /// 1 clears the Escape flag with the highlight (`menu::screen_loop`).
    pub clears_escape_flag: u32,
    /// The `sparksmode` setting of `var\ral.cfg`.
    pub sparks_mode: u32,
    pub stage: StageChoice,
    pub service: ServiceState,
    pub trial: TrialState,
    pub shown: Showcase,
}

/// The event's state.
pub struct EventState(GameThread<RefCell<EventValues>>);

impl EventState {
    pub const fn new() -> Self {
        EventState(GameThread::new(RefCell::new(EventValues {
            event_type: 0x1,
            event_kind: 0x18,
            section: 0x0,
            stage_in_section: 0x0,
            stages_done: 0x0,
            player_count: 0x1,
            current_player: 0x0,
            event_goes_on: 0x1,
            event_out: 0x0,
            event_over: false,
            championship_end: 0x0,
            continuing_save: 0x0,
            standings_count: 0x0,
            clears_escape_flag: 0x0,
            sparks_mode: 0x0,
            stage: StageChoice {
                group: 0x0,
                number: 0x1,
                sub: 0x0,
                first_on_page: 0x0,
                pages: 0x0,
                page: 0x0,
            },
            service: ServiceState {
                due: 0x0,
                minutes: 0x0,
                next: 0x0,
                spread_first: 0x0,
                spread_step: 0x0,
            },
            trial: TrialState {
                qualifies: 0x0,
                any_qualified: false,
            },
            shown: Showcase {
                class: 0x0,
                car: 0x0,
                car_page: 0x0,
                picture: 0x0,
            },
        })))
    }

    /// A copy of the values: no borrow outlives the statement that reads them.
    pub fn read(&self) -> EventValues {
        *self.0.get().borrow()
    }

    /// The values, to write.
    pub fn write(&self) -> RefMut<'_, EventValues> {
        self.0.get().borrow_mut()
    }
}

/// One of the event's dwords, named for the tables that reach a variable by a code (the
/// flows' variable lists, the settings file).
#[derive(Clone, Copy)]
pub enum EventWord {
    EventType,
    EventKind,
    Section,
    StageInSection,
    StagesDone,
    PlayerCount,
    CurrentPlayer,
    EventGoesOn,
    ContinuingSave,
    ChampionshipEnd,
    StageGroup,
    StageNumber,
    StageSub,
    ServiceDue,
    ServiceMinutes,
    ServiceNext,
    SpreadFirst,
    TrialQualifies,
}

impl EventWord {
    pub fn get(self, event: &EventState) -> u32 {
        let v = event.read();
        match self {
            EventWord::EventType => v.event_type,
            EventWord::EventKind => v.event_kind,
            EventWord::Section => v.section,
            EventWord::StageInSection => v.stage_in_section,
            EventWord::StagesDone => v.stages_done,
            EventWord::PlayerCount => v.player_count,
            EventWord::CurrentPlayer => v.current_player,
            EventWord::EventGoesOn => v.event_goes_on,
            EventWord::ContinuingSave => v.continuing_save,
            EventWord::ChampionshipEnd => v.championship_end,
            EventWord::StageGroup => v.stage.group,
            EventWord::StageNumber => v.stage.number,
            EventWord::StageSub => v.stage.sub,
            EventWord::ServiceDue => v.service.due,
            EventWord::ServiceMinutes => v.service.minutes,
            EventWord::ServiceNext => v.service.next,
            EventWord::SpreadFirst => v.service.spread_first,
            EventWord::TrialQualifies => v.trial.qualifies,
        }
    }

    pub fn set(self, event: &EventState, value: u32) {
        let mut v = event.write();
        match self {
            EventWord::EventType => v.event_type = value,
            EventWord::EventKind => v.event_kind = value,
            EventWord::Section => v.section = value,
            EventWord::StageInSection => v.stage_in_section = value,
            EventWord::StagesDone => v.stages_done = value,
            EventWord::PlayerCount => v.player_count = value,
            EventWord::CurrentPlayer => v.current_player = value,
            EventWord::EventGoesOn => v.event_goes_on = value,
            EventWord::ContinuingSave => v.continuing_save = value,
            EventWord::ChampionshipEnd => v.championship_end = value,
            EventWord::StageGroup => v.stage.group = value,
            EventWord::StageNumber => v.stage.number = value,
            EventWord::StageSub => v.stage.sub = value,
            EventWord::ServiceDue => v.service.due = value,
            EventWord::ServiceMinutes => v.service.minutes = value,
            EventWord::ServiceNext => v.service.next = value,
            EventWord::SpreadFirst => v.service.spread_first = value,
            EventWord::TrialQualifies => v.trial.qualifies = value,
        }
    }
}
