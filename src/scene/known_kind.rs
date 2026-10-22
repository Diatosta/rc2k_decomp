//! `scene::known_kind` (0x41F469): a known kind of car's object kind.
//!
//! In:  eax = which of the known kinds (held to 0 .. 0x21, signed)
//! Out: eax = the number; everything else preserved
//!
//! `g_knownObjectKinds` at the place: 0x22 dwords, one for each known kind of car.
//! Two callers.
//! Manual patch with a private copy: a conditional jump in the first bytes.
//! Self-test: 14 places.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, SelfTest, game_read, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("scene::known_kind", rva(0x41F469), thunk, &ORIGINAL)
    .private_copy(0x1C)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const TABLE_RVA: usize = rva(0x5C5790); // g_knownObjectKinds

// ---- Rust version ----

pub unsafe fn known_kind(place: u32) -> u32 {
    // Converted: `rc2k_core::scene::kinds::known_place`.
    let place = rc2k_core::scene::kinds::known_place(place);
    unsafe { game_read(TABLE_RVA + place * 4) }
}

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, unsafe { known_kind(r.eax) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    for place in [
        0u32,
        1,
        2,
        0x10,
        0x20,
        0x21,
        0x22,
        0x23,
        0x7FFF_FFFF,
        0x8000_0000,
        0xFFFF_FFFF,
        0xFFFF_FFDF,
        0x100,
        5,
    ] {
        check_state(
            &mut t,
            &format!("place {place:#x}"),
            call_original,
            Regs {
                eax: place,
                ..Regs::SENTINEL
            },
            &|| {},
            &[],
            &rust,
        );
    }
    t.finish()
}
