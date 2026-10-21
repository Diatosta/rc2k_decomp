//! `scene::find_object_kind` (0x41F4BD): an object kind's place in the known kinds
//! (`g_knownObjectKinds`, 0x22 dwords): the first whose value, or value + 1, is eax;
//! 0x22 for none.
//!
//! In:  eax the kind
//! Out: eax the place; everything else preserved
//! Callers: 0x425E3A (`scene::load_object`, whose Rust version has its own) and
//! 0x4522A7. Shadow mode: both, compared.
//! Self-test: the original against the Rust version for each known kind, each + 1, and
//! some unknown.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, SelfTest, game_read, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::find_object_kind", rva(0x41F4BD), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// g_knownObjectKinds.
const KINDS_RVA: usize = rva(0x5C5790);

// ---- Rust version ----

pub fn find_object_kind(kind: u32) -> u32 {
    // Converted: `rc2k_core::scene::kinds::find_known`.
    rc2k_core::scene::kinds::find_known(|n| unsafe { game_read(KINDS_RVA + 4 * n) }, kind)
}

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, find_object_kind(r.eax)))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut kinds: Vec<u32> = (0..rc2k_core::scene::kinds::KNOWN_KINDS)
        .flat_map(|n| {
            let k: u32 = unsafe { game_read(KINDS_RVA + 4 * n as usize) };
            [k, k.wrapping_add(1), k.wrapping_add(2), k.wrapping_sub(1)]
        })
        .collect();
    kinds.extend([0, 0xFFFF_FFFF, 0x8000_0000]);
    for kind in kinds {
        check_state(
            &mut t,
            &format!("kind {kind:#x}"),
            call_original,
            Regs {
                eax: kind,
                ..Regs::SENTINEL
            },
            &|| {},
            &[],
            &rust,
        );
    }
    t.finish()
}
