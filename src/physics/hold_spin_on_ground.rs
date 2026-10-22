//! `physics::hold_spin_on_ground` (0x43BAA4): with all four wheels down (the four dwords
//! from `qword_71A5D8` not 0), the spin's x and z (`dword_71A15C`, `dword_71A164`) held
//! to -`flt_613EA0` .. `flt_613EA0` (by negating that limit in place, twice for each, so
//! it comes back), and `dword_614C24` = whether either now has the limit's size (C3: also
//! when no number). A reading: no tumbling on the ground.
//!
//! In:  nothing
//! Out: ecx the wheels down, ebx 4; the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B986), every physics step.
//! Self-test: `physics/copy_hooks_test.rs` (a copy on test variables against the
//! converted version), 24 cases; compares and copies of floats only: exact at any
//! precision.

use crate::errors::Refused;
use crate::framework::{EBX, ECX, RawHook, Regs, game_addr, hook_asm, rva};
use crate::physics::copy_hooks_test::{lot, test_moves};
use crate::screens::harness::Left;
use crate::service::harness::{dispatch_state, state_entry};
use rc2k_core::physics::motion::hold_spin;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::hold_spin_on_ground",
    rva(0x43BAA4),
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

/// The four wheels, the limit, x, z, the flag.
const VARIABLES: [usize; 8] = [
    0x71A5D8, 0x71A5DC, 0x71A5E0, 0x71A5E4, 0x613EA0, 0x71A15C, 0x71A164, 0x614C24,
];

/// The converted version (`rc2k_core::physics::motion`) on the variables at `v`; `r` as
/// the original leaves it.
// Converted: `rc2k_core::physics::motion::hold_spin`.
pub unsafe fn hold_at(v: &[u32], r: &mut Regs) {
    unsafe {
        let get = |k: usize| (v[k] as *const u32).read_unaligned();
        let put = |k: usize, x: u32| (v[k] as *mut u32).write_unaligned(x);
        r.ecx = (0..4).filter(|&k| get(k) != 0).count() as u32;
        r.ebx = 4;
        if r.ecx != 4 {
            return;
        }
        let f = |k: usize| f32::from_bits(get(k));
        let (mut limit, mut x, mut z) = (f(4), f(5), f(6));
        let flag = hold_spin(&mut limit, &mut x, &mut z);
        put(4, limit.to_bits());
        put(5, x.to_bits());
        put(6, z.to_bits());
        put(7, flag);
    }
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version (compares and copies of floats only: the same at any precision); `r`
/// as the original leaves it.
pub unsafe fn hold_spin_on_ground(r: &mut Regs) {
    unsafe { hold_at(&game(), r) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let mut g = *r;
    unsafe { hold_at(&game(), &mut g) };
    Ok(Left::nothing().with(ECX, g.ecx).with(EBX, g.ebx))
}

extern "C" fn dispatch(r: &mut Regs) {
    let v = game();
    let regions: Vec<_> = [4usize, 5, 6, 7].iter().map(|&k| (v[k], 4)).collect();
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn fill(seed: u32, out: &mut [u32]) {
    lot(seed, out);
    for (k, o) in out.iter_mut().enumerate().take(4) {
        if seed % 5 == k as u32 {
            *o = 0;
        }
    }
    let f = |x: f32| x.to_bits();
    out[4] = f([2.0f32, 0.5, 0.0, f32::NAN][(seed % 4) as usize]);
    out[5] = f([3.0f32, -3.0, 1.0, 2.0, -0.25, f32::NAN][(seed % 6) as usize]);
    out[6] = f([-2.0f32, 0.5, 4.0][(seed % 3) as usize]);
}

fn self_test() -> (usize, usize) {
    test_moves(
        &HOOK,
        0x43BAA4,
        0x127,
        &[0xB9, 0x00, 0x00, 0x00, 0x00],
        &VARIABLES,
        31,
        24,
        &fill,
        &|v, r| {
            unsafe { hold_at(v, r) };
            Left::nothing().with(ECX, r.ecx).with(EBX, r.ebx)
        },
    )
}
