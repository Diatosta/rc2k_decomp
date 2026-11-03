//! `menu::run_championship` (0x4A0E60): a whole championship (event type 4), from its
//! start or from a saved game (`dword_755970` 1, which also makes `dword_755950` 2), to
//! its end. In order:
//! - `dword_755250` 0; unless continuing: `menu::begin_event`, `shuffle_drivers`,
//!   `clear_result_slots`, `reset_drivers`, `reset_driver_condition`, `dword_755254` 0,
//!   `clear_players_284`; then `take_player1_units`, `reset_cars_750724`;
//!   `g_yesNoAnswer`, `dword_755561`, `dword_75555D` 0, `g_eventMode` 2, `dword_75AB40`
//!   2; with no section to start at (`dword_7554C0` 0) the welcome
//!   (`menu::championship_welcome`, or `menu::a8_welcome` were `g_eventMode` not 2);
//!   `sound::stop_stream`; a yes ends it; `dword_755970` 0;
//! - each section: `dword_755A3C`, `dword_75593C`, `dword_755938`, `dword_755928` 0,
//!   `dword_75592C` 1, `dword_755250` 0, `section` `dword_7554C0` - 1 when that is set;
//! - each record of the section (`records::find_record` on `g_recordTable`, `section`,
//!   `index`; `dword_75AB20` 0 first) by its kind (+0): 3, the championship's end
//!   (`dword_755928` 1, then the standings below); 5, a welcome (`dword_755924` its
//!   +0x38, `index` + 1, `dword_755934` 1, `begin_event`, `menu::welcome_loop`); 4, the
//!   section's end: the championship standings (`standings_b_from_all`, sorted, the stage
//!   points, sorted twice, `class_championship_pass`), and unless the section was given
//!   (`dword_7554C0`) or it was the end, the overall points, the table numbered and shown
//!   (`show_championship_standings`), `section_finish`, `review_options_480d46_loop`,
//!   and on to the next section (`begin_event`); 1, a service due (`dword_755938` 1,
//!   `dword_75593C` its +0x14, `dword_755940` its +0xC, `dword_755944` and
//!   `dword_75594C` 1, `dword_755254` + 1, `index` + 1); 0, a stage: the rally's stages
//!   (`rally_stages_loop`), the stage (`next_stage_loop`), the service (`service_pass`),
//!   the races (`run_event_races`, unless `dword_755250`), the times, the stage's and
//!   (from the second stage on, `dword_755950` >= 1) the rally's results, the record
//!   bests (`keep_record_best`, `record_entries_pass`, `best_times_loop` when
//!   `dword_755920`), then by `menu::next_record_kind`: 3 the next stage at once, 4 the
//!   section's end, 1 a service next (`dword_755944`, `dword_75594C` 1), else the
//!   inspection (`inspection_pass`, `dword_75594C` 0); then the review
//!   (`review_options_47d5a5_loop` when `dword_75AAD8` or `dword_75594C`, else
//!   `review_options_48108e_loop`) and the next record (`index`, `dword_755950` + 1,
//!   `dword_755254` 0). A yes on the stage's screens resets the players' times
//!   (`dword_755250` 1) and goes to the rally's results (`dword_75AB20` 1, the leave
//!   flags and `dword_75592C` 0, `walk_section_records`, the rally's tables shown) and
//!   then the section's end; any other kind is fatal error 0x8070DBA0 "Invalid Level
//!   Type";
//! - the end: unless the section was given, `championship_finish`, `clear_score_marks`,
//!   `best_class_scores`, `best_overall_scores`; `review_options_481429_loop`;
//! - a yes to leaving on most screens ends it at once; at the very end, with the
//!   registers of the entry, `menu::show_in_game_options`.
//!
//! In:  nothing
//! Out: every register as `menu::show_in_game_options` leaves the entry's (all kept)
//!
//! `run_event` is shared with `menu::run_a8_championship` (`a8`: its differences). Callers:
//! `saves::load_savegame` and the five starters (`menu::start_championship_*`). The calls through
//! `event_flow::Call`. Refuses what its callees refuse, when they are reached.

use crate::errors::Refused;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::flow_game::Game;
use crate::menu::flow_store::SCREEN_FLOW;
use crate::menu::pages_store::MENU_PAGES;
use crate::session::Dword;

pub const STAGES_DONE: &Dword = &EVENT_STATE.stages_done; // dword_755950
pub const RESET_TIMES: &Dword = &MENU_PAGES.reset_times; // dword_755250
pub const SERVICES: &Dword = &MENU_PAGES.services; // dword_755254
pub const START_SECTION: &Dword = &SCREEN_FLOW.start_section; // dword_7554C0 (the A8's dword_7554C4)
pub const START_SECTION_A8: &Dword = &SCREEN_FLOW.start_section_a8;
pub const SERVICE_TIME: &Dword = &EVENT_STATE.service_minutes; // dword_75593C
pub const SERVICE_DUE: &Dword = &EVENT_STATE.service_due; // dword_755938
pub const THE_END: &Dword = &EVENT_STATE.championship_end; // dword_755928
pub const GOES_ON: &Dword = &EVENT_STATE.event_goes_on; // dword_75592C
pub const WELCOME_FIGURE: &Dword = &EVENT_STATE.welcome_figure; // dword_755924
pub const WELCOMED: &Dword = &EVENT_STATE.welcomed; // dword_755934
pub const SERVICE_940: &Dword = &EVENT_STATE.service_940; // dword_755940
pub const SERVICE_944: &Dword = &EVENT_STATE.service_944; // dword_755944
pub const SERVICE_NEXT: &Dword = &EVENT_STATE.service_next; // dword_75594C
// dword_755920

/// The two championships' driver: `a8` for `menu::run_a8_championship`'s differences.
pub fn run_event(a8: bool) -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::championship::run`.
    Game::new().run(|g| rc2k_core::menu::championship::run(g, a8))
}

pub fn run_championship() -> Result<(), Refused> {
    run_event(false)
}
