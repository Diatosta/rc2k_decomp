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
use crate::physics::flow::{a, g, p};
use crate::scene::matrix3::{extended, rounded};
use rc2k_core::physics::clear_place;

// g_logEnabled
const PORKY: &std::ffi::CStr = c"porky"; // aPorky

/// One try's calls (0x43BE4D .. 0x43BE61).
fn try_place() -> Result<(), Refused> {
    unsafe { crate::physics::probe_ground::probe_ground() }?;
    crate::physics::crossed_edge::check()?;
    crate::physics::crossed_edge::steps()?;
    crate::physics::wall_contacts::check()?;
    crate::physics::wall_contacts::steps()?;
    unsafe { crate::physics::car_collisions::car_collisions() }?;
    crate::physics::face_contacts_around::face_contacts_around()
}

/// The steps, not guarded.
// Converted: `rc2k_core::physics::clear_place`.
pub fn steps() -> Result<(), Refused> {
    p(0x614C28, clear_place::TRY_MARK);
    let factor = extended(g(0x613FDC));
    unsafe { crate::physics::scale_vector::scale_vector(a(0x71A170), factor) };
    p(0x62400C, rounded(factor));
    unsafe { crate::physics::restore_saved_car::restore_saved_car() }?;
    unsafe { crate::physics::lowest_body_point::lowest_body_point() }?;
    p(0x614118, g(0x6D3068));
    let found = clear_place::tries(clear_place::first_try(g(0x719FE0)), |n| {
        p(0x614738, 0);
        let m = clear_place::try_move(n);
        for (table, place) in [(0x614014, 0x71A10C), (0x614054, 0x71A114)] {
            let (part, _) = clear_place::move_part(m.step, g(table + 4 * m.entry as usize));
            p(place, g(place).wrapping_add(part));
        }
        try_place()?;
        Ok::<_, Refused>(g(0x614738) == 0)
    })?;
    let Some(n) = found else {
        return unsafe { crate::physics::reset_car::reset_car() };
    };
    unsafe { crate::physics::lowest_body_point::lowest_body_point() }?;
    let depth = g(0x6D3068);
    if let Some(height) = clear_place::lifted(g(0x71A110), depth) {
        p(0x71A110, height);
    }
    unsafe {
        crate::logfile::log_write::log_write(PORKY.as_ptr().cast(), false, 1);
        crate::logfile::log_int::log_int(n as i32, false, 1);
        crate::logfile::log_hex8::log_hex8(g(0x614118), false, 1);
        crate::logfile::log_hex8::log_hex8(g(0x6D3068), true, 1);
    }
    p(0x61473C, g(0x61473C).wrapping_add(1));
    unsafe { crate::physics::limit_motion::limit_motion() };
    Ok(())
}

/// The Rust version.
pub unsafe fn find_clear_place() -> Result<(), Refused> {
    crate::physics::wall_contacts::check()?;
    steps()
}
