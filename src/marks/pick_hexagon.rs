//! `marks::pick_hexagon` (0x4BC6F1): one step of the marks' random (`g_marksSeed`, times
//! 0x9D plus 0x47) split into a place (bits 0 .. 10, 11 .. 20) and a kind (bits 22 .. 24)
//! for `marks::spawn_hexagon`.
//!
//! In:  nothing
//! Out: eax, ebx, ecx the place and the kind
//!
//! Caller: `marks::hexagons_spawn` (twice). The call is `spawn_hexagon`'s Rust version.
//! Self-test: the original and the Rust version on the game's items, seeds by lot; put back.
//!
//! Converted 2026-10-21: the hook's Rust version is `pick_hexagon_plain`, through
//! `rc2k_core::marks::items::pick_hexagon`; compared exactly as before (the earlier version
//! stays for the Rust callers in unconverted code).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write};
use crate::marks::setup::SEED_RVA;
use crate::marks::spawn_hexagon::{self, spawn_hexagon};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn pick_hexagon(entry: &Regs) -> Regs {
    let mut g = *entry;
    unsafe {
        let s = game_read::<u32>(SEED_RVA)
            .wrapping_mul(0x9D)
            .wrapping_add(0x47);
        game_write(SEED_RVA, s);
        g.eax = s & 0x7FF;
        g.ebx = s.rotate_right(11) & 0x3FF;
        g.ecx = s.rotate_right(22) & 7;
        spawn_hexagon(g.eax, g.ebx, g.ecx);
    }
    g
}

/// The converted version: the hook's.
// Converted: `rc2k_core::marks::items::pick_hexagon`
pub unsafe fn pick_hexagon_plain(entry: &Regs) -> Regs {
    unsafe {
        let p = rc2k_core::marks::items::pick_hexagon(game_read(SEED_RVA));
        game_write(SEED_RVA, p.random);
        spawn_hexagon::spawn_hexagon_plain(p.x, p.z, p.kind);
        Regs {
            eax: p.x,
            ebx: p.z,
            ecx: p.kind,
            ..*entry
        }
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { pick_hexagon_plain(r) }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
