//! A whole championship (event type 4), from its start or from a saved game (`CONTINUING` set,
//! which also makes `STAGES_DONE` 2), to its end.
//!
//! Start: unless continuing, the event, drivers and players are set up; with no section to
//! start at the welcome is shown. Then, per section, each record is run by its kind: 3 the
//! championship's end (standings), 5 a welcome, 4 the section's end (standings, points, review),
//! 1 a service due, 0 a stage (stages, service, races, results, record bests, inspection,
//! review); any other kind is fatal error 0x8070DBA0 "Invalid Level Type". A yes to leaving on
//! most screens ends it at once. The end shows the finish, the best scores and the end review,
//! then `menu::show_in_game_options`.
//!
//! `run_event` is shared with `menu::run_a8_championship` (`a8`: its differences). The calls go
//! through `event_flow::Call`; `saves::load_savegame` and the five starters
//! (`menu::start_championship_*`) run it. Refuses what its callees refuse.

use crate::errors::Refused;
use crate::menu::flow_game::Game;

/// The two championships' driver: `a8` for `menu::run_a8_championship`'s differences.
pub fn run_event(o: crate::owners::Owners<'static>, a8: bool) -> Result<(), Refused> {
    Game::new(o).run(|g| rc2k_core::menu::championship::run(g, a8))
}

pub fn run_championship_in(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    run_event(o, false)
}
