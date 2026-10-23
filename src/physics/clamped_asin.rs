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

use crate::math::x87_run::clamped_asin_80;

// dword_62400C
// flt_5C5D38, 1.0

/// The Rust core: the scratch float and the limit at `scratch`, `limit`.
pub unsafe fn clamped_asin_at(x: [u8; 10], scratch: u32, limit: u32) -> [u8; 10] {
    unsafe { clamped_asin_80(x, scratch as *mut u32, limit as *mut u32) }
}
