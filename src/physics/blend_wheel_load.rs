//! `physics::blend_wheel_load` (0x440F70): unless `flt_71A208` is below `flt_614C64`
//! (C0): a slot by the signs of 0x71A1FC, 0x71A200, 0x71A204 (4 + 2 + 1 for each below 0)
//! and `flt_71A03C[slot]` moved towards 1 by `flt_71A208` / `flt_5C5D60` held to 0 .. 1
//! (through `dword_62400C`): a + (1 - a) share. What the eight are is a reading at best.
//!
//! In:  nothing
//! Out: edx the slot (when reached); the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B99A), every physics step.
//! Self-test: `physics/copy_hooks_test.rs` (`test_moves_floats`: a copy on test
//! variables against the converted version), 16 cases, exact at double precision.

use crate::errors::Refused;
use crate::framework::{EDX, RawHook, Regs, game_addr, game_read, hook_asm, rva};
use crate::math::x87;
use crate::math::x87_run::{compare_full, toward_one};
use crate::physics::copy_hooks_test::test_moves_floats;
use crate::screens::harness::Left;
use crate::service::floats::{FLOAT, Floats, dispatch_state_floats, floats_entry};
use rc2k_core::physics::motion::{LoadBlend, blend_load};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::blend_wheel_load", rva(0x440F70), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The figure, its three signs, the scratch, then the eight slots.
const VARIABLES: [usize; 13] = [
    0x71A208, 0x71A1FC, 0x71A200, 0x71A204, 0x62400C, 0x71A03C, 0x71A040, 0x71A044, 0x71A048,
    0x71A04C, 0x71A050, 0x71A054, 0x71A058,
];

/// The Rust core on the variables at `v`; `r` as the original leaves it.
pub unsafe fn blend_at(v: &[u32], r: &mut Regs) {
    unsafe {
        let get = |k: usize| (v[k] as *const u32).read_unaligned();
        let put = |k: usize, x: u32| (v[k] as *mut u32).write_unaligned(x);
        let less = |a: u32, b: u32| compare_full(a, b) & 0x100 != 0;
        if less(get(0), game_read(rva(0x614C64))) {
            return;
        }
        let zero: u32 = game_read(rva(0x5C5D90));
        let one: u32 = game_read(rva(0x5C5D38));
        r.edx = 0;
        for (k, add) in [(1, 4), (2, 2), (3, 1)] {
            if less(get(k), zero) {
                r.edx += add;
            }
        }
        put(4, x87::div(get(0), game_read(rva(0x5C5D60))));
        if less(get(4), zero) {
            put(4, x87::loaded(zero));
        }
        if !less(get(4), one) {
            put(4, x87::loaded(one));
        }
        let slot = 5 + r.edx as usize;
        put(slot, toward_one(get(slot), get(4)));
    }
}

/// The converted version (`rc2k_core::physics::motion`, plain floats) on the variables
/// at `v`; `r` as the original leaves it.
// Converted: `rc2k_core::physics::motion::blend_load`.
pub unsafe fn blend_at_plain(v: &[u32], r: &mut Regs) {
    unsafe {
        let get = |k: usize| f32::from_bits((v[k] as *const u32).read_unaligned());
        let put = |k: usize, x: f32| (v[k] as *mut u32).write_unaligned(x.to_bits());
        let k = |x: usize| f32::from_bits(game_read::<u32>(rva(x)));
        let figure = get(0);
        let threshold = k(0x614C64);
        // The signs, the constants and the slots read only past the first compare.
        if rc2k_core::physics::x87::below(figure as f64, threshold as f64) {
            return;
        }
        let b = LoadBlend {
            figure,
            signs: [get(1), get(2), get(3)],
            threshold,
            divisor: k(0x5C5D60),
            zero: k(0x5C5D90),
            one: k(0x5C5D38),
        };
        let mut slots: [f32; 8] = std::array::from_fn(|n| get(5 + n));
        if let Some((slot, share)) = blend_load(&b, &mut slots) {
            r.edx = slot;
            put(4, share);
            put(5 + slot as usize, slots[slot as usize]);
        }
    }
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version (the exact one, for `physics::car_physics_step`); `r` as the
/// original leaves it.
pub unsafe fn blend_wheel_load(r: &mut Regs) {
    unsafe { blend_at(&game(), r) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let mut g = *r;
    unsafe { blend_at_plain(&game(), &mut g) };
    Ok(Left::nothing().with(EDX, g.edx))
}

extern "C" fn dispatch(r: &mut Regs) {
    let v = game();
    let floats = Floats {
        memory: &[((v[4], 4), FLOAT), ((v[5], 0x20), FLOAT)],
        regs: &[],
    };
    dispatch_state_floats(
        &HOOK,
        r,
        call_original,
        &[(v[4], 4), (v[5], 0x20)],
        &floats,
        &rust,
    );
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}

fn fill(seed: u32, out: &mut [u32]) {
    let f = |x: f32| x.to_bits();
    out[0] = f([0.0f32, 5.0, 50.0, -3.0, 1000.0, f32::NAN, 12.0, 0.5][(seed % 8) as usize]);
    for (k, o) in out.iter_mut().enumerate().take(4).skip(1) {
        *o = f(if (seed >> (k - 1)) & 1 == 0 {
            1.5
        } else {
            -2.5
        });
    }
    out[4] = 0xAAAA_AAAA;
    for (k, o) in out.iter_mut().enumerate().take(13).skip(5) {
        *o = f((k as f32 - 5.0) / 9.0);
    }
}

fn self_test() -> (usize, usize) {
    test_moves_floats(
        &HOOK,
        0x440F70,
        0xD6,
        &[0xD9, 0x05, 0x08, 0xA2, 0x71, 0x00],
        &VARIABLES,
        14,
        16,
        &fill,
        &|v, r| {
            unsafe { blend_at_plain(v, r) };
            Left::nothing().with(EDX, r.edx)
        },
        &[4, 5, 6, 7, 8, 9, 10, 11, 12].map(|k| (k, FLOAT)),
        &[],
    )
}
