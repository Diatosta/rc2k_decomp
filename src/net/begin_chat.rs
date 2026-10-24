//! `net::begin_chat` (0x4180C5): Enter on the lobby's chat item starts a message: the input
//! (`byte_52A4C8`) emptied and edited (`net::begin_edit`, the item `g_listChoice`, width
//! 0x30C, measured with `dword_51E424`).
//!
//! In:  nothing
//! Out: eax = the item, ebx = the input, ecx = 0x30C, edx = `dword_51E424`'s address; no
//!      `pusha`
//! Caller: `net::lobby_loop`.

use crate::net::lobby::{CHAT_INPUT, CHOICE, a, g, pb};

const CHAT_FONT: usize = 0x51E424; // dword_51E424

/// The Rust version. The registers the original leaves (eax the item, ebx the input, ecx
/// 0x30C, edx the font) are read by no caller (`lobby_loop` only; liveness: ebp, edi, edx,
/// esi live after, edx the caller's own).
pub fn begin_chat() {
    pb(CHAT_INPUT, 0);
    crate::net::begin_edit::begin_edit(g(CHOICE), a(CHAT_INPUT), 0x30C, a(CHAT_FONT));
}
