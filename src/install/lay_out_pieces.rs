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
use crate::race::viewer::{Saved, check_original, dispatch_regs, regs_entry};
use rc2k_core::install::layout;
use rc2k_core::install::pieces::{self, LayoutError, ListMemory};
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

pub const PIECE: u32 = layout::PIECE;
pub const MOST_PIECES: u32 = pieces::MOST_PIECES;
const TITLE: usize = 0x51DD78; // unk_51DD78: the title's item
const TEXT_FONTS: usize = 0x51D52C; // byte_51D52C

/// The game's lists, read where the original reads them.
struct GameList;

impl ListMemory for GameList {
    fn byte(&self, at: u32) -> u8 {
        rb(at)
    }
    fn word(&self, at: u32) -> u16 {
        rw(at)
    }
    fn dword(&self, at: u32) -> u32 {
        crate::install::installer::rd(at)
    }
    fn text_font(&self, font: u8) -> u8 {
        rb(a(TEXT_FONTS) + 8 * font as u32)
    }
}

pub fn lay_out_pieces(entry: &Regs) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::install::pieces::lay_out`; the pieces written here.
    let made = match pieces::lay_out(entry.edi, a(TITLE), &GameList) {
        Ok(made) => made,
        Err(LayoutError::UnknownKind) => return Err(fatal(pieces::UNKNOWN_KIND)),
        Err(LayoutError::TooMany) => {
            return Err(BadInput("more pieces than the page's table holds").into());
        }
    };
    let base = a(LAYOUT);
    for (n, piece) in made.iter().enumerate() {
        let at = base + 4 + PIECE * n as u32;
        if let Some(text) = piece.text {
            ww(at + 2, text);
        }
        wb(at, piece.marks);
        wb(at + 1, piece.kind);
        wd(at + 4, piece.item);
        for k in 0..4 {
            wd(at + 8 + 4 * k, 0);
        }
    }
    wd(base, made.len() as u32);
    Ok(*entry)
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = [region(LAYOUT, 4 + (PIECE * MOST_PIECES) as usize)];
    dispatch_regs(&HOOK, r, call_original, &regions, 0, &lay_out_pieces);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &lay_out_pieces)
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
