//! `install::item_action` (0x414A4B): Enter on an action item (kind 6): its action
//! (`[esi+8]`), if it has one.
//!
//! In:  esi the item (edi the list, for the action)
//! Out: with an action as it leaves them; else the flags of `cmp [esi+8], 1` (carry set)
//! Caller `install::item_enter`. Ungated: the installer's actions are whole pages' work
//! (`install::install_files`). Shadow mode runs the original alone. Self-test: a copy, the
//! action a noting stand-in, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{CF, Part, ZF, both, cmp, copy, quiet, rd, stand_in};
use crate::install::list_test::TestList;
use crate::race::viewer::{Saved, dispatch_parts};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("install::item_action", rva(0x414A4B), thunk, &ORIGINAL)
    .private_copy(0xA)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

pub fn item_action(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let action = rd(r.esi + 8);
    cmp(&mut r, action, 1);
    if action >= 1 {
        part(action, &mut r)?;
    }
    Ok(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        item_action(r, &mut crate::install::installer::game_part)
    });
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // cmp dword ptr [esi+8], 1; jb short +3
    let copy = match copy(
        0x414A4B,
        0xA,
        &[0x83, 0x7E, 0x08, 0x01, 0x72, 0x03],
        &[],
        &[],
        &[],
        &[],
        &[0xC0DE_0001],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    quiet();
    let list = TestList::new();
    let regions = [list.region()];
    let saved = Saved::new(&regions);
    for action in [true, false] {
        let prepare = || {
            list.set(0, &[6, 6], 0, 0, 0, 0, 0, b"");
            if action {
                list.set_action(1, stand_in(0));
            }
        };
        let before = Regs {
            esi: list.item(1),
            edi: list.base(),
            ..Regs::SENTINEL
        };
        both(
            &mut t,
            &format!("action {action}"),
            &copy,
            before,
            &regions,
            CF | ZF,
            &[],
            &prepare,
            &|r, part, _| item_action(r, part),
        );
    }
    drop(saved);
    t.finish()
}
