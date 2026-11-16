//! The savegame preview's variables: the `usesavegamekeys` setting, the slot shown and the
//! saved game's figures the preview and details screens draw. A struct of plain cells (only
//! the game thread reaches it), one static.

use crate::memory::session::PlainDword;

/// The savegame preview.
#[derive(rc2k_derive::Cells)]
pub struct SavegamePreview {
    /// The `usesavegamekeys` setting of `var\ral.cfg`; 0 lets `saves::load_savegame` call
    /// `keep_player_settings`.
    pub use_savegame_keys: PlainDword,
    /// The slot whose saved game is shown (0x65 = none yet).
    pub shown_slot: PlainDword,
    /// 1 when there is nothing to show (`screens::draw_savegame_preview`).
    pub nothing_to_show: PlainDword,
    /// The seven dwords of the shown saved game, in the order they are saved: [0] the number
    /// of players, [2] the stage's index in its section, [3] the section, [5] the mode (0x13
    /// takes the title from the table); the others are not read
    /// (`screens::draw_savegame_details`).
    pub saved: [PlainDword; 7],
    /// The saved cars in the file's buffer. Not used: nothing reads it (the readers take the
    /// cars by their offset in the file).
    pub saved_cars: PlainDword,
}

/// The first values (`tests::savegame_preview_starts_as_the_games`).
impl SavegamePreview {
    pub const fn new() -> Self {
        SavegamePreview {
            use_savegame_keys: PlainDword::new(0),
            shown_slot: PlainDword::new(0x65),
            nothing_to_show: PlainDword::new(1),
            saved: [const { PlainDword::new(0) }; 7],
            saved_cars: PlainDword::new(0),
        }
    }
}
