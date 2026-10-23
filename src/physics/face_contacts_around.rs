//! `physics::face_contacts_around` (0x45E196): the car's place (`unk_71A10C`, `unk_71A114`)
//! into `dword_639824`, `dword_63982C`, then `physics::cell_face_contacts` for its cell
//! and the eight about it (± 0x200000 each way, in the original's order; the place read
//! back from `dword_639824` .. before each).
//!
//! In:  nothing
//! Out: the registers as the last `cell_face_contacts` leaves them; no flags read
//!
//! Caller: `physics::face_contacts` (0x45E190, its only code).
//! Refuses what `cell_face_contacts` refuses.
//! Self-test (in a race; skipped otherwise; `physics::face_contacts` shares it): the
//! original and the Rust version on the stage (everything written saved and put back),
//! the car where it is and moved onto object faces.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::face_contact::guarded;
use crate::physics::flow::{g, p};
use crate::screens::harness::Left;

/// The steps, not guarded.
// Converted: `rc2k_core::physics::ground_search::cells_around`.
pub fn steps(r: &mut Regs) -> Result<(), Refused> {
    r.eax = g(0x71A10C);
    r.ecx = g(0x71A114);
    p(0x639824, r.eax);
    p(0x63982C, r.ecx);
    rc2k_core::physics::ground_search::cells_around(
        || (g(0x639824), g(0x63982C)),
        |x, z| {
            r.eax = x;
            r.ecx = z;
            crate::physics::cell_face_contacts::steps(r)
        },
    )
}

/// The Rust version.
pub unsafe fn face_contacts_around(entry: &Regs) -> Result<Left, Refused> {
    guarded(entry, steps)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::physics::face_contact::faces_entry(r, steps)
}
