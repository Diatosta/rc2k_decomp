//! Module saves's owner (step 11, owned state): the saved games' and best times' stores in
//! one struct, reached through `saves()` by the code that does not take it as a parameter.

use crate::saves::best_times_store::{BestTimes, RallyTimes};
use crate::saves::preview_store::SavegamePreview;
use crate::saves::write_savegame_slot::SaveGames;
use crate::session::{Bytes, GameThread, PlainDword};

/// The file names and kept values that belong to no larger store.
pub struct Files {
    /// `aSavedataSavega`: the saved game's file name, the number's two digits last.
    pub savegame_template: Bytes<27>,
    /// `aVarRalDat`: the options file's name.
    pub options_path: Bytes<12>,
    /// `unk_4F66DD`, `unk_4F66E1`, `unk_4F66E5` (-1 at start), `unk_4F66E9`: four dwords the
    /// options file keeps (here and in `saves::save_options`, by address); nothing else of
    /// the game reads them (a table of the developers' variables names the first). The fifth
    /// (0x4F66ED) is a gap.
    pub options_kept: [PlainDword; 5],
}

impl Files {
    pub const fn new() -> Self {
        Files {
            savegame_template: Bytes::new(*b"savedata\\savegame\\svgame00\0"),
            options_path: Bytes::new(*b"var\\ral.dat\0"),
            options_kept: [
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0xFFFF_FFFF),
                PlainDword::new(0),
                PlainDword::new(0),
            ],
        }
    }
}

/// The saves' state.
pub struct Saves {
    pub best_times: BestTimes,
    pub rally_times: RallyTimes,
    pub savegame_preview: SavegamePreview,
    pub save_games: SaveGames,
    pub files: Files,
}

/// The one instance. Reached through `saves()`; `moved` registers its stores' places.
pub static SAVES: GameThread<Saves> = GameThread::new(Saves {
    best_times: BestTimes::new(),
    rally_times: RallyTimes::new(),
    savegame_preview: SavegamePreview::new(),
    save_games: SaveGames::new(),
    files: Files::new(),
});

/// The saves' state, for a caller that does not take it as a parameter yet.
pub fn saves() -> &'static Saves {
    SAVES.get()
}
