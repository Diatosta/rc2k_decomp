//! `physics::probe_ground` (0x45D305): the ground under each of the car's
//! `dword_71A238` probe points (the work copy's lists x `flt_71AAC8`, y `flt_71ABC8`, z
//! `flt_71ACC8`, 0x40 each; the first four by the look of it the wheels, a reading).
//! Nothing when the count is 0. Else `dword_6D3B84` = -1 and for each point b:
//! - for b < 4 its y is replaced: `flt_71ABE0` (the list's seventh y) less `flt_614BF4`,
//!   plus `flt_613E8C` unless bit b + 5 of `dword_71A270` is set and then unless bit
//!   b + 1 is; less `flt_613F8C` times `flt_614BF8` (the point's own y only goes to
//!   `dword_62400C`);
//! - the point turned by `g_crashMatrix` (`scene::rotate_point`), its x y z kept as floats
//!   (0x6D2214, 0x6D2314, 0x6D2414 [b]; x and z also in 0x6D3628 and 0x6D3630), and as
//!   whole numbers times 65536 plus the car's place (`dword_71A10C` ..) at 0x6D1E14,
//!   0x6D1F14, 0x6D2014 [b];
//! - the ground under (x, z) by `physics::find_ground`, the mask `dword_620470` 0x80, the
//!   last face `dword_71ADC8[b]`; `dword_6D2714[b]` the found;
//! - not found: `dword_6D2114[b]` 0, `dword_6D3B84` = b;
//! - found: `dword_6D2114[b]` the height (`dword_639828`) plus a bump: the noise bytes
//!   (`g_noiseBytes`, 64 x 64) at (x >> 14, z >> 14) mixed by the low 14 bits
//!   (0x6D3044 .. 0x6D3064 the steps), times `flt_71A038`, 256, over 100; the face's half
//!   and shares (0x6D2914, 0x6D2A14, 0x6D2B14 [b]); `dword_6D2514[b]` = y less that
//!   ground; for b < 4 that also into `dword_6D2614[b]` and `dword_6D2514[b]` times
//!   `flt_71A0A0` (held to at least `flt_5C5D90`); `dword_71ADC8[b]` = the face found.
//!
//! Then the first four of 0x6D1E14, 0x6D2114, 0x6D2014 into 0x71AEE8 .. 0x71AF14.
//!
//! In:  nothing
//! Out: (count not 0) ebx the count; eax, ecx, edx, esi as the last point leaves them
//!      (see `probe_ground_at`); edi, ebp kept. No caller reads the flags.
//!
//! Callers (6): `sub_43BDD1`, `sub_43F1A8`, `race::settle_car` (twice), `sub_456056`,
//! `sub_45DC12`. Refuses (before anything: what it wrote is put back) a count above 0x40
//! (the lists' room), and what `find_ground` refuses on any point.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::math::x87_run::{int_times_times_over_to_int, less_plus_less_product_80};
use crate::physics::car_store::CAR;
use crate::physics::contact_store::CONTACT_WORK;
use crate::physics::ground::*;
use crate::physics::ground_store::GROUND_WORK;
use crate::physics::tables_store::PHYSICS;
use crate::race::course_store::COURSE;
use crate::race::numbers_store::NUMBERS;
use crate::scene::draw_store::DRAW;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;
use rc2k_core::physics::ground;

pub const MOST_POINTS: u32 = 0x40;

/// The Rust core. The registers the original leaves (ebx the count, eax, ecx, edx, esi
/// the last point's) are read by none of its callers.
// Converted: `rc2k_core::physics::ground::{wheel_lifted, noise_cell, bump_noise}` (the
// turns and the bump's scaling stay x87).
pub unsafe fn probe_ground_at(g: &Ground) -> Result<(), Refused> {
    unsafe {
        let n = CAR.probe_count.get();
        if n < 1 {
            return Ok(());
        }
        if n > MOST_POINTS {
            return Err(BadInput("more than 0x40 probe points (the lists' room)").into());
        }

        probe_points(g, n)
    }
}

