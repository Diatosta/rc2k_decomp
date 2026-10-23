//! `race::sample_noise` (0x44FD38): a 64 x 64 byte map sampled at the work car's place
//! (`g_car` +0x6A4 and +0x6AC: the top byte the cell, the next two the fraction), blended
//! between its four corners (`dword_637620` .. `dword_637634`; `imul` and `shrd` by 16),
//! shifted left 8.
//!
//! In:  esi = the map (`g_noiseBytes` from `race::fog_table`)
//! Out: eax the value, ebx and ecx the fractions, edx the last product's high half
//!
//! See `race/frame_draw.rs`. Callers: `race::fog_table` (0x44FC3E) and `sub_43BC68`
//! (0x43BC6E, 0x43BCB1; `mov` follows). Shadow mode: by state. Self-test: a test map, the
//! work car's place set (cells at the edges, fractions); all put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { sample_noise(r.esi) })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
