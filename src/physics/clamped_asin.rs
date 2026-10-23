//! `physics::clamped_asin` (0x42120B): the angle whose sine is st(0). The value is stored
//! as a float in `dword_62400C`, held there to -`flt_5C5D38` ..= `flt_5C5D38` (1.0; the
//! limit is negated in place for the second compare and put back), and the result is
//! `fpatan` of st(0) (unheld, 80 bits) over the root of 1 less the held float's square.
//! A value past the limit gives +-pi/2.
//!
//! In:  st(0) the sine
//! Out: st(0) the angle (in its place); every register kept; flags changed (no caller
//!      reads them)
//!
//! The thunk (as `physics::scale_vector`'s) pops st(0) into `INPUT`, runs `dispatch`
//! inside the gate and loads `RESULT`; `original_with_input` runs the original on
//! `INPUT` and pops its result into `RESULT`, so shadow mode compares it.
//! Callers: `physics::matrix_angles` (twice), unowned code at 0x42119E.
//! Self-test: a copy of the original (its scratch and limit the test's) on sines by lot
//! (80-bit ones, +-1, past the limit, NaN) and three limits, under three precisions.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{game_addr, rva};
use crate::math::x87_run::clamped_asin_80;
use crate::scene::matrix3::{extended, test_floats};

const SCRATCH_RVA: usize = rva(0x62400C); // dword_62400C
const LIMIT_RVA: usize = rva(0x5C5D38); // flt_5C5D38, 1.0

/// The Rust core: the scratch float and the limit at `scratch`, `limit`.
pub unsafe fn clamped_asin_at(x: [u8; 10], scratch: u32, limit: u32) -> [u8; 10] {
    unsafe { clamped_asin_80(x, scratch as *mut u32, limit as *mut u32) }
}

/// The Rust version.
pub unsafe fn clamped_asin(x: [u8; 10]) -> [u8; 10] {
    unsafe {
        clamped_asin_at(
            x,
            game_addr(SCRATCH_RVA) as u32,
            game_addr(LIMIT_RVA) as u32,
        )
    }
}

/// Runs code at `addr` with the 80-bit `x` in st(0); its st(0) into `out`.
#[unsafe(naked)]
pub unsafe extern "C" fn run_80(_addr: u32, _x: *const [u8; 10], _out: *mut [u8; 10]) {
    core::arch::naked_asm!(
        "mov eax, [esp + 8]",
        "fld tbyte ptr [eax]",
        "call dword ptr [esp + 4]",
        "mov eax, [esp + 12]",
        "fstp tbyte ptr [eax]",
        "ret",
    )
}

/// Test sines: by lot in -1.5 .. 1.5, with odd low bits now and then (not floats), and
/// the edge cases.
pub fn test_sine(seed: u32) -> [u8; 10] {
    match seed % 8 {
        0 => extended(0x3F80_0000 ^ ((seed & 8) << 28)), // +-1
        1 => extended(0x7FC0_0000),                      // NaN
        2 => extended(0x3FC0_0000 ^ ((seed & 8) << 28)), // +-1.5
        _ => {
            let mut f = [0u32; 1];
            test_floats(seed * 5 + 1, &mut f);
            let v = (f32::from_bits(f[0]) % 1.5).to_bits();
            let mut x = extended(v);
            if seed & 1 != 0 {
                x[0] ^= (seed as u8) | 1;
            }
            x
        }
    }
}
