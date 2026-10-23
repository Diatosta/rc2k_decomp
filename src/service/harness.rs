//! The hook and self-test machinery for functions that only change game variables. Shadow
//! mode cannot simply run such a function twice, so it runs the Rust version, notes what it
//! left, puts the variables back, runs the original and compares.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr};
use crate::screens::harness::Left;
use std::slice::{from_raw_parts, from_raw_parts_mut};

/// A piece of game memory a function may change: (address, bytes).
pub type Region = (u32, usize);

pub fn at(rva: usize) -> Region {
    (game_addr(rva) as u32, 4)
}

pub fn capture(regions: &[Region]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for &(addr, len) in regions {
        bytes.extend_from_slice(unsafe { from_raw_parts(addr as *const u8, len) });
    }
    bytes
}

pub fn restore(regions: &[Region], mut bytes: &[u8]) {
    for &(addr, len) in regions {
        unsafe { from_raw_parts_mut(addr as *mut u8, len) }.copy_from_slice(&bytes[..len]);
        bytes = &bytes[len..];
    }
}

/// The Rust version as the harness runs it: it gets the registers at entry and says what it
/// leaves in them. It must not have written anything when it returns an error.
pub type Rust<'a> = &'a dyn Fn(&Regs) -> Result<Left, Refused>;

/// `dispatch_state` outside shadow mode, without its hook (typed tables, 2026-10-22: the entry the menus' and draw lists' enums run, with no hook): the Rust version, the
/// registers it leaves applied. A refusal is the caller's; the Rust version has written
/// nothing then.
pub fn state_entry(r: &mut Regs, rust: Rust) -> Result<(), Refused> {
    rust(r)?.apply(r);
    Ok(())
}
