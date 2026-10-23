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
//! Not the physics step's loop (that is `sub_43B635`'s caller): a bounded search inside
//! one step, so the hook is gated as usual.
//!
//! In:  nothing
//! Out: the registers as the steps and calls leave them (ebp the try, kept around each
//!      try's calls); no flags read (the caller returns)
//!
//! Caller: `sub_43B635` (0x43B8DB).
//! Refuses (before anything) a probe count of 0 or past 0x40; what its calls refuse.
//! Shadow mode: the Rust version with `g_logEnabled` 0 (its log lines not written
//! twice), then the original; `regions()` and the faces touched compared.
//! Self-test (in a race; skipped otherwise; `g_logEnabled` 0 for it): the original and
//! the Rust version on the stage (everything written saved and put back), the car where
//! it is and moved onto object faces (so tries fail and the search goes on).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::flow::{a, g, p};
use crate::race::stage_prefs::left_of;
use crate::scene::matrix3::{extended, rounded};
use crate::screens::harness::Left;
use rc2k_core::physics::clear_place;

// g_logEnabled
const PORKY: usize = 0x4DBAFB; // aPorky

/// One try's calls (0x43BE4D .. 0x43BE61).
fn try_place(r: &mut Regs) -> Result<(), Refused> {
    unsafe { crate::physics::probe_ground::probe_ground(r) }?;
    crate::physics::crossed_edge::check()?;
    crate::physics::crossed_edge::steps(r)?;
    crate::physics::wall_contacts::check()?;
    crate::physics::wall_contacts::steps(r)?;
    unsafe { crate::physics::car_collisions::car_collisions(r) }?.apply(r);
    crate::physics::face_contacts_around::steps(r)
}

/// The steps, not guarded.
// Converted: `rc2k_core::physics::clear_place` (the tries' contacts are the hooks').
pub fn steps(r: &mut Regs) -> Result<(), Refused> {
    p(0x614C28, clear_place::TRY_MARK);
    let factor = extended(g(0x613FDC));
    unsafe { crate::physics::scale_vector::scale_vector(a(0x71A170), factor) };
    p(0x62400C, rounded(factor));
    unsafe { crate::physics::restore_saved_car::restore_saved_car() }?;
    unsafe { crate::physics::lowest_body_point::lowest_body_point(r) }?;
    r.eax = g(0x6D3068);
    p(0x614118, r.eax);
    let found = clear_place::tries(clear_place::first_try(g(0x719FE0)), |n| {
        r.ebp = n;
        p(0x614738, 0);
        let m = clear_place::try_move(n);
        r.ebx = m.entry;
        r.ecx = m.shift;
        for (table, place) in [(0x614014, 0x71A10C), (0x614054, 0x71A114)] {
            (r.eax, r.edx) = clear_place::move_part(m.step, g(table + 4 * r.ebx as usize));
            p(place, g(place).wrapping_add(r.eax));
        }
        r.eax = m.step;
        try_place(r)?;
        r.ebp = n;
        Ok::<_, Refused>(g(0x614738) == 0)
    })?;
    if found.is_none() {
        r.ebp = clear_place::TRIES_END;
        *r = unsafe { crate::physics::reset_car::reset_car(r) }?;
        return Ok(());
    }
    unsafe { crate::physics::lowest_body_point::lowest_body_point(r) }?;
    let depth = g(0x6D3068);
    if let Some(height) = clear_place::lifted(g(0x71A110), depth) {
        r.eax = depth;
        p(0x71A110, height);
    }
    unsafe {
        crate::logfile::log_write::log_write(a(PORKY) as *const u8, false, 1);
        crate::logfile::log_int::log_int(r.ebp as i32, false, 1);
        crate::logfile::log_hex8::log_hex8(g(0x614118), false, 1);
        crate::logfile::log_hex8::log_hex8(g(0x6D3068), true, 1);
    }
    p(0x61473C, g(0x61473C).wrapping_add(1));
    unsafe { crate::physics::limit_motion::limit_motion() }.apply(r);
    Ok(())
}

/// The Rust version.
pub unsafe fn find_clear_place(entry: &Regs) -> Result<Left, Refused> {
    crate::physics::wall_contacts::check()?;
    let mut r = *entry;
    steps(&mut r)?;
    Ok(left_of(&r))
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { find_clear_place(r) }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::service::harness::state_entry(r, &rust)
}
