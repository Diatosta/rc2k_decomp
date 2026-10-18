//! `physics::face_contacts` (0x45E190): `physics::face_contacts_around` (its only code,
//! `call` and `retn`): the course faces about the work car tried against it.
//!
//! In:  nothing
//! Out: the registers as `face_contacts_around` leaves them; no flags read
//!
//! Callers: `sub_43B635` (every physics step), `sub_43BDD1`. The hook's five bytes are
//! the `call`: a private copy (six bytes, the call relocated) and a hand-made patch, as
//! `scene::end_scene`.
//! Refuses what `face_contacts_around` refuses (everything written put back).
//! Self-test: `face_contacts_around`'s (in a race; skipped otherwise).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::physics::face_contact::in_race_test;
use crate::physics::face_contacts_around::{face_contacts_around, test};
use crate::screens::harness::Left;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("physics::face_contacts", rva(0x45E190), thunk, &ORIGINAL)
    .private_copy(6)
    .relocate_calls(&[0])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version.
pub unsafe fn face_contacts(entry: &Regs) -> Result<Left, Refused> {
    unsafe { face_contacts_around(entry) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { face_contacts(r) }
}

extern "C" fn dispatch(r: &mut Regs) {
    crate::physics::face_contact::dispatch_faces(
        &HOOK,
        r,
        call_original,
        crate::physics::face_contacts_around::steps,
    );
}

fn self_test() -> (usize, usize) {
    in_race_test(&HOOK, &mut |t, put_back| {
        test(t, put_back, call_original, &rust)
    })
}
