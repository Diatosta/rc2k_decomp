//! `physics::car_lighting` (0x44ECE4): the light on the work car from the face under it.
//! The car's x / z into the probe point, the old light (+0x244) into `dword_5CCCA8`,
//! `dword_62DAFC` / `62DB00` 0, the last face (+0xE00) into `dword_5CCC78`;
//! `physics::find_ground`; no face: done. With `dword_5CCC6C` 1 the shares
//! (`dword_639800`, `804`) taken from 0x10000; kept in `dword_62DB08` / `0C`. The face's
//! four corner lights (+0x14 ..; with its texture's +0x2C 1 each scaled by
//! ((byte +0x22 + k) >> 1 ^ 0x7F) + 0x80, `mul`, >> 8) blended by the shares (`imul`,
//! `shrd` 16) into +0x244; with +0xFB4 set the change held to 10 a step (or none with
//! `dword_6235E8`); `g_lightTable[it]` into `dword_62DAFC` and +0x240. With bit 2 of the
//! face's +2 also two colour blends (+0x28 .., +0x2C ..) through `dword_635518[]` /
//! `dword_635918[]` into `dword_62DB00` / `62DB04`, each added to +0x240 byte by byte
//! with saturation (four bytes with `g_hasMmx`, else the low three).
//!
//! In:  nothing
//! Out: eax, ebx (the texture's record), ecx, edx, esi (the face) as left; no flags read
//!
//! Caller: `sub_43B635` (0x43B990), every physics step.
//! Refuses a light index past 0xFF (the tables are 256) and no faces; what was written is
//! put back. Shadow mode runs the original alone (`find_ground` through
//! `physics/reset_parts.rs`).
//! Self-test: a copy whose `find_ground` goes to a stand-in, run on the game's variables
//! (saved and put back) with test faces, against the Rust version: faces found and not,
//! textures, both MMX paths.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::physics::reset_parts::FIND_GROUND;
use rc2k_core::physics::lighting;

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

unsafe fn byte(a: u32) -> u32 {
    unsafe { (a as *const u8).read() as u32 }
}

/// The two-way blend of the four corners now in `dword_5CCC90` ..: into `5CCCA4`,
/// `5CCCA0`, returned.
fn corners(r: &mut Regs) -> u32 {
    let c = [0, 1, 2, 3].map(|k| g(0x5CCC90 + 4 * k));
    let ([a, b], v, edx) = lighting::corners(c, g(0x639800), g(0x639804));
    p(0x5CCCA4, a);
    p(0x5CCCA0, b);
    r.edx = edx;
    v
}

/// The steps on `r`.
// Converted: `rc2k_core::physics::lighting`.
pub fn car_lighting_in(r: &mut Regs) -> Result<(), Refused> {
    p(0x639824, g(0x71A10C));
    p(0x63982C, g(0x71A114));
    p(0x5CCCA8, g(0x71A21C));
    r.eax = 0;
    p(0x62DAFC, 0);
    p(0x62DB00, 0);
    r.eax = g(0x71ADD8);
    p(0x5CCC78, r.eax);
    crate::physics::reset_parts::game_part(FIND_GROUND, r)?;
    r.ebx = g(0x5CCC74);
    if r.ebx == u32::MAX {
        return Ok(());
    }
    if g(0x5CCCBC) == 0 {
        return Err(BadInput("no course faces").into());
    }
    if g(0x5CCC6C) == 1 {
        let (s, t) = lighting::half_shares(1, g(0x639800), g(0x639804));
        p(0x639800, s);
        p(0x639804, t);
        r.eax = t;
    }
    p(0x62DB08, g(0x639800));
    p(0x62DB0C, g(0x639804));
    let f = (r.ebx << 6).wrapping_add(g(0x5CCCBC));
    r.esi = f;
    unsafe {
        for k in 0..4u32 {
            p(0x5CCC90 + 4 * k as usize, byte(f + 0x14 + k));
        }
        r.ebx = (game_addr(rva(0x979F30)) as u32).wrapping_add(byte(f + 0x21) << 6);
        if byte(r.ebx + 0x2C) == 1 {
            for k in 0..4u32 {
                let at = 0x5CCC90 + 4 * k as usize;
                let (v, edx) = lighting::shaded(g(at), byte(f + 0x22 + k) as u8);
                r.edx = edx;
                p(at, v);
            }
        }
        let light = corners(r);
        p(0x71A21C, light);
        if g(0x71AF8C) != 0 {
            let held = lighting::hold_change(light, g(0x5CCCA8), g(0x6235E8) != 0);
            p(0x71A21C, held.light);
            if let Some(eax) = held.eax {
                r.eax = eax;
            }
            if let Some(ecx) = held.ecx {
                r.ecx = ecx;
            }
        }
        let light = g(0x71A21C);
        if light >= lighting::TABLE {
            return Err(BadInput("a light index past 0xFF").into());
        }
        r.eax = g(0x635118 + 4 * light as usize);
        p(0x62DAFC, r.eax);
        p(0x71A218, r.eax);
        if byte(f + 2) & 4 == 0 {
            return Ok(());
        }
        for (first, table, to) in [
            (0x28u32, 0x635518usize, 0x62DB00usize),
            (0x2C, 0x635918, 0x62DB04),
        ] {
            for k in 0..4u32 {
                p(0x5CCC90 + 4 * k as usize, byte(f + first + k));
            }
            let i = corners(r);
            if i >= lighting::TABLE {
                return Err(BadInput("a colour index past 0xFF").into());
            }
            r.eax = g(table + 4 * i as usize);
            p(to, r.eax);
            r.eax = lighting::add_colour(r.eax, g(0x71A218), g(0x5183A8) != 0);
            p(0x71A218, r.eax);
        }
    }
    Ok(())
}

/// The Rust version: what was written is put back on a refusal.
pub unsafe fn car_lighting(entry: &Regs) -> Result<Regs, Refused> {
    let mut r = *entry;
    match car_lighting_in(&mut r) {
        Ok(()) => Ok(r),
        Err(e) => Err(e),
    }
}
