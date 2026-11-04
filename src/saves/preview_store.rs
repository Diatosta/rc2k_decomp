//! The savegame preview's variables, out of RAL.EXE's image (IDA 0x755144 .. 0x755170;
//! phase 7, typed): the `usesavegamekeys` setting, the slot shown and the saved game's
//! figures the preview and details screens draw. A `#[repr(C)]` struct of atomic cells with
//! the image's layout, one static; `moved::MOVED` points the range at it. (Split 2026-10-25
//! from the front end's block `menu::store`.)

use crate::session::Dword;

/// The savegame preview (IDA 0x755144 .. 0x755170).
#[repr(C)]
pub struct SavegamePreview {
    /// `dword_755144`: the `usesavegamekeys` setting of `var\ral.cfg`; 0 lets
    /// `saves::load_savegame` call `maybeCopySomeData`.
    pub use_savegame_keys: Dword,
    /// `dword_755148`: the slot whose saved game is shown (0x65 = none yet).
    pub shown_slot: Dword,
    /// `dword_75514C`: 1 when there is nothing to show (`screens::draw_savegame_preview`).
    pub nothing_to_show: Dword,
    /// `dword_755150` .. `dword_755168`: the seven dwords of the shown saved game, in the
    /// order they are saved: [0] the number of players, [2] the stage's index in its
    /// section, [3] the section, [5] the mode (0x13 takes the title from the table); the
    /// others are not read (`screens::draw_savegame_details`).
    pub saved: [Dword; 7],
    /// `dword_75516C`: the saved cars in the file's buffer (the address after the figures).
    /// Not used: nothing reads it (the readers take the cars by their offset in the file).
    pub saved_cars: Dword,
}

/// First values the image's (`tests::savegame_preview_starts_as_the_games`).
pub static SAVEGAME_PREVIEW: SavegamePreview = SavegamePreview {
    use_savegame_keys: Dword::new(0),
    shown_slot: Dword::new(0x65),
    nothing_to_show: Dword::new(1),
    saved: [const { Dword::new(0) }; 7],
    saved_cars: Dword::new(0),
};

/// IDA 0X755144, the first byte of the piece.
pub const START: usize = 0x755144;
/// IDA 0X755170, the end of the piece.
pub const END: usize = 0x755170;

/// Where IDA 0X755144 is now (`moved::MOVED`'s home).
pub fn savegame_preview_addr() -> u32 {
    crate::pointers::game_address(&raw const SAVEGAME_PREVIEW)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moved::image_bytes;
    use core::mem::{offset_of, size_of};

    #[test]
    fn savegame_preview_starts_as_the_games() {
        assert!(END - START <= size_of::<SavegamePreview>());
        assert!(size_of::<SavegamePreview>() <= (END - START).next_multiple_of(4));
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host(savegame_preview_addr()).cast_const(),
                END - START,
            )
        };
        assert_eq!(mine, &image_bytes(START, END - START)[..]);
        let at = |field: usize, ida: usize| assert_eq!(field, ida - START, "{ida:#X}");
        at(offset_of!(SavegamePreview, shown_slot), 0x755148);
        at(offset_of!(SavegamePreview, saved), 0x755150);
        at(offset_of!(SavegamePreview, saved_cars), 0x75516C);
    }
}
