//! The event's state: its type, kind, section and stage, the players, the flags the flows set,
//! the stage chosen, the service, the time trial's results and what the car pages show. One
//! store, split by what each part is for; the saved game's dwords are read through
//! `EventState::save`.

use crate::memory::session::{PlainDword, PlainFlag};

/// The stage chosen on the stage pages and the pages' own position.
#[derive(rc2k_derive::Cells)]
pub struct StageChoice {
    /// The stage's group, 0 or 1 (`menu::stage_choice`).
    pub group: PlainDword,
    /// The stage's number in its group, 1 .. 6.
    pub number: PlainDword,
    /// The stage's sub-group, 0 .. 5.
    pub sub: PlainDword,
    /// The first stage of the page shown.
    pub first_on_page: PlainDword,
    /// The number of stage pages.
    pub pages: PlainDword,
    /// The stage page shown.
    pub page: PlainDword,
}

/// The championship's service and the computer drivers' time spreads.
#[derive(rc2k_derive::Cells)]
pub struct ServiceState {
    /// 1 when the service is due.
    pub due: PlainDword,
    /// The service time left.
    pub minutes: PlainDword,
    /// 1 while the next service review is pending.
    pub next: PlainDword,
    /// 1 when the next computer drivers' times start a batch (set by a service record and the
    /// condition-reset step), 0 once `menu::calc_cpu_times` has drawn the first spread: with
    /// `spread_step` the state of `rc2k_core::menu::event::Spread`.
    pub spread_first: PlainDword,
    /// How many spreads of the batch `menu::calc_cpu_times` has drawn after the first: the
    /// multiplier of its second random number (under 1000); 0 while the batch starts.
    pub spread_step: PlainDword,
}

/// The time trial's verdict on the player's time.
#[derive(rc2k_derive::Cells)]
pub struct TrialState {
    /// 1 when the time qualifies / beats the target.
    pub qualifies: PlainDword,
    /// 1 when any player's time qualified.
    pub any_qualified: PlainFlag,
}

/// What the class and car pages show.
#[derive(rc2k_derive::Cells)]
pub struct Showcase {
    /// The class shown on the class pages.
    pub class: PlainDword,
    /// The number of classes (`menu::calc_cpu_times`).
    pub classes: PlainDword,
    /// The car shown (an entry of the carousel).
    pub car: PlainDword,
    /// The car page shown.
    pub car_page: PlainDword,
    /// The slideshow's picture, 0 .. 99.
    pub picture: PlainDword,
}

/// The event's state.
#[derive(rc2k_derive::Cells)]
pub struct EventState {
    /// 3 time trial, 4 and 5 the championships (`menu::event_flow`).
    pub event_type: PlainDword,
    /// The event's kind (9 .. 0x18, `menu::run_event_races`).
    pub event_kind: PlainDword,
    /// The event's section.
    pub section: PlainDword,
    /// The event's stage in its section.
    pub stage_in_section: PlainDword,
    /// The stages done in the event.
    pub stages_done: PlainDword,
    /// The players in the event.
    pub player_count: PlainDword,
    /// The player whose turn it is.
    pub current_player: PlainDword,
    /// 1 while the event goes on.
    pub event_goes_on: PlainDword,
    /// 1 when the player is out of the event.
    pub event_out: PlainDword,
    /// 1 when an event is over.
    pub event_over: PlainFlag,
    /// The championship is over.
    pub championship_end: PlainDword,
    /// 1 when a saved game goes on.
    pub continuing_save: PlainDword,
    /// The last number given to table A's standings (`menu::number_all`); the stage list's
    /// last row is derived from it.
    pub standings_count: PlainDword,
    /// 1 clears the Escape flag with the highlight (`menu::screen_loop`).
    pub clears_escape_flag: PlainDword,
    /// The `sparksmode` setting of `var\ral.cfg`.
    pub sparks_mode: PlainDword,
    pub stage: StageChoice,
    pub service: ServiceState,
    pub trial: TrialState,
    pub shown: Showcase,
}

/// The five event dwords a saved game holds (the other two are the championship's event mode
/// and the class mode), by reference.
pub struct EventSave<'a> {
    pub player_count: &'a PlainDword,
    pub stages_done: &'a PlainDword,
    pub stage_in_section: &'a PlainDword,
    pub section: &'a PlainDword,
    pub event_kind: &'a PlainDword,
}

impl EventState {
    pub const fn new() -> Self {
        EventState {
            event_type: PlainDword::new(0x1),
            event_kind: PlainDword::new(0x18),
            section: PlainDword::new(0x0),
            stage_in_section: PlainDword::new(0x0),
            stages_done: PlainDword::new(0x0),
            player_count: PlainDword::new(0x1),
            current_player: PlainDword::new(0x0),
            event_goes_on: PlainDword::new(0x1),
            event_out: PlainDword::new(0x0),
            event_over: PlainFlag::new(false),
            championship_end: PlainDword::new(0x0),
            continuing_save: PlainDword::new(0x0),
            standings_count: PlainDword::new(0x0),
            clears_escape_flag: PlainDword::new(0x0),
            sparks_mode: PlainDword::new(0x0),
            stage: StageChoice {
                group: PlainDword::new(0x0),
                number: PlainDword::new(0x1),
                sub: PlainDword::new(0x0),
                first_on_page: PlainDword::new(0x0),
                pages: PlainDword::new(0x0),
                page: PlainDword::new(0x0),
            },
            service: ServiceState {
                due: PlainDword::new(0x0),
                minutes: PlainDword::new(0x0),
                next: PlainDword::new(0x0),
                spread_first: PlainDword::new(0x0),
                spread_step: PlainDword::new(0x0),
            },
            trial: TrialState {
                qualifies: PlainDword::new(0x0),
                any_qualified: PlainFlag::new(false),
            },
            shown: Showcase {
                class: PlainDword::new(0x0),
                classes: PlainDword::new(0x4),
                car: PlainDword::new(0x0),
                car_page: PlainDword::new(0x0),
                picture: PlainDword::new(0x0),
            },
        }
    }

    /// The dwords a saved game holds.
    pub fn save(&self) -> EventSave<'_> {
        EventSave {
            player_count: &self.player_count,
            stages_done: &self.stages_done,
            stage_in_section: &self.stage_in_section,
            section: &self.section,
            event_kind: &self.event_kind,
        }
    }
}
