//! `scene::no_mips_for` (0x46512F): ebp (the mip levels wanted) cleared for the texture
//! sizes that get none: 0x80 .. 0xFF and 0x140 on (esi, unsigned).
//!
//! In:  esi the size, ebp the mip levels
//! Out: ebp 0 for those sizes, else as it came; everything else preserved
//! One caller, `gfx::allocate_textures`'s caller 0x42263E (`scene::try_allocate_textures`,
//! whose Rust version inlines it). Shadow mode: both, compared.
//! Self-test: the original against the Rust version on sizes around the bounds.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EBP, Regs};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// ---- Rust version ----

/// True when the size gets no mip levels.
pub fn no_mips_for(size: u32) -> bool {
    // Converted: `rc2k_core::scene::textures::no_mips_for`.
    rc2k_core::scene::textures::no_mips_for(size)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(if no_mips_for(r.esi) {
        Left::nothing().with(EBP, 0)
    } else {
        Left::nothing()
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
