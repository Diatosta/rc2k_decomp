//! `net::lobby_frame` (0x417D95): the frame of the lobby (`net::lobby_loop`): on the locked
//! surface the background; the table's headings (strings 0x15F at 0x21A and 0x160 at 0x2C0,
//! `stru_51E564`, y 0x6F); the players sorted (`net::sort_players`) and one row each from y
//! 0x8C: the nationality's flag (sprite +0x27 + 0x1C at 0x18), the name (`stru_51E784` when
//! ready, else `stru_51E6C4`), the car's sprite (+0x25 + 0x44, 0x18 and 0x19 swapped), the
//! race's time and the total (`text::draw_time_text` with the hours always shown, or string
//! 0x161 for no time; with gaps shown, `dword_52AC30` 1, each as its gap to the first row's
//! with a '+' over its first character when within 0x36EE7F); the chat's eight lines (under
//! the received messages' lock) and its input (`net::chat_input_line`, item 3); the four
//! buttons right to left from 0x310 at 0x223 (`net::lobby_button`: ready 0x15D, the car 0x15B
//! and the stage 0x15C when they can be chosen, the chat 0x15E); two lines (`gfx::draw_hline`
//! at y 0x156 and 0x217, 0x320 long, colour 7); unlocked and shown.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! More than eight players would read past the sorted copy: refused. Called by
//! `screens::run_frame` for `net::lobby_loop`. Its calls through `part`
//! (`strings::append_nul` inline). Shadow mode runs the original alone (it draws).
//! Self-test: a copy whose calls go to stand-ins (the texts written), against the Rust
//! version: the lock failing; no players; three players with and without gaps, ready or
//! not, times and none; the buttons' switches.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::field;
use crate::framework::Regs;
use crate::net::lobby::{
    APPEND_STRING, APPEND_TIME, BLIT, BLOCK, CAR_ITEM, CHAT, CHAT_INPUT, COPY_PICTURE, DRAW_HLINE,
    DRAW_STRING, DRAW_TEXT, DRAW_TIME, FONT, LOCK, LOCK_RECEIVED, PLAYERS, Part, SHOW_FRAME,
    SHOW_GAPS, SORTED, SPRITES, STAGE_ITEM, STR_APPEND, TEXT, UNLOCK, UNLOCK_RECEIVED, a, g,
    lobby_entry, p,
};
use crate::net::types::NetPlayerBlock;
use rc2k_core::net::players::{self, READY};
use rc2k_core::net::screens;

const HEADING_FONT: usize = 0x51E564; // stru_51E564
const NAME_FONT: usize = 0x51E6C4; // stru_51E6C4
const READY_FONT: usize = 0x51E784; // stru_51E784
const CHAT_FONT: usize = 0x51E584; // stru_51E584
const SHOW_HOURS: usize = 0x4E0148; // g_timeShowHours
const FONT_PALETTES: usize = 0x51DEB0; // g_fontPalettes
const SORT_PLAYERS: u32 = 0x418108; // net::sort_players
const CHAT_INPUT_LINE: u32 = 0x418083; // net::chat_input_line
const LOBBY_BUTTON: u32 = 0x41802F; // net::lobby_button

fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

fn byte(at: u32) -> u32 {
    unsafe { (at as *const u8).read() as u32 }
}

fn nul(r: &mut Regs) {
    unsafe { (r.ebx as *mut u8).write(0) };
    r.ebx = r.ebx.wrapping_add(1);
}

/// A time, or its gap to `first` with a '+' over its first character, into the text buffer:
/// eax the time; the registers as the original leaves them before its `append_nul`.
fn time_text(r: &mut Regs, gaps: bool, first: u32, part: Part) -> Result<(), Refused> {
    let (value, plus) = screens::shown_time(r.eax, first, gaps);
    r.eax = value;
    if plus {
        r.ecx = r.ebx;
        part(APPEND_TIME, r)?;
        unsafe { (r.ecx as *mut u8).write(b'+') };
        return Ok(());
    }
    part(APPEND_TIME, r)
}

