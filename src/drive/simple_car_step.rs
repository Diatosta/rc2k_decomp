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
//! Caller: `drive::race_tick`. Refuses what its calls refuse.

use crate::errors::Refused;
use crate::physics::flow::{a, g, p};
use rc2k_core::drive::roaming::{roam_distance, roam_turn};

const B: usize = 0x70F3EC; // g_carBlockStore

/// The Rust version. The registers it leaves are read by no caller (liveness: ecx, edi
/// and esi after `drive::race_tick`'s call have no read point).
// Converted: `rc2k_core::drive::roaming::{roam_distance, roam_turn}`.
pub unsafe fn simple_car_step() -> Result<(), Refused> {
    unsafe {
        crate::drive::car_input_words::car_input_words()?;
        crate::drive::read_car_controls::read_car_controls()?;
        crate::drive::steer_inputs::steer_inputs();
        crate::drive::pedal_inputs::pedal_inputs();
    }
    // The keys `drive::pedal_inputs` leaves in ebx (`reset_request` loads the same again
    // before `physics::reset_car`): the turn's y.
    let keys = g(0x71A440);
    unsafe { crate::drive::reset_request::reset_request() }?;
    let distance = roam_distance(
        g(B + 0x638),
        g(B + 0x63C),
        unsafe { (a(crate::input::store::DeviceSlot::at(0x4F6B79)) as *const u8).read() } != 0,
    );
    let [x, _, z, _] = unsafe {
        crate::math::turn_point_xz::turn_point_xz(0, keys as i32, distance as i32, g(B + 0x9C0))
    };
    p(B + 0x134, g(B + 0x134).wrapping_add(x));
    p(B + 0x13C, g(B + 0x13C).wrapping_add(z));
    p(
        B + 0x9C0,
        g(B + 0x9C0).wrapping_add(roam_turn(g(B + 0x644))),
    );
    unsafe { crate::physics::wheel_surfaces::wheel_surfaces() }?;
    p(0x639824, g(0x5CCDA0));
    p(0x63982C, g(0x5CCDE0));
    unsafe { crate::physics::find_ground::find_ground() }?;
    if g(0x5CCC74) != u32::MAX {
        let y = g(0x639828).wrapping_add(g(0x613FC4));
        p(0x5CCDC0, y);
        p(B + 0x138, y);
    }
    p(B + 0x9C4, 0);
    p(B + 0x9C8, 0);
    // `race::get_mat_crash` reads no register at entry (liveness): a scratch.
    unsafe { crate::race::stage_prefs::get_mat_crash() };
    Ok(())
}
