//! The saves' owner: the saved games' and best times' stores in one struct, reached through
//! `saves()` by the code that does not take it as a parameter.

use crate::saves::best_times_store::{BestTimes, RallyTimes};
use crate::saves::preview_store::SavegamePreview;
use crate::saves::write_savegame_slot::SaveGames;
use crate::session::{GameThread, PlainBytes, PlainDword};

/// The file names and kept values that belong to no larger store.
pub struct Files {
    /// The saved game's file name, the number's two digits last.
    pub savegame_template: PlainBytes<27>,
    /// The options file's name.
    pub options_path: PlainBytes<12>,
    /// Four dwords the options file keeps (the third is -1 at start; here and in
    /// `saves::save_options`); nothing else of the game reads them (a table of the developers'
    /// variables names the first). The fifth is a gap.
    pub options_kept: [PlainDword; 5],
}

impl Files {
    pub const fn new() -> Self {
        Files {
            savegame_template: PlainBytes::new(*b"savedata\\savegame\\svgame00\0"),
            options_path: PlainBytes::new(*b"var\\ral.dat\0"),
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

impl Saves {
    /// The state at start-up: every store in its first state.
    pub const fn new() -> Self {
        Self {
            best_times: BestTimes::new(),
            rally_times: RallyTimes::new(),
            savegame_preview: SavegamePreview::new(),
            save_games: SaveGames::new(),
            files: Files::new(),
        }
    }
}

/// The one instance, reached through `saves()`.
pub static SAVES: GameThread<Saves> = GameThread::new(Saves::new());

/// The saves' state, for a caller that does not take it as a parameter yet.
pub fn saves() -> &'static Saves {
    SAVES.get()
}
