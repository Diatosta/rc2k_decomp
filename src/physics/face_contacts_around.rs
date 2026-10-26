//! `physics::face_contacts_around` (0x45E196): the car's place (`unk_71A10C`, `unk_71A114`)
//! into `dword_639824`, `dword_63982C`, then `physics::cell_face_contacts` for its cell
//! and the eight about it (± 0x200000 each way, in the original's order; the place read
//! back from `dword_639824` .. before each).
//!
//! In:  nothing
//! Out: the registers as the last `cell_face_contacts` leaves them; no flags read
//!
//! Caller: `physics::face_contacts` (0x45E190, its only code).
//! Refuses what `cell_face_contacts` refuses (everything written put back, the faces
//! touched included).
//! Self-test (in a race; skipped otherwise; `physics::face_contacts` shares it): the
//! original and the Rust version on the stage (everything written saved and put back),
//! the car where it is and moved onto object faces.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::face_contact::{
    self, Touched, faces_region, guarded, in_race_test, move_car_onto, object_faces,
};
use crate::physics::flow::{g, p};
use crate::screens::harness::{CallOriginal, Left};
use crate::service::harness::check_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::face_contacts_around",
    rva(0x45E196),
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

/// The steps, not guarded.
// Converted: `rc2k_core::physics::ground_search::cells_around`.
pub fn steps(r: &mut Regs, touched: &mut Touched) -> Result<(), Refused> {
    r.eax = g(0x71A10C);
    r.ecx = g(0x71A114);
    p(0x639824, r.eax);
    p(0x63982C, r.ecx);
    rc2k_core::physics::ground_search::cells_around(
        || (g(0x639824), g(0x63982C)),
        |x, z| {
            r.eax = x;
            r.ecx = z;
            crate::physics::cell_face_contacts::steps(r, touched)
        },
    )
}

/// The Rust version.
pub unsafe fn face_contacts_around(entry: &Regs) -> Result<Left, Refused> {
    guarded(entry, steps)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { face_contacts_around(r) }
}

/// The regions both hooks' tests compare (the whole face array).
pub fn test_regions() -> Vec<crate::service::harness::Region> {
    let mut all = face_contact::regions();
    all.push(faces_region());
    all
}

extern "C" fn dispatch(r: &mut Regs) {
    crate::physics::face_contact::dispatch_faces(&HOOK, r, call_original, steps);
}

/// The test of this and of `face_contacts`: the original `call_original`, the car where
/// it is and moved onto object faces.
pub fn test(
    t: &mut SelfTest,
    put_back: &dyn Fn(),
    call_original: CallOriginal,
    rust: crate::service::harness::Rust,
) {
    let all = test_regions();
    let mut moves = vec![None];
    moves.extend(object_faces(10).into_iter().map(Some));
    for moved in moves {
        let set = || {
            put_back();
            if let Some(k) = moved {
                move_car_onto(k);
            }
        };
        check_state(
            t,
            &format!("car moved onto {moved:?}"),
            call_original,
            Regs::SENTINEL,
            &set,
            &all,
            rust,
        );
    }
}

fn self_test() -> (usize, usize) {
    in_race_test(&HOOK, &mut |t, put_back| {
        test(t, put_back, call_original, &rust)
    })
}
