//! `hud::disc_vertex` (0x4B6D10): vertex edi of the HUD's disc from its ring entry ecx:
//! the texture place (`flt_75C210`[ecx], -`flt_75C254`[ecx], times `dword_75C050` times
//! `dword_75C04C`, plus `flt_75C01C` and `dword_75C054` / `dword_75C058`, over
//! `flt_75C020`); the place (`flt_75C188`[ecx], -`flt_75C1CC`[ecx], times `dword_75C050`,
//! plus `dword_75C044` / `dword_75C048`); colour white under the quad record's alpha,
//! specular 0.
//!
//! In:  ecx the ring entry (0 the centre .. 0x10), edi the vertex
//! Out: eax the colour; the rest kept
//!
//! Caller: `hud::draw_disc` (three a triangle). Refuses an entry past 0x10, before
//! anything. Self-test: the original and the Rust version on a vertex of the test's,
//! every entry.
//!
//! Converted 2026-10-21: the hook's Rust version is `disc_vertex_plain`, through
//! `rc2k_core::hud::disc::vertex` (plain floats; the x87 version stays for the Rust callers
//! in unconverted code). Shadow mode and the self-test compare within `service::floats`'
//! tolerances (the self-test at both test precisions, exact at double).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read};
use crate::hud::add_quad::QUAD_RVA;

pub use rc2k_core::hud::disc::ENTRIES;

unsafe fn place(b: u32, k: u32, e: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + {k} + 0x35C210]",
            "fmul dword ptr [{b} + 0x35C050]",
            "fmul dword ptr [{b} + 0x35C04C]",
            "fadd dword ptr [{b} + 0x35C01C]",
            "fadd dword ptr [{b} + 0x35C054]",
            "fdiv dword ptr [{b} + 0x35C020]",
            "fstp dword ptr [{e} + 0x18]",
            "fld dword ptr [{b} + {k} + 0x35C254]",
            "fchs",
            "fmul dword ptr [{b} + 0x35C050]",
            "fmul dword ptr [{b} + 0x35C04C]",
            "fadd dword ptr [{b} + 0x35C01C]",
            "fadd dword ptr [{b} + 0x35C058]",
            "fdiv dword ptr [{b} + 0x35C020]",
            "fstp dword ptr [{e} + 0x1C]",
            "fld dword ptr [{b} + {k} + 0x35C188]",
            "fmul dword ptr [{b} + 0x35C050]",
            "fadd dword ptr [{b} + 0x35C044]",
            "fstp dword ptr [{e}]",
            "fld dword ptr [{b} + {k} + 0x35C1CC]",
            "fchs",
            "fmul dword ptr [{b} + 0x35C050]",
            "fadd dword ptr [{b} + 0x35C048]",
            "fstp dword ptr [{e} + 4]",
            b = in(reg) b,
            k = in(reg) k,
            e = in(reg) e,
            out("st(0)") _,
            options(nostack),
        )
    }
}

/// The Rust version (see the header): ring entry `entry` into the vertex at `vertex`.
pub unsafe fn disc_vertex(entry: u32, vertex: u32) -> Result<(), Refused> {
    if entry >= ENTRIES {
        return Err(BadInput("a disc entry past the ring").into());
    }
    unsafe {
        place(game_addr(0) as u32, entry * 4, vertex);
        let colour = (game_read::<u32>(QUAD_RVA) << 24) | 0xFF_FFFF;
        ((vertex + 0x10) as *mut u32).write_unaligned(colour);
        ((vertex + 0x14) as *mut u32).write_unaligned(0);
    }
    Ok(())
}
