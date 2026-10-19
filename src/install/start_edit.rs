//! `install::start_edit` (0x4148D6): an edit item (kind 4) begins to be edited: its record's
//! bit 0 set, its text cut at its most bytes (`[record+1]`: a NUL there) and that and two
//! bytes more kept in `byte_95EAF8` (for Escape, `install::edit_key`).
//!
//! In:  esi the item (+8 the text, +0xC the record: +0 flags, +1 the most bytes)
//! Out: every register kept (`pusha` / `popa`); zero set, carry clear (`cmp eax, eax`)
//! Callers `install::item_enter`, `install::next_edit`. Shadow mode compares. Self-test:
//! lists of the test's against the original.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{CF, SCRIPT_PATH, ZF, a, rb, rd, region, wb};
use crate::install::list_test::TestList;
use crate::race::viewer::{Saved, check_original, dispatch_regs};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::start_edit", rva(0x4148D6), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub fn start_edit(entry: &Regs) -> Regs {
    let record = rd(entry.esi + 0xC);
    wb(record, rb(record) | 1);
    let most = rb(record + 1) as u32;
    let text = rd(entry.esi + 8);
    wb(text + most, 0);
    let backup = a(SCRIPT_PATH);
    for n in 0..most + 2 {
        wb(backup + n, rb(text + n));
    }
    let mut r = *entry;
    r.set_zero(true);
    r.set_carry(false);
    r
}

fn rust(r: &Regs) -> Result<Regs, Refused> {
    Ok(start_edit(r))
}

extern "C" fn dispatch(r: &mut Regs) {
    let record = rd(r.esi + 0xC);
    let most = rb(record + 1) as usize;
    let regions = [
        (record, 1),
        (rd(r.esi + 8), most + 2),
        region(SCRIPT_PATH, most + 2),
    ];
    dispatch_regs(&HOOK, r, call_original, &regions, CF | ZF, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let list = TestList::new();
    let regions = [list.region(), region(SCRIPT_PATH, 0x102)];
    let saved = Saved::new(&regions);
    for (edit, text) in [
        (0u8, &b"c:\\games\\rally"[..]),
        (6, b""),
        (1, b"a text that is longer than the most bytes allowed"),
    ] {
        let prepare = || list.set(0, &[4, 4], 0, 0, 0, edit, 100, text);
        prepare();
        let before = Regs {
            esi: list.item(1),
            ..Regs::SENTINEL
        };
        check_original(
            &mut t,
            &format!(
                "edit {edit:#x}, text {}",
                crate::strings::utils::bytes_str(text)
            ),
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
