//! `physics::find_clear_place` (0x43BDD1): the work car moved to a place where nothing
//! touches it, after a reset. `dword_614C28` = 0x5A; its speed times `flt_613FDC`
//! (`physics::scale_vector`); `physics::restore_saved_car`;
//! `physics::lowest_body_point`, the depth into `dword_614118`. Then tries n = 0x70 +
//! (`unk_719FE0` & 0xF) .. 0xFF: `dword_614738` (counted by `restore_saved_car`) = 0, the
//! place moved by 2^((n >> 4) + 1) x (`dword_614014[n & 0xF]`, `dword_614054[n & 0xF]`)
//! (x, z; `imul`, low dwords), `physics::probe_ground`, `physics::crossed_edge`,
//! `physics::wall_contacts`, `physics::car_collisions`, `physics::face_contacts`; a try
//! that restored nothing ends it. None: `physics::reset_car`. Found:
//! `lowest_body_point`, a depth below 0 taken off the height (+0x800); "porky", n, the
//! first depth and the last logged (level 1, in `pusha` / `popa`); `dword_61473C` + 1;
//! `physics::limit_motion`.
//!
//! In:  nothing
//! Out: the registers as the steps and calls leave them (ebp the try, kept around each
//!      try's calls); no flags read (the caller returns)
//!
//! Caller: `sub_43B635` (0x43B8DB). Refuses (before anything) a probe count of 0 or past
//! 0x40; what its calls refuse.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::scene::matrix3::{extended, rounded};
use rc2k_core::physics::clear_place;

// g_logEnabled
const PORKY: &std::ffi::CStr = c"porky"; // aPorky

/// One try's calls (0x43BE4D .. 0x43BE61).
fn try_place(race: &Race) -> Result<(), Refused> {
    { crate::physics::probe_ground::probe_ground(race) }?;
    crate::physics::crossed_edge::check(&race.physics)?;
    crate::physics::crossed_edge::steps(race)?;
    crate::physics::wall_contacts::check(&race.shared)?;
    crate::physics::wall_contacts::steps(race)?;
    { crate::physics::car_collisions::car_collisions(race) }?;
    crate::physics::face_contacts_around::face_contacts_around(race)
}

/// The steps, not guarded.
// Converted: `rc2k_core::physics::clear_place`.
pub fn steps(race: &Race) -> Result<(), Refused> {
    let scene = crate::scene::owner::scene();
    let physics_store = &race.physics;
    race.shared
        .physics_tables
        .clear_place_mark
        .set(clear_place::TRY_MARK);
    let factor = extended(race.shared.physics_tables.clear_place_share.bits());
    crate::physics::scale_vector::scale_vector(&race.shared.car.velocity, factor);
    scene.drawing.scene.scratch.set(rounded(factor));
    { crate::physics::restore_saved_car::restore_saved_car(race) }?;
    { crate::physics::lowest_body_point::lowest_body_point(race) }?;
    race.shared
        .physics_tables
        .clear_place_face
        .set(physics_store.contact.lowest_depth.get());
    let found = clear_place::tries(
        clear_place::first_try(race.shared.car.step_clock.get()),
        |n| {
            race.shared.physics_tables.clear_place_count.set(0);
            let m = clear_place::try_move(n);
            for (table, place) in [
                (
                    &race.shared.physics_tables.clear_place_steps_a,
                    &race.shared.car.position[0],
                ),
                (
                    &race.shared.physics_tables.clear_place_steps_b,
                    &race.shared.car.position[2],
                ),
            ] {
                let (part, _) = clear_place::move_part(m.step, table[m.entry as usize].get());
                place.set(place.get().wrapping_add(part));
            }
            try_place(race)?;
            Ok::<_, Refused>(race.shared.physics_tables.clear_place_count.get() == 0)
        },
    )?;
    let Some(n) = found else {
        return crate::physics::reset_car::reset_car(race);
    };
    { crate::physics::lowest_body_point::lowest_body_point(race) }?;
    let depth = physics_store.contact.lowest_depth.get();
    if let Some(height) = clear_place::lifted(race.shared.car.position[1].get(), depth) {
        race.shared.car.position[1].set(height);
    }
    crate::logfile::log_write::log_nul(&PORKY, false, 1);
    crate::logfile::log_int::log_int(n as i32, false, 1);
    crate::logfile::log_hex8::log_hex8(race.shared.physics_tables.clear_place_face.get(), false, 1);
    crate::logfile::log_hex8::log_hex8(physics_store.contact.lowest_depth.get(), true, 1);
    race.shared.physics_tables.clear_place_tries.set(
        race.shared
            .physics_tables
            .clear_place_tries
            .get()
            .wrapping_add(1),
    );
    crate::physics::limit_motion::limit_motion(race, scene);
    Ok(())
}

/// The Rust version.
pub fn find_clear_place(race: &Race) -> Result<(), Refused> {
    crate::physics::wall_contacts::check(&race.shared)?;
    steps(race)
}
