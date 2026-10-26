//! `drive::tyre_quad` (0x41DD3B): a screen quad of the tyre-change screen: the screen
//! rectangle at ebx (x, y, w, h, whole numbers) as the four vertices' places
//! (`g_polygonVertices`), z 0, rhw 1, the colour ecx, the texture rectangle at eax
//! (`drive::tyre_parts`' `quad_tail`, scaled to the screen), drawn (place 5, the device's
//! DrawPrimitive: a call into the game). All in `pusha` / `popa`.
//!
//! In:  eax the texture rectangle, ebx the screen rectangle, ecx the colour
//! Out: every register kept; no flags read
//!
//! Callers: `drive::tyre_screen` (two), `drive::tyre_screen_parts` (ten).
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
use crate::physics::flow::p;
use crate::race::car_starts::Part;
use rc2k_core::drive::tyre_screen::quad_corners;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { tyre_quad(r, &mut game_part) }?;
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
// Converted: `rc2k_core::drive::tyre_screen::quad_corners` (the shared tail stays x87).
pub unsafe fn tyre_quad(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let (uv, rect, colour) = (entry.eax, entry.ebx, entry.ecx);
    let at = |o: u32| unsafe { ((rect + o) as *const u32).read_unaligned() };
    let corners = quad_corners([at(0), at(4), at(8), at(0xC)]);
    for (k, [x, y]) in corners.into_iter().enumerate() {
        p(VERTS + 0x20 * k, x.to_bits());
        p(VERTS + 0x20 * k + 4, y.to_bits());
    }
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
