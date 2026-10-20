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

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "settings::apply_feature_switches",
    rva(0x4114BB),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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

// ---- hook ----

fn regions() -> [Region; 3] {
    FEATURES.map(|(used, _, _)| at(used))
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { apply_feature_switches() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let all: Vec<Region> = FEATURES
        .iter()
        .flat_map(|&(used, ignore, has)| [at(used), at(ignore), at(has)])
        .collect();
    let saved = capture(&all);
    for (ignore, has) in [
        (0u32, 0u32),
        (0, 1),
        (1, 1),
        (1, 0),
        (0, 3),
        (0, 2),
        (0xFFFF_FF00, 1),
        (0, 0xFFFF_FFFF),
    ] {
        for feature in 0..3 {
            let prepare = || unsafe {
                for (n, (used, ignore_at, has_at)) in FEATURES.into_iter().enumerate() {
                    game_write(used, 0xAAAA_AAAAu32);
                    // The others the other way round.
                    let (i, h) = if n == feature {
                        (ignore, has)
                    } else {
                        (has, ignore)
                    };
                    game_write(ignore_at, i);
                    game_write(has_at, h);
                }
            };
            check_state(
                &mut t,
                &format!("feature {feature}: ignore={ignore:#x} has={has:#x}"),
                call_original,
                Regs::SENTINEL,
                &prepare,
                &regions(),
                &rust,
            );
        }
    }
    restore(&all, &saved);
    t.finish()
}