unsafe fn probe_points(g: &Ground, n: u32) -> Result<(), Refused> {
    unsafe {
        let k65536 = NUMBERS.k65536.bits();
        CONTACT_WORK.crossed_probe.set(u32::MAX);
        for b in 0..n {
            let i = b as usize;
            let work = &CONTACT_WORK;
            let z = CAR.probe_point_z[b as usize].bits();
            let y = CAR.probe_point_y[b as usize].bits();
            let y = if b < 4 {
                DRAW.scratch.set(x87::loaded(y));
                let add;
                (add, _) = ground::wheel_lifted(CAR.parts_present.get(), b);
                less_plus_less_product_80(
                    CAR.probe_point_y[6].bits(),
                    PHYSICS.handling[2].bits(),
                    add.then(|| PHYSICS.probe_lift.bits()),
                    PHYSICS.probe_share.bits(),
                    PHYSICS.handling[3].bits(),
                )
            } else {
                extended(y)
            };
            let x = extended(CAR.probe_point_x[b as usize].bits());
            let out = rotate_point(&CAR.crash_matrix, [x, y, extended(z)]);
            let (ox, oy, oz) = (rounded(out[0]), rounded(out[1]), rounded(out[2]));
            CONTACT_WORK.probe_turned_x_last.set_bits(ox);
            work.probe_turned_x[i].set_bits(ox);
            work.probe_turned_y[i].set_bits(oy);
            CONTACT_WORK.probe_turned_z_last.set_bits(oz);
            work.probe_turned_z[i].set_bits(oz);
            for (v, place, list) in [
                (ox, &CAR.position[0], &work.probe_x[i]),
                (oy, &CAR.position[1], &work.probe_y[i]),
                (oz, &CAR.position[2], &work.probe_z[i]),
            ] {
                let w = (x87::mul_to_int(v, k65536) as u32).wrapping_add(place.get());
                list.set(w);
            }
            GROUND_WORK.ground_x.set(work.probe_x[i].get());
            GROUND_WORK.ground_z.set(work.probe_z[i].get());
            COURSE
                .ground_face_last
                .set(CAR.probe_point_face[b as usize].get());
            crate::physics::ground_probe_store::GROUND_PROBE
                .settle_mark
                .set(0x80);
            ground_search_at(g)?;
            work.probe_ground_height[i].set(0);
            if GROUND_WORK.ground_found.get() == 0 {
                CONTACT_WORK.crossed_probe.set(b);
                work.probe_found[i].set(0);
                continue;
            }
            work.probe_found[i].set(1);
            work.probe_ground_height[i].set(GROUND_WORK.ground_height.get());
            let cell = ground::noise_cell(GROUND_WORK.ground_x.get(), GROUND_WORK.ground_z.get());
            CONTACT_WORK.ground_cell_row[0].set(cell.row);
            CONTACT_WORK.ground_cell_row[1].set(cell.row_offset);
            let trap = crate::physics::cells::Trap::new();
            let noise = |i: u32| {
                u32::from(
                    trap.byte(
                        crate::scene::engine_tables_store::ENGINE_TABLES
                            .noise_bytes
                            .addr()
                            + i,
                        "the ground noise table",
                    ),
                )
            };
            let n = cell.corners.map(noise);
            trap.finish()?;
            for (c, x) in [0, 1, 3, 2].into_iter().zip(n) {
                work.ground_noise_corners[c].set(x);
            }
            let (n58, n54, n5c, _) =
                ground::bump_noise(n, GROUND_WORK.ground_x.get(), GROUND_WORK.ground_z.get());
            CONTACT_WORK.ground_noise_mix[1].set(n58);
            CONTACT_WORK.ground_noise_mix[0].set(n54);
            CONTACT_WORK.ground_noise_mix[2].set(n5c);
            DRAW.scratch.set(n5c);
            let bump = int_times_times_over_to_int(
                n5c as i32,
                CAR.ground_bump_scale.bits(),
                NUMBERS.k256.bits(),
                NUMBERS.hundred.bits(),
            );
            DRAW.scratch.set(bump as u32);
            work.probe_ground_height[i]
                .set(work.probe_ground_height[i].get().wrapping_add(bump as u32));
            work.probe_face_flip[i].set(COURSE.ground_half.get());
            work.probe_share_u[i].set(GROUND_WORK.plane_f.bits());
            work.probe_share_v[i].set(GROUND_WORK.plane_g.bits());
            let d = work.probe_y[i]
                .get()
                .wrapping_sub(work.probe_ground_height[i].get());
            work.probe_height[i].set(d);
            if b < 4 {
                DRAW.scratch.set(x87::loaded(CAR.crash_matrix[5].bits()));
                if x87::compare_status(DRAW.scratch.get(), NUMBERS.zero.bits()) & 0x100 != 0 {
                    DRAW.scratch.set(x87::loaded(NUMBERS.zero.bits()));
                }
                work.wheel_height_raw[i].set(d);
                work.probe_height[i]
                    .set(x87::int_times_to_int(d as i32, DRAW.scratch.get()) as u32);
            }
            CAR.probe_point_face[b as usize].set(COURSE.ground_slot.get());
        }
        for i in 0..4 {
            CAR.wheel_ground_x[i].set(CONTACT_WORK.probe_x[i].get());
            CAR.wheel_ground_height[i].set(CONTACT_WORK.probe_ground_height[i].get());
            CAR.wheel_ground_z[i].set(CONTACT_WORK.probe_z[i].get());
        }
        Ok(())
    }
}

/// The Rust version.
pub unsafe fn probe_ground() -> Result<(), Refused> {
    unsafe { probe_ground_at(&Ground::game()) }
}
