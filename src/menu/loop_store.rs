//! The menu loop's flags, out of RAL.EXE's image (IDA 0x755525 .. 0x755595): the
//! item highlighted, the leaving flags, the text entry's counters, the sprite table and the menu
//! shown. The game keeps these dwords at addresses one past a multiple of four (IDA 0x755525 ..);
//! here they lie at the same distances from the start. A `#[repr(C)]` struct with the image's
//! layout, one static.

use crate::session::{PlainDword, PlainFlag};

/// The menu loop's flags (IDA 0x755525 .. 0x755595).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct LoopFlags {
    /// `dword_755525`: The highlighted menu item, from 1.
    pub item: PlainDword,
    /// `dword_755551`: 1 after a long time without a key (`menu::update_blink`).
    pub idle: PlainFlag,
    /// `dword_755555`: The main menu's frames without a key.
    pub idle_frames: PlainDword,
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
    /// `LoopFlags.menu.0`: The current menu (by handle, `menus_store::MenuId`).
    pub menu: crate::menu::menus_store::MenuCell,
    /// `dword_755591`: Not worked out.
    pub dword_755591: PlainDword,
}

impl LoopFlags {
    pub const fn new() -> Self {
        LoopFlags {
            item: PlainDword::new(0x1),
            idle: PlainFlag::new(false),
            idle_frames: PlainDword::new(0x0),
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
            menu: crate::menu::menus_store::MenuCell::new(),
            dword_755591: PlainDword::new(0x0),
        }
    }
}
