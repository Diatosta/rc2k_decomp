//! `net::add_chat_line` (0x419428): a message added to the lobby's chat (`byte_52A508`, eight
//! lines): "name> text" built in `word_52A888` (the player's name from its block of
//! `g_netReceived`), then cut into lines no wider than 0x30C (`text::measure_text` with
//! `dword_51E424`, one character more at a time into `byte_52A808`), each line going in at
//! the bottom and the top one dropped.
//!
//! In:  `player` (eax), `message` (ebx; nothing if empty). The original also reads ecx: the
//!      x of its first `text::measure_text`, which in measure mode changes only the x the
//!      measure leaves, not the width it reads (`rc2k_core::text::draw`); dropped.
//! Out: nothing (every register kept, `pusha` .. `popa`)
//! A character wider than a line alone leaves the line empty and the text where it was: the
//! original never ends then (refused, as is a chat with no line end in its 0x300 bytes).
//! Callers: `net::send_chat` and `net::chat_line_locked` (each with the received messages'
//! lock held).

use crate::errors::{BadInput, Refused};
use crate::net::lobby::a;
use crate::net::store::NET;
use crate::strings::str_append::str_append;
use crate::text::draw_text::GameCanvas;
use crate::text::measure_text::measure_text_at;

const CHAT_FONT: usize = 0x51E424; // dword_51E424
const LINE_WIDTH: u32 = 0x30C;
const CHAT_LEN: u32 = 0x300;

fn byte(at: u32) -> u8 {
    unsafe { (at as *const u8).read() }
}

fn set_byte(at: u32, v: u8) {
    unsafe { (at as *mut u8).write(v) }
}

/// The Rust version.
pub fn add_chat_line(player: u32, message: *const u8) -> Result<(), Refused> {
    if byte(message as u32) == 0 {
        return Ok(());
    }
    // "name> text"
    let mut cursor = unsafe {
        str_append(
            NET.player_block(player).name.addr() as *const u8,
            (NET.chat_text.addr() as u32) as *mut u8,
        )
    };
    unsafe {
        (cursor as *mut u16).write_unaligned(0x203E);
        cursor = str_append(message, cursor.add(2));
        cursor.write(0);
    }
    let mut text = NET.chat_text.addr() as u32;
    loop {
        // The next line: one character more at a time while it fits.
        let mut end = NET.chat_line.addr() as u32;
        loop {
            let c = byte(text);
            set_byte(end, c);
            if c == 0 {
                break;
            }
            text += 1;
            end += 1;
            set_byte(end, 0);
            let width = unsafe {
                measure_text_at(
                    a(CHAT_FONT) as *mut _,
                    (NET.chat_line.addr() as u32) as *const u8,
                    0,
                    &mut GameCanvas,
                )
            }?
            .width;
            if width > LINE_WIDTH {
                set_byte(end - 1, 0);
                text -= 1;
                break;
            }
        }
        if byte(NET.chat_line.addr() as u32) == 0 && byte(text) != 0 {
            return Err(
                BadInput("a character wider than a chat line: the original never ends").into(),
            );
        }
        // The top line dropped, the rest moved up.
        let mut from = NET.chat.addr() as u32;
        let mut to = from;
        loop {
            if from >= (NET.chat.addr() as u32) + CHAT_LEN {
                return Err(BadInput("a chat with no line end").into());
            }
            let c = byte(from);
            from += 1;
            if c == 0x0A {
                break;
            }
        }
        loop {
            let c = byte(from);
            from += 1;
            set_byte(to, c);
            to += 1;
            if c == 0 {
                break;
            }
        }
        // The new line at the bottom.
        from = NET.chat_line.addr() as u32;
        set_byte(to - 1, 0x0A);
        loop {
            let c = byte(from);
            from += 1;
            set_byte(to, c);
            to += 1;
            if c == 0 {
                break;
            }
        }
        if byte(text) == 0 {
            return Ok(());
        }
    }
}
