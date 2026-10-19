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

fn i(v: u32) -> i32 {
    v as i32
}

fn add(at: u32, v: u32) {
    wd(at, rd(at).wrapping_add(v));
}

pub fn place_row(entry: &Regs) -> Result<Regs, Refused> {
    let (first, count, the_box) = (entry.ebx, entry.ecx, entry.esi);
    if count == 0 {
        return Err(crate::errors::BadInput("a box of no pieces").into());
    }
    let mut eax = entry.eax;
    // The widest label (414F15).
    let mut ebp = 0u32;
    let (mut ebx, mut ecx) = (first, count);
    loop {
        let kind = rb(ebx + 1);
        if kind & 1 != 0 {
            if i(ebp) < i(rd(ebx + 0x10)) {
                ebp = rd(ebx + 0x10);
            }
        } else if kind & 2 != 0 {
            if ecx == 1 {
                break;
            }
            if rb(ebx + 0x31) & 2 != 0 {
                eax = rd(ebx + 8).wrapping_add(rd(ebx + 0x10)).wrapping_add(0xA);
                if i(eax) > i(rd(ebx + 0x38)) {
                    wd(ebx + 0x38, eax);
                }
            }
        }
        ebx = ebx.wrapping_add(PIECE);
        ecx = ecx.wrapping_sub(1);
        if ecx == 0 {
            break;
        }
    }
    let labels = ebp;
    // The wrap (414F4D .. 414FAB).
    let (mut ebx, mut ecx) = (first, count);
    let edi = rd(the_box + 8).wrapping_sub(labels);
    let mut line = first;
    #[derive(PartialEq)]
    enum At {
        Test,
        Wrap,
        Taller,
        Height,
        Next,
    }
    let mut state = At::Height;
    loop {
        state = match state {
            At::Test => {
                if rb(ebx + 1) & 2 == 0 {
                    At::Wrap
                } else {
                    eax = rd(ebx + 8).wrapping_add(rd(ebx + 0x10));
                    if i(eax) <= i(edi) {
                        At::Taller
                    } else {
                        eax = rd(ebx + 8);
                        At::Wrap
                    }
                }
            }
            At::Wrap => {
                let mut at = ebx;
                for _ in 0..ecx {
                    if rb(at + 1) & 2 == 0 {
                        eax = 0;
                    }
                    wd(at + 8, rd(at + 8).wrapping_sub(eax));
                    add(at + 0xC, ebp);
                    at = at.wrapping_add(PIECE);
                }
                let mut edx = line;
                loop {
                    eax = ebp.wrapping_sub(rd(edx + 0x14));
                    add(edx + 0xC, eax);
                    edx = edx.wrapping_add(PIECE);
                    if edx == ebx {
                        break;
                    }
                }
                line = ebx;
                At::Height
            }
            At::Taller => {
                if i(ebp) >= i(rd(ebx + 0x14)) {
                    At::Next
                } else {
                    At::Height
                }
            }
            At::Height => {
                ebp = rd(ebx + 0x14);
                At::Next
            }
            At::Next => {
                ebx = ebx.wrapping_add(PIECE);
                ecx = ecx.wrapping_sub(1);
                if ecx == 0 {
                    break;
                }
                At::Test
            }
        };
    }
    let _ = state == At::Next;
    let ebp = labels;
    // Across (414FB8).
    let (mut ebx, mut ecx) = (first, count);
    let mut edi = 0u32;
    loop {
        let kind = rb(ebx + 1);
        if kind & 1 != 0 {
            add(ebx + 8, ebp);
        } else if kind & 2 != 0 {
            eax = rd(ebx + 8).wrapping_add(ebp).wrapping_add(0xA);
            wd(ebx + 8, eax);
            eax = eax.wrapping_add(rd(ebx + 0x10));
            if i(edi) < i(eax) {
                edi = eax;
            }
        } else {
            let centred = rb(ebx) & 0x20 != 0 || kind & 0xC == 0;
            if centred {
                eax = (i(rd(the_box + 8)) >> 1) as u32;
                add(ebx + 8, eax);
            } else if kind & 4 == 0 {
                eax = rd(the_box + 8).wrapping_sub(1);
                add(ebx + 8, eax);
            }
        }
        ebx = ebx.wrapping_add(PIECE);
        ecx = ecx.wrapping_sub(1);
        if ecx == 0 {
            break;
        }
    }
    // The choices centred across, the height found (415007).
    let (mut ebx, mut ecx) = (first, count);
    let ebp = (i(rd(the_box + 8).wrapping_sub(edi)) >> 1) as u32;
    let mut edi = 0u32;
    loop {
        if rb(ebx + 1) & 3 != 0 {
            add(ebx + 8, ebp);
        }
        eax = rd(ebx + 0xC).wrapping_add(rd(ebx + 0x14));
        if i(edi) < i(eax) {
            edi = eax;
        }
        ebx = ebx.wrapping_add(PIECE);
        ecx = ecx.wrapping_sub(1);
        if ecx == 0 {
            break;
        }
    }
    // The whole moved to the box (415032).
    eax = rd(the_box);
    let mut edx = (i(rd(the_box + 0xC).wrapping_sub(edi)) >> 1) as u32;
    if the_box == a(TITLE_BOX) {
        edx &= !1;
    }
    edx = edx.wrapping_add(rd(the_box + 4));
    let mut ebx = first;
    for _ in 0..count {
        add(ebx + 8, eax);
        add(ebx + 0xC, edx);
        ebx = ebx.wrapping_add(PIECE);
    }
    Ok(Regs {
        eax,
        ebx,
        ecx: 0,
        edx,
        edi,
        ebp,
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
