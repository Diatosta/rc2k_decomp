//! `physics::cell_face_contacts` (0x45E291): `physics::record_face_contacts` for the
//! course record of the grid cell holding (eax x, ecx z): the key ((z >> 21) << 11) +
//! (x >> 21), `physics::find_course_record`; nothing when it finds none.
//!
//! In:  eax x, ecx z
//! Out: the registers as `find_course_record` and `record_face_contacts` leave them; no
//!      flags read (the caller goes on)
//!
//! Callers: `physics::face_contacts_around` (nine).
//! Refuses what `find_course_record` (before anything) and `record_face_contacts`
//! refuse (what was written put back).
//! Self-test (in a race; skipped otherwise): the original and the Rust version on the
//! stage (everything written saved and put back): the nine cells about the car, where it
//! is and moved onto object faces.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::physics::face_contact::{
    self, Touched, faces_region, guarded, in_race_test, move_car_onto, object_faces,
};
use crate::physics::find_course_record::find_course_record;
use crate::physics::flow::g;
use crate::screens::harness::Left;
use crate::service::harness::check_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::cell_face_contacts",
    rva(0x45E291),
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
pub fn steps(r: &mut Regs, touched: &mut Touched) -> Result<(), Refused> {
    r.ecx = (r.ecx >> 21) << 11;
    r.eax = (r.eax >> 21).wrapping_add(r.ecx);
    let found = unsafe { find_course_record(r) }?;
    found.apply(r);
    if found.carry.unwrap_or(false) {
        return Ok(());
    }
    crate::physics::record_face_contacts::steps(r, touched)
}

/// The Rust version.
pub unsafe fn cell_face_contacts(entry: &Regs) -> Result<Left, Refused> {
    guarded(entry, steps)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { cell_face_contacts(r) }
}

extern "C" fn dispatch(r: &mut Regs) {
    crate::physics::face_contact::dispatch_faces(&HOOK, r, call_original, steps);
}

/// The nine cells' offsets, in `face_contacts_around`'s order.
pub const AROUND: [(u32, u32); 9] = [
    (0, 0),
    (0xFFE0_0000, 0xFFE0_0000),
    (0, 0xFFE0_0000),
    (0x20_0000, 0xFFE0_0000),
    (0x20_0000, 0),
    (0x20_0000, 0x20_0000),
    (0, 0x20_0000),
    (0xFFE0_0000, 0x20_0000),
    (0xFFE0_0000, 0),
];

fn self_test() -> (usize, usize) {
    in_race_test(&HOOK, &mut |t, put_back| {
        let mut all = face_contact::regions();
        all.push(faces_region());
        let mut moves = vec![None];
        moves.extend(object_faces(6).into_iter().map(Some));
        for moved in moves {
            for (n, &(dx, dz)) in AROUND.iter().enumerate() {
                let set = || {
                    put_back();
                    if let Some(k) = moved {
                        move_car_onto(k);
                    }
                };
                set();
                let (x, z) = (g(0x71A10C).wrapping_add(dx), g(0x71A114).wrapping_add(dz));
                check_state(
                    t,
                    &format!("cell {n}, car moved onto {moved:?}"),
                    call_original,
                    Regs {
                        eax: x,
                        ecx: z,
                        ..Regs::SENTINEL
                    },
                    &set,
                    &all,
                    &rust,
                );
            }
        }
    })
}
