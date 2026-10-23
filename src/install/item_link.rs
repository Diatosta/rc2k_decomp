//! `install::item_link` (0x414A28): Enter on a link item (kind 1): its action (`[esi+8]`,
//! edx its `[esi+0Ch]`) if it has one; else, unless its next list (`[esi+1]`) is negative,
//! that number into `byte_51D415` and the page left (`g_listLeave` 1).
//!
//! In:  esi the item (edi the list, for the action)
//! Out: with an action as it leaves them; else bl the next list and the flags of
//!      `test bl, bl` (zero for list 0, carry clear)
//! Caller `install::item_enter`. Ungated: an action can be a whole page's work. Shadow mode
//! runs the original alone. Self-test: a copy, the action a noting stand-in, against the
//! Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{
    CF, KEY_TAKEN, Part, ZF, both, cmp, copy, logic, pb, quiet, rb, rd, region, stand_in,
};
use crate::install::list_test::TestList;
use crate::race::viewer::{Saved, dispatch_parts, regs_entry};
use rc2k_core::install::items::{self, LinkWay};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("install::item_link", rva(0x414A28), thunk, &ORIGINAL)
    .private_copy(0x23)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

pub fn item_link(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let action = rd(r.esi + 8);
    // Converted: `rc2k_core::install::items::link`.
    let next = || {
        let next = rb(r.esi + 1);
        r.ebx = r.ebx & !0xFF | next as u32;
        logic(&mut r, next as u32);
        next
    };
    match items::link(action, next) {
        LinkWay::Action => {
            r.edx = rd(r.esi + 0xC);
            cmp(&mut r, action, 1);
            part(action, &mut r)?;
        }
        LinkWay::Leave(next) => {
            pb(crate::install::installer::LIST_ANSWER, next);
            pb(crate::install::installer::LIST_LEAVE, 1);
        }
        LinkWay::Stay => {}
    }
    Ok(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        item_link(r, &mut crate::install::installer::game_part)
    });
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| {
        item_link(r, &mut crate::install::installer::game_part)
    })
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // cmp dword ptr [esi+8], 1; jb short +8
    let copy = match copy(
        0x414A28,
        0x23,
        &[0x83, 0x7E, 0x08, 0x01, 0x72, 0x08],
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
    let regions = [list.region(), region(KEY_TAKEN, 3)];
    let saved = Saved::new(&regions);
    for (action, next) in [
        (true, 3u8),
        (false, 3),
        (false, 0),
        (false, 0x80),
        (false, 0xFF),
    ] {
        let prepare = || {
            list.set(0, &[1, 1], next, 0, 0, 0, 0, b"");
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
            &format!("action {action}, next {next:#x}"),
            &copy,
            before,
            &regions,
            CF | ZF,
            &[],
            &prepare,
            &|r, part, _| item_link(r, part),
        );
    }
    drop(saved);
    t.finish()
}
