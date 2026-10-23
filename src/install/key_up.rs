//! `install::key_up` (0x414784): key 0x66 on an item: for choices shown upright (kind
//! 0x22 of `[esi] & 0x2F`) the choice before (`install::choice_back`).
//!
//! In:  esi the item
//! Out: bl the kind (`[esi] & 0x2F`); with choices as `choice_back` leaves them (zero set),
//!      else the flags of `cmp bl, 22h` (zero clear)
//! Caller `install::page_key`, reading zero. Shadow mode compares. Self-test: items of the
//! test's against the original.
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{CF, ZF, cmp, rb, rd};
use crate::race::viewer::{dispatch_regs, regs_entry};
use rc2k_core::install::items::{Helper, ItemKey};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::key_up", rva(0x414784), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// A key on the item at esi: its kind into bl, then the helper the key has for it, or the
/// flags of the last compare.
pub fn item_key(entry: &Regs, key: ItemKey) -> Result<Regs, Refused> {
    let mut r = *entry;
    let kind = rb(r.esi) & key.kind_mask();
    r.ebx = r.ebx & !0xFF | kind as u32;
    // Converted: `rc2k_core::install::items::ItemKey::helper`.
    use crate::install as i;
    match key.helper(kind) {
        Some(Helper::ChoiceBack) => i::choice_back::choice_back(&r),
        Some(Helper::ChoiceOn) => i::choice_on::choice_on(&r),
        Some(Helper::ChoiceByNumber) => i::choice_by_number::choice_by_number(&r),
        Some(Helper::ValueDown) => i::value_down::value_down(&r),
        Some(Helper::ValueUp) => i::value_up::value_up(&r),
        Some(Helper::ValueByNumber) => i::value_by_number::value_by_number(&r),
        None => {
            cmp(&mut r, kind as u32, key.last_compared() as u32);
            Ok(r)
        }
    }
}

pub fn key_up(entry: &Regs) -> Result<Regs, Refused> {
    item_key(entry, ItemKey::Up)
}

extern "C" fn dispatch(r: &mut Regs) {
    let target = (rd(r.esi + 8), 1usize);
    dispatch_regs(&HOOK, r, call_original, &[target], CF | ZF, &key_up);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &key_up)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::item_test::run(
        &mut t,
        call_original,
        &key_up,
        crate::install::item_test::any,
    );
    t.finish()
}
