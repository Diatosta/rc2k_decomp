//! `scene::identity_matrix3` (0x46574E): the 3 x 3 matrix at esi made the identity.
//!
//! In:  esi the matrix
//! Out: every register kept
//!
//! See `scene/matrix3.rs`. Callers: `sub_4BEFD2` and code outside a function. Self-test:
//! the original and the Rust version on a buffer of the test's.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::stage_prefs::left_of;
use crate::scene::matrix3::*;
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::identity_matrix3", rva(0x46574E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { identity_matrix3(r.esi) };
    Ok(left_of(r))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = [(r.esi, 0x24)];
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut buffer = [0xAAu8; 0x30];
    let len = buffer.len();
    let b = buffer.as_mut_ptr() as u32;
    for at in [0u32, 1, 4] {
        check_state(
            &mut t,
            &format!("at +{at}"),
            call_original,
            Regs {
                esi: b + at,
                ..Regs::SENTINEL
            },
            &|| unsafe { std::ptr::write_bytes(b as *mut u8, 0xAA, len) },
            &[(b, len)],
            &rust,
        );
    }
    t.finish()
}
