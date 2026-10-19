//! `install::folder_key` (0x41534E): the folder page's key action (its list's +0x14): with
//! ah bit 3 (an edit's key), Enter (1, not while the edit holds 0x10 of `[edi]`) sets the
//! answer to 0 and leaves, Escape (0x1C) sets it to 1 and leaves. Any other key does
//! nothing.
//!
//! In:  al the key, ah its bits, edi the list
//! Out: every register kept; the flags of its last test (no caller reads them read back)
//! Caller `install::run_page` (the folder page). Shadow mode compares. Self-test: lists of
//! the test's against the original, every key and both bit-3 states.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{
    CF, KEY_TAKEN, LIST_ANSWER, LIST_LEAVE, ZF, logic, pb, rb, region,
};
use crate::install::list_test::TestList;
use crate::race::viewer::{Saved, check_original, dispatch_regs};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::folder_key", rva(0x41534E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub fn folder_key(entry: &Regs) -> Regs {
    let mut r = *entry;
    if r.eax & 0x800 == 0 {
        // test eax, eax
        let v = r.eax;
        logic(&mut r, v);
        return r;
    }
    match r.eax as u8 {
        1 => {
            // test byte [edi], 10h
            let held = rb(r.edi) & 0x10;
            r.set_zero(held == 0);
            r.set_carry(false);
            if held == 0 {
                pb(LIST_ANSWER, 0);
                pb(LIST_LEAVE, 1);
            }
        }
        0x1C => {
            pb(LIST_ANSWER, 1);
            pb(LIST_LEAVE, 1);
            // cmp al, 1Ch: equal
            r.set_zero(true);
            r.set_carry(false);
        }
        other => {
            // cmp al, 1Ch: not equal
            logic(&mut r, other.wrapping_sub(0x1C) as u32);
        }
    }
    r
}

fn rust(r: &Regs) -> Result<Regs, Refused> {
    Ok(folder_key(r))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_regs(
        &HOOK,
        r,
        call_original,
        &[region(KEY_TAKEN, 3)],
        CF | ZF,
        &rust,
    );
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let list = TestList::new();
    let regions = [list.region(), region(KEY_TAKEN, 3)];
    let saved = Saved::new(&regions);
    for flags in [0u8, 0x10] {
        for eax in [0x801u32, 0x81C, 0x830, 1, 0x1C, 0x839] {
            let prepare = || {
                list.set(flags, &[1], 0, 0, 0, 0, 0, b"");
                pb(LIST_ANSWER, 9);
                pb(LIST_LEAVE, 0);
            };
            prepare();
            let before = Regs {
                eax: 0x5500_0000 | eax,
                edi: list.base(),
                ..Regs::SENTINEL
            };
            check_original(
                &mut t,
                &format!("flags {flags:#x}, eax {eax:#x}"),
                call_original,
                before,
                &regions,
                CF | ZF,
                &prepare,
                &rust,
            );
        }
    }
    drop(saved);
    t.finish()
}
