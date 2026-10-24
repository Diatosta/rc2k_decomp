//! `drive::reset_request` (0x43A0D3): the reset key held. Once the race is past 0x320
//! (`unk_719FEC`, by the sign): with the keys' bit 0x80 (`unk_71A440`), outside mode 2
//! (`dword_62389C`) and the car's speed below `flt_613828` (or unordered): a notice set up
//! on the first step (`unk_71A6F4` 0xCD, `unk_71A708` 0x19, `unk_71A704` 0x12C), the count
//! `unk_71A23C` + 1, at 100 the time penalty (0x2BC less `unk_71A1C0`) added to
//! `unk_71A060`, the notice cleared and `physics::reset_car`. Else a count under way is
//! cleared with the notice.
//!
//! In:  nothing
//! Out: ebx `unk_71A440` past the first test; at the reset as `physics::reset_car` leaves
//!      the registers (eax the penalty); the rest kept; no flags read
//!
//! Callers: `drive::car_controls`, `sub_460835`, `sub_460B2F`.
//! Refuses what `physics::reset_car` refuses. Shadow mode runs the original alone at the
//! reset (`physics/reset_parts.rs`), else compares.
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back) by lot, counts below, at and past 100; the reset's call through `reset_parts`'
//! stand-in in a copy.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::physics::flow::{c0, g, p};
use rc2k_core::drive::controls::{
    RESET_NOTICE, reset_counts, reset_held, reset_penalty, reset_step,
};

const COUNT: usize = 0x71A23C;

fn clear_notice() {
    for x in [0x71A6FC, 0x71A6F4, 0x71A704, 0x71A700] {
        p(x, 0);
    }
}

/// The Rust version, then `physics::reset_car`.
// Converted: `rc2k_core::drive::controls::{reset_counts, reset_held, reset_step, reset_penalty}`.
pub unsafe fn reset_request() -> Result<(), Refused> {
    if !reset_counts(g(0x719FEC)) {
        return Ok(());
    }
    if !reset_held(g(0x71A440), g(0x62389C), || c0(g(0x71A19C), g(0x613828))) {
        if g(COUNT) != 0 {
            clear_notice();
            p(COUNT, 0);
        }
        return Ok(());
    }
    let (first, count, reset) = reset_step(g(COUNT));
    if first {
        p(0x71A6F4, RESET_NOTICE);
        p(0x71A708, 0x19);
        p(0x71A704, 0x12C);
    }
    p(COUNT, count);
    if !reset {
        return Ok(());
    }
    p(
        0x71A060,
        g(0x71A060).wrapping_add(reset_penalty(g(0x71A1C0))),
    );
    clear_notice();
    unsafe { crate::physics::reset_car::reset_car() }?;
    Ok(())
}
