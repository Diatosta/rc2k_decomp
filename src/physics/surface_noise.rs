//! `physics::surface_noise` (0x43BC68): the ground's roughness under the work car from two
//! noise maps (`race::sample_noise`, esi `unk_A2AB34` and `unk_A2AB04`): each sample
//! scaled into its range (lo + (hi - lo) x it >> 16, `imul` / `shrd`: `qword_609344` and
//! `dword_60AD18` .. `1C`) and held to 0 ..= 0xFF (by the sign). The first into +0x900 and
//! +0x904; the second into +0x908, and at least `dword_60AD10` (unsigned) through
//! `dword_62400C`, / 256 into +0x280 (`unk_71A258`). Then with `dword_60934C` set +0x900
//! 0, else +0x904 0.
//!
//! In:  nothing
//! Out: eax the second sample (held), ebx its range, ecx and edx as left, esi
//!      `unk_A2AB04`; no flags read
//!
//! Caller: `sub_43B635` (0x43B714), every physics step.
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back): maps, places and ranges by lot.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::math::x87_run::{ext_div2, int_80};
use crate::race::frame_draw::sample_noise;
use crate::screens::harness::Left;
use rc2k_core::physics::ground::{noise_level, rough_floor};

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

/// lo + (hi - lo) x sample >> 16, held to 0 ..= 0xFF; ebx the range, edx the product's
/// high half.
fn scaled(r: &mut Regs, lo: u32, hi: u32) -> u32 {
    let (v, range, high) = noise_level(r.eax, lo, hi);
    r.ebx = range;
    r.edx = high;
    v
}

/// The Rust version.
// Converted: `rc2k_core::physics::ground::{noise_level, rough_floor}` (the share / 256
// stays x87).
pub unsafe fn surface_noise(entry: &Regs) -> Left {
    let mut r = *entry;
    let a = |x: usize| game_addr(rva(x)) as u32;
    unsafe {
        r.esi = a(0xA2AB34);
        sample_noise(r.esi).apply(&mut r);
        r.eax = scaled(&mut r, g(0x609344), g(0x609348));
        p(0x71A8D8, r.eax);
        p(0x71A8DC, r.eax);
        r.esi = a(0xA2AB04);
        sample_noise(r.esi).apply(&mut r);
        r.eax = scaled(&mut r, g(0x60AD18), g(0x60AD1C));
        p(0x71A8E0, r.eax);
        let floor = rough_floor(r.eax, g(0x60AD10));
        r.eax = floor;
        p(0x62400C, floor);
        p(
            0x71A258,
            ext_div2(int_80(floor as i32, false), g(0x5C5E2C), 0x3F80_0000),
        );
        if g(0x60934C) != 0 {
            p(0x71A8D8, 0);
        } else {
            p(0x71A8DC, 0);
        }
    }
    crate::race::stage_prefs::left_of(&r)
}
