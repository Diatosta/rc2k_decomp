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
//! Caller: `drive::start_cars_behind` (0x43703E; four calls).
//! Refuses (before anything) a start number (`dword_6051A0`) above 0x400 (not worked
//! out), and what the two calls refuse.
//! Self-test: a copy whose two calls go to stream A's stand-ins (`race::car_starts`),
//! against the Rust version with the same, the calls and their registers compared, the
//! work car and the start's variables compared (the share exact at double precision,
//! within `FLOAT` at extended); all put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::math::x87;
use crate::race::car_starts::{Part, game_part};
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;
use rc2k_core::drive::start_places;

fn g(x: u32) -> u32 {
    unsafe { game_read(rva(x as usize)) }
}

fn p(x: u32, v: u32) {
    unsafe { game_write(rva(x as usize), v) }
}

/// The Rust core, the two calls through `part` (places 6 and 9).
// Converted: `rc2k_core::drive::start_places` (the share in plain floats; the motion's
// turn stays x87).
pub unsafe fn start_car_on_line(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    unsafe {
        let start = g(0x6051A0);
        start_places::check_start(start)?;
        let mut r = *entry;
        p(0x71AA80, g(0x605034 + 4 * start));
        p(0x71A234, start_places::STARTED_MODE);
        r.eax = start_places::start_checkpoint(g(0x70F484), r.ebx);
        for v in [0x71A070, 0x71A06C, 0x71A074, 0x6365E4, 0x6365EC] {
            p(v, r.eax);
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
        p(0x71A650, share);
        part(6, &mut r)?;
        part(9, &mut r)?;
        p(0x71A17C, 0);
        p(0x71A180, 0);
        p(0x71A184, x87::loaded(g(0x6051B0)));
        let m = game_addr(rva(0x71A08C)) as u32;
        let out = rotate_point(
            m,
            [
                extended(g(0x71A17C)),
                extended(g(0x71A180)),
                extended(g(0x71A184)),
            ],
        );
        p(0x71A170, rounded(out[0]));
        p(0x71A174, rounded(out[1]));
        p(0x71A178, rounded(out[2]));
        Ok(r)
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let g = unsafe { start_car_on_line(r, &mut game_part) }?;
    *r = Regs {
        esp: r.esp,
        eflags: r.eflags,
        ..g
    };
    Ok(())
}
