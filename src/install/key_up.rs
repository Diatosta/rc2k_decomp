//! `install::key_up` (0x414784): key 0x66 on an item: for choices shown upright (kind
//! 0x22 of `[esi] & 0x2F`) the choice before (`install::choice_back`).
//!
//! In:  esi the item
//! Out: bl the kind (`[esi] & 0x2F`); with choices as `choice_back` leaves them (zero set),
//!      else the flags of `cmp bl, 22h` (zero clear)
//! Caller `install::page_key`, reading zero. Shadow mode compares. Self-test: items of the
//! test's against the original.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{cmp, rb};
use crate::race::viewer::regs_entry;
use rc2k_core::install::items::{Helper, ItemKey};

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

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &key_up)
}
