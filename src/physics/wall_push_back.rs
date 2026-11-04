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
//! Callers: `physics::wall_contacts`, `physics::wall_edge_contact`. Refuses what
//! `place_sound_e1` refuses (what was written put back).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::math::x87_run::compare_80;
use crate::physics::car_store::CAR;
use crate::physics::contact_force::contact_force;
use crate::physics::contact_store::CONTACT_WORK;
use crate::physics::flow::{copy_cells, fill_cells, negate};
use crate::physics::tables_store::PHYSICS;
use crate::physics::vector_length::vector_length;
use crate::race::norms::normalise_work_vector;
use crate::race::numbers_store::NUMBERS;
use crate::race::scratch_store::FRAME_SCRATCH;

/// The steps, not guarded. The registers the original leaves are read by none of its
/// callers.
pub fn steps() -> Result<(), Refused> {
    let work = &FRAME_SCRATCH.work_vector;
    copy_cells(work, &CAR.velocity);
    negate(&work[0]);
    negate(&work[1]);
    negate(&work[2]);
    if compare_80(vector_length(&work), NUMBERS.zero.bits()) & 0x4000 != 0 {
        fill_cells(work, 0);
    }
    normalise_work_vector();
    copy_cells(&CONTACT_WORK.contact_normal, work);
    copy_cells(&CONTACT_WORK.contact_car_speed, &CAR.velocity);
    copy_cells(&CONTACT_WORK.contact_turn_speed, &CAR.spin);
    fill_cells(&CONTACT_WORK.contact_point, 0);
    CONTACT_WORK.contact_slide_limit.set_bits(0);
    contact_force(&PHYSICS.contacts[4]);
    { crate::physics::place_sound_e1::place_sound_e1() }?;
    PHYSICS.wall_heading.set(u32::MAX);
    Ok(())
}
