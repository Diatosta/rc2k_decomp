//! `install::mark_page` (0x414B53): a page's items marked for drawing, item by item (cl
//! counts them, `[edi+1]` of them, 256 for 0):
//! - a choice item (kind 2) marked 0x80 (not to be chosen) gives every unmarked choice
//!   0x80 too; one unmarked has its free choices counted, and with at most one its chosen
//!   byte becomes that one (`neg al; add al, [esi+1]`: with none, the choices less 2) and
//!   the item 0x80;
//! - the highlighted item (`g_listChoice`), if unmarked: an edit (kind 4) whose record has
//!   bit 2 is begun (`install::next_edit`, ebx 0), and an edit being edited, or any item
//!   while the clock's bit 3 (`dword_51D41F - dword_51D423`) is clear, gets 0xC0 (lit);
//! - a choice item's chosen choice, if it is one, gets 0xC0.
//!
//! In:  edi the list
//! Out: every register kept (`pusha` / `popa`); no flags read
//! Callers: `install::run_page` (twice). Shadow mode compares. Self-test: lists of the
//! test's (every kind, marks, highlights and clocks) against the original.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{
    CF, CHOICE, FRAME_CLOCK, MOVE_CLOCK, SCRIPT_PATH, ZF, g, list_regions, p, rb, rd, region, wb,
};
use crate::install::list_test::TestList;
use crate::race::viewer::{Saved, check_original, dispatch_regs, regs_entry};
use rc2k_core::install::items::{self, MARKS};
use rc2k_core::install::pages::UNUSABLE;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::mark_page", rva(0x414B53), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub fn mark_page(entry: &Regs) -> Regs {
    let list = entry.edi;
    let mut cl = 0u8;
    let mut esi = list + 0x18;
    // Converted: `rc2k_core::install::items` (`walked`, `mark_choices`, `lone_choice`,
    // `lit`).
    let table = |item: u32| unsafe {
        std::slice::from_raw_parts_mut(rd(item + 0xC) as *mut u8, 2 * items::walked(rb(item + 1)))
    };
    loop {
        let byte = rb(esi);
        if byte & 0xF == 2 {
            let ah = byte & MARKS;
            if ah == UNUSABLE {
                items::mark_choices(table(esi), ah);
            } else if ah == 0
                && let Some(chosen) = items::lone_choice(table(esi), rb(esi + 1))
            {
                wb(rd(esi + 8), chosen);
                wb(esi, rb(esi) | UNUSABLE);
            }
        }
        let byte = rb(esi);
        if byte & MARKS == 0 && cl as u32 == g(CHOICE) {
            let mut editing = None;
            if byte & 0xF == 4 {
                let record = rd(esi + 0xC);
                if rb(record) & 4 != 0 {
                    crate::install::next_edit::next_edit(&Regs {
                        ebx: 0,
                        edi: list,
                        ..*entry
                    });
                }
                editing = Some(rb(record) & 1 != 0);
            }
            let clock = || g(FRAME_CLOCK).wrapping_sub(g(MOVE_CLOCK));
            if editing == Some(true) || items::lit(editing, clock()) {
                wb(esi, rb(esi) | MARKS);
            }
        }
        if byte & 0xF == 2 {
            let chosen = rb(rd(esi + 8));
            if chosen < rb(esi + 1) {
                let at = rd(esi + 0xC)
                    .wrapping_add(1)
                    .wrapping_add(2 * chosen as u32);
                wb(at, rb(at) | MARKS);
            }
        }
        esi = esi.wrapping_add(0x10);
        cl = cl.wrapping_add(1);
        if cl == rb(list + 1) {
            break;
        }
    }
    let mut r = *entry;
    r.set_zero(true);
    r.set_carry(false);
    r
}

fn rust(r: &Regs) -> Result<Regs, Refused> {
    Ok(mark_page(r))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = list_regions(r.edi);
    dispatch_regs(&HOOK, r, call_original, &regions, 0, &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let list = TestList::new();
    let regions = [
        list.region(),
        region(CHOICE, 4),
        region(SCRIPT_PATH, 0x102),
        region(FRAME_CLOCK, 8),
    ];
    let saved = Saved::new(&regions);
    let pages: [&[u8]; 6] = [
        &[1, 2, 3, 4, 5, 6],
        &[0x82, 0x42, 2, 0xC2, 0x22, 1],
        &[4, 0x44, 1, 2, 6, 0x14],
        &[2, 2, 2, 2],
        &[0x81, 0x41, 4, 0x82, 2, 5, 6, 3],
        &[5, 1],
    ];
    for kinds in pages {
        for (choices, marks, chosen) in [
            (4u8, 0u32, 1u8),
            (4, 0b1110, 3),
            (4, 0b1111, 0),
            (5, 0b10110, 9),
            (16, 0xFFFF, 2),
            (1, 0, 0),
        ] {
            for (choice, edit, clock) in [
                (0u32, 0u8, 0u32),
                (2, 4, 8),
                (2, 1, 0),
                (3, 5, 0x17),
                (7, 0, 0),
            ] {
                let prepare = || {
                    list.set(0, kinds, choices, marks, chosen, edit, 100, b"ab");
                    p(CHOICE, choice);
                    p(FRAME_CLOCK, 0x100 + clock);
                    p(MOVE_CLOCK, 0x100);
                };
                prepare();
                let before = Regs {
                    edi: list.base(),
                    ..Regs::SENTINEL
                };
                check_original(
                    &mut t,
                    &format!(
                        "kinds {kinds:x?}, {choices} choices marked {marks:#b}, chosen {chosen}, \
                         highlight {choice}, edit {edit:#x}, clock {clock:#x}"
                    ),
                    call_original,
                    before,
                    &regions,
                    0,
                    &prepare,
                    &rust,
                );
            }
        }
    }
    let _ = (CF, ZF);
    drop(saved);
    t.finish()
}
