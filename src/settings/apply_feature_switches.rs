//! `settings::apply_feature_switches` (0x4114BB): which of the processor's extensions the
//! game will use.
//!
//! Three times the same: a feature is used if the processor has it (bit 0 of what the processor
//! test left) and the setting that says to ignore it is 0. MMX: `has_mmx` from
//! `found_mmx` and `ignoremmx`; SSE: `has_sse` from `found_sse` and `ignorekatmai`; 3DNow:
//! `has_3dnow` from `found_3dnow` and `ignoreamdk3d`. One caller, `start`, right after the settings are
//! read.

use crate::session::Dword;
use crate::system::owner::System;

// ---- game data ----

/// (used, ignore setting, the processor has it): `has_mmx`, `ignore_mmx`, `found_mmx`;
/// `has_sse`, `ignore_katmai`, `found_sse`; `has_3dnow`, `ignore_3dnow`, `found_3dnow`.
fn features(system: &System) -> [(&Dword, &Dword, &Dword); 3] {
    let m = &system.session.machine;
    [
        (&m.has_mmx, &m.ignore_mmx, &m.found_mmx),
        (&m.has_sse, &m.ignore_katmai, &m.found_sse),
        (&m.has_3dnow, &m.ignore_3dnow, &m.found_3dnow),
    ]
}

pub fn apply_feature_switches(system: &System) {
    // Converted: `rc2k_core::files::settings::feature_switches`.
    {
        let has = features(system).map(|(_, _, has)| has.get());
        let ignore = features(system).map(|(_, ignore, _)| ignore.get());
        let used = rc2k_core::files::settings::feature_switches(has, ignore);
        for ((at, _, _), v) in features(system).into_iter().zip(used) {
            at.set(v);
        }
    }
}
