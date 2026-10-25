//! `drive::start_car_on_line` (0x437613): the work car put at a start on the line:
//! `dword_71AA80` = `dword_605034[dword_6051A0]` (the start's number's entry); the grades'
//! mode `dword_71A234` 2; the checkpoint (`dword_70F484`, the lead car's, + ebx) into
//! `dword_71A070`, `6C`, `74`, `dword_6365E4` and `dword_6365EC`; a share `flt_6051AC` /
//! `flt_71A650` = lo + ((m - 1) - n) / (m - 1) (hi - lo) (n `dword_6051A0`, m
//! `dword_6051B4`, lo / hi `flt_6051A4` / `flt_6051A8`); `race::place_car_on_line`,
//! `race::reset_car_motion`; then the motion: 0, 0 and the speed `flt_6051B0` turned by
//! `g_crashMatrix` (`scene::rotate_point`) into 0x71A170 .. 0x71A178 (`g_carSpeed` the
//! speed). By the look of it a computer car placed at the start of a run (a reading).
//!
//! In:  ebx the checkpoints on from the lead's
//! Out: as `race::reset_car_motion` leaves them; no flags read (`drive::store_car` next)
//!
//! Caller: `drive::start_cars_behind` (0x43703E; four calls). Refuses (before anything) a
//! start number (`dword_6051A0`) above 0x400 (not worked out), and what the two calls
//! refuse.

use crate::errors::Refused;
use crate::framework::{game_read, game_write, rva};
use crate::math::x87;
use crate::physics::car_store::CAR;
use crate::race::blocks_store::CAR_BLOCKS;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;
use rc2k_core::drive::start_places;

fn g(x: u32) -> u32 {
    unsafe { game_read(rva(x as usize)) }
}

fn p(x: u32, v: u32) {
    unsafe { game_write(rva(x as usize), v) }
}

/// The Rust version: `distance` (ebx) checkpoints on from the lead car. The registers it
/// leaves are read by no caller (`drive::start_cars_behind` sets them again).
// Converted: `rc2k_core::drive::start_places` (the share in plain floats; the motion's
// turn stays x87).
pub unsafe fn start_car_on_line(distance: u32) -> Result<(), Refused> {
    unsafe {
        let start = g(0x6051A0);
        start_places::check_start(start)?;
        CAR.start_slot_value.set(g(0x605034 + 4 * start));
        CAR.driver_kind.set(start_places::STARTED_MODE);
        let checkpoint =
            start_places::start_checkpoint(CAR_BLOCKS.cars[0].checkpoint.get(), distance);
        for cell in [&CAR.checkpoint, &CAR.reset_checkpoint, &CAR.last_pace_call] {
            cell.set(checkpoint);
        }
        for v in [0x6365E4, 0x6365EC] {
            p(v, checkpoint);
        }
        let f = |x: u32| f32::from_bits(g(x));
        let share = start_places::start_share(
            start as i32,
            g(0x6051B4) as i32,
            f(0x5C5D38),
            f(0x6051A8),
            f(0x6051A4),
        )
        .to_bits();
        p(0x6051AC, share);
        CAR.start_share.set_bits(share);
        crate::race::start_place::place_car_on_line()?;
        crate::race::car_starts::reset_car_motion()?;
        CAR.local_velocity[0].set_bits(0);
        CAR.local_velocity[1].set_bits(0);
        CAR.local_velocity[2].set_bits(x87::loaded(g(0x6051B0)));
        let m = CAR.crash_matrix[0].addr() as u32;
        let out = rotate_point(
            m,
            [
                extended(CAR.local_velocity[0].bits()),
                extended(CAR.local_velocity[1].bits()),
                extended(CAR.local_velocity[2].bits()),
            ],
        );
        CAR.velocity[0].set_bits(rounded(out[0]));
        CAR.velocity[1].set_bits(rounded(out[1]));
        CAR.velocity[2].set_bits(rounded(out[2]));
        Ok(())
    }
}
