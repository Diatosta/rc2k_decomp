//! `marks::pick_square` (0x4C0623): one step of the squares' own random (`dword_8BC256`,
//! times 0xC5 plus 0x5D) split into a place (bits 0 .. 10, 11 .. 20) and a kind (bits 21 ..
//! 23) for `marks::spawn_square`.
//!
//! In:  nothing
//! Out: eax, ebx the place; ecx the kind (as `spawn_square` leaves it)
//!
//! Caller: `marks::squares_spawn` (twice). The call is `spawn_square`'s Rust version.
//! Self-test: the original and the Rust version on the game's items, randoms by lot.
//!
//! Converted 2026-10-21: the hook's Rust version is `pick_square_plain`, through
//! `rc2k_core::marks::items::pick_square`; compared exactly as before (the earlier version
//! stays for the Rust callers in unconverted code).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};
use crate::marks::spawn_square::{self, spawn_square};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

const RANDOM_RVA: usize = rva(0x8BC256); // dword_8BC256

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn pick_square(entry: &Regs) -> Regs {
    let mut g = *entry;
    unsafe {
        let s = game_read::<u32>(RANDOM_RVA)
            .wrapping_mul(0xC5)
            .wrapping_add(0x5D);
        game_write(RANDOM_RVA, s);
        g.eax = s & 0x7FF;
        g.ebx = (s >> 11) & 0x3FF;
        g.ecx = (s >> 21) & 7;
        g.ecx = spawn_square(g.eax, g.ebx, g.ecx);
    }
    g
}

/// The converted version: the hook's.
// Converted: `rc2k_core::marks::items::pick_square`
pub unsafe fn pick_square_plain(entry: &Regs) -> Regs {
    unsafe {
        let p = rc2k_core::marks::items::pick_square(game_read(RANDOM_RVA));
        game_write(RANDOM_RVA, p.random);
        let ecx = spawn_square::spawn_square_plain(p.x, p.z, p.kind);
        Regs {
            eax: p.x,
            ebx: p.z,
            ecx,
            ..*entry
        }
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { pick_square_plain(r) }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
