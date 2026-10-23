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
//! `sub_45DC12`.
//! Refuses (before anything: what it wrote is put back) a count above 0x40 (the lists'
//! room), and what `find_ground` refuses on any point.
//! Self-test: a copy of the original (its call to a copy of `find_ground`, its
//! `rotate_point` the game's hook) on the test's ground and work copy: counts 0, 1, 4,
//! 6, places and turns by lot over the test's grid, the switch bits, under three
//! precisions; the points found counted; the refusal.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{EAX, EBX, ECX, EDX, ESI, Regs, rva};
use crate::math::x87;
use crate::math::x87_run::{int_times_times_over_to_int, less_plus_less_product_80};
use crate::physics::ground::*;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use rc2k_core::physics::ground;

pub const MOST_POINTS: u32 = 0x40;
const COUNT: u32 = 0x71A238; // dword_71A238
const NOISE: u32 = 0xA2AAE4; // g_noiseBytes, 0x1000

/// The Rust core; `r` as the original leaves it.
// Converted: `rc2k_core::physics::ground::{wheel_lifted, noise_cell, bump_noise}` (the
// turns and the bump's scaling stay x87).
pub unsafe fn probe_ground_at(g: &Ground, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        let n = g.get(COUNT);
        if n < 1 {
            return Ok(());
        }
        if n > MOST_POINTS {
            return Err(BadInput("more than 0x40 probe points (the lists' room)").into());
        }

        probe_points(g, r, n)
    }
}

unsafe fn probe_points(g: &Ground, r: &mut Regs, n: u32) -> Result<(), Refused> {
    unsafe {
        let k65536 = g.get(0x5C5D98);
        g.put(0x6D3B84, u32::MAX);
        for b in 0..n {
            r.ebx = b;
            let at = |base: u32| base + 4 * b;
            let z = g.get(at(0x71ACC8));
            let y = g.get(at(0x71ABC8));
            let y = if b < 4 {
                g.put(0x62400C, x87::loaded(y));
                let add;
                (add, r.ecx) = ground::wheel_lifted(g.get(0x71A270), b);
                less_plus_less_product_80(
                    g.get(0x71ABE0),
                    g.get(0x614BF4),
                    add.then(|| g.get(0x613E8C)),
                    g.get(0x613F8C),
                    g.get(0x614BF8),
                )
            } else {
                extended(y)
            };
            let x = extended(g.get(at(0x71AAC8)));
            r.esi = g.a(0x71A08C);
            let out = rotate_point(r.esi, [x, y, extended(z)]);
            let (ox, oy, oz) = (rounded(out[0]), rounded(out[1]), rounded(out[2]));
            g.put(0x6D3628, ox);
            g.put(at(0x6D2214), ox);
            g.put(at(0x6D2314), oy);
            g.put(0x6D3630, oz);
            g.put(at(0x6D2414), oz);
            for (v, place, list) in [
                (ox, 0x71A10C, 0x6D1E14),
                (oy, 0x71A110, 0x6D1F14),
                (oz, 0x71A114, 0x6D2014),
            ] {
                let w = (x87::mul_to_int(v, k65536) as u32).wrapping_add(g.get(place));
                g.put(at(list), w);
            }
            g.put(0x639824, g.get(at(0x6D1E14)));
            g.put(0x63982C, g.get(at(0x6D2014)));
            r.eax = g.get(at(0x71ADC8));
            g.put(0x5CCC78, r.eax);
            g.put(0x620470, 0x80);
            let mut inner = *r;
            ground_search_at(g, &mut inner)?;
            g.put(at(0x6D2114), 0);
            if g.get(0x639830) == 0 {
                g.put(0x6D3B84, b);
                g.put(at(0x6D2714), 0);
                continue;
            }
            g.put(at(0x6D2714), 1);
            g.put(at(0x6D2114), g.get(0x639828));
            let cell = ground::noise_cell(g.get(0x639824), g.get(0x63982C));
            g.put(0x6D3060, cell.row);
            g.put(0x6D3064, cell.row_offset);
            let esi = cell.below;
            let noise = |i: u32| {
                u32::from(
                    (crate::framework::game_addr(rva(NOISE as usize) + i as usize) as *const u8)
                        .read(),
                )
            };
            let n = cell.corners.map(noise);
            for (v, x) in [0x6D3044, 0x6D3048, 0x6D3050, 0x6D304C].into_iter().zip(n) {
                g.put(v, x);
            }
            let (n58, n54, n5c, high) = ground::bump_noise(n, g.get(0x639824), g.get(0x63982C));
            let fz = g.get(0x63982C) & 0x3FFF;
            g.put(0x6D3058, n58);
            g.put(0x6D3054, n54);
            g.put(0x6D305C, n5c);
            g.put(0x62400C, n5c);
            let bump = int_times_times_over_to_int(
                n5c as i32,
                g.get(0x71A038),
                g.get(0x5C5E2C),
                g.get(0x5C5D14),
            );
            g.put(0x62400C, bump as u32);
            g.put(at(0x6D2114), g.get(at(0x6D2114)).wrapping_add(bump as u32));
            g.put(at(0x6D2914), g.get(0x5CCC6C));
            g.put(at(0x6D2A14), g.get(0x639810));
            g.put(at(0x6D2B14), g.get(0x639814));
            let d = g.get(at(0x6D1F14)).wrapping_sub(g.get(at(0x6D2114)));
            g.put(at(0x6D2514), d);
            if b < 4 {
                g.put(0x62400C, x87::loaded(g.get(0x71A0A0)));
                if x87::compare_status(g.get(0x62400C), g.get(0x5C5D90)) & 0x100 != 0 {
                    g.put(0x62400C, x87::loaded(g.get(0x5C5D90)));
                }
                g.put(at(0x6D2614), d);
                g.put(
                    at(0x6D2514),
                    x87::int_times_to_int(d as i32, g.get(0x62400C)) as u32,
                );
            }
            r.eax = g.get(0x5CCC74);
            g.put(at(0x71ADC8), r.eax);
            r.ecx = fz;
            r.esi = esi;
            r.edx = high;
        }
        for k in 0..4u32 {
            g.put(0x71AEE8 + 4 * k, g.get(0x6D1E14 + 4 * k));
            g.put(0x71AEF8 + 4 * k, g.get(0x6D2114 + 4 * k));
            g.put(0x71AF08 + 4 * k, g.get(0x6D2014 + 4 * k));
        }
        r.ebx = n;
        Ok(())
    }
}

/// The Rust version: `r` as the original leaves it.
pub unsafe fn probe_ground(r: &mut Regs) -> Result<(), Refused> {
    unsafe { probe_ground_at(&Ground::game(), r) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let mut g = *r;
    unsafe { probe_ground(&mut g) }?;
    Ok(Left::nothing()
        .with(EAX, g.eax)
        .with(EBX, g.ebx)
        .with(ECX, g.ecx)
        .with(EDX, g.edx)
        .with(ESI, g.esi))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
