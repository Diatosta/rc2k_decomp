//! `net::begin_edit` (0x416E1A): an item's text starts being edited: the item (`dword_52A4B8`),
//! the text (`dword_52A4B4`), the widest it may be (`dword_52A4BC`) and the font it is
//! measured with (`dword_52A4C0`) noted, and the text's 64 bytes kept (`byte_95EAF8`) for
//! Escape.
//!
//! In:  `item` (eax), `text` (ebx), `width` (ecx), `font` (edx)
//! Out: nothing (every register kept, `pusha` .. `popa`)
//! Callers: `net::begin_setting_edit`, `net::begin_chat`.

use crate::net::lobby::{EDIT_SAVED, a, movsb};
use crate::net::store::NET;

/// The Rust version.
pub fn begin_edit(item: u32, text: u32, width: u32, font: u32) {
    NET.edited.set(item);
    NET.edit_text.set(text);
    NET.edit_width.set(width);
    NET.edit_font.set(font);
    movsb(a(EDIT_SAVED), text, 0x40);
}
