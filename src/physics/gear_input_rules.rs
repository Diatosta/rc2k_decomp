//! `physics::gear_input_rules` (0x439BF6): the gearbox's input state (0x613890 ..
//! 0x6138A8) by the car's state (the work copy's `dword_71A244`, `dword_71A010`,
//! `dword_719FEC`, `dword_71A07C`, `dword_71A078`, `dword_71A080`): with `dword_71A244`
//! neither 0 nor -1 all cleared, 0x61389C = 500 and `g_carGear` 0 (nothing else); with
//! `dword_71A010` 2 the two inputs 0x613894 / 0x613898 joined into the first (in neutral,
//! `dword_6138A8` = 1, or -1 when the second is the larger) or swapped (in reverse);
//! with `dword_719FEC` below 800 (the sign of the difference) 0x61389C = 500 and, unless
//! `dword_62389C` is 1, four cleared; each of three switches clears them and the gear.
//! What they are is a reading at best (an automatic box's held inputs?).
//!
//! In:  nothing
//! Out: eax the joined input when they were joined; the rest kept; no flags read
//!
//! Callers: race init's loop (`sub_4608BB`), `sub_460835`, `sub_460B2F` (each calls
//! `sub_43B635` or `sub_439DB1` next).
//! Self-test: `physics/copy_hooks_test.rs` (a copy on test variables), 48 cases.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, game_addr, hook_asm, rva};
use crate::physics::copy_hooks_test::{lot, test_moves};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::gear_input_rules", rva(0x439BF6), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The variables (IDA addresses), in the order the core uses them.
const VARIABLES: [usize; 15] = [
    0x71A244, 0x6138A4, 0x6138A0, 0x613894, 0x613898, 0x61389C, 0x71A1AC, 0x71A010, 0x6138A8,
    0x719FEC, 0x62389C, 0x613890, 0x71A07C, 0x71A078, 0x71A080,
];
const HELD: u32 = 0x1F4;

unsafe fn get(v: &[u32], k: usize) -> u32 {
    unsafe { (v[k] as *const u32).read_unaligned() }
}

unsafe fn put(v: &[u32], k: usize, x: u32) {
    unsafe { (v[k] as *mut u32).write_unaligned(x) }
}

/// The Rust core on the variables at `v`; `r` as the original leaves it.
pub unsafe fn rules_at(v: &[u32], r: &mut Regs) {
    unsafe {
        let all_clear = |gear_too: bool| {
            put(v, 1, 0);
            put(v, 2, 0);
            put(v, 3, 0);
            put(v, 4, 0);
            put(v, 5, HELD);
            if gear_too {
                put(v, 6, 0);
            }
        };
        let state = get(v, 0);
        if state != 0 && state != u32::MAX {
            all_clear(true);
            return;
        }
        if get(v, 7) == 2 {
            let gear = get(v, 6);
            if gear == 0 {
                put(v, 8, 1);
                let (a, b) = (get(v, 3), get(v, 4));
                r.eax = a;
                if (a.wrapping_sub(b) as i32) < 0 {
                    put(v, 8, u32::MAX);
                    r.eax = b;
                }
                put(v, 3, r.eax);
                put(v, 4, 0);
            } else if (gear as i32) < 0 {
                let (a, b) = (get(v, 3), get(v, 4));
                put(v, 3, b);
                put(v, 4, a);
            }
        }
        if (get(v, 9).wrapping_sub(0x320) as i32) < 0 {
            put(v, 5, HELD);
            if get(v, 10) != 1 {
                put(v, 4, 0);
                put(v, 11, 0);
                put(v, 2, 0);
                put(v, 1, 0);
            }
        }
        if get(v, 12) != 0 {
            all_clear(true);
        }
        if get(v, 13) != 0 {
            all_clear(true);
        }
        if get(v, 14) != 0 {
            put(v, 3, 0);
            put(v, 4, 0);
            put(v, 5, HELD);
            put(v, 6, 0);
        }
    }
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version; `r` as the original leaves it.
pub unsafe fn gear_input_rules(r: &mut Regs) {
    unsafe { rules_at(&game(), r) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let mut g = *r;
    unsafe { gear_input_rules(&mut g) };
    Ok(Left::nothing().with(EAX, g.eax))
}

extern "C" fn dispatch(r: &mut Regs) {
    let v = game();
    let regions: Vec<_> = [1usize, 2, 3, 4, 5, 6, 8, 11]
        .iter()
        .map(|&k| (v[k], 4))
        .collect();
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn fill(seed: u32, out: &mut [u32]) {
    lot(seed, out);
    out[0] = [0, u32::MAX, 0, 5][(seed % 4) as usize];
    out[7] = if seed.is_multiple_of(3) { 1 } else { 2 };
    out[6] = [0, 1, u32::MAX, 3][(seed / 4 % 4) as usize];
    out[9] = [0x31F, 0x320, 0x1000, 0x8000_0000][(seed / 3 % 4) as usize];
    out[10] = seed % 2;
    for (k, o) in out.iter_mut().enumerate().skip(12) {
        *o = u32::from(seed.wrapping_mul(7 + k as u32).is_multiple_of(5));
    }
}

fn self_test() -> (usize, usize) {
    test_moves(
        &HOOK,
        0x439BF6,
        0x1BB,
        &[0x83, 0x3D, 0x44, 0xA2, 0x71, 0x00, 0x00],
        &VARIABLES,
        47,
        48,
        &fill,
        &|v, r| {
            unsafe { rules_at(v, r) };
            Left::nothing().with(EAX, r.eax)
        },
    )
}
