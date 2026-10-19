//! `install::lay_out_pieces` (0x414C7E): a page's pieces (`dword_51C407` of them, 0x30 bytes
//! each after it: +0 the item's kind and marks, +1 the piece's kind, +2 its text, +4 its
//! item, +8 x, +0xC y, +0x10 width, +0x14 height) made from its list. The title (unless the
//! list's flags are 0x40 of 0xC0: `test; jg`) is kind 0x10 of `unk_51DD78`; then each item
//! not hidden (0x40 of 0xC0): a link a piece of kind 0; a choice a label (1) and a piece
//! (2) for each of its choices not hidden, its marks from the choice's high bits; a value
//! a label (1) and a piece (0x12); an edit the same, without the label when the item has
//! 0x10; a text kind 0x10 with its font's bits (`byte_51D52C`); an action 0x14. The places
//! and sizes are zeroed.
//!
//! In:  edi the list
//! Out: every register kept (`pusha` / `popa`); no flags read
//! Any other kind is fatal error 0x80A97451. More pieces than the 0xFFC bytes hold (85) is
//! refused (the original writes over the list variables after them). Caller
//! `install::run_page`. Shadow mode compares. Self-test: lists of the test's against the
//! original (the fatal kind only on the Rust version).

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{LAYOUT, a, fatal, rb, region, rw, wb, wd, ww};
use crate::install::list_test::TestList;
use crate::race::viewer::{Saved, check_original, dispatch_regs};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::lay_out_pieces", rva(0x414C7E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const PIECE: u32 = 0x30;
pub const MOST_PIECES: u32 = 0xFFC / PIECE;
const TITLE: usize = 0x51DD78; // unk_51DD78: the title's item
const TEXT_FONTS: usize = 0x51D52C; // byte_51D52C

fn hidden(b: u8) -> bool {
    b & 0xC0 == 0x40
}

/// The pieces a list makes, or the fatal error of a kind it does not know.
fn count(list: u32) -> Result<u32, Refused> {
    let mut n = if hidden(rb(list)) { 0 } else { 1 };
    let items = match rb(list + 1) {
        0 => 0x100,
        c => c as u32,
    };
    for k in 0..items {
        let item = list + 0x18 + 0x10 * k;
        let b = rb(item);
        if hidden(b) {
            continue;
        }
        n += match b & 0xF {
            1 | 3 | 5 | 6 => 1 + (b & 0xF != 1 && b & 0xF != 5 && b & 0xF != 6) as u32,
            4 => 1 + (b & 0x10 == 0) as u32,
            2 => {
                let choices = match rb(item + 1) {
                    0 => 0x100,
                    c => c as u32,
                };
                let table = crate::install::installer::rd(item + 0xC);
                1 + (0..choices)
                    .filter(|&c| !hidden(rb(table + 2 * c + 1)))
                    .count() as u32
            }
            _ => return Err(fatal(0x80A9_7451)),
        };
    }
    Ok(n)
}

fn piece(at: u32, kind: u8, piece_kind: u8, text: u16, item: u32) {
    ww(at + 2, text);
    second_piece(at, kind, piece_kind, item);
}

/// A value's or an edit's second piece: its text word is not written (it keeps what the
/// table held there).
fn second_piece(at: u32, kind: u8, piece_kind: u8, item: u32) {
    wb(at, kind);
    wb(at + 1, piece_kind);
    wd(at + 4, item);
    for k in 0..4 {
        wd(at + 8 + 4 * k, 0);
    }
}

