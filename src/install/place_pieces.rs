//! `install::place_pieces` (0x414ED1): a page's pieces placed, box by box: the pieces in a
//! row whose items share a box (the item's +4: `{x, y, width, height}`; none: the title's
//! box `unk_51DD98`) go to `install::place_row` together.
//!
//! In:  nothing (the pieces at `dword_51C407`)
//! Out: every register kept (`pusha` / `popa`); no flags read
//! No pieces would go round 2^32 times: refused. Caller `install::run_page`. Shadow mode
//! compares. Self-test: pages laid out by `install::lay_out_pieces` from lists of the
//! test's, measured as the test says, against the original.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{LAYOUT, a, g, rd, region, wd};
use crate::install::lay_out_pieces::{MOST_PIECES, PIECE};
use crate::install::list_test::TestList;
use crate::race::viewer::{Saved, check_original, dispatch_regs, regs_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::place_pieces", rva(0x414ED1), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const TITLE_BOX: usize = 0x51DD98; // unk_51DD98

pub fn place_pieces(entry: &Regs) -> Result<Regs, Refused> {
    let base = a(LAYOUT) + 4;
    let at = |k: u32| base.wrapping_add(PIECE * k);
    // Converted: `rc2k_core::install::layout::rows` (each piece's box read through its item).
    let rows = rc2k_core::install::layout::rows(g(LAYOUT), |k| rd(rd(at(k) + 4) + 4))?;
    let mut r = *entry;
    for (first, n, the_box) in rows {
        let place = if the_box == 0 { a(TITLE_BOX) } else { the_box };
        r = crate::install::place_row::place_row(&Regs {
            eax: the_box,
            ebx: at(first),
            ecx: n,
            esi: place,
            ..r
        })?;
    }
    Ok(*entry)
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = [region(LAYOUT, 4 + (PIECE * MOST_PIECES) as usize)];
    dispatch_regs(&HOOK, r, call_original, &regions, 0, &place_pieces);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &place_pieces)
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
    let pages: [&[u8]; 4] = [&[1, 2, 3, 4, 5, 6], &[1, 1, 1], &[2, 0x22, 4, 0x14], &[6]];
    for kinds in pages {
        for flags in [0u8, 0x40] {
            for boxes in [0u32, 1, 2] {
                let prepare = || {
                    list.set(flags, kinds, 3, 0, 0, 0, 100, b"");
                    // the items' boxes: in the spare of the list's memory
                    let spare = list.base() + 0x500;
                    for k in 0..kinds.len() as u32 {
                        let the_box = match boxes {
                            0 => 0,
                            1 => spare,
                            _ => spare + 0x10 * (k % 2),
                        };
                        wd(list.item(k) + 4, the_box);
                    }
                    for (k, v) in [40u32, 100, 400, 300, 10, 20, 200, 90].iter().enumerate() {
                        wd(spare + 4 * k as u32, *v);
                    }
                    let _ = crate::install::lay_out_pieces::lay_out_pieces(&Regs {
                        edi: list.base(),
                        ..Regs::default()
                    });
                    // sizes as a measuring would give them
                    for p in 0..g(LAYOUT) {
                        let at = a(LAYOUT) + 4 + PIECE * p;
                        wd(at + 0x10, 20 + 7 * p);
                        wd(at + 0x14, 10 + p % 3);
                    }
                };
                prepare();
                let before = Regs::SENTINEL;
                check_original(
                    &mut t,
                    &format!("kinds {kinds:x?}, flags {flags:#x}, boxes {boxes}"),
                    call_original,
                    before,
                    &regions,
                    0,
                    &prepare,
                    &place_pieces,
                );
            }
        }
    }
    drop(saved);
    t.finish()
}
