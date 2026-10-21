//! `scene::approximate_distance` (0x44EB78): how long a line is, near enough, without a
//! square root.
//!
//! In:  eax, ebx, ecx = the three differences
//! Out: eax = the length; ebx and ecx = the two smaller parts' shares of it;
//!      everything else preserved
//!
//! Each difference over 256 (signed) and without its sign; the largest times 0xF2, the
//! middle one times 0x60 and the smallest times 0x4B, added.
//! Callers: `scene::add_point_lights` and 0x44F8C7.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, ECX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::approximate_distance",
    rva(0x44EB78),
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

// ---- Rust version ----

/// Returns the length and what is left in ebx and ecx. Converted:
/// `rc2k_core::scene::distance::approximate`.
pub fn approximate_distance(differences: [u32; 3]) -> [u32; 3] {
    let d = rc2k_core::scene::distance::approximate(differences.map(|d| d as i32));
    [d.length, d.middle, d.least]
}

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    let [length, middle, least] = approximate_distance([r.eax, r.ebx, r.ecx]);
    Ok(Left::nothing()
        .with(EAX, length)
        .with(EBX, middle)
        .with(ECX, least))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let values = [
        0u32,
        0x100,
        0xFF,
        0x1_0000,
        0x12_3456,
        0xFFFF_FF00,
        0xFFED_CBA9,
        0x7FFF_FFFF,
        0x8000_0000,
        0x8000_0100,
    ];
    for a in values {
        for b in values {
            for c in values {
                let before = Regs {
                    eax: a,
                    ebx: b,
                    ecx: c,
                    ..Regs::SENTINEL
                };
                check_state(
                    &mut t,
                    &format!("{a:#x} {b:#x} {c:#x}"),
                    call_original,
                    before,
                    &|| {},
                    &[],
                    &rust,
                );
            }
        }
    }
    t.finish()
}
