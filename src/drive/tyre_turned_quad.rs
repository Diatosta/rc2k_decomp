//! `drive::tyre_turned_quad` (0x41C969): the tyre-change screen's turning quad (the
//! steering wheel): the device's turn (esi +0x4EC, x `flt_5BEAC8` x 2 pi, `fsincos` into
//! `flt_5BEBF4` cos, `flt_5BEBF0` sin) turning the half-sizes of the rectangle at ebx
//! (floats: x, y, w, h) round its place plus `dword_5BEAE0` / `5BEAE4`, the colour ecx, the
//! texture rectangle at eax (`quad_tail`), drawn (place 5). All in `pusha` / `popa`.
//!
//! In:  eax the texture rectangle, ebx the rectangle, ecx the colour, esi the input device
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
use crate::physics::flow::{a, base, p, x87_run};
use crate::race::car_starts::Part;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { tyre_turned_quad(r, &mut game_part) }?;
        let esp = r.esp;
        *r = Regs {
            eflags: r.eflags,
            ..g
        };
        r.esp = esp;
    }
    Ok(())
}

/// The Rust version, the draw through `part`.
pub unsafe fn tyre_turned_quad(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let (uv, rect, colour, device) = (entry.eax, entry.ebx, entry.ecx, entry.esi);
    p(0x5183CC, 0x40C9_0FDB);
    x87_run!(
        "fild dword ptr [{e} + 0x4EC]",
        "fmul dword ptr [{b} + 0x1BEAC8]",
        "fmul dword ptr [{b} + 0x1183CC]",
        ".byte 0xD9, 0xFB", // fsincos
        "fstp dword ptr [{b} + 0x1BEBF4]",
        "fstp dword ptr [{b} + 0x1BEBF0]",
        "fld dword ptr [{d} + 0xC]",
        "fmul dword ptr [{b} + 0x1BEBF0]",
        "fstp dword ptr [{b} + 0x1BEBF8]",
        "fld dword ptr [{d} + 0xC]",
        "fmul dword ptr [{b} + 0x1BEBF4]",
        ".byte 0xD9, 0xE0", // fchs
        "fstp dword ptr [{b} + 0x1BEBFC]",
        "fld dword ptr [{d} + 0x8]",
        "fmul dword ptr [{b} + 0x1BEBF4]",
        "fstp dword ptr [{b} + 0x1BEC00]",
        "fld dword ptr [{d} + 0x8]",
        "fmul dword ptr [{b} + 0x1BEBF0]",
        "fstp dword ptr [{b} + 0x1BEC04]",
        "fld dword ptr [{b} + 0x1BEC00]",
        ".byte 0xD9, 0xE0",
        "fadd dword ptr [{d}]",
        "fiadd dword ptr [{b} + 0x1BEAE0]",
        "fst dword ptr [{v} + 0x40]",
        "fadd dword ptr [{b} + 0x1BEBF8]",
        "fstp dword ptr [{v}]",
        "fld dword ptr [{b} + 0x1BEC00]",
        "fadd dword ptr [{d}]",
        "fiadd dword ptr [{b} + 0x1BEAE0]",
        "fst dword ptr [{v} + 0x60]",
        "fadd dword ptr [{b} + 0x1BEBF8]",
        "fstp dword ptr [{v} + 0x20]",
        "fld dword ptr [{b} + 0x1BEC04]",
        ".byte 0xD9, 0xE0",
        "fadd dword ptr [{d} + 0x4]",
        "fiadd dword ptr [{b} + 0x1BEAE4]",
        "fst dword ptr [{v} + 0x44]",
        "fadd dword ptr [{b} + 0x1BEBFC]",
        "fstp dword ptr [{v} + 0x4]",
        "fld dword ptr [{b} + 0x1BEC04]",
        "fadd dword ptr [{d} + 0x4]",
        "fiadd dword ptr [{b} + 0x1BEAE4]",
        "fst dword ptr [{v} + 0x64]",
        "fadd dword ptr [{b} + 0x1BEBFC]",
        "fstp dword ptr [{v} + 0x24]";
        b = base(), e = device, d = rect, v = a(VERTS)
    );
    quad_words(colour);
    quad_tail(uv);
    draw(
        &Regs {
            esi: uv,
            edi: rect,
            ..*entry
        },
        part,
    )?;
    Ok(*entry)
}
