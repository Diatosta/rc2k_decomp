//! `settings::apply_feature_switches` (0x4114BB): which of the processor's extensions the
//! game will use.
//!
//! In:  nothing
//! Out: every register preserved
//! Three times the same: a feature is used if the processor has it (bit 0 of what the
//! processor test left) and the setting that says to ignore it is 0. MMX: `g_hasMmx`
//! (0x5183A8) from `dword_5183F8` and `ignoremmx` (0x5183AC); the second, by its setting
//! `ignorekatmai` SSE: `dword_5183B0` from `dword_5183FC` and 0x5183B4; 3DNow:
//! `g_has3dNow` (0x5183B8) from `dword_518400` and `ignoreamdk3d` (0x5183BC).
//! One caller, `start`, right after the settings are read.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::framework::{game_read, game_write, rva};

// ---- game data ----

/// (used, ignore setting, the processor has it)
const FEATURES: [(usize, usize, usize); 3] = [
    (rva(0x5183A8), rva(0x5183AC), rva(0x5183F8)), // g_hasMmx, dword_5183AC, dword_5183F8
    (rva(0x5183B0), rva(0x5183B4), rva(0x5183FC)), // dword_5183B0, dword_5183B4, dword_5183FC
    (rva(0x5183B8), rva(0x5183BC), rva(0x518400)), // g_has3dNow, dword_5183BC, dword_518400
];

// ---- Rust version ----

pub unsafe fn apply_feature_switches() {
    // Converted: `rc2k_core::files::settings::feature_switches`.
    unsafe {
        let has = FEATURES.map(|(_, _, has)| game_read::<u32>(has));
        let ignore = FEATURES.map(|(_, ignore, _)| game_read::<u32>(ignore));
        let used = rc2k_core::files::settings::feature_switches(has, ignore);
        for ((at, _, _), v) in FEATURES.into_iter().zip(used) {
            game_write(at, v);
        }
    }
}
