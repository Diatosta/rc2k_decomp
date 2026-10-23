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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{a, rb, rd, wd};
use crate::install::lay_out_pieces::PIECE;
use crate::race::viewer::regs_entry;
use rc2k_core::install::layout::{self, Area, Piece};

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

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &place_row)
}
