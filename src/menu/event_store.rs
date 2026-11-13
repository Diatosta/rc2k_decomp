//! The event's state, out of RAL.EXE's image: the stage
//! chosen, the classes, the times and flags of the event's players, the service, the event's
//! type, kind and sections, the current player and the car shown. One static.

use crate::session::{PlainDword, PlainFlag};

/// The event's state.
#[derive(rc2k_derive::Cells)]
pub struct Event {
    /// The controller (0 the keyboard, n the input device n - 1) of the player picked on the
    /// controller page, written by `screens::pick_player_n` (player +0x2C); read by no code.
    pub picked_controller: PlainDword,
    /// The stage chosen: its group, 0 or 1 (`menu::stage_choice`).
    pub stage_group: PlainDword,
    /// The stage chosen: its number in its group, 1 .. 6.
    pub stage_number: PlainDword,
    /// The stage chosen: its sub-group, 0 .. 5.
    pub stage_sub: PlainDword,
    /// `| Not worked out.`:
    pub dword_7558d4: PlainDword,
    /// 1 clears the Escape flag with the highlight (`menu::screen_loop`).
    pub clears_escape_flag: PlainDword,
    /// The `sparksmode` setting of `var\ral.cfg`.
    pub sparks_mode: PlainDword,
    pub dword_7558e0: PlainDword,
    pub dword_7558e4: PlainDword,
    pub dword_7558e8: PlainDword,
    pub dword_7558ec: PlainDword,
    pub dword_7558f0: PlainDword,
    /// The class shown on the class pages.
    pub class: PlainDword,
    /// The number of classes (`menu::calc_cpu_times`).
    pub classes: PlainDword,
    /// `| Not worked out.`:
    pub dword_7558fc: PlainDword,
    pub dword_755900: PlainDword,
    pub dword_755904: PlainDword,
    pub dword_755908: PlainDword,
    pub dword_75590c: PlainDword,
    pub dword_755910: PlainDword,
    /// The player's time (`menu::time_trial_qualifies`).
    pub player_time: PlainDword,
    /// 1 when the time qualifies / beats the target.
    pub time_qualifies: PlainDword,
    /// `| Not worked out.`:
    pub dword_75591c: PlainDword,
    /// 1 when any player's time qualified.
    pub any_time_qualified: PlainFlag,
    /// The welcome page's figure (`menu::run_championship`).
    pub welcome_figure: PlainDword,
    /// The championship is over.
    pub championship_end: PlainDword,
    /// 1 while the event goes on.
    pub event_goes_on: PlainDword,
    pub dword_755930: PlainDword,
    /// 1 once the welcome page was shown.
    pub welcomed: PlainDword,
    /// 1 when the service is due.
    pub service_due: PlainDword,
    /// The service time left.
    pub service_minutes: PlainDword,
    /// The +0xC word of the service record the championship has reached (`menu::run_championship`);
    /// written there, read by no code.
    pub service_record_c: PlainDword,
    /// 1 when the next computer drivers' times start a batch (set by a service record and the
    /// condition-reset step), 0 once `menu::calc_cpu_times` has drawn the first spread: with
    /// `cpu_spread_step` the state of `rc2k_core::menu::event::Spread`.
    pub cpu_spread_first: PlainDword,
    /// How many spreads of the batch `menu::calc_cpu_times` has drawn after the first: the
    /// multiplier of its second random number (under 1000); 0 while the batch starts.
    pub cpu_spread_step: PlainDword,
    /// The next service.
    pub service_next: PlainDword,
    /// The stages done in the event.
    pub stages_done: PlainDword,
    /// 3 time trial, 4 and 5 the championships (`menu::event_flow`).
    pub event_type: PlainDword,
    /// `index`: The event's stage in its section.
    pub index: PlainDword,
    /// `section`: The event's section.
    pub section: PlainDword,
    /// The first stage of the page shown.
    pub first_page_stage: PlainDword,
    /// The section of the stage list being viewed (a copy of `section` made by
    /// `menu::view_stage_loop`); read by no code.
    pub viewed_section: PlainDword,
    /// The stage pages.
    pub stage_pages: PlainDword,
    /// The stage page shown.
    pub stage_page: PlainDword,
    /// 1 when a saved game goes on.
    pub continuing_save: PlainDword,
    /// 1 when the player is out of the event.
    pub event_out: PlainDword,
    /// The count of the standings (`menu::number_all_a`).
    pub count_a: PlainDword,
    /// 1 when an event is over.
    pub event_over: PlainFlag,
    /// The players in the event.
    pub player_count: PlainDword,
    /// The event's kind (9 .. 0x18, `menu::run_event_races`).
    pub event_kind: PlainDword,
    /// The player whose turn it is.
    pub current_player: PlainDword,
    /// The car shown (an entry of the carousel).
    pub shown_car: PlainDword,
    /// The slideshow's picture, 0 .. 99.
    pub picture: PlainDword,
    /// The car page shown.
    pub car_page: PlainDword,
}

impl Event {
    pub const fn new() -> Self {
        Event {
            picked_controller: PlainDword::new(0x0),
            stage_group: PlainDword::new(0x0),
            stage_number: PlainDword::new(0x1),
            stage_sub: PlainDword::new(0x0),
            dword_7558d4: PlainDword::new(0x0),
            clears_escape_flag: PlainDword::new(0x0),
            sparks_mode: PlainDword::new(0x0),
            dword_7558e0: PlainDword::new(0x0),
            dword_7558e4: PlainDword::new(0x0),
            dword_7558e8: PlainDword::new(0x0),
            dword_7558ec: PlainDword::new(0x0),
            dword_7558f0: PlainDword::new(0x0),
            class: PlainDword::new(0x0),
            classes: PlainDword::new(0x4),
            dword_7558fc: PlainDword::new(0x0),
            dword_755900: PlainDword::new(0x0),
            dword_755904: PlainDword::new(0x0),
            dword_755908: PlainDword::new(0x0),
            dword_75590c: PlainDword::new(0x0),
            dword_755910: PlainDword::new(0x2),
            player_time: PlainDword::new(0x0),
            time_qualifies: PlainDword::new(0x0),
            dword_75591c: PlainDword::new(0x0),
            any_time_qualified: PlainFlag::new(false),
            welcome_figure: PlainDword::new(0x3DC),
            championship_end: PlainDword::new(0x0),
            event_goes_on: PlainDword::new(0x1),
            dword_755930: PlainDword::new(0x0),
            welcomed: PlainDword::new(0x0),
            service_due: PlainDword::new(0x0),
            service_minutes: PlainDword::new(0x0),
            service_record_c: PlainDword::new(0x0),
            cpu_spread_first: PlainDword::new(0x0),
            cpu_spread_step: PlainDword::new(0x0),
            service_next: PlainDword::new(0x0),
            stages_done: PlainDword::new(0x0),
            event_type: PlainDword::new(0x1),
            index: PlainDword::new(0x0),
            section: PlainDword::new(0x0),
            first_page_stage: PlainDword::new(0x0),
            viewed_section: PlainDword::new(0x0),
            stage_pages: PlainDword::new(0x0),
            stage_page: PlainDword::new(0x0),
            continuing_save: PlainDword::new(0x0),
            event_out: PlainDword::new(0x0),
            count_a: PlainDword::new(0x0),
            event_over: PlainFlag::new(false),
            player_count: PlainDword::new(0x1),
            event_kind: PlainDword::new(0x18),
            current_player: PlainDword::new(0x0),
            shown_car: PlainDword::new(0x0),
            picture: PlainDword::new(0x0),
            car_page: PlainDword::new(0x0),
        }
    }
}
