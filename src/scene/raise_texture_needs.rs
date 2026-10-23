//! `scene::raise_texture_needs` (0x423F97): the textures of one tag are needed at a
//! level at least.
//!
//! In:  eax = the level, ebx = the tag
//! Out: ecx = 0x400, edx = 1 if any entry changed, else 0; zero set if any changed,
//!      carry set if none; everything else preserved
//!
//! For every texture whose tag (`dword_5CDAE0`) is ebx: its level in each of the two
//! tables (`dword_5CEAE0`, `dword_5CFAE0`) becomes eax if eax is above it, unsigned
//! (so -1, "not needed", is never raised).
//! 57 callers.
//! Self-test: 40 cases by lot on the game's tables, put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::scene::texture_needs::{SYSTEM_LEVELS_RVA, TAGS_RVA, TEXTURES, VIDEO_LEVELS_RVA};

// ---- Rust version ----

/// Whether any entry changed.
pub unsafe fn raise_texture_needs(level: u32, tag: u32) -> bool {
    // Converted: `rc2k_core::scene::textures::Needs::raise`.
    use crate::scene::textures::slice;
    let n = TEXTURES as u32;
    unsafe {
        rc2k_core::scene::textures::Needs {
            tags: slice(TAGS_RVA, n),
            system: slice(SYSTEM_LEVELS_RVA, n),
            video: slice(VIDEO_LEVELS_RVA, n),
        }
        .raise(level, tag)
    }
}
