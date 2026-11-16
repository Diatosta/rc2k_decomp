//! The menu loop's flags: the item highlighted, the leaving flags, the text entry's counters, the
//! sprite table and the menu shown. The game keeps these dwords at addresses one past a multiple
//! of four; here they lie at the same distances from the start. One static.

use crate::memory::session::{PlainDword, PlainFlag};

/// The menu loop's flags.
#[derive(rc2k_derive::Cells)]
pub struct LoopFlags {
    /// The highlighted menu item, from 1.
    pub item: PlainDword,
    /// 1 after a long time without a key (`menu::update_blink`).
    pub idle: PlainFlag,
    /// The main menu's frames without a key.
    pub idle_frames: PlainDword,
    /// 1 when the event is left too, not only the screen (`go_back` clears it, confirmed
    /// leaving sets it; the loop leaves on it after an action, the flows read it).
    pub event_left: PlainFlag,
    /// 1 leaves the screen at once. Set by Escape, `go_back`, `leave_with_carry`; cleared by the
    /// key poll, set-up and the loop's action check.
    pub leave_screen: PlainFlag,
    /// Set with `leave_screen` by `leave_with_carry`; the class result pages set it while a page
    /// with the player is shown; a review ends the event when it is not 1.
    pub left_with_carry: PlainFlag,
    /// An action asks to leave the screen (`ask_to_leave`): the loop leaves after the action, and a
    /// screen with `also_if` puts the leave question.
    pub ask_leave: PlainFlag,
    /// The characters typed so far (`menu::text_entry`).
    pub text_length: PlainDword,
    /// The most characters the text takes.
    pub text_max: PlainDword,
    /// Flipped by key 0x20 (`menu::handle_keys`).
    pub toggle: PlainFlag,
    /// The championship row highlighted.
    pub champ_row: PlainDword,
    /// 1 once the options file has been read.
    pub options_loaded: PlainFlag,
    /// The address of the menu sprites' glyph table.
    pub sprite_table: PlainDword,
    /// `LoopFlags.menu.0`: The current menu (by handle, `menus_store::MenuId`).
    pub menu: crate::menu::menus_store::MenuCell,
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
