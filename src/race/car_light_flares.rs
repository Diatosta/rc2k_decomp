//! `race::car_light_flares` (0x457036): a car's lights as flares (see `race/frame_draw.rs`;
//! `sub_457241` a pair of lights, `sub_4572B0` one placed in the car's frame,
//! `sub_457332` added to the flare tables and the depth list, `sub_45721A`,
//! `sub_457229`, `sub_41F4E1` covered here: their only callers are in this tree).
//!
//! In:  esi = the car's block
//! Out: the registers as it leaves them (its caller calls it inside `pusha`)
//!
//! With `dword_621870` 8 one flare at `dword_641744` .. in white (0xF0F0F0). Else, with
//! `dword_621480`, for a known kind (+0x24 below 0x22 with `dword_5C5C58`) only below 100.0
//! of +0x310 (its +0x298 to `dword_641754`), and for any other kind always: the kind's
//! light record (`g_lightRecords`, by `dword_5C59B0` of the kind held to 0 .. 0x21) gives
//! up to nine pairs at fixed offsets, each light lit by the block's byte at its place, in
//! three colours (0x606060, 0xC00000, 0x800000). The original's second kind test and its
//! log line ("ERROR - light attempted on car type") are never reached (`notes/bugs.md`).
//! Refuses (before anything) a depth list without room for 24 flares. Shadow mode: by
//! state. Self-test: a test block (frame, place, kind, the lights' bytes), the switches,
//! the flare and depth counts; all put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { car_light_flares(r)? }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
