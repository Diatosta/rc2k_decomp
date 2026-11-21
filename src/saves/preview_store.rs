//! The savegame preview's variables: the `usesavegamekeys` setting, the slot shown and the
//! saved game's figures the preview and details screens draw. A struct of plain cells (only
//! the game thread reaches it), one static.

use std::cell::Cell;

/// The savegame preview.
pub struct SavegamePreview {
    /// The `usesavegamekeys` setting of `var\ral.cfg`; 0 lets `saves::load_savegame` call
    /// `keep_player_settings`.
    pub use_savegame_keys: Cell<u32>,
    /// The slot whose saved game is shown (0x65 = none yet).
    pub shown_slot: Cell<u32>,
    /// 1 when there is nothing to show (`screens::draw_savegame_preview`).
    pub nothing_to_show: Cell<u32>,
    /// The seven dwords of the shown saved game, in the order they are saved: [0] the number
    /// of players, [2] the stage's index in its section, [3] the section, [5] the mode (0x13
    /// takes the title from the table); the others are not read
    /// (`screens::draw_savegame_details`).
    pub saved: [Cell<u32>; 7],
}

/// The first values (`tests::savegame_preview_starts_as_the_games`).
impl SavegamePreview {
    pub const fn new() -> Self {
        SavegamePreview {
            use_savegame_keys: Cell::new(0),
            shown_slot: Cell::new(0x65),
            nothing_to_show: Cell::new(1),
            saved: [const { Cell::new(0) }; 7],
        }
    }
}
