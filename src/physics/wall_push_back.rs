//! `physics::wall_push_back` (0x45D8FB): the work car pushed back off a wall against its
//! motion: minus its speed (`unk_71A170` ..) in `g_workVector` (0 when its length is 0,
//! `physics::vector_length`), made one long (`race::normalise_work_vector`) as the
//! normal; the speed and the turn speed taken, the point 0, no slide limit;
//! `physics::contact_force` into contact record 4 (`g_contactRecords` + 0x80),
//! `physics::place_sound_e1`; `dword_6141C8` = -1.
//!
//! In:  nothing
//! Out: the registers as `contact_force` leaves them; no flags read (callers go on)
//!
//! Callers: `physics::wall_contacts`, `physics::wall_edge_contact`.
//! Refuses what `place_sound_e1` refuses (what was written put back).
//! Self-test: `physics/wall_contacts.rs` (`test_walls`): the original and the Rust version
//! on the game's variables (saved and put back), speeds and figures by lot (standing
//! still too), under three precisions.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::math::x87_run::compare_80;
use crate::physics::contact_force::{RECORD, contact_force};
use crate::physics::flow::{a, copy3, fill3, g, guard, negate, p};
use crate::physics::vector_length::vector_length;
use crate::physics::wall_contacts::{WALL_RECORD, WORK, regions};
use crate::race::norms::normalise_work_vector;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The steps, not guarded.
pub fn steps(r: &mut Regs) -> Result<(), Refused> {
    copy3(WORK, 0x71A170);
    negate(WORK);
    negate(WORK + 4);
    negate(WORK + 8);
    if compare_80(unsafe { vector_length(a(WORK)) }, g(0x5C5D90)) & 0x4000 != 0 {
        fill3(WORK, 0);
    }
    unsafe { normalise_work_vector() };
    copy3(0x6D3078, WORK);
    copy3(0x6D30A8, 0x71A170);
    copy3(0x6D30B4, 0x71A15C);
    fill3(0x6D3090, 0);
    p(0x6D3100, 0);
    p(RECORD, a(WALL_RECORD));
    unsafe { contact_force() }.apply(r);
    unsafe { crate::physics::place_sound_e1::place_sound_e1() }?;
    p(0x6141C8, u32::MAX);
    Ok(())
}

/// The Rust version.
pub unsafe fn wall_push_back(entry: &Regs) -> Result<Left, Refused> {
    guard(&regions(), || {
        let mut r = *entry;
        steps(&mut r)?;
        Ok(left_of(&r))
    })
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { wall_push_back(r) }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
