//! `menu::championship_finish`: the championship finish routine. The finish flag and the kept
//! value are cleared; the leader is looked for in standings table B (event mode 0x13:
//! `menu::first_player_b` in group 0; otherwise `menu::first_player_unplaced`). When it is
//! found, the finish flag and its twin are set and the leader's player index is used; otherwise
//! the finish flag is cleared and the first player done with the section is taken
//! (`menu::find_section_player`; none: nothing but a key taken). Then come the race's set-up
//! from it (`finish_calls::set_up`), the sounds' and screens' set-up, `drive::finish_race_0`,
//! the finish flag cleared, the front end's set-up again (with the race's sounds once more) and
//! a key taken at the end.
//!
//! The finders' fatal errors are raised. Refuses a leader's index past 3 (it would read
//! past the players) and what the finders refuse, before anything.

use crate::errors::Refused;
use crate::menu::finish_calls::Game;
use rc2k_core::menu::finish;

pub fn championship_finish(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    Game::run(o, finish::championship_finish)
}
