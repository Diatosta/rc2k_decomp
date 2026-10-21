//! `physics::cap_reverse_inputs` (0x46514A): in reverse (`g_carGear` -1) while the car's
//! speed (`g_carSpeed`) is not below 0 (an unordered compare counts as below), the two
//! floats `flt_614234` and `flt_614238` held to at most `flt_5C5D3C` (2.0; one that is
//! below it, or unordered, is left).
//!
//! In:  nothing
//! Out: every register kept; no flags read (the caller's next is a call)
//!
//! Caller: `sub_43DFB1` (0x43E519).
//! Self-test: a copy of the original (its four variables the test's) against the converted
//! version (compares and copies of floats only: exact at any precision):
//! gears, speeds about 0 and NaN, values about the cap.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{capture, dispatch_state};
use rc2k_core::physics::drive_forces::cap_reverse;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::cap_reverse_inputs",
    rva(0x46514A),
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

const CAP_RVA: usize = rva(0x5C5D3C); // flt_5C5D3C, 2.0
const ZERO_RVA: usize = rva(0x5C5D90); // flt_5C5D90, 0.0

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The converted version (`rc2k_core::physics::drive_forces`) on the gear, the speed and
/// the two values at these places.
// Converted: `rc2k_core::physics::drive_forces::cap_reverse`.
pub unsafe fn cap_reverse_inputs_at(gear: u32, speed: u32, values: [u32; 2]) {
    unsafe {
        if at(gear) != u32::MAX {
            return; // the speed and the values not read, as the original
        }
        let f = |a: u32| f32::from_bits(at(a));
        let zero = f32::from_bits(game_read(ZERO_RVA));
        let cap = f32::from_bits(game_read(CAP_RVA));
        let Some(held) = cap_reverse(at(gear), f(speed), zero, [f(values[0]), f(values[1])], cap)
        else {
            return;
        };
        for (v, h) in values.iter().zip(held) {
            if at(*v) != h.to_bits() {
                (*v as *mut u32).write_unaligned(h.to_bits());
            }
        }
    }
}

fn game() -> (u32, u32, [u32; 2]) {
    let g = |a: usize| game_addr(rva(a)) as u32;
    (g(0x71A1AC), g(0x71A184), [g(0x614234), g(0x614238)])
}

/// The Rust version (compares and copies of floats only: the same at any precision).
pub unsafe fn cap_reverse_inputs() {
    let (gear, speed, values) = game();
    unsafe { cap_reverse_inputs_at(gear, speed, values) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    let (gear, speed, values) = game();
    unsafe { cap_reverse_inputs_at(gear, speed, values) };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let (_, _, values) = game();
    dispatch_state(&HOOK, r, call_original, &[(values[0], 8)], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut cells = vec![0u32; 4];
    let base = cells.as_mut_ptr() as u32;
    let (gear, speed, values) = (base, base + 4, [base + 8, base + 12]);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x46514A,
            len: 0x62,
            entry: &[0x83, 0x3D, 0xAC, 0xA1, 0x71, 0x00, 0xFF],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[
            (0x71A1AC, gear),
            (0x71A184, speed),
            (0x614234, values[0]),
            (0x614238, values[1]),
        ],
        6,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let all = [(base, 16)];
    let floats = [0.0f32, -0.0, 1.5, 2.0, 3.0, -4.0];
    let mut n = 0u32;
    for g in [u32::MAX, 0, 1] {
        for s in [
            0x0000_0000u32,
            0x8000_0000,
            0x4000_0000,
            0xC000_0000,
            0x7FC0_0000,
        ] {
            for k in 0..3usize {
                n += 1;
                let (a, b) = (
                    floats[k * 2 % 6].to_bits(),
                    if k == 2 {
                        0x7FC0_0000
                    } else {
                        floats[(k * 2 + 3) % 6].to_bits()
                    },
                );
                let fill = || unsafe {
                    (base as *mut [u32; 4]).write([g, s, a, b]);
                };
                fill();
                let mut ro = Regs::SENTINEL;
                unsafe { call_game(&mut ro, copy.addr()) };
                let theirs = capture(&all);
                fill();
                unsafe { cap_reverse_inputs_at(gear, speed, values) };
                let mine = capture(&all);
                let regs_ok = ro.preserved_except(&Regs::SENTINEL, 0);
                t.check(regs_ok && theirs == mine, || {
                    format!("case {n}: gear {g:#x}, speed {s:#x}, values {a:#x} {b:#x}: regs_ok={regs_ok} original {theirs:x?}; rust {mine:x?}")
                });
            }
        }
    }
    drop((copy, cells));
    t.finish()
}
