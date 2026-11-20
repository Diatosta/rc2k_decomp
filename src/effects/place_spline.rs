//! `effects::place_spline`: spline `n` (of the 8) placed: the spline offset = its number times
//! `SPLINE_LEN`; the settings' `listener_scale` by one constant (unrounded) into the size of its
//! A, B and C records, and that by another into D's; A and D by `effects::place_shape_points`;
//! B's second size = its size by one constant when under a limit, else by another, the grade flag
//! 0, B by `effects::place_graded_points`; C's second size = its size by a constant when above a
//! limit (the compare's status 0), else by another, the grade flag 1, C by
//! `place_graded_points`. Each compare's status flags go to the shared FPU status cell.
//!
//! Refuses null settings, a spline number of 8 or more (before anything), and its steps' refusals
//! (all it may write is put back first).

use crate::effects::paths_store::Paths;
use crate::effects::place_graded_points::place_graded_points;
use crate::effects::splines::{SPLINE_LEN, Spline, place_shape_points as place_shape};
use crate::errors::{BadInput, Refused};
use crate::math::x87::{self, Flt};

const SPLINES: u32 = 8;

/// The four records of spline `n`: A, B, C, D.
fn records(n: u32) -> [Spline; 4] {
    [0, 1, 2, 3].map(|t| Spline::record(t, n))
}

/// The sizes as float bits: `a` times `k` (the first, for A, B and C) and `a` times `k` times
/// `k2` (the second, for D).
fn sizes(a: impl Flt + Copy, k: impl Flt + Copy, k2: impl Flt) -> (u32, u32) {
    let abc = x87::mul(a, k);
    let d = x87::mul2(a, k, k2);
    (abc.to_bits(), d.to_bits())
}

pub fn place_spline(
    paths: &Paths,
    n: u32,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let setup = &effects.setup;
    let settings: u32 = setup.settings.is_set.get();
    if settings == 0 {
        return Err(BadInput("no marks settings").into());
    }
    let marks = effects.settings()?;
    if n >= SPLINES {
        return Err(BadInput("a spline number of 8 or more").into());
    }
    {
        let k = paths.spline_constants.place_k.each_ref();
        paths.spline_work.offset.set(n * SPLINE_LEN);
        let [a, b, c, d] = records(n);
        let (abc, dsize) = sizes(&marks.listener_scale, k[0], k[1]);
        for r in [a, b, c] {
            r.put(paths, 0x24, abc)?;
        }
        d.put(paths, 0x24, dsize)?;
        for r in [a, d] {
            place_shape(&effects.stage()?.object_points, paths, r)?;
        }
        let status = x87::compare_status(b.get(paths, 0x24)?, k[2]);
        setup.work.fpu_status.set(status as u32);
        let f = if status == 0x100 { k[4] } else { k[3] };
        b.put(paths, 0x2C, x87::mul(b.get(paths, 0x24)?, f).to_bits())?;
        paths.spline_head.switch.set(0);
        place_graded_points(&effects.stage()?.object_points, paths, b)?;
        let status = x87::compare_status(c.get(paths, 0x24)?, k[5]);
        setup.work.fpu_status.set(status as u32);
        let f = if status == 0 { k[7] } else { k[6] };
        c.put(paths, 0x2C, x87::mul(c.get(paths, 0x24)?, f).to_bits())?;
        paths.spline_head.switch.set(1);
        place_graded_points(&effects.stage()?.object_points, paths, c)?;
    }
    Ok(())
}
