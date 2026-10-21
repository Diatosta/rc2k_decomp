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

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::math::x87_run::{ext_div2, int_80};
use crate::race::frame_draw::sample_noise;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use rc2k_core::physics::ground::{noise_level, rough_floor};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::surface_noise", rva(0x43BC68), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { surface_noise(r) })
}

fn regions() -> Vec<Region> {
    let a = |x: usize| game_addr(rva(x)) as u32;
    vec![
        (a(0x71A8D8), 12),
        (a(0x62400C), 4),
        (a(0x71A258), 4),
        (a(0x637620), 0x18),
    ]
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let a = |x: usize| game_addr(rva(x)) as u32;
    let mut all = regions();
    all.extend([(a(0x71A67C), 12), (a(0x609344), 12), (a(0x60AD10), 16)]);
    // The two maps (64 x 64 bytes at 0xA2AB34 and 0xA2AB04) are the game's, read only.
    let saved = capture(&all);
    for seed in 0..8u32 {
        let set = || {
            p(0x71A67C, seed.wrapping_mul(0x1357_9BDF));
            p(0x71A684, seed.wrapping_mul(0x2468_ACE1));
            p(0x609344, seed * 10);
            p(0x609348, 200 + seed * 40);
            p(0x60934C, seed % 2);
            p(0x60AD10, 20 + seed);
            p(0x60AD18, if seed == 3 { 300 } else { 5 });
            p(0x60AD1C, 150 + seed * 30);
        };
        check_state(
            &mut t,
            &format!("seed {seed}"),
            call_original,
            Regs::SENTINEL,
            &set,
            &all,
            &rust,
        );
    }
    restore(&all, &saved);
    t.finish()
}
