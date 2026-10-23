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

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{
    COLOURS, DRAW_STRING, DRAW_TEXT, FONTS, LAYOUT, Part, TEXT, a, append_byte,
    append_string_by_id, at, both, copy, g, quiet, rb, rd, region, rw, stand_in, str_append, wb,
    wd,
};
use crate::install::lay_out_pieces::{MOST_PIECES, PIECE};
use crate::install::list_test::TestList;
use crate::race::viewer::{Saved, dispatch_parts, regs_entry};
use rc2k_core::install::pieces::{self, Way};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::draw_pieces", rva(0x415058), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        draw_pieces(r, &mut crate::install::installer::game_part)
    });
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| {
        draw_pieces(r, &mut crate::install::installer::game_part)
    })
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; lea esi, dword_51C407
    let copy = match copy(
        0x415058,
        0x13B,
        &[0x60, 0x8D, 0x35, 0x07, 0xC4, 0x51, 0x00],
        &[
            (0x40, 0x40234A),
            (0x9E, 0x401F7D),
            (0xA8, 0x401FA3),
            (0xAD, 0x401F16),
            (0xC1, 0x4023D5),
            (0xD9, 0x401FA3),
            (0xEA, 0x401F16),
            (0x115, 0x4023D5),
        ],
        &[0x40234A, 0x4023D5],
        &[],
        &[],
        &[0xC0DE_0005, 0xC0DE_0006],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    quiet();
    let list = TestList::new();
    let regions = [
        region(LAYOUT, 4 + (PIECE * MOST_PIECES) as usize),
        list.region(),
        region(TEXT, 0x100),
    ];
    let saved = Saved::new(&regions);
    let pages: [&[u8]; 4] = [
        &[1, 2, 3, 4, 5, 6],
        &[0xC4, 0x84, 0x44, 0x14],
        &[5, 0x85],
        &[6, 0xC1],
    ];
    for kinds in pages {
        for edit in [0u8, 1, 3] {
            for with_action in [false, true] {
                let prepare = || {
                    list.set(0, kinds, 2, 0, 0, edit, 77, b"typed");
                    for k in 0..kinds.len() as u32 {
                        let item = list.item(k);
                        match kinds[k as usize] & 0xF {
                            5 => {
                                wb(item + 1, (k % 3) as u8);
                                wd(item + 8, 0x1234);
                                wd(item + 0xC, if with_action { stand_in(2) } else { 0 });
                            }
                            6 => wd(item + 0xC, stand_in(3)),
                            _ => {}
                        }
                    }
                    let _ = crate::install::lay_out_pieces::lay_out_pieces(&Regs {
                        edi: list.base(),
                        ..Regs::default()
                    });
                    for p in 0..g(LAYOUT) {
                        let at = a(LAYOUT) + 4 + PIECE * p;
                        wd(at + 8, 100 + p);
                        wd(at + 0xC, 200 + 3 * p);
                    }
                };
                both(
                    &mut t,
                    &format!("kinds {kinds:x?}, edit {edit}, text action {with_action}"),
                    &copy,
                    Regs::SENTINEL,
                    &regions,
                    0,
                    &[],
                    &prepare,
                    &|r, part, _| draw_pieces(r, part),
                );
            }
        }
    }
    drop(saved);
    t.finish()
}