pub fn lay_out_pieces(entry: &Regs) -> Result<Regs, Refused> {
    let list = entry.edi;
    if count(list)? > MOST_PIECES {
        return Err(BadInput("more pieces than the page's table holds").into());
    }
    let base = a(LAYOUT);
    let mut n = 0u32;
    let at = |n: &mut u32| {
        let place = base + 4 + PIECE * *n;
        *n += 1;
        place
    };
    if !hidden(rb(list)) {
        piece(at(&mut n), 5, 0x10, rw(list + 2), a(TITLE));
    }
    let items = match rb(list + 1) {
        0 => 0x100,
        c => c as u32,
    };
    for k in 0..items {
        let item = list + 0x18 + 0x10 * k;
        let b = rb(item);
        if hidden(b) {
            continue;
        }
        let text = rw(item + 2);
        match b & 0xF {
            1 => piece(at(&mut n), b, 0, text, item),
            2 => {
                piece(at(&mut n), b, 1, text, item);
                let marks = b & 0x3F;
                let table = crate::install::installer::rd(item + 0xC);
                let choices = match rb(item + 1) {
                    0 => 0x100,
                    c => c as u32,
                };
                for c in 0..choices {
                    let word = rw(table + 2 * c);
                    let high = (word >> 8) as u8 & 0xC0;
                    if high == 0x40 {
                        continue;
                    }
                    piece(
                        at(&mut n),
                        high | marks,
                        2,
                        word - ((high as u16) << 8),
                        item,
                    );
                }
            }
            3 => {
                piece(at(&mut n), b, 1, text, item);
                second_piece(at(&mut n), b, 0x12, item);
            }
            4 => {
                let label = at(&mut n);
                piece(label, b, 1, text, item);
                if b & 0x10 != 0 {
                    n -= 1;
                }
                let place = base + 4 + PIECE * n;
                n += 1;
                second_piece(place, b & 0xEF, 0x12, item);
            }
            5 => {
                let place = at(&mut n);
                piece(place, b, 0, text, item);
                let font = rb(a(TEXT_FONTS) + 8 * rb(item + 1) as u32);
                wb(place + 1, font | 0x10);
            }
            _ => piece(at(&mut n), b, 0x14, text, item),
        }
    }
    wd(base, n);
    Ok(*entry)
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = [region(LAYOUT, 4 + (PIECE * MOST_PIECES) as usize)];
    dispatch_regs(&HOOK, r, call_original, &regions, 0, &lay_out_pieces);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let list = TestList::new();
    let regions = [
        region(LAYOUT, 4 + (PIECE * MOST_PIECES) as usize),
        list.region(),
    ];
    let saved = Saved::new(&regions);
    // Stale text words in the table, as the second pieces keep them.
    crate::install::installer::fill(a(LAYOUT), 4 + (PIECE * MOST_PIECES) as usize, 0x5A);
    let pages: [&[u8]; 6] = [
        &[1, 2, 3, 4, 5, 6],
        &[0x41, 0x82, 0xC3, 0x14, 0x44, 0x26],
        &[2, 2, 0x42],
        &[4, 0x14, 0x84],
        &[5],
        &[0x46, 0x41],
    ];
    for kinds in pages {
        for flags in [0u8, 0x40, 0x80, 0xC0, 0x10] {
            for (choices, marks) in [(3u8, 0u32), (5, 0b10100), (2, 0b11)] {
                let prepare = || {
                    list.set(flags, kinds, choices, 0, 0, 0, 100, b"");
                    // choices hidden by their high bits (0x40 of 0xC0)
                    for k in 0..kinds.len() as u32 {
                        if kinds[k as usize] & 0xF == 2 {
                            let table = crate::install::installer::rd(list.item(k) + 0xC);
                            for c in 0..0x10u32 {
                                let high = if marks >> c & 1 != 0 {
                                    0x40
                                } else {
                                    (c as u8 & 2) << 6
                                };
                                wb(table + 2 * c + 1, high | 2);
                            }
                        }
                    }
                };
                prepare();
                let before = Regs {
                    edi: list.base(),
                    ..Regs::SENTINEL
                };
                check_original(
                    &mut t,
                    &format!(
                        "flags {flags:#x}, kinds {kinds:x?}, {choices} choices, hidden {marks:#b}"
                    ),
                    call_original,
                    before,
                    &regions,
                    0,
                    &prepare,
                    &lay_out_pieces,
                );
            }
        }
    }
    // A kind it does not know: the fatal error (the original's would end the game).
    list.set(0, &[1, 7], 0, 0, 0, 0, 100, b"");
    let refused = lay_out_pieces(&Regs {
        edi: list.base(),
        ..Regs::SENTINEL
    });
    t.check(
        matches!(refused, Err(Refused::Fatal(f)) if f.code == 0x80A9_7451),
        || format!("kind 7: {refused:?}", refused = refused.map(|_| ())),
    );
    drop(saved);
    t.finish()
}
