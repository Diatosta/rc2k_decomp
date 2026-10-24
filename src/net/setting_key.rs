//! `net::setting_key` (0x416C76): a key for the setting being edited (`net::edit_key`); when
//! that ends the editing: the player's name (item 1) left empty gets its old text back
//! (`byte_95EAF8`, 64 bytes); the most players (item 2) read from its text
//! (`settings::parse_number`), held to 2 .. 8 (signed), and to 4 with `g_noticeKind` 2, into
//! `dword_52ABD0`.
//!
//! In:  the key
//! Out: as `net::edit_key` leaves them while editing goes on; after it, eax = the item (or
//!      the number), the copy's esi, edi, ecx; no `pusha`; the flags of the last test (no
//!      caller reads them)
//! Caller: `net::settings_loop`. Its calls through `part`. Shadow mode runs the original
//! alone. Self-test: a copy whose calls go to stand-ins (the editing ended or not, the
//! number given), against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::net::lobby::{
    CHOICE, EDIT_SAVED, MACHINE_NAME, MOST_PLAYERS, NUMBER_TEXT, a, g, gb, movsb, p,
};
use rc2k_core::net::chat;

/// The Rust version: the key `key` for the setting being edited. The registers the
/// original leaves (the item or the number in eax, the copy's esi, edi and ecx) are read by
/// no caller (`settings_loop`).
pub fn setting_key(key: u8) -> Result<(), Refused> {
    if !crate::net::edit_key::edit_key(key)? {
        return Ok(());
    }
    // Converted: `rc2k_core::net::chat` (`NAME`, `MOST_PLAYERS`, `most_players`).
    match g(CHOICE) {
        chat::NAME => {
            if gb(MACHINE_NAME) == 0 {
                movsb(a(MACHINE_NAME), a(EDIT_SAVED), 0x40);
            }
        }
        chat::MOST_PLAYERS => {
            let number =
                unsafe { crate::settings::parse_number::parse_number(a(NUMBER_TEXT) as *const u8) };
            p(
                MOST_PLAYERS,
                chat::most_players(number, crate::strings::load_language::notice_kind()),
            );
        }
        _ => {}
    }
    Ok(())
}
