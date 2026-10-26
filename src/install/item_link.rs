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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{Part, cmp, logic, pb, rb, rd};
use crate::race::viewer::regs_entry;
use rc2k_core::install::items::{self, LinkWay};

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

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| {
        item_link(r, &mut crate::install::installer::game_part)
    })
}
