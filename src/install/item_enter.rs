//! `install::item_enter` (0x4147FE): Enter on an item, by its kind (`[esi] & 0xF`): 1 a link
//! (`install::item_link`), 4 an edit begun (`install::start_edit`), 6 an action
//! (`install::item_action`).
//!
//! In:  esi the item, edi the list
//! Out: bl the kind; as the callee leaves them, else the flags of `cmp bl, 6`
//! Caller `install::page_key`, reading zero. Its calls through `part`. Ungated (an action
//! can be a page's work). Shadow mode runs the original alone. Self-test: a copy whose three
//! calls go to stand-ins, against the Rust version, every kind.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{CF, Part, ZF, at, both, cmp, copy, quiet, rb};
use crate::install::list_test::TestList;
use crate::race::viewer::{Saved, dispatch_parts};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::item_enter", rva(0x4147FE), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const LINK: usize = 0x414A28;
const START_EDIT: usize = 0x4148D6;
const ACTION: usize = 0x414A4B;

pub fn item_enter(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let kind = rb(r.esi) & 0xF;
    r.ebx = r.ebx & !0xFF | kind as u32;
    let callee = match kind {
        1 => LINK,
        4 => START_EDIT,
        6 => ACTION,
        _ => {
            cmp(&mut r, kind as u32, 6);
            return Ok(r);
        }
    };
    cmp(&mut r, kind as u32, kind as u32);
    part(at(callee), &mut r)?;
    Ok(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        item_enter(r, &mut crate::install::installer::game_part)
    });
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // mov bl, [esi]; and bl, 0Fh
    let copy = match copy(
        0x4147FE,
        0x28,
        &[0x8A, 0x1E, 0x80, 0xE3, 0x0F],
        &[(0xA, 0x414A28), (0x16, 0x4148D6), (0x22, 0x414A4B)],
        &[0x414A28, 0x4148D6, 0x414A4B],
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
    for kind in [0u8, 1, 2, 3, 4, 5, 6, 7, 0x41, 0x86, 0xF4] {
        let prepare = || list.set(0, &[kind, kind], 1, 0, 0, 0, 0, b"");
        let before = Regs {
            esi: list.item(1),
            edi: list.base(),
            ..Regs::SENTINEL
        };
        both(
            &mut t,
            &format!("kind {kind:#x}"),
            &copy,
            before,
            &regions,
            CF | ZF,
            &[],
            &prepare,
            &|r, part, _| item_enter(r, part),
        );
    }
    drop(saved);
    t.finish()
}
