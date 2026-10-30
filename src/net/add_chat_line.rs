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
use crate::net::store::NET;
use crate::pointers::GamePtr;
use crate::session::Bytes;
use crate::strings::str_append::str_append;
use crate::text::draw_text::GameCanvas;
use crate::text::measure_text::measure_text_at;

const CHAT_FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::CHAT_LINE;
const LINE_WIDTH: u32 = 0x30C;
const CHAT_LEN: usize = 0x300;

/// Byte `i` of `b` (refused past its end).
fn at<const N: usize>(b: &Bytes<N>, i: usize) -> Result<u8, BadInput> {
    if i < N {
        Ok(b.get(i))
    } else {
        Err(BadInput("past the end of a chat buffer"))
    }
}

/// Byte `i` of `b` set to `v` (refused past its end).
fn put<const N: usize>(b: &Bytes<N>, i: usize, v: u8) -> Result<(), BadInput> {
    if i < N {
        b.set(i, v);
        Ok(())
    } else {
        Err(BadInput("past the end of a chat buffer"))
    }
}

/// The Rust version.
pub fn add_chat_line(player: u32, message: GamePtr) -> Result<(), Refused> {
    if message.read::<u8>("the chat's text")? == 0 {
        return Ok(());
    }
    // "name> text"
    let cursor = str_append(
        GamePtr::to(&NET.player_block(player).name),
        GamePtr::to(&NET.chat_text),
    )?;
    cursor.write(0x203Eu16, "the chat's text")?;
    let cursor = str_append(message, cursor.offset(2))?;
    cursor.write(0u8, "the chat's text")?;
    let (line, chat, said) = (&NET.chat_line, &NET.chat, &NET.chat_text);
    let mut text = 0usize;
    loop {
        // The next line: one character more at a time while it fits.
        let mut end = 0usize;
        loop {
            let c = at(said, text)?;
            put(line, end, c)?;
            if c == 0 {
                break;
            }
            text += 1;
            end += 1;
            put(line, end, 0)?;
            let width = {
                measure_text_at(
                    CHAT_FONT.game_ptr(),
                    crate::pointers::GamePtr::to(line),
                    0,
                    &mut GameCanvas,
                )
            }?
            .width;
            if width > LINE_WIDTH {
                put(line, end - 1, 0)?;
                text -= 1;
                break;
            }
        }
        if at(line, 0)? == 0 && at(said, text)? != 0 {
            return Err(
                BadInput("a character wider than a chat line: the original never ends").into(),
            );
        }
        // The top line dropped, the rest moved up.
        let (mut from, mut to) = (0usize, 0usize);
        loop {
            if from >= CHAT_LEN {
                return Err(BadInput("a chat with no line end").into());
            }
            let c = at(chat, from)?;
            from += 1;
            if c == 0x0A {
                break;
            }
        }
        loop {
            let c = at(chat, from)?;
            from += 1;
            put(chat, to, c)?;
            to += 1;
            if c == 0 {
                break;
            }
        }
        // The new line at the bottom.
        put(chat, to - 1, 0x0A)?;
        let mut from = 0usize;
        loop {
            let c = at(line, from)?;
            from += 1;
            put(chat, to, c)?;
            to += 1;
            if c == 0 {
                break;
            }
        }
        if at(said, text)? == 0 {
            return Ok(());
        }
    }
}
