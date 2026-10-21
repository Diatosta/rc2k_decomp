//! `install::place_row` (0x414F11): the pieces of one box of a page (`install::place_pieces`:
//! the pieces of one item row record `{x, y, width, height}`) placed in it: labels (bit 0 of
//! the piece's kind) in a column as wide as the widest, choices (bit 1) after their label
//! and each other 10 pixels apart, wrapped to a new line where they would pass the box's
//! width, the rest centred (or, with bit 2, at 0, with bit 3, at the right edge); then the
//! whole centred across and down in the box (by an even number of pixels down in the
//! title's box `unk_51DD98`), moved to its place.
//!
//! In:  ebx the first piece, ecx the pieces, esi the box, eax as it comes (a value the
//!      wrap reads before setting it)
//! Out: eax the box's x, ebx past the pieces, ecx 0, edx the centring down, edi the height,
//!      ebp the centring across; esi kept; no flags read
//! Caller `install::place_pieces`. A loop target at +4: private copy, manual patch. Shadow
//! mode compares. Self-test: pieces of the test's (labels, choices, the rest, every bit)
//! in boxes narrow and wide, against the original.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{LAYOUT, a, fill, rb, rd, region, wb, wd};
use crate::install::lay_out_pieces::{MOST_PIECES, PIECE};
use crate::race::viewer::{Saved, check_original, dispatch_regs};
use rc2k_core::install::layout::{self, Area, Piece};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("install::place_row", rva(0x414F11), thunk, &ORIGINAL)
    .private_copy(0x147)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const TITLE_BOX: usize = 0x51DD98; // unk_51DD98

pub fn place_row(entry: &Regs) -> Result<Regs, Refused> {
    let (first, count, the_box) = (entry.ebx, entry.ecx, entry.esi);
    let at = |k: u32| first.wrapping_add(PIECE * k);
    // Converted: `rc2k_core::install::layout::place_row`, on the pieces' fields it reads;
    // their x and y written back.
    let mut pieces: Vec<Piece> = (0..count)
        .map(|k| Piece {
            marks: rb(at(k)),
            kind: rb(at(k) + 1),
            x: rd(at(k) + 8),
            y: rd(at(k) + 0xC),
            width: rd(at(k) + 0x10),
            height: rd(at(k) + 0x14),
        })
        .collect();
    let area = Area {
        x: rd(the_box),
        y: rd(the_box + 4),
        width: rd(the_box + 8),
        height: rd(the_box + 0xC),
    };
    let title = the_box == a(TITLE_BOX);
    let placed = layout::place_row(&mut pieces, area, title, entry.eax)?;
    for (k, p) in (0..).zip(&pieces) {
        wd(at(k) + 8, p.x);
        wd(at(k) + 0xC, p.y);
    }
    Ok(Regs {
        eax: placed.x,
        ebx: at(count),
        ecx: 0,
        edx: placed.down,
        edi: placed.height,
        ebp: placed.across,
        ..*entry
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = [region(LAYOUT, 4 + (PIECE * MOST_PIECES) as usize)];
    dispatch_regs(&HOOK, r, call_original, &regions, 0, &place_row);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = [region(LAYOUT, 4 + (PIECE * MOST_PIECES) as usize)];
    let saved = Saved::new(&regions);
    let base = a(LAYOUT) + 4;
    // (the pieces' kinds and their own bits, widths) in boxes of (width, height)
    let rows: [&[(u8, u8, u32, u32)]; 6] = [
        &[(0, 0, 80, 12)],
        &[
            (1, 0, 60, 12),
            (2, 0, 30, 10),
            (2, 0, 40, 10),
            (2, 0, 50, 14),
        ],
        &[(1, 0, 60, 12), (0x12, 0, 90, 12)],
        &[
            (0, 0x20, 30, 12),
            (4, 0, 30, 12),
            (8, 0, 30, 12),
            (0xC, 0, 10, 8),
            (0x10, 0, 70, 20),
        ],
        &[
            (1, 0, 20, 12),
            (2, 0, 90, 10),
            (2, 0, 90, 16),
            (2, 0, 90, 10),
            (1, 0, 70, 9),
            (2, 0, 30, 10),
        ],
        &[(2, 0, 30, 10), (2, 0, 40, 10)],
    ];
    for pieces in rows {
        for (width, height) in [(300u32, 200u32), (150, 61), (90, 7)] {
            for title in [false, true] {
                for eax in [0u32, 0x1234] {
                    let the_box = if title { a(TITLE_BOX) } else { base + 0xF00 };
                    if title && width != 300 {
                        continue;
                    }
                    let prepare = || {
                        fill(base, 0xF00, 0x33);
                        for (n, &(kind, own, w, h)) in pieces.iter().enumerate() {
                            let at = base + PIECE * n as u32;
                            wb(at, own);
                            wb(at + 1, kind);
                            wd(at + 8, 0);
                            wd(at + 0xC, 0);
                            wd(at + 0x10, w);
                            wd(at + 0x14, h);
                        }
                        if !title {
                            wd(the_box, 50);
                            wd(the_box + 4, 150);
                            wd(the_box + 8, width);
                            wd(the_box + 0xC, height);
                        }
                    };
                    let before = Regs {
                        eax,
                        ebx: base,
                        ecx: pieces.len() as u32,
                        esi: the_box,
                        ..Regs::SENTINEL
                    };
                    check_original(
                        &mut t,
                        &format!(
                            "pieces {pieces:?}, box {width} x {height}, title {title}, eax {eax:#x}"
                        ),
                        call_original,
                        before,
                        &regions,
                        0,
                        &prepare,
                        &place_row,
                    );
                }
            }
        }
    }
    drop(saved);
    t.finish()
}
