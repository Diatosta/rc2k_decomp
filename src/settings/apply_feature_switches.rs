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

use crate::session::{Dword, MACHINE};

// ---- game data ----

/// (used, ignore setting, the processor has it): `g_hasMmx`, `dword_5183AC`, `dword_5183F8`;
/// `g_hasSse`, `dword_5183B4`, `dword_5183FC`; `g_has3dNow`, `dword_5183BC`, `dword_518400`.
fn features() -> [(&'static Dword, &'static Dword, &'static Dword); 3] {
    let m = &MACHINE;
    [
        (&m.has_mmx, &m.ignore_mmx, &m.found_mmx),
        (&m.has_sse, &m.ignore_katmai, &m.found_sse),
        (&m.has_3dnow, &m.ignore_3dnow, &m.found_3dnow),
    ]
}

// ---- Rust version ----

pub unsafe fn apply_feature_switches() {
    // Converted: `rc2k_core::files::settings::feature_switches`.
    {
        let has = features().map(|(_, _, has)| has.get());
        let ignore = features().map(|(_, ignore, _)| ignore.get());
        let used = rc2k_core::files::settings::feature_switches(has, ignore);
        for ((at, _, _), v) in features().into_iter().zip(used) {
            at.set(v);
        }
    }
}
