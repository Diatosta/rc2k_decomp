//! `physics::air_drag` (0x43F2DA): for the motion's three (`dword_71A17C`,
//! `dword_71A180`, `g_carSpeed`): its square times `flt_613FCC` over `flt_5C5E34` into
//! `flt_614810` .. `flt_614818`, made against the motion (|it| when the motion is below 0
//! or no number, else -|it|). By the look of it the air's drag (a reading).
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B80E), every physics step.
//! Self-test: `physics/copy_hooks_test.rs` (`test_moves_floats`: a copy on test
//! variables against the converted version), 8 fillings (signs, zeros, NaN), exact at
//! double precision.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, game_addr, game_read, hook_asm, rva};
use crate::math::x87;
use crate::math::x87_run::{abs_of, compare_full, square_times_over};
use crate::physics::copy_hooks_test::test_moves_floats;
use crate::screens::harness::Left;
use crate::service::floats::{FLOAT, Floats, dispatch_state_floats, floats_entry};
use rc2k_core::physics::motion::air_drag as air_drag_core;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::air_drag", rva(0x43F2DA), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const VARIABLES: [usize; 6] = [0x71A17C, 0x71A180, 0x71A184, 0x614810, 0x614814, 0x614818];

/// The Rust core on the variables at `v`.
pub unsafe fn drag_at(v: &[u32]) {
    unsafe {
        let get = |k: usize| (v[k] as *const u32).read_unaligned();
        let put = |k: usize, x: u32| (v[k] as *mut u32).write_unaligned(x);
        let (k, d, zero) = (
            game_read::<u32>(rva(0x613FCC)),
            game_read::<u32>(rva(0x5C5E34)),
            game_read::<u32>(rva(0x5C5D90)),
        );
        for i in 0..3 {
            put(3 + i, square_times_over(get(i), k, d));
            let a = abs_of(get(3 + i));
            let below = compare_full(get(i), zero) & 0x100 != 0;
            put(3 + i, if below { a } else { x87::negated(a) });
        }
    }
}

/// The converted version (`rc2k_core::physics::motion`, plain floats) on the variables
/// at `v`.
// Converted: `rc2k_core::physics::motion::air_drag`.
pub unsafe fn drag_at_plain(v: &[u32]) {
    unsafe {
        let get = |k: usize| f32::from_bits((v[k] as *const u32).read_unaligned());
        let k = |x: usize| f32::from_bits(game_read::<u32>(rva(x)));
        let drag = air_drag_core(
            [get(0), get(1), get(2)],
            k(0x613FCC),
            k(0x5C5E34),
            k(0x5C5D90),
        );
        for (n, d) in drag.iter().enumerate() {
            (v[3 + n] as *mut u32).write_unaligned(d.to_bits());
        }
    }
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version (the exact one, for `physics::car_physics_step`).
pub unsafe fn air_drag() {
    unsafe { drag_at(&game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { drag_at_plain(&game()) };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let v = game();
    let floats = Floats {
        memory: &[((v[3], 12), FLOAT)],
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, &[(v[3], 12)], &floats, &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}

fn fill(seed: u32, out: &mut [u32]) {
    let motion = [
        [3.0f32, -4.0, 25.0],
        [0.0, -0.0, 1.0],
        [-17.5, 2.25, f32::NAN],
        [100.0, -100.0, 0.5],
    ][(seed % 4) as usize];
    for k in 0..3 {
        out[k] = (motion[k] * (1.0 + seed as f32 / 10.0)).to_bits();
        out[3 + k] = 0xAAAA_AAAA;
    }
}

fn self_test() -> (usize, usize) {
    test_moves_floats(
        &HOOK,
        0x43F2DA,
        0xBB,
        &[0xD9, 0x05, 0x7C, 0xA1, 0x71, 0x00],
        &VARIABLES,
        15,
        8,
        &fill,
        &|v, _| {
            unsafe { drag_at_plain(v) };
            Left::nothing()
        },
        &[(3, FLOAT), (4, FLOAT), (5, FLOAT)],
        &[],
    )
}
