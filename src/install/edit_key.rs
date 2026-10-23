//! `install::edit_key` (0x4148F7): a key while an edit item (kind 4) is being edited.
//! - ah bit 0: al is the character itself;
//! - Escape (1): the edit ended (record bit 0 off) and the text put back from
//!   `byte_95EAF8` (its most bytes and two); ah gets bit 3;
//! - Enter (0x1C): the edit ended; a text of nothing but spaces and control bytes is cleared
//!   (its most bytes and two) and the record's default text (`[record+2]`, a string id)
//!   put in; ah gets bit 3;
//! - 0x6E: the text cleared; Backspace (0xE): its last byte cut;
//! - the four arrows: nothing;
//! - any other: the key's character (`input::key_to_char`, with `dword_4F07F8` as the
//!   layout while `byte_4F07D2` is 2) added if it is not below a space and there is room
//!   (the most bytes), and cut again if the text is then wider than `[record+6]`
//!   (`text::measure_text` with the font of the record's kind: `byte_51D52D`, `dword_51D4C4`).
//!
//! In:  al the key, ah its bits, esi the item (+8 the text, +0xC the record)
//! Out: esi, edi, ebp kept; zero set and carry clear, but for Escape and Enter (the flags of
//!      `or ah, 8`: zero clear); eax, ebx, ecx, edx as each way leaves them
//! Caller `install::page_key`. Shadow mode compares (the measuring only writes the font's
//! two results). Self-test: an edit of the test's and the game's fonts, against the
//! original, every key's way.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::bridge::{Raw, read_through};
use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{
    COLOURS, FONTS, SCRIPT_PATH, a, append_string_by_id, g, logic, rb, rd, rw, wb,
};
use crate::race::viewer::regs_entry;
use rc2k_core::install::edits::{self, EDITING, EditKey};

const LAYOUT_MODE: usize = 0x4F07D2; // byte_4F07D2
const LAYOUT: usize = 0x4F07D4; // g_keyboardLayout
const OTHER_LAYOUT: usize = 0x4F07F8; // dword_4F07F8

/// The font of an edit record's kind.
pub fn edit_font(record: u32) -> u32 {
    let kind = rb(record + 4) as u32;
    let font = rb(a(COLOURS) + 5 + 8 * kind) as u32;
    rd(a(FONTS) + 4 * font)
}

fn past_nul(mut at: u32) -> u32 {
    loop {
        let b = rb(at);
        at = at.wrapping_add(1);
        if b < 1 {
            return at;
        }
    }
}

fn clear(text: u32, most: u32, r: &mut Regs) {
    for n in 0..most + 2 {
        wb(text + n, 0);
    }
    r.ecx = 0;
}

pub fn edit_key(entry: &Regs) -> Result<Regs, Refused> {
    let mut r = *entry;
    let text = rd(r.esi + 8);
    let record = rd(r.esi + 0xC);
    let most = rb(record + 1) as u32;
    let key = r.eax as u8;
    let done = |mut r: Regs| {
        r.set_zero(true);
        r.set_carry(false);
        Ok(r)
    };
    let with_bit_3 = |mut r: Regs| {
        r.eax |= 0x800;
        let ah = (r.eax >> 8) & 0xFF;
        logic(&mut r, ah);
        Ok(r)
    };
    let backspace = |mut r: Regs| {
        r.ebx = text;
        if rb(text) != 0 {
            r.ebx = past_nul(text);
            wb(r.ebx.wrapping_sub(2), 0);
        }
        done(r)
    };
    // Converted: `rc2k_core::install::edits` (`EditKey`, `blank`, `takes`, `too_wide`).
    let ch = if r.eax & 0x100 != 0 {
        key
    } else {
        match EditKey::of(key) {
            EditKey::Escape => {
                wb(record, rb(record) & !EDITING);
                let backup = a(SCRIPT_PATH);
                for n in 0..most + 2 {
                    wb(text + n, rb(backup + n));
                }
                r.ecx = 0;
                return with_bit_3(r);
            }
            EditKey::Enter => {
                wb(record, rb(record) & !EDITING);
                let words = unsafe { Raw::new(text as *const u8) };
                let (blank, stop) = read_through(edits::blank(&words));
                r.ebx = text.wrapping_add(stop as u32);
                if !blank {
                    return with_bit_3(r);
                }
                clear(text, most, &mut r);
                let mut s = Regs {
                    eax: rw(record + 2) as u32,
                    ebx: text,
                    ..r
                };
                append_string_by_id(&mut s);
                r.ebx = s.ebx;
                return with_bit_3(r);
            }
            EditKey::Clear => {
                clear(text, most, &mut r);
                return done(r);
            }
            EditKey::Backspace => return backspace(r),
            EditKey::Arrow => return done(r),
            EditKey::Other => {
                let swap = rb(a(LAYOUT_MODE)) == 2;
                let kept = g(LAYOUT);
                if swap {
                    crate::install::installer::p(LAYOUT, a(OTHER_LAYOUT));
                }
                let out = unsafe { crate::input::key_to_char::key_to_char(r.eax) };
                if swap {
                    crate::install::installer::p(LAYOUT, kept);
                }
                out? as u8
            }
        }
    };
    r.edx = r.edx & !0xFF | ch as u32;
    if ch < 0x20 {
        return done(r);
    }
    r.ebx = past_nul(text);
    r.ecx = most.wrapping_add(text).wrapping_add(1);
    let length = r.ebx.wrapping_sub(text).wrapping_sub(1) as usize;
    if !edits::takes(ch, length, most as u8) {
        return done(r);
    }
    wb(r.ebx.wrapping_sub(1), ch);
    wb(r.ebx, 0);
    let mut m = Regs {
        eax: edit_font(record),
        ebx: text,
        ..r
    };
    crate::install::installer::game_part(a(crate::install::installer::MEASURE_TEXT), &mut m)?;
    r.ebx = m.ebx;
    r.ecx = m.ecx;
    r.edx = m.edx;
    r.eax = rw(record + 6) as u32;
    if edits::too_wide(r.ecx, r.eax as u16) {
        return backspace(r);
    }
    done(r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &edit_key)
}
