//! `drive::simple_car_step` (0x43A1CA): the free-roaming car's step (no physics, the race
//! mode `dword_621884` 0). The inputs (`drive::car_input_words`, `drive::read_car_controls`,
//! `drive::steer_inputs`, `drive::pedal_inputs`, `drive::reset_request`); the car moved
//! along its heading (+0x9C0) by (+0x638 - +0x63C) << 5 (<< 7 with `byte_4F6B79`,
//! `math::turn_point_xz` of (0, ebx, it)), turned by +0x644 >> 4; the wheels' surfaces;
//! the ground under `dword_5CCDA0`, `5CCDE0` (`physics::find_ground`), found: the height
//! `dword_639828` + `flt_613FC4`'s bits into `dword_5CCDC0` and +0x138; pitch and roll 0;
//! `race::get_mat_crash`. The first car block (`g_carBlockStore`) throughout.
//!
//! In:  nothing
//! Out: the registers as the steps and calls leave them; no flags read
//!
//! Caller: `drive::race_tick`. It begins with a call: hooked by a private copy of its
//! 0xB4 bytes (its nine calls relocated) and a hand-made patch.
//! Refuses what its calls refuse.
//! Self-test (in a race; skipped otherwise): the original and the Rust version from the
//! same state (the first block, the inputs and the ground saved and put back).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::flow::{a, g, p};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use rc2k_core::drive::roaming::{roam_distance, roam_turn};

const B: usize = 0x70F3EC; // g_carBlockStore

/// The Rust version.
// Converted: `rc2k_core::drive::roaming::{roam_distance, roam_turn}`.
pub unsafe fn simple_car_step(entry: &Regs) -> Result<Left, Refused> {
    let mut r = *entry;
    unsafe {
        crate::drive::car_input_words::car_input_words(&r)?.apply(&mut r);
        let mut c = r;
        crate::drive::read_car_controls::read_car_controls(&mut c)?;
        r.eax = c.eax;
        r.esi = c.esi;
        crate::drive::steer_inputs::steer_inputs(&r).apply(&mut r);
        crate::drive::pedal_inputs::pedal_inputs().apply(&mut r);
        r = crate::drive::reset_request::reset_request(
            &r,
            &mut crate::physics::reset_parts::game_part,
        )?;
    }
    r.eax = 0;
    r.ecx = roam_distance(
        g(B + 0x638),
        g(B + 0x63C),
        unsafe { (a(0x4F6B79) as *const u8).read() } != 0,
    );
    r.edx = g(B + 0x9C0);
    let [x, _, z, edx] =
        unsafe { crate::math::turn_point_xz::turn_point_xz(0, r.ebx as i32, r.ecx as i32, r.edx) };
    r.eax = x;
    r.ecx = z;
    r.edx = edx;
    p(B + 0x134, g(B + 0x134).wrapping_add(x));
    p(B + 0x13C, g(B + 0x13C).wrapping_add(z));
    r.eax = roam_turn(g(B + 0x644));
    p(B + 0x9C0, g(B + 0x9C0).wrapping_add(r.eax));
    unsafe { crate::physics::wheel_surfaces::wheel_surfaces() }?.apply(&mut r);
    r.eax = g(0x5CCDA0);
    p(0x639824, r.eax);
    r.eax = g(0x5CCDE0);
    p(0x63982C, r.eax);
    unsafe { crate::physics::find_ground::find_ground() }?;
    if g(0x5CCC74) != u32::MAX {
        r.eax = g(0x639828).wrapping_add(g(0x613FC4));
        p(0x5CCDC0, r.eax);
        p(B + 0x138, r.eax);
    }
    p(B + 0x9C4, 0);
    p(B + 0x9C8, 0);
    r = unsafe { crate::race::stage_prefs::get_mat_crash(&r) };
    Ok(left_of(&r))
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { simple_car_step(r) }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
