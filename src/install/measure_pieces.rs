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

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{
    COLOURS, FONTS, LAYOUT, MEASURE_STRING, MEASURE_TEXT, Part, TEXT, a, answer, append_byte,
    append_string_by_id, at, both, copy, g, quiet, rb, rd, region, rw, stand_in, wb, wd,
};
use crate::install::lay_out_pieces::{MOST_PIECES, PIECE};
use crate::install::list_test::TestList;
use crate::race::viewer::{Saved, dispatch_parts};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::measure_pieces", rva(0x414DF5), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn font_of(index: u32) -> u32 {
    rd(a(FONTS) + 4 * index)
}

pub fn measure_pieces(entry: &Regs, part: Part) -> Result<Regs, Refused> {
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
        if rb(esi) & 0x20 != 0 {
            let kind = rb(esi + 1);
            let low = kind & 3;
            wb(esi + 1, (low << 2) | (kind - low));
        }
        if rb(esi + 1) & 0x10 == 0 {
            r.eax = (rb(esi) & 0x10) as u32 | (rb(esi + 1) & 0xF) as u32;
            let font = rb(a(COLOURS) + 5 + 8 * r.eax) as u32;
            r.eax = rw(esi + 2) as u32;
            r.ebx = font_of(font);
            part(at(MEASURE_STRING), &mut r)?;
        } else {
            let kind = rb(esi) & 0xF;
            r.eax = r.eax & !0xFF | kind as u32;
            match kind {
                4 => {
                    r.edi = rd(esi + 4);
                    let record = rd(r.edi + 0xC);
                    r.ecx = rw(record + 6) as u32;
                    r.edx = rb(a(COLOURS) + 7 + 8 * rb(record + 4) as u32) as u32;
                    r.edx = (r.edx as u8) as u32;
                }
                5 => {
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
                    let font = rb(a(COLOURS) + 5 + 8 * rb(r.edi + 1) as u32) as u32;
                    r.ebx = a(TEXT);
                    r.eax = font_of(font);
                    part(at(MEASURE_TEXT), &mut r)?;
                }
                6 => {
                    r.edi = rd(esi + 4);
                    r.eax = rd(r.edi + 4);
                    r.ecx = rd(r.eax + 8);
                    r.edx = rd(r.eax + 0xC);
                }
                _ => {}
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

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        measure_pieces(r, &mut crate::install::installer::game_part)
    });
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; xor eax, eax; lea esi, dword_51C407
    let copy = match copy(
        0x414DF5,
        0xDC,
        &[0x60, 0x33, 0xC0, 0x8D, 0x35, 0x07, 0xC4, 0x51, 0x00],
        &[
            (0x4A, 0x40232C),
            (0x82, 0x401FA3),
            (0x93, 0x401F16),
            (0xB0, 0x40230B),
        ],
        &[0x40232C, 0x40230B],
        &[],
        &[],
        &[0xC0DE_0005],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let list = TestList::new();
    let regions = [
        region(LAYOUT, 4 + (PIECE * MOST_PIECES) as usize),
        list.region(),
        region(TEXT, 0x100),
    ];
    let saved = Saved::new(&regions);
    let pages: [&[u8]; 4] = [
        &[1, 2, 3, 4, 5, 6],
        &[0x21, 0x22, 0x14],
        &[5, 5],
        &[0x26, 0x23],
    ];
    for kinds in pages {
        for with_action in [false, true] {
            let prepare = || {
                list.set(0, kinds, 2, 0, 0, 0, 77, b"");
                let spare = list.base() + 0x500;
                for (k, v) in [40u32, 100, 400, 300].iter().enumerate() {
                    wd(spare + 4 * k as u32, *v);
                }
                for k in 0..kinds.len() as u32 {
                    let item = list.item(k);
                    wd(item + 4, spare);
                    if kinds[k as usize] & 0xF == 5 {
                        wb(item + 1, (k % 3) as u8);
                        wd(item + 8, 0x1234);
                        wd(item + 0xC, if with_action { stand_in(2) } else { 0 });
                    }
                }
                let _ = crate::install::lay_out_pieces::lay_out_pieces(&Regs {
                    edi: list.base(),
                    ..Regs::default()
                });
                answer(Box::new(|place, r| {
                    if place != 0xC0DE_0005 {
                        r.ecx = 30 + (r.eax & 0x3F);
                        r.edx = 12 + (r.ebx & 3);
                    }
                }));
            };
            both(
                &mut t,
                &format!("kinds {kinds:x?}, text action {with_action}"),
                &copy,
                Regs::SENTINEL,
                &regions,
                0,
                &[],
                &prepare,
                &|r, part, _| measure_pieces(r, part),
            );
        }
    }
    quiet();
    drop(saved);
    t.finish()
}
