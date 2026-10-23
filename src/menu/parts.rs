//! For the menu set-ups and item actions ported on 2026-10-19 (agent E): the calls a Rust
//! version makes go through a `Part` by place (its callee's place in its list), so that a
//! self-test can run a copy of the original whose calls go to stand-ins of those places and
//! the Rust version on the same stand-ins' notes. Each call is noted with the seven
//! registers; what a stand-in answers (registers, the carry, a game dword written) is set
//! per place, in turn (the last answer stays).

use crate::errors::Refused;
use crate::framework::Regs;

/// A call by place.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// A Rust version with its calls through a `Part`: the registers it leaves.
pub type Parted<'a> = &'a dyn Fn(&Regs, Part) -> Result<Regs, Refused>;

/// `dispatch_parted` outside shadow mode, without its hook (typed tables, 2026-10-22: the entry the menus' and draw lists' enums run, with no hook).
pub fn parted_entry(
    r: &mut Regs,
    rust: Parted,
    game_part: fn(u32, &mut Regs) -> Result<(), Refused>,
) -> Result<(), Refused> {
    let mut part = game_part;
    let left = rust(r, &mut part)?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
