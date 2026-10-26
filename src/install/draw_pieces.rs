//! `install::draw_pieces` (0x415058): each of a page's pieces drawn at its place. A plain
//! text piece (no 0x10 in its kind) its string (`text::draw_string_by_id`) in the font of
//! its marks (`byte_51D528`: bits 6, 7 of its first byte, its kind and 0x10); else by the
//! item's kind: an edit its text (`text::draw_text`) in the font of its record's kind and
//! marks, with the caret (string 4) after it while it is edited, lit and its record has
//! bit 1; a text its string and the item's appended text in `text`; an action its drawing
//! action (`[item+0xC]`, ecx and edx the place).
//!
//! In:  nothing (the pieces at `dword_51C407`)
//! Out: every register kept (`pusha` / `popa`); no flags read
//! No pieces would go round 2^32 times: refused. Caller `install::run_page` (the surface
//! locked). Its calls through `part`; the appending done here. Shadow mode runs the
//! original alone. Self-test: a copy whose draws go to stand-ins, against the Rust version,
//! on pages laid out by `install::lay_out_pieces` from lists of the test's.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::Regs;
use crate::install::installer::{
    COLOURS, DRAW_STRING, DRAW_TEXT, FONTS, LAYOUT, Part, TEXT, a, append_byte,
    append_string_by_id, at, g, rb, rd, rw, str_append,
};
use crate::install::lay_out_pieces::PIECE;
use crate::race::viewer::regs_entry;
use rc2k_core::install::pieces::{self, Way};

fn font(index: u32) -> u32 {
    rd(a(FONTS) + 4 * index)
}

fn colour(marks: u8, kind: u32) -> u32 {
    rb(a(COLOURS) + pieces::colour_at(marks, kind)) as u32
}

pub fn draw_pieces(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let count = g(LAYOUT);
    if count == 0 {
        return Err(BadInput("a page of no pieces").into());
    }
    let mut r = *entry;
    r.esi = a(LAYOUT) + 4;
    r.ecx = count;
    loop {
        let left = r.ecx;
        let esi = r.esi;
        let first = rb(esi);
        let marks = (first >> 6) as u32 & 3;
        // Converted: `rc2k_core::install::pieces` (`Way`, `colour_at`, `caret`).
        match Way::of(first, rb(esi + 1)) {
            Way::String { font_kind } => {
                r.ebx = rb(a(COLOURS) + pieces::colour_at(first, font_kind)) as u32;
                r.eax = rw(esi + 2) as u32;
                r.ecx = rd(esi + 8);
                r.edx = rd(esi + 0xC);
                r.ebx = font(r.ebx);
                part(at(DRAW_STRING), &mut r)?;
            }
            way => {
                r.eax = r.eax & !0xFF | (first & 0xF) as u32;
                match way {
                    Way::Edit => {
                        r.edi = rd(esi + 4);
                        r.ebx = marks;
                        r.ecx = rd(r.edi + 0xC);
                        let record = r.ecx;
                        r.eax = colour(first, rb(record + 4) as u32);
                        r.ebx = rd(r.edi + 8);
                        if pieces::caret(first, rb(record)) {
                            let mut s = Regs {
                                eax: r.ebx,
                                ebx: a(TEXT),
                                ..r
                            };
                            str_append(&mut s);
                            s.eax = 4;
                            append_string_by_id(&mut s);
                            append_byte(&mut s, 0);
                            r.ebx = a(TEXT);
                            r.edx = s.edx;
                        }
                        r.ecx = rd(esi + 8);
                        r.edx = rd(esi + 0xC);
                        r.eax = font(r.eax);
                        part(at(DRAW_TEXT), &mut r)?;
                    }
                    Way::Text => {
                        r.edi = rd(esi + 4);
                        r.eax = rw(esi + 2) as u32;
                        r.ebx = a(TEXT);
                        append_string_by_id(&mut r);
                        let action = rd(r.edi + 0xC);
                        if action != 0 {
                            r.eax = rd(r.edi + 8);
                            part(action, &mut r)?;
                        }
                        append_byte(&mut r, 0);
                        r.eax = colour(first, rb(r.edi + 1) as u32);
                        r.ebx = a(TEXT);
                        r.ecx = rd(esi + 8);
                        r.edx = rd(esi + 0xC);
                        r.eax = font(r.eax);
                        part(at(DRAW_TEXT), &mut r)?;
                    }
                    Way::Action => {
                        r.edi = rd(esi + 4);
                        r.ecx = rd(esi + 8);
                        r.edx = rd(esi + 0xC);
                        part(rd(r.edi + 0xC), &mut r)?;
                    }
                    _ => {}
                }
            }
        }
        r.ecx = left.wrapping_sub(1);
        r.esi = esi.wrapping_add(PIECE);
        if r.ecx == 0 {
            break;
        }
    }
    Ok(*entry)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| {
        draw_pieces(r, &mut crate::install::installer::game_part)
    })
}
