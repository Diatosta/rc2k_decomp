//! `install::next_edit` (0x41530F): with ebx 0, the highlight (`g_listChoice`) moved on to
//! the first edit item (kind 4) from it, which begins to be edited (`install::start_edit`);
//! none before the list's end: the highlight back to 0. Any other ebx: nothing.
//!
//! In:  ebx 0 (a list action's start), edi the list
//! Out: every register kept (`pusha` / `popa`); the flags of what it did last (no caller
//!      reads them)
//! Callers `install::folder_page` (the folder page's action), `install::mark_page`.
//! Shadow mode compares. Self-test: lists of the test's against the original.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{
    CF, CHOICE, SCRIPT_PATH, ZF, cmp, g, list_regions, logic, p, rb, region,
};
use crate::install::list_test::TestList;
use crate::race::viewer::{Saved, check_original, dispatch_regs};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("install::next_edit", rva(0x41530F), thunk, &ORIGINAL)
    .private_copy(0x3F)
    .relocate_calls(&[0x38])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub fn next_edit(entry: &Regs) -> Regs {
    let mut r = *entry;
    if entry.ebx != 0 {
        logic(&mut r, entry.ebx);
        return r;
    }
    let count = rb(entry.edi + 1) as u32;
    let item = |n: u32| entry.edi + 0x18 + 0x10 * n;
    // Converted: `rc2k_core::install::items::next_edit` (the highlight stored once, where
    // it ends).
    let (n, found) = rc2k_core::install::items::next_edit(g(CHOICE), count, |n| rb(item(n)));
    if found {
        p(CHOICE, n);
        r = crate::install::start_edit::start_edit(&Regs { esi: item(n), ..r });
    } else {
        cmp(&mut r, count, n);
        p(CHOICE, 0);
    }
    let flags = r.eflags;
    Regs {
        eflags: flags,
        ..*entry
    }
}

fn rust(r: &Regs) -> Result<Regs, Refused> {
    Ok(next_edit(r))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = list_regions(r.edi);
    dispatch_regs(&HOOK, r, call_original, &regions, CF | ZF, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let list = TestList::new();
    let regions = [list.region(), region(CHOICE, 4), region(SCRIPT_PATH, 0x102)];
    let saved = Saved::new(&regions);
    for (kinds, choice, ebx) in [
        (&[1u8, 1, 4, 1][..], 0u32, 0u32),
        (&[1, 1, 4, 1], 2, 0),
        (&[1, 4, 1, 1], 2, 0),
        (&[1, 1, 1, 1], 1, 0),
        (&[4, 1, 6, 0x14], 3, 0),
        (&[4, 1, 6, 1], 1, 1),
        (&[4, 1, 6, 1], 1, 0xFFFF_FFFF),
    ] {
        let prepare = || {
            list.set(0, kinds, 0, 0, 0, 0, 100, b"edit");
            p(CHOICE, choice);
        };
        prepare();
        let before = Regs {
            ebx,
            edi: list.base(),
            ..Regs::SENTINEL
        };
        check_original(
            &mut t,
            &format!("kinds {kinds:x?}, highlight {choice}, ebx {ebx:#x}"),
            call_original,
            before,
            &regions,
            CF | ZF,
            &prepare,
            &rust,
        );
    }
    drop(saved);
    t.finish()
}
