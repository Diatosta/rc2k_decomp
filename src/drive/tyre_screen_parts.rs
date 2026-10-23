//! `drive::tyre_screen_parts` (0x41C790): the tyre-change screen's fixed parts: ten quads
//! (`drive::tyre_quad`, place 2) from texture rectangles set in `dword_5C2A1C` and screen
//! rectangles in `dword_5C2A0C`, placed from the layout (`dword_5BEBD0` .., `dword_5BEAE0`
//! .., the car's half by `dword_5BEBEC`). All in `pusha` / `popa`.
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `drive::tyre_screen`.
//!
//! Shadow mode runs the original alone (it draws). Self-test: a copy whose calls go to
//! stand-ins noting them (`drive/tyre_parts.rs`), against the Rust version with the same.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::drive::tyre_parts::*;
use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::flow::{a, g, p};
use rc2k_core::drive::tyre_screen::fixed_parts;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { tyre_screen_parts(r) }?;
        let esp = r.esp;
        *r = Regs {
            eflags: r.eflags,
            ..g
        };
        r.esp = esp;
    }
    Ok(())
}

const UV: usize = 0x5C2A1C;
const RECT: usize = 0x5C2A0C;

fn set(at: usize, v: [u32; 4]) {
    for (k, x) in v.iter().enumerate() {
        p(at + 4 * k, *x);
    }
}

/// The Rust version, the quads through `part`.
// Converted: `rc2k_core::drive::tyre_screen::fixed_parts`.
pub unsafe fn tyre_screen_parts(entry: &Regs) -> Result<Regs, Refused> {
    let mut q = Regs {
        eax: a(UV),
        ebx: a(RECT),
        ecx: 0xFF_FFFF,
        ..*entry
    };
    let four = |at: usize| std::array::from_fn(|k| g(at + 4 * k));
    for (uv, rect) in fixed_parts(four(0x5BEBD0), four(0x5BEAE0), g(0x5BEBEC) != 0) {
        set(UV, uv);
        set(RECT, rect);
        crate::drive::tyre_parts::game_part(QUAD, &mut q)?;
    }
    Ok(*entry)
}
