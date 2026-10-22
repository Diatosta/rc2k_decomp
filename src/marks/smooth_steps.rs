//! `marks::smooth_steps` (0x4BE928): two cosine-eased steps from ebx to ecx (floats), step
//! eax, into ecx and ebx.
//!
//! In:  eax the step, ebx a, ecx b (floats)
//! Out: ecx, ebx the two steps; eax kept
//!
//! See `marks/turns.rs`. Callers: `sub_4BD122` (twice), `sub_4BEB25` (four). Self-test:
//! the original and the Rust version on values and steps by lot; the two scratch
//! variables put back afterwards.
//!
//! Converted 2026-10-21: the hook's Rust version is `turns::smooth_steps_plain`, through
//! `rc2k_core::marks::turns::smooth_pair` (plain floats; the x87 version stays for the Rust
//! callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).

use crate::errors::Refused;
use crate::framework::{EBX, ECX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::marks::turns::*;
use crate::race::stage_prefs::left_of;
use crate::scene::matrix::float_places;
use crate::screens::harness::Left;
use crate::service::floats::*;
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::smooth_steps", rva(0x4BE928), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { smooth_steps_plain(r) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    let places = float_places(&step_regions());
    let floats = Floats {
        memory: &places,
        regs: &[(ECX, FLOAT), (EBX, FLOAT)],
    };
    dispatch_state_floats(&HOOK, r, call_original, &step_regions(), &floats, &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = step_regions();
    let saved = capture(&regions);
    let tolerant_places = float_places(&regions);
    let tolerant = Floats {
        memory: &tolerant_places,
        regs: &[(ECX, FLOAT), (EBX, FLOAT)],
    };
    at_test_precisions(&tolerant, &mut |word, held| {
        for seed in 1..=8u32 {
            for step in [0u32, 1, 0x3F, 0x7F, 0x80, u32::MAX] {
                let mut v = [0u32; 2];
                crate::scene::matrix3::test_floats(seed, &mut v);
                check_state_floats(
                    &mut t,
                    "marks::smooth_steps",
                    &format!("control word {word:#x}, seed {seed}, step {step:#x}"),
                    call_original,
                    Regs {
                        eax: step,
                        ebx: v[0],
                        ecx: v[1],
                        ..Regs::SENTINEL
                    },
                    &|| {},
                    &regions,
                    held,
                    &rust,
                );
            }
        }
    });
    restore(&regions, &saved);
    t.finish()
}
