//! `scene::scale_texture_coordinates` (0x44E2FB): the polygon's texture coordinates
//! from whole numbers to floats.
//!
//! In:  nothing
//! Out: eax = the last value worked on, ecx = the number of vertices, esi = the last
//!      entry; nothing changed for a polygon of no vertices; everything else preserved
//!
//! For each entry of the polygon, with the switch `dword_6240E0` its first pair of
//! coordinates (+8, +0xC) and with `dword_6240E4` its second (+0x10, +0x14): a value
//! that is not 0 becomes a float (`fild`, `fstp`) whose bits then lose 0x0C000000: the
//! number over 2^24. The two work variables `dword_5183C0` and `flt_5183C4` keep the
//! last value and its float.
//! Eight callers.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::d3d::draw_fan_unclipped::COUNT_RVA;
use crate::errors::Refused;
use crate::framework::{EAX, ECX, ESI, game_addr, game_read};
use crate::scene::draw_polygon::{SWITCH_A_RVA, SWITCH_B_RVA};
use crate::scene::polygon::{ENTRIES_RVA, dword, listed, put};
use crate::screens::harness::Left;
use crate::session::MACHINE;

// ---- game data ----

// ---- Rust version ----

// Converted: `rc2k_core::scene::texture_coordinates`
pub unsafe fn scale_texture_coordinates() -> Result<Left, Refused> {
    use rc2k_core::scene::texture_coordinates as coordinates;
    unsafe {
        if game_read::<i32>(COUNT_RVA) <= 0 {
            return Ok(Left::nothing());
        }
        let entries = listed()?;
        let switches: [u32; 2] = [game_read(SWITCH_A_RVA), game_read(SWITCH_B_RVA)];
        let mut eax = 0;
        for &entry in &entries {
            eax = (entry - game_addr(ENTRIES_RVA)) as u32;
            for (pair, switch) in switches.into_iter().enumerate() {
                if switch == 0 {
                    continue;
                }
                for at in [entry + 8 + pair * 8, entry + 0xC + pair * 8] {
                    eax = dword(at);
                    if let Some(float) = coordinates::float_of(eax) {
                        MACHINE.work.set(eax);
                        MACHINE.work_float.set(float);
                    }
                    eax = coordinates::scaled(eax);
                    put(at, eax);
                }
            }
        }
        Ok(Left::nothing()
            .with(EAX, eax)
            .with(ECX, entries.len() as u32)
            .with(
                ESI,
                *entries.last().expect("the polygon has a vertex") as u32,
            ))
    }
}
