//! The menu loop's flags, out of RAL.EXE's image (IDA 0x755525 .. 0x755595): the
//! item highlighted, the leaving flags, the text entry's counters, the sprite table and the menu
//! shown. The game keeps these dwords at addresses one past a multiple of four (IDA 0x755525 ..);
//! here they lie at the same distances from the start. A `#[repr(C)]` struct with the image's
//! layout, one static; `moved::MOVED` points the range at it.

use crate::session::{PlainBytes, PlainDword, PlainFlag};

/// The menu loop's flags (IDA 0x755525 .. 0x755595).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct LoopFlags {
    /// `dword_755525`: The highlighted menu item, from 1.
    pub item: PlainDword,
    /// Unreferenced bytes (IDA 0x755529 .. 0x755551).
    pub _gap_755529: PlainBytes<40>,
    /// `dword_755551`: 1 after a long time without a key (`menu::update_blink`).
    pub idle: PlainFlag,
    /// `dword_755555`: The main menu's frames without a key.
    pub idle_frames: PlainDword,
    /// Unreferenced bytes (IDA 0x755559 .. 0x75555D).
    pub _gap_755559: PlainBytes<4>,
    /// `dword_75555D`: 1 when the event is left too, not only the screen (`go_back` clears it,
    /// confirmed leaving sets it; the loop leaves on it after an action, the flows read it).
    pub event_left: PlainFlag,
    /// `dword_755561`: 1 leaves the screen at once. Set by Escape, `go_back`,
    /// `leave_with_carry`; cleared by the key poll, set-up and the loop's action check.
    pub leave_screen: PlainFlag,
    /// `dword_755565`: set with `leave_screen` by `leave_with_carry`; the class result pages set
    /// it while a page with the player is shown; a review ends the event when it is not 1.
    pub left_with_carry: PlainFlag,
    /// `dword_755569`: an action asks to leave the screen (`ask_to_leave`): the loop leaves
    /// after the action, and a screen with `also_if` puts the leave question.
    pub ask_leave: PlainFlag,
    /// `dword_75556D`: The characters typed so far (`menu::text_entry`).
    pub text_length: PlainDword,
    /// `dword_755571`: The most characters the text takes.
    pub text_max: PlainDword,
    /// `dword_755575`: Flipped by key 0x20 (`menu::handle_keys`).
    pub toggle: PlainFlag,
    /// `g_champRow`: The championship row highlighted.
    pub champ_row: PlainDword,
    /// `g_optionsLoaded`: 1 once the options file has been read.
    pub options_loaded: PlainFlag,
    /// `g_spriteTable`: The address of the menu sprites' glyph table.
    pub sprite_table: PlainDword,
    /// Unreferenced bytes (IDA 0x755585 .. 0x75558D).
    pub _gap_755585: PlainBytes<8>,
    /// `LoopFlags.menu.0`: The current menu (by handle, `menus_store::MenuId`).
    pub menu: crate::menu::menus_store::MenuCell,
    /// `dword_755591`: Not worked out.
    pub dword_755591: PlainDword,
}

impl LoopFlags {
    /// First values the image's (`tests::loop_flags_starts_as_the_games`).
    pub const fn new() -> Self {
        LoopFlags {
            item: PlainDword::new(0x1),
            _gap_755529: PlainBytes::new([0; 40]),
            idle: PlainFlag::new(false),
            idle_frames: PlainDword::new(0x0),
            _gap_755559: PlainBytes::new([0; 4]),
            event_left: PlainFlag::new(false),
            leave_screen: PlainFlag::new(false),
            left_with_carry: PlainFlag::new(false),
            ask_leave: PlainFlag::new(false),
            text_length: PlainDword::new(0x0),
            text_max: PlainDword::new(0x1E),
            toggle: PlainFlag::new(true),
            champ_row: PlainDword::new(0x0),
            options_loaded: PlainFlag::new(false),
            sprite_table: PlainDword::new(0x0),
            _gap_755585: PlainBytes::new([0; 8]),
            menu: crate::menu::menus_store::MenuCell::new(),
            dword_755591: PlainDword::new(0x0),
        }
    }
}

/// IDA 0x755525, the first byte of the piece.
pub const START: usize = 0x755525;
/// IDA 0x755595, the end of the piece.
pub const END: usize = 0x755595;

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn loop_flags_starts_as_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            &crate::menu::owner::menus().screens.loop_flags,
        );
        assert_eq!(offset_of!(LoopFlags, item), 0x0);
        assert_eq!(offset_of!(LoopFlags, idle), 0x2c);
        assert_eq!(offset_of!(LoopFlags, idle_frames), 0x30);
        assert_eq!(offset_of!(LoopFlags, event_left), 0x38);
        assert_eq!(offset_of!(LoopFlags, leave_screen), 0x3c);
        assert_eq!(offset_of!(LoopFlags, left_with_carry), 0x40);
        assert_eq!(offset_of!(LoopFlags, ask_leave), 0x44);
        assert_eq!(offset_of!(LoopFlags, text_length), 0x48);
        assert_eq!(offset_of!(LoopFlags, text_max), 0x4c);
        assert_eq!(offset_of!(LoopFlags, toggle), 0x50);
        assert_eq!(offset_of!(LoopFlags, champ_row), 0x54);
        assert_eq!(offset_of!(LoopFlags, options_loaded), 0x58);
        assert_eq!(offset_of!(LoopFlags, sprite_table), 0x5c);
        assert_eq!(offset_of!(LoopFlags, menu), 0x68);
        assert_eq!(offset_of!(LoopFlags, dword_755591), 0x6c);
    }
}