/// The Rust version: the registers it leaves (the entry's).
pub fn lobby_frame(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    // Converted: `rc2k_core::net::screens` (`shown_time`, `flag_sprite`, `car_sprite`,
    // `NO_TIME`, `lobby_buttons`), the ready bit and no time of `rc2k_core::net::players`.
    let done = |r: &Regs| {
        Ok(Regs {
            eflags: r.eflags,
            ..*entry
        })
    };
    part(LOCK, &mut r)?;
    if r.carry() {
        return done(&r);
    }
    if g(PLAYERS) > 8 {
        return Err(BadInput("more than eight players").into());
    }
    part(COPY_PICTURE, &mut r)?;
    r.ebp = 0x6F;
    for (id, x) in [(0x15F, 0x21A), (0x160, 0x2C0)] {
        r.eax = id;
        r.ebx = a(HEADING_FONT);
        r.ecx = x;
        r.edx = r.ebp;
        part(DRAW_STRING, &mut r)?;
    }
    r.ebp += 0x1D;
    if g(PLAYERS) != 0 {
        part(SORT_PLAYERS, &mut r)?;
        let gaps = g(SHOW_GAPS) == 1;
        let (first_time, first_total) = (get(a(SORTED) + 0x2C), get(a(SORTED) + 0x30));
        for n in 0..g(PLAYERS) {
            r.eax = n;
            let block = a(SORTED).wrapping_add(n.wrapping_mul(BLOCK));
            r.esi = block;
            r.edi = 0x18;
            let blit = |r: &mut Regs, glyph: u32, y: u32, part: Part| -> Result<(), Refused> {
                r.eax = glyph;
                r.ecx = r.edi;
                r.edx = y;
                r.esi = g(SPRITES);
                part(BLIT, r)?;
                r.esi = block;
                Ok(())
            };
            let y = r.ebp;
            blit(
                &mut r,
                screens::flag_sprite(byte(block + 0x27) as u8),
                y,
                part,
            )?;
            r.edi += 0x28;
            r.eax = block;
            r.ebx = a(TEXT);
            part(STR_APPEND, &mut r)?;
            nul(&mut r);
            r.eax = if byte(block + field!(NetPlayerBlock, record.state)) as u8 & READY != 0 {
                a(READY_FONT)
            } else {
                a(NAME_FONT)
            };
            r.ebx = a(TEXT);
            r.ecx = r.edi;
            r.edx = r.ebp;
            part(DRAW_TEXT, &mut r)?;
            r.edi += 0x14A;
            let y = r.ebp + 1;
            blit(
                &mut r,
                screens::car_sprite(byte(block + 0x25) as u8),
                y,
                part,
            )?;
            r.edi += 0x48;
            let kept_hours = g(SHOW_HOURS);
            p(SHOW_HOURS, 1);
            r.eax = get(block + 0x2C);
            r.ebx = a(TEXT);
            if r.eax == players::NO_TIME {
                r.eax = screens::NO_TIME;
                part(APPEND_STRING, &mut r)?;
                nul(&mut r);
                r.eax = a(NAME_FONT);
                r.ebx = a(TEXT);
                r.ecx = r.edi;
                r.edx = r.ebp;
                part(DRAW_TEXT, &mut r)?;
            } else {
                time_text(&mut r, gaps, first_time, part)?;
                nul(&mut r);
                r.eax = a(FONT);
                r.ebx = a(TEXT);
                r.ecx = r.edi;
                r.edx = r.ebp;
                part(DRAW_TIME, &mut r)?;
            }
            r.edi += 0xA5;
            r.eax = get(block + 0x30);
            r.ebx = a(TEXT);
            let timed = get(block + 0x2C) != players::NO_TIME;
            time_text(&mut r, gaps && timed, first_total, part)?;
            nul(&mut r);
            r.eax = a(FONT);
            r.ebx = a(TEXT);
            r.ecx = r.edi;
            part(DRAW_TIME, &mut r)?;
            p(SHOW_HOURS, kept_hours);
            r.ebp = r.ebp.wrapping_add(get(a(NAME_FONT) + 8));
            r.eax = n + 1;
        }
    }
    r.ebp = 0x158;
    part(LOCK_RECEIVED, &mut r)?;
    r.eax = a(CHAT_FONT);
    r.ebx = a(CHAT);
    r.ecx = 0xA;
    r.edx = r.ebp;
    part(DRAW_TEXT, &mut r)?;
    part(UNLOCK_RECEIVED, &mut r)?;
    r.eax = get(a(CHAT_FONT) + 8).wrapping_mul(8);
    r.ebp = r.ebp.wrapping_add(r.eax);
    r.esi = 3;
    r.eax = a(CHAT_INPUT);
    part(CHAT_INPUT_LINE, &mut r)?;
    r.ebp = 0x223;
    r.edi = 0x310;
    r.esi = 0;
    for (n, (id, shown)) in screens::lobby_buttons(g(CAR_ITEM), g(STAGE_ITEM))
        .into_iter()
        .enumerate()
    {
        if shown {
            r.eax = id;
            part(LOBBY_BUTTON, &mut r)?;
        }
        if n != 3 {
            r.esi += 1;
        }
    }
    r.eax = 7;
    r.ebx = 0x320;
    r.ecx = 0;
    r.edx = 0x156;
    r.esi = g(FONT_PALETTES);
    part(DRAW_HLINE, &mut r)?;
    r.edx = 0x217;
    part(DRAW_HLINE, &mut r)?;
    part(UNLOCK, &mut r)?;
    part(SHOW_FRAME, &mut r)?;
    done(&r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, lobby_frame)
}
