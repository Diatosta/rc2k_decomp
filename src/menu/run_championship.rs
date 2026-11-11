//! A whole championship (event type 4), from its start or from a saved game (`CONTINUING`
//! set, which also makes `STAGES_DONE` 2), to its end. In order:
//! - `RESET_TIMES` cleared; unless continuing: `menu::begin_event`, `shuffle_drivers`,
//!   `clear_result_slots`, `reset_drivers`, `reset_driver_condition`, `SERVICES` cleared,
//!   `clear_hud_bits`; then `take_player1_units`, `hide_all_in_game_options`; the yes/no
//!   answer, `LEAVE_SCREEN` and `EVENT_LEFT` cleared, the event mode set to 2 and the
//!   class mode to 2. With no section to start at (`START_SECTION` 0) the welcome is shown
//!   (`menu::championship_welcome`, or `menu::a8_welcome` were the event mode not 2); the
//!   stream stops (`sound::stop_stream`); a yes ends it; `CONTINUING` is cleared.
//! - each section: `ESCAPE_PRESSED`, `SERVICE_TIME`, `SERVICE_DUE` and `THE_END` cleared,
//!   `GOES_ON` set, `RESET_TIMES` cleared, and `section` taken from `START_SECTION` - 1
//!   when that is set.
//! - each record of the section (`records::find_record` on the record table, `section`,
//!   `index`; the rally-done flag cleared first) by its kind (+0):
//!   - 3, the championship's end: `THE_END` set, then the standings below;
//!   - 5, a welcome: `WELCOME_FIGURE` takes its +0x38, `index` + 1, `WELCOMED` set,
//!     `begin_event`, `menu::welcome_loop`;
//!   - 4, the section's end: the championship standings (`standings_b_from_all`, sorted,
//!     the stage points, sorted twice, `class_championship_pass`), and unless the section
//!     was given (`START_SECTION`) or it was the end, the overall points, the table
//!     numbered and shown (`show_championship_standings`), `section_finish`,
//!     `championship_standings_review_loop`, and on to the next section (`begin_event`);
//!   - 1, a service due: `SERVICE_DUE` set, `SERVICE_TIME` takes its +0x14,
//!     `SERVICE_RECORD_C` its +0xC, `CPU_SPREAD_FIRST` and `SERVICE_NEXT` are set,
//!     `SERVICES` + 1, `index` + 1;
//!   - 0, a stage: the rally's stages (`rally_stages_loop`), the stage (`next_stage_loop`),
//!     the service (`service_pass`), the races (`run_event_races`, unless `RESET_TIMES`),
//!     the times, the stage's and (from the second stage on, `STAGES_DONE` >= 1) the
//!     rally's results, the record bests (`keep_record_best`, `record_entries_pass`,
//!     `best_times_loop` when any time qualified), then by `menu::next_record_kind`: 3 the
//!     next stage at once, 4 the section's end, 1 a service next (`CPU_SPREAD_FIRST`,
//!     `SERVICE_NEXT` set), else the inspection (`inspection_pass`, `SERVICE_NEXT`
//!     cleared); then the review (`championship_service_review_loop` when the review kind
//!     or `SERVICE_NEXT` is set, else `championship_races_review_loop`) and the next record
//!     (`index`, `STAGES_DONE` + 1, `SERVICES` cleared). A yes on the stage's screens
//!     resets the players' times (`RESET_TIMES` set) and goes to the rally's results (the
//!     rally-done flag set, the leave flags and `GOES_ON` cleared, `walk_section_records`,
//!     the rally's tables shown) and then the section's end;
//!   - any other kind is fatal error 0x8070DBA0 "Invalid Level Type".
//! - the end: unless the section was given, `championship_finish`, `clear_score_marks`,
//!   `best_class_scores`, `best_overall_scores`; `championship_end_review_loop`.
//! - a yes to leaving on most screens ends it at once; at the very end,
//!   `menu::show_in_game_options`.
//!
//! `run_event` is shared with `menu::run_a8_championship` (`a8`: its differences). The
//! `saves::load_savegame` and the five starters (`menu::start_championship_*`) run it. The
//! calls go through `event_flow::Call`. Refuses what its callees refuse, when they are
//! reached.

use crate::errors::Refused;
use crate::menu::flow_game::Game;
use crate::session::Dword;

pub const STAGES_DONE: &Dword = &crate::menu::owner::MENUS.screens.event.stages_done;
pub const RESET_TIMES: &Dword = &crate::menu::owner::MENUS.tables.pages.reset_times;
pub const SERVICES: &Dword = &crate::menu::owner::MENUS.tables.pages.services;
pub const START_SECTION: &Dword = &crate::menu::owner::MENUS.screens.flow.start_section;
pub const START_SECTION_A8: &Dword = &crate::menu::owner::MENUS.screens.flow.start_section_a8;
pub const SERVICE_TIME: &Dword = &crate::menu::owner::MENUS.screens.event.service_minutes;
pub const SERVICE_DUE: &Dword = &crate::menu::owner::MENUS.screens.event.service_due;
pub const THE_END: &Dword = &crate::menu::owner::MENUS.screens.event.championship_end;
pub const GOES_ON: &Dword = &crate::menu::owner::MENUS.screens.event.event_goes_on;
pub const WELCOME_FIGURE: &Dword = &crate::menu::owner::MENUS.screens.event.welcome_figure;
pub const WELCOMED: &Dword = &crate::menu::owner::MENUS.screens.event.welcomed;
pub const SERVICE_RECORD_C: &Dword = &crate::menu::owner::MENUS.screens.event.service_record_c;
pub const CPU_SPREAD_FIRST: &Dword = &crate::menu::owner::MENUS.screens.event.cpu_spread_first;
pub const SERVICE_NEXT: &Dword = &crate::menu::owner::MENUS.screens.event.service_next;

/// The two championships' driver: `a8` for `menu::run_a8_championship`'s differences.
pub fn run_event(menus: &crate::menu::owner::Menus, a8: bool) -> Result<(), Refused> {
    // The logic is in `rc2k_core::menu::championship::run`.
    Game::new(menus).run(|g| rc2k_core::menu::championship::run(g, a8))
}

pub fn run_championship(menus: &crate::menu::owner::Menus) -> Result<(), Refused> {
    run_event(menus, false)
}
