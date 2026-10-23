//! `install::measure_pieces` (0x414DF5): each of a page's pieces measured into its width
//! (+0x10) and height (+0x14). A piece whose item has 0x20 first has bits 0, 1 of its kind
//! moved to bits 2, 3. A piece of a plain text kind (no 0x10) is its string measured
//! (`text::measure_string_by_id`) in the font of its marks and kind (`byte_51D52D`,
//! `dword_51D4C4`); else by the item's kind: an edit its record's width and its font's
//! height (`byte_51D52F`); a text its string and the item's appended text (`[item+0xC]`
//! with eax `[item+8]`, if any) in `text`, measured (`text::measure_text`); an action its
//! box's size; any other what ecx and edx hold.
//!
//! In:  nothing (the pieces at `dword_51C407`)
//! Out: every register kept (`pusha` / `popa`); no flags read
//! No pieces would go round 2^32 times: refused. Caller `install::run_page`. Its calls
//! through `part` (the measuring, and the items' text actions); the appending done here.
//! Shadow mode runs the original alone (an item's action is the game's). Self-test: a copy
//! whose two measures go to stand-ins (answering sizes), against the Rust version, on
//! pages laid out by `install::lay_out_pieces` from lists of the test's.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::Regs;
use crate::install::installer::{
    COLOURS, FONTS, LAYOUT, MEASURE_STRING, MEASURE_TEXT, TEXT, a, append_byte,
    append_string_by_id, at, g, rb, rd, rw, wb, wd,
};
use crate::install::lay_out_pieces::PIECE;
use crate::race::viewer::regs_entry;
use rc2k_core::install::pieces::{self, Way};

fn font_of(index: u32) -> u32 {
    rd(a(FONTS) + 4 * index)
}

pub fn measure_pieces(entry: &Regs) -> Result<Regs, Refused> {
    let count = g(LAYOUT);
    if count == 0 {
        return Err(BadInput("a page of no pieces").into());
    }
    let mut r = *entry;
    r.eax = 0;
    r.esi = a(LAYOUT) + 4;
    r.ecx = count;
    loop {
        let left = r.ecx;
        let esi = r.esi;
        // Converted: `rc2k_core::install::pieces` (`measured_kind`, `Way`,
        // `measure_colour_at`).
        let marks = rb(esi);
        if marks & 0x20 != 0 {
            wb(esi + 1, pieces::measured_kind(marks, rb(esi + 1)));
        }
        match Way::of(marks, rb(esi + 1)) {
            Way::String { font_kind } => {
                r.eax = font_kind;
                let font = rb(a(COLOURS) + pieces::measure_colour_at(font_kind)) as u32;
                r.eax = rw(esi + 2) as u32;
                r.ebx = font_of(font);
                crate::install::installer::game_part(at(MEASURE_STRING), &mut r)?;
            }
            way => {
                r.eax = r.eax & !0xFF | (marks & 0xF) as u32;
                match way {
                    Way::Edit => {
                        r.edi = rd(esi + 4);
                        let record = rd(r.edi + 0xC);
                        r.ecx = rw(record + 6) as u32;
                        r.edx = rb(a(COLOURS) + 7 + 8 * rb(record + 4) as u32) as u32;
                    }
                    Way::Text => {
                        r.edi = rd(esi + 4);
                        r.eax = rw(esi + 2) as u32;
                        r.ebx = a(TEXT);
                        append_string_by_id(&mut r);
                        let action = rd(r.edi + 0xC);
                        if action != 0 {
                            r.eax = rd(r.edi + 8);
                            crate::install::installer::game_part(action, &mut r)?;
                        }
                        append_byte(&mut r, 0);
                        let kind = rb(r.edi + 1) as u32;
                        let font = rb(a(COLOURS) + pieces::measure_colour_at(kind)) as u32;
                        r.ebx = a(TEXT);
                        r.eax = font_of(font);
                        crate::install::installer::game_part(at(MEASURE_TEXT), &mut r)?;
                    }
                    Way::Action => {
                        r.edi = rd(esi + 4);
                        r.eax = rd(r.edi + 4);
                        r.ecx = rd(r.eax + 8);
                        r.edx = rd(r.eax + 0xC);
                    }
                    _ => {}
                }
            }
        }
        wd(esi + 0x10, r.ecx);
        wd(esi + 0x14, r.edx);
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
    regs_entry(r, &|r| measure_pieces(r))
}
