//! `install::folder_page` (0x4142F5): the install-folder page. At the start (ebx 0) the
//! first edit item is begun (`install::next_edit`). At the end (ebx negative) `dword_51BA00`
//! is 0. Then by `dword_51BA00`: not 1, the items at +0x38 and +0x48 get 0x40 (hidden);
//! not 2, +0x58 and +0x68 get 0x40.
//!
//! In:  ebx the state, edi the list
//! Out: every register kept but what `next_edit` leaves; the flags of the last `cmp`
//! A page action. Caller `install::run_page`. Its call through `part`. Shadow mode runs
//! the original alone (`next_edit` begins an edit). Self-test: a copy whose call goes to a
//! stand-in, against the Rust version, every state and problem.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{
    NEXT_EDIT, PATH_PROBLEM, Part, ZF, at, both, cmp, copy, g, p, quiet, rb, wb,
};
use crate::install::list_test::TestList;
use crate::race::viewer::{Saved, dispatch_parts};
use rc2k_core::install::pages;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("install::folder_page", rva(0x4142F5), thunk, &ORIGINAL)
    .private_copy(0x3A)
    .relocate_calls(&[0x4])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub fn folder_page(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    if r.ebx == 0 {
        r.ebx = 0;
        part(at(NEXT_EDIT), &mut r)?;
    }
    if (entry.ebx as i32) < 0 {
        p(PATH_PROBLEM, 0);
    }
    // Converted: `rc2k_core::install::pages::hidden_notices`.
    let hide = |r: &Regs, offsets: [u32; 2]| {
        for o in offsets {
            wb(r.edi + o, rb(r.edi + o) | pages::HIDDEN);
        }
    };
    cmp(&mut r, g(PATH_PROBLEM), pages::BAD_FOLDER);
    if pages::hidden_notices(g(PATH_PROBLEM)).0 {
        hide(&r, [0x38, 0x48]);
    }
    cmp(&mut r, g(PATH_PROBLEM), pages::NO_ROOM);
    if pages::hidden_notices(g(PATH_PROBLEM)).1 {
        hide(&r, [0x58, 0x68]);
    }
    Ok(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        folder_page(r, &mut crate::install::installer::game_part)
    });
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // test ebx, ebx; jnz +5; call sub_41530F
    let copy = match copy(
        0x4142F5,
        0x3A,
        &[0x85, 0xDB, 0x75, 0x05, 0xE8, 0x11, 0x10, 0x00, 0x00],
        &[(0x4, 0x41530F)],
        &[0x41530F],
        &[],
        &[],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    quiet();
    let list = TestList::new();
    let regions = [list.region()];
    let saved = Saved::new(&regions);
    for ebx in [0u32, 1, 0xFFFF_FFFF] {
        for problem in [0u32, 1, 2] {
            let prepare = || {
                list.set(0, &[4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], 0, 0, 0, 1, 100, b"x");
                p(PATH_PROBLEM, problem);
            };
            let before = Regs {
                ebx,
                edi: list.base(),
                ..Regs::SENTINEL
            };
            both(
                &mut t,
                &format!("ebx {ebx:#x}, problem {problem}"),
                &copy,
                before,
                &regions,
                ZF,
                &[],
                &prepare,
                &|r, part, _| folder_page(r, part),
            );
        }
    }
    drop(saved);
    t.finish()
}
