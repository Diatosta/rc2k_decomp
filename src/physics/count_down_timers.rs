//! `physics::count_down_timers` (0x43D4B5): the work car's step counters:
//! `dword_71AF8C`, `dword_71AF88`, `dword_71A1CC`, `dword_71A1A8`, `dword_71A438` less 1
//! when not 0; `dword_71A240` counts the steps standing in neutral in mode 1 (0 below 1000
//! steps, with a switch of 0x71A078 .. 0x71A080 set, with bits 1 .. 4 of `dword_71A270` all
//! set, or in gear; kept in other modes); `dword_71A244` on by one unless 0, -1 or 1;
//! `flt_71A5D4` raised to `flt_71AEE0` (the highest so far). Readings, all.
//!
//! In:  nothing
//! Out: eax `dword_71A270` & 0x1E when that is looked at, else kept; no flags read
//!
//! Callers: `sub_43B635` (0x43B70A) and `sub_421848` (the car specification's run).
//! Self-test: `physics/copy_hooks_test.rs` (a copy on test variables against the
//! converted version), 48 cases; one compare and copy of floats: exact at any precision.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, game_addr, hook_asm, rva};
use crate::physics::copy_hooks_test::{lot, test_moves};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use rc2k_core::physics::motion::{StepCounters, StepFacts, count_down};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::count_down_timers",
    rva(0x43D4B5),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const VARIABLES: [usize; 16] = [
    0x71AF8C, 0x71AF88, 0x719FEC, 0x71A080, 0x71A07C, 0x71A078, 0x62389C, 0x71A270, 0x71A1AC,
    0x71A240, 0x71A244, 0x71A1CC, 0x71A1A8, 0x71AEE0, 0x71A5D4, 0x71A438,
];

/// The converted version (`rc2k_core::physics::motion`) on the variables at `v`; `r` as
/// the original leaves it.
// Converted: `rc2k_core::physics::motion::count_down`.
pub unsafe fn count_at(v: &[u32], r: &mut Regs) {
    unsafe {
        let get = |k: usize| (v[k] as *const u32).read_unaligned();
        let put = |k: usize, x: u32| (v[k] as *mut u32).write_unaligned(x);
        let mut c = StepCounters {
            first: [get(0), get(1)],
            standing: get(9),
            running: get(10),
            later: [get(11), get(12), get(15)],
            highest: f32::from_bits(get(14)),
        };
        let facts = StepFacts {
            steps: get(2),
            switches: [get(3), get(4), get(5)],
            mode: get(6),
            bits: get(7),
            gear: get(8),
            figure: f32::from_bits(get(13)),
        };
        if let Some(eax) = count_down(&mut c, &facts) {
            r.eax = eax;
        }
        for (k, x) in [
            (0, c.first[0]),
            (1, c.first[1]),
            (9, c.standing),
            (10, c.running),
            (11, c.later[0]),
            (12, c.later[1]),
            (14, c.highest.to_bits()),
            (15, c.later[2]),
        ] {
            if get(k) != x {
                put(k, x);
            }
        }
    }
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version (one compare and copy of floats: the same at any precision); `r` as
/// the original leaves it.
pub unsafe fn count_down_timers(r: &mut Regs) {
    unsafe { count_at(&game(), r) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let mut g = *r;
    unsafe { count_at(&game(), &mut g) };
    Ok(Left::nothing().with(EAX, g.eax))
}

extern "C" fn dispatch(r: &mut Regs) {
    let v = game();
    let regions: Vec<_> = [0usize, 1, 9, 10, 11, 12, 14, 15]
        .iter()
        .map(|&k| (v[k], 4))
        .collect();
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn fill(seed: u32, out: &mut [u32]) {
    lot(seed, out);
    for k in [0usize, 1, 11, 12, 15] {
        out[k] = if (seed + k as u32).is_multiple_of(3) {
            0
        } else {
            out[k] % 50
        };
    }
    out[2] = [500u32, 1000, 5000][(seed % 3) as usize];
    for (k, o) in out.iter_mut().enumerate().take(6).skip(3) {
        *o = u32::from((seed >> k).is_multiple_of(5));
    }
    out[6] = seed % 3;
    out[7] = if seed.is_multiple_of(4) { 0x1E } else { out[7] };
    out[8] = [0u32, 1, u32::MAX][(seed / 3 % 3) as usize];
    out[10] = [0u32, u32::MAX, 1, 2, 7][(seed % 5) as usize];
    out[13] = (seed as f32 * 1.5).to_bits();
    out[14] = 6.0f32.to_bits();
}

fn self_test() -> (usize, usize) {
    test_moves(
        &HOOK,
        0x43D4B5,
        0xE5,
        &[0x83, 0x3D, 0x8C, 0xAF, 0x71, 0x00, 0x00],
        &VARIABLES,
        27,
        48,
        &fill,
        &|v, r| {
            unsafe { count_at(v, r) };
            Left::nothing().with(EAX, r.eax)
        },
    )
}
