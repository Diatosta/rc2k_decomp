//! `net::add_chat_line` (0x419428): a message added to the lobby's chat (`byte_52A508`, eight
//! lines): "name> text" built in `word_52A888` (the player's name from its block of
//! `g_netReceived`), then cut into lines no wider than 0x30C (`text::measure_text` with
//! `dword_51E424`, one character more at a time into `byte_52A808`), each line going in at
//! the bottom and the top one dropped.
//!
//! In:  eax = the player, ebx = the message (nothing if empty)
//! Out: every register kept (`pusha` .. `popa`)
//! A character wider than a line alone leaves the line empty and the text where it was: the
//! original never ends then (refused, as is a chat with no line end in its 0x300 bytes).
//! Callers: `net::send_chat` and `net::chat_line_locked` (each with the received messages'
//! lock held). Its calls through `part` (`net::lobby_calls`: the Rust versions;
//! `strings::append_nul` inline). Shadow mode runs the original alone. Self-test: a copy
//! whose calls go to stand-ins (`str_append` done, the width one character's 0x50 times
//! their number), against the Rust version: an empty message, a short one, one of three
//! lines.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::lobby::{
    CHAT, CHAT_LINE, CHAT_TEXT, MEASURE_TEXT, Part, STR_APPEND, Saved, a, answer, block,
    dispatch_lobby, fill, put_text, region, run_both,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::add_chat_line", rva(0x419428), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const CHAT_FONT: usize = 0x51E424; // dword_51E424
const LINE_WIDTH: u32 = 0x30C;
const CHAT_LEN: u32 = 0x300;

fn byte(at: u32) -> u8 {
    unsafe { (at as *const u8).read() }
}

fn set_byte(at: u32, v: u8) {
    unsafe { (at as *mut u8).write(v) }
}

/// The Rust version: the registers it leaves (the entry's).
pub fn add_chat_line(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    if byte(r.ebx) == 0 {
        return Ok(*entry);
    }
    r.esi = r.ebx;
    r.eax = block(r.eax);
    r.ebx = a(CHAT_TEXT);
    part(STR_APPEND, &mut r)?;
    unsafe { (r.ebx as *mut u16).write_unaligned(0x203E) };
    r.ebx += 2;
    r.eax = r.esi;
    part(STR_APPEND, &mut r)?;
    set_byte(r.ebx, 0);
    r.ebx += 1;
    r.ebp = a(CHAT_TEXT);
    let low = |r: &mut Regs, c: u8| r.eax = r.eax & !0xFF | c as u32;
    loop {
        r.edi = a(CHAT_LINE);
        loop {
            let c = byte(r.ebp);
            low(&mut r, c);
            set_byte(r.edi, c);
            if c == 0 {
                break;
            }
            r.ebp += 1;
            r.edi += 1;
            set_byte(r.edi, 0);
            r.eax = a(CHAT_FONT);
            r.ebx = a(CHAT_LINE);
            part(MEASURE_TEXT, &mut r)?;
            if r.ecx > LINE_WIDTH {
                set_byte(r.edi - 1, 0);
                r.ebp -= 1;
                break;
            }
        }
        if byte(a(CHAT_LINE)) == 0 && byte(r.ebp) != 0 {
            return Err(
                BadInput("a character wider than a chat line: the original never ends").into(),
            );
        }
        // The top line dropped, the rest moved up.
        r.esi = a(CHAT);
        r.edi = r.esi;
        loop {
            if r.esi >= a(CHAT) + CHAT_LEN {
                return Err(BadInput("a chat with no line end").into());
            }
            let c = byte(r.esi);
            r.esi += 1;
            low(&mut r, c);
            if c == 0x0A {
                break;
            }
        }
        loop {
            let c = byte(r.esi);
            r.esi += 1;
            set_byte(r.edi, c);
            r.edi += 1;
            low(&mut r, c);
            if c == 0 {
                break;
            }
        }
        r.esi = a(CHAT_LINE);
        set_byte(r.edi - 1, 0x0A);
        loop {
            let c = byte(r.esi);
            r.esi += 1;
            set_byte(r.edi, c);
            r.edi += 1;
            low(&mut r, c);
            if c == 0 {
                break;
            }
        }
        if byte(r.ebp) == 0 {
            break;
        }
    }
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_lobby(&HOOK, r, call_original, add_chat_line);
}

/// The test's answers: `str_append` done; a line's width 0x50 a character.
pub fn chat_answers() -> crate::net::lobby::Answer {
    Box::new(|place, r| match place {
        STR_APPEND => {
            r.ebx = unsafe {
                crate::strings::str_append::str_append(r.eax as *const u8, r.ebx as *mut u8)
            } as u32
        }
        MEASURE_TEXT => {
            let mut n = 0;
            while byte(r.ebx + n) != 0 {
                n += 1;
            }
            r.eax = 0;
            r.ecx = 0x50 * n;
            r.ebx = 1;
            r.edx = 0x22;
        }
        _ => {}
    })
}

/// The chat as a test starts it: "a\nb\nc\nd\ne\nf\ng\nh".
pub fn test_chat() {
    put_text(CHAT, b"a\nb\nc\nd\ne\nf\ng\nh");
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; cmp byte ptr [ebx], 0; jz loc_4194CA
    let copy = match crate::net::lobby::copy(
        0x419428,
        0xA4,
        &[0x60, 0x80, 0x3B, 0x00, 0x0F, 0x84, 0x98, 0x00, 0x00, 0x00],
        &[0x1E, 0x2D, 0x32, 0x5D],
        &[STR_APPEND, MEASURE_TEXT],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let all = [
        region(CHAT, CHAT_LEN as usize),
        region(CHAT_LINE, 0x80),
        region(CHAT_TEXT, 0x80),
        region(0x52AD9C, 0x20),
    ];
    let saved = Saved::new(&all);
    for message in [&b"\0"[..], b"hi\0", b"a message of twenty-five\0"] {
        run_both(
            &mut t,
            &format!("{:?}", String::from_utf8_lossy(message)),
            copy.addr(),
            Regs {
                eax: 0,
                ebx: message.as_ptr() as u32,
                ..Regs::SENTINEL
            },
            &all,
            0,
            &[],
            &|| {
                fill(CHAT, CHAT_LEN as usize, 7);
                fill(CHAT_LINE, 0x80, 8);
                fill(CHAT_TEXT, 0x80, 9);
                test_chat();
                put_text(0x52AD9C, b"Ann");
                answer(chat_answers());
            },
            &add_chat_line,
        );
    }
    drop(saved);
    t.finish()
}
