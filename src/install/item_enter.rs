//! `install::item_enter` (0x4147FE): Enter on an item, by its kind (`[esi] & 0xF`): 1 a link
//! (`install::item_link`), 4 an edit begun (`install::start_edit`), 6 an action
//! (`install::item_action`).
//!
//! In:  esi the item, edi the list
//! Out: bl the kind; as the callee leaves them, else the flags of `cmp bl, 6`
//! Caller `install::page_key`, reading zero. Its calls through `part`. Ungated (an action
//! can be a page's work). Shadow mode runs the original alone. Self-test: a copy whose three
//! calls go to stand-ins, against the Rust version, every kind.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{at, cmp, rb};
use crate::race::viewer::regs_entry;
use rc2k_core::install::items::EnterWay;

const LINK: usize = 0x414A28;
const START_EDIT: usize = 0x4148D6;
const ACTION: usize = 0x414A4B;

pub fn item_enter(entry: &Regs) -> Result<Regs, Refused> {
    let mut r = *entry;
    let kind = rb(r.esi) & 0xF;
    r.ebx = r.ebx & !0xFF | kind as u32;
    // Converted: `rc2k_core::install::items::EnterWay`.
    let callee = match EnterWay::of(kind) {
        Some(EnterWay::Link) => LINK,
        Some(EnterWay::StartEdit) => START_EDIT,
        Some(EnterWay::Action) => ACTION,
        None => {
            cmp(&mut r, kind as u32, 6);
            return Ok(r);
        }
    };
    cmp(&mut r, kind as u32, kind as u32);
    crate::install::installer::game_part(at(callee), &mut r)?;
    Ok(r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| item_enter(r))
}
