//! `race::fog_table` (0x44FC33): the fog's table for the frame (see `race/frame_draw.rs`):
//! `race::draw_distance_settings`, the noise at the car (`race::sample_noise` of
//! `g_noiseBytes`) spread into a step (`dword_62DAF0`), two shares from `dword_6415DC` x
//! `dword_6415D8` (`dword_6365F0` / `dword_6365F4`), `g_fogTableLength` = `dword_63DC58` x 8
//! bytes of `g_fogTable` by the summed step, and `dword_636610` the value past it.
//!
//! In:  nothing
//! Out: the registers as it leaves them (a `mov` follows in `sub_44FBD9`)
//!
//! It begins with its call: private copy (0x105, both calls relocated), manual patch.
//! Caller: `sub_44FBD9` (0x44FC04). Refuses (before anything) a length of 0 (the loop tests
//! at its end) or past `g_fogTable`'s 0x1000 bytes. Shadow mode: by state. Self-test: the
//! game's tables and noise, the inputs set (levels, the car's place, the shares, lengths
//! 0x10 .. 0x1000); all put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};
use crate::race::draw_distance_settings::distance_regions;
use crate::race::frame_draw::*;
use crate::race::sample_noise::noise_regions;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, state_entry};

pub fn fog_regions() -> Vec<Region> {
    let mut v = distance_regions();
    v.extend(noise_regions());
    v.extend([
        (game_addr(FOG_TABLE_RVA) as u32, 0x1000),
        at(rva(0x63761C)),
        at(rva(0x62DAF0)),
        at(rva(0x6365F0)),
        at(rva(0x6365F4)),
        at(rva(0x636610)),
    ]);
    v
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { fog_table(r)? }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
