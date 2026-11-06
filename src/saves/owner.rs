//! Module saves's owner (step 11, owned state): the saved games' and best times' stores in
//! one struct, reached through `saves()` by the code that does not take it as a parameter.

use crate::saves::best_times_store::{BestTimes, RallyTimes};
use crate::saves::preview_store::SavegamePreview;
use crate::saves::write_savegame_slot::SaveGames;

/// The saves' state.
pub struct Saves {
    pub best_times: BestTimes,
    pub rally_times: RallyTimes,
    pub savegame_preview: SavegamePreview,
    pub save_games: SaveGames,
}

/// The one instance. Reached through `saves()`; `moved` registers its stores' places.
pub static SAVES: Saves = Saves {
    best_times: BestTimes::new(),
    rally_times: RallyTimes::new(),
    savegame_preview: SavegamePreview::new(),
    save_games: SaveGames::new(),
};

/// The saves' state, for a caller that does not take it as a parameter yet.
pub fn saves() -> &'static Saves {
    &SAVES
}
