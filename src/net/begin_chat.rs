//! `net::begin_chat` (0x4180C5): Enter on the lobby's chat item starts a message: the input
//! (`byte_52A4C8`) emptied and edited (`net::begin_edit`, the item `g_listChoice`, width
//! 0x30C, measured with `dword_51E424`).
//!
//! In:  nothing
//! Out: eax = the item, ebx = the input, ecx = 0x30C, edx = `dword_51E424`'s address; no
//!      `pusha`
//! Caller: `net::lobby_loop`.

use crate::install::vars::INSTALLER_PAGES;
use crate::net::store::NET;

const CHAT_FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::CHAT_LINE;

/// The Rust version. The registers the original leaves (eax the item, ebx the input, ecx
/// 0x30C, edx the font) are read by no caller (`lobby_loop` only; liveness: ebp, edi, edx,
/// esi live after, edx the caller's own).
pub fn begin_chat() -> Result<(), crate::errors::Refused> {
    NET.chat_input.set(0, 0);
    crate::net::begin_edit::begin_edit(
        INSTALLER_PAGES.choice.get(),
        NET.chat_input.addr() as u32,
        0x30C,
        CHAT_FONT.addr() as u32,
    )
}
