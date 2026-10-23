//! `install::key_number` (0x4147DC): a number key (al 2 .. 0xA) on an item (`[esi] & 0xF`):
//! kind 2 that choice (`install::choice_by_number`), kind 3 that eighth of the value's
//! span (`install::value_by_number`); ecx the key less 2.
//!
//! In:  al the key, esi the item
//! Out: ecx the key less 2, bl the kind; as the helper leaves them (zero set), else the
//!      flags of `cmp bl, 3`
//! Caller `install::page_key`, reading zero. Shadow mode compares. Self-test: items of the
//! test's against the original, keys 2, 5, 0xA.
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{CF, ZF, rd};
use crate::install::key_up::item_key;
use crate::race::viewer::{dispatch_regs, regs_entry};
use rc2k_core::install::items::ItemKey;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::key_number", rva(0x4147DC), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub fn key_number(entry: &Regs) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::install::items` (`number_of_key`, `ItemKey::helper`).
    let r = Regs {
        ecx: rc2k_core::install::items::number_of_key(entry.eax as u8),
        ..*entry
    };
    item_key(&r, ItemKey::Number)
}

extern "C" fn dispatch(r: &mut Regs) {
    let target = (rd(r.esi + 8), 1usize);
    dispatch_regs(&HOOK, r, call_original, &[target], CF | ZF, &key_number);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &key_number)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::item_test::run(
        &mut t,
        call_original,
        &key_number,
        crate::install::item_test::any,
    );
    t.finish()
}
