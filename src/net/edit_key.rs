//! `net::edit_key` (0x416E43): a key for the text being edited (`net::begin_edit`), if one
//! is (`dword_52A4B8` not -1): Escape (1) puts the kept text back and ends the editing,
//! Enter (0x1C) ends it, 0x6E clears the text's 64 bytes, backspace (0x0E) drops its last
//! character; any other key is made a character (`input::key_to_char`), which, if it is one
//! (0x20 or more), goes at the text's end (if it has fewer than 0x3F) with a NUL after it,
//! and is taken off again if the text is then wider (`text::measure_text` with the editing's
//! font, signed) than allowed.
//!
//! In:  al = the key
//! Out: every register kept (`pusha` .. `popa`); the flags of `cmp dword_52A4B8, -1`: zero
//!      set when no text is edited (any more)
//! Callers: `net::setting_key`, `net::chat_key`. Its calls through `part` (`net::lobby_calls`:
//! the Rust versions). Shadow mode runs the original alone (the dead key's state). Self-test:
//! a copy whose calls go to stand-ins (the character and the width given), against the Rust
//! version: not editing, each key, a character below 0x20, too wide, a full text.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::bridge::{Raw, read_through};
use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::{
    EDIT_FONT, EDIT_SAVED, EDIT_TEXT, EDIT_WIDTH, EDITED, KEY_TO_CHAR, MEASURE_TEXT, Part, a, g,
    lobby_entry, movsb, p, set_sign,
};
use rc2k_core::net::editing::{self, EditKey};

/// The flags of `cmp dword_52A4B8, -1`.
pub fn editing_flags(r: &mut Regs) {
    let v = g(EDITED);
    r.set_zero(v == u32::MAX);
    r.set_carry(v != u32::MAX);
    set_sign(r, v.wrapping_add(1) as i32);
}

fn set_byte(at: u32, v: u8) {
    unsafe { (at as *mut u8).write(v) }
}

/// The Rust version: the registers it leaves (the entry's, the flags of the last compare).
pub fn edit_key(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    // Converted: `rc2k_core::net::editing` (`EditKey`, `last_char`, `takes`,
    // `end_for_char`, `too_wide`).
    if g(EDITED) != editing::NOT_EDITING {
        let text = g(EDIT_TEXT);
        let words = unsafe { Raw::new(text as *const u8) };
        r.ebx = text;
        match EditKey::of(r.eax as u8) {
            EditKey::Escape => {
                movsb(text, a(EDIT_SAVED), editing::TEXT_LEN as u32);
                p(EDITED, editing::NOT_EDITING);
            }
            EditKey::Enter => p(EDITED, editing::NOT_EDITING),
            EditKey::Clear => unsafe {
                std::ptr::write_bytes(text as *mut u8, 0, editing::TEXT_LEN)
            },
            EditKey::Backspace => {
                if let Some(last) = read_through(editing::last_char(&words)) {
                    set_byte(text + last as u32, 0);
                }
            }
            EditKey::Other => {
                part(KEY_TO_CHAR, &mut r)?;
                if editing::takes(r.eax as u8) {
                    r.eax &= !0xFF00;
                    if let Some(at) = read_through(editing::end_for_char(&words)) {
                        let at = at as u32;
                        r.ecx = at;
                        unsafe { ((text + at) as *mut u16).write_unaligned(r.eax as u16) };
                        let mut m = Regs {
                            eax: g(EDIT_FONT),
                            ..r
                        };
                        part(MEASURE_TEXT, &mut m)?;
                        if editing::too_wide(m.ecx, g(EDIT_WIDTH)) {
                            set_byte(text + at, 0);
                        }
                    }
                }
            }
        }
    }
    let mut left = Regs {
        eflags: r.eflags,
        ..*entry
    };
    editing_flags(&mut left);
    Ok(left)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, edit_key)
}
