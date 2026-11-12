//! The savegame preview's variables, with the image's layout (IDA 0x755144 .. 0x755170):
//! the `usesavegamekeys` setting, the slot shown and the saved game's figures the preview and details screens draw. A `#[repr(C)]` struct of atomic cells with
//! the image's layout, one static; `moved::MOVED` points the range at it.

use crate::session::PlainDword;

/// The savegame preview (IDA 0x755144 .. 0x755170).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SavegamePreview {
    /// `dword_755144`: the `usesavegamekeys` setting of `var\ral.cfg`; 0 lets
    /// `saves::load_savegame` call `maybeCopySomeData`.
    pub use_savegame_keys: PlainDword,
    /// `dword_755148`: the slot whose saved game is shown (0x65 = none yet).
    pub shown_slot: PlainDword,
    /// `dword_75514C`: 1 when there is nothing to show (`screens::draw_savegame_preview`).
    pub nothing_to_show: PlainDword,
    /// `dword_755150` .. `dword_755168`: the seven dwords of the shown saved game, in the
    /// order they are saved: [0] the number of players, [2] the stage's index in its
    /// section, [3] the section, [5] the mode (0x13 takes the title from the table); the
    /// others are not read (`screens::draw_savegame_details`).
    pub saved: [PlainDword; 7],
    /// `dword_75516C`: the saved cars in the file's buffer (the address after the figures).
    /// Not used: nothing reads it (the readers take the cars by their offset in the file).
    pub saved_cars: PlainDword,
}

/// First values the image's (`tests::savegame_preview_starts_as_the_games`).
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

/// IDA 0X755144, the first byte of the piece.
pub const START: usize = 0x755144;
/// IDA 0X755170, the end of the piece.
pub const END: usize = 0x755170;

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::{offset_of, size_of};

    #[test]
    fn savegame_preview_starts_as_the_games() {
        assert!(END - START <= size_of::<SavegamePreview>());
        assert!(size_of::<SavegamePreview>() <= (END - START).next_multiple_of(4));
        crate::test_support::first_values::assert_block_starts_as_the_games(
            &crate::saves::owner::SAVES.get().savegame_preview,
        );
        assert_eq!(offset_of!(SavegamePreview, shown_slot), 0x4);
        assert_eq!(offset_of!(SavegamePreview, saved), 0xc);
        assert_eq!(offset_of!(SavegamePreview, saved_cars), 0x28);
    }
}
