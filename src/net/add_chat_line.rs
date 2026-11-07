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
use crate::net::owner::Net;
use crate::net::owner::net;
use crate::session::Bytes;
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

/// "name> text" and its NUL, cut to `chat_text`'s room (the original runs on over what follows
/// it, as far as a sender's long message takes it).
fn said_text(name: &[u8], message: &[u8]) -> Vec<u8> {
    let mut text = [name, b"> ", message].concat();
    text.truncate(net().vars.chat_text.read().len() - 1);
    text.push(0);
    text
}

/// The text in `bytes`, up to its NUL (refused when none ends it).
pub fn text_in(bytes: &[u8]) -> Result<&[u8], BadInput> {
    bytes
        .iter()
        .position(|&b| b == 0)
        .map(|end| &bytes[..end])
        .ok_or(BadInput("a text without its end in its block"))
}

/// The Rust version.
pub fn add_chat_line(net: &Net, player: u32, message: &[u8]) -> Result<(), Refused> {
    let message = text_in(message)?;
    if message.is_empty() {
        return Ok(());
    }
    let text = said_text(&net.vars.player_block(player).name.text(), message);
    net.vars.chat_text.write_at(0, &text, "the chat's text")?;
    let (line, chat, said) = (&net.vars.chat_line, &net.vars.chat, &net.vars.chat_text);
    let mut text = 0usize;
    let line_room = line.read().len();
    loop {
        // The next line, built as a value: one character more at a time while it fits.
        // (The original builds it in `line`, a NUL after each character, and measures the
        // buffer; the same bytes are stored once the line is cut.)
        let mut built: Vec<u8> = Vec::new();
        let mut cut = false;
        loop {
            let c = at(said, text)?;
            if built.len() >= line_room {
                return Err(BadInput("past the end of a chat buffer").into());
            }
            if c == 0 {
                break;
            }
            if built.len() + 1 >= line_room {
                return Err(BadInput("past the end of a chat buffer").into());
            }
            text += 1;
            built.push(c);
            // Measured as the original measures `line`: this line's bytes, its NUL, and the
            // bytes earlier lines left after it (a control code's argument may be the NUL).
            let mut shown = built.clone();
            shown.push(0);
            let left = line.read();
            shown.extend_from_slice(left.get(shown.len()..).unwrap_or(&[]));
            let width = { measure_text_at(CHAT_FONT, &shown, 0, &mut GameCanvas) }?.width;
            if width > LINE_WIDTH {
                built.pop();
                text -= 1;
                cut = true;
                break;
            }
        }
        // The line as the original leaves it in `line`: its NUL, and a second one where the
        // character that did not fit was.
        built.push(0);
        if cut {
            built.push(0);
        }
        line.write_at(0, &built, "the chat's line")?;
        built.truncate(built.iter().position(|&b| b == 0).unwrap_or(0) + 1);
        if built[0] == 0 && at(said, text)? != 0 {
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
        for &c in &built {
            put(chat, to, c)?;
            to += 1;
        }
        if at(said, text)? == 0 {
            return Ok(());
        }
    }
}

#[cfg(test)]
mod said_tests {
    use super::said_text;

    #[test]
    fn a_line_is_kept_and_a_long_one_cut_to_its_room() {
        assert_eq!(said_text(b"Ana", b"hi"), b"Ana> hi\0");
        let long = said_text(&[b'n'; 31], &[b'm'; 255]);
        assert_eq!(long.len(), 128);
        assert_eq!(long.last(), Some(&0));
    }
}
