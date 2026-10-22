//! `physics::mark_car_started` (0x436AA7): `g_car` +0xA0 (`unk_71A078`) set to 1 when
//! it is 0 and +0x29C (`unk_71A274`) is not (a reading: the car has got going).
//!
//! In:  nothing
//! Out: every register kept; no flags read (the callers return next)
//!
//! Caller: `sub_436790` (four calls).
//! Self-test: `physics/copy_hooks_test.rs`, 8 fillings over both words' zero and not.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, game_addr, hook_asm, rva};
use crate::physics::cells::{get, put_changed};
use crate::physics::copy_hooks_test::{lot, test_moves};
use crate::screens::harness::Left;
use crate::service::harness::{dispatch_state, state_entry};
use rc2k_core::physics::start::CarStart;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::mark_car_started", rva(0x436AA7), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The variables (IDA addresses).
const VARIABLES: [usize; 2] = [0x71A078, 0x71A274];

/// The Rust version on the variables' addresses.
// Converted: `rc2k_core::physics::start::CarStart::mark_started`.
unsafe fn core(v: &[u32]) -> Left {
    unsafe {
        let was = CarStart {
            started: get(v[0]),
            moving: get(v[1]),
            ..CarStart::default()
        };
        let mut c = was;
        c.mark_started();
        put_changed(v[0], was.started, c.started);
        Left::nothing()
    }
}

/// The Rust version.
pub unsafe fn mark_car_started() -> Left {
    unsafe { core(&game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { mark_car_started() })
}

extern "C" fn dispatch(r: &mut Regs) {
    let v = game();
    dispatch_state(&HOOK, r, call_original, &[(v[0], 4)], &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    test_moves(
        &HOOK,
        0x436AA7,
        0x1D,
        &[0x83, 0x3D, 0x78, 0xA0, 0x71, 0x00, 0x00],
        &VARIABLES,
        3,
        8,
        &|seed, out| {
            lot(seed, out);
            if seed & 1 != 0 {
                out[0] = 0;
            }
            if seed & 2 != 0 {
                out[1] = 0;
            }
        },
        &|v, _| unsafe { core(v) },
    )
}
