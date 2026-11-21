//! `make_masks`' rings: a shape's ring of `n` quads, each from two entries of a corner table (here
//! the table's number: 0x18 bytes an entry), spanned into the mask's runs; its angles kept in a
//! second table.

use crate::effects::mask_store::Masks;
use crate::effects::spans::{span_quad, spans_fit, triangle_rows};
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::math::x87_float as bh;

/// The ring's floats are `MASKS.ring_*` (from the two angles' ends to the step).
const MOST_QUADS: u32 = 0x80;
const ENTRY: u32 = 0x18;

/// The number of the corner table in use (held in the variables: 0 for A and 1 for B).
fn ring_corners(masks: &Masks) -> Result<usize, BadInput> {
    let table = masks.vars.borrow().ring_corners as usize;
    if table < 2 {
        Ok(table)
    } else {
        Err(BadInput("a ring corner table other than A and B"))
    }
}

/// The dword at byte `at` of corner table `table`.
fn corner_get(masks: &Masks, table: usize, at: u32) -> Result<u32, BadInput> {
    let vars = masks.vars.borrow();
    let bytes = vars.corners[table]
        .get(at as usize..)
        .and_then(|rest| rest.first_chunk::<4>())
        .ok_or(BadInput("the ring corners"))?;
    Ok(u32::from_le_bytes(*bytes))
}

/// `v` put at byte `at` of ring angle table `table`.
fn angle_put(masks: &Masks, table: usize, at: u32, v: u32) -> Result<(), BadInput> {
    let mut vars = masks.vars.borrow_mut();
    let bytes = vars.ring_angle_tables[table]
        .get_mut(at as usize..)
        .and_then(|rest| rest.first_chunk_mut::<4>())
        .ok_or(BadInput("the ring angles"))?;
    *bytes = v.to_le_bytes();
    Ok(())
}

/// The quad of corner entries k and k + 1 of corner table `table`: the first two values of k, the
/// first two of k + 1, then the fourth and fifth of k + 1, and the fourth and fifth of k.
fn quad(masks: &Masks, table: usize, k: u32) -> Result<[u32; 8], Refused> {
    Ok({
        let e = k * ENTRY;
        crate::effects::try_from_fn(|i__| {
            let o = [0, 4, 0x18, 0x1C, 0x24, 0x28, 0xC, 0x10][i__];
            Ok(corner_get(masks, table, e + o)?)
        })?
    })
}

/// `effects::span_ring_quad`: the two angles' sines and minus cosines into the ring's
/// floats, the quad they give by the shape's `radii` and `middle`
/// into the ring's quad, then all eight of its floats overwritten by corner entries k and
/// k + 1 (so the computed quad is lost: a slip kept as is), and that spanned (`span_quad`).
/// `shape` the shape's number.
fn span_ring_quad(
    masks: &Masks,
    shape: usize,
    k: u32,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    {
        let [a1, a2]: [u32; 2] = masks.vars.borrow().ring_angles.map(f32::to_bits);
        let turns = [
            x87::sine(a1),
            x87::cosine(a1, true),
            x87::sine(a2),
            x87::cosine(a2, true),
        ];
        let (r1, r2, cx, cy) = {
            let mut vars = masks.vars.borrow_mut();
            vars.ring_turns = turns;
            let s = &vars.shapes[shape];
            (s.radii[0], s.radii[1], s.middle[0], s.middle[1])
        };
        let [s1, c1, s2, c2] = turns;
        masks.vars.borrow_mut().ring_quad = [
            x87::mul_add(r1, s1, cx),
            x87::mul_add(r1, c1, cy),
            x87::mul_add(r1, s2, cx),
            x87::mul_add(r1, c2, cy),
            x87::mul_add(r2, s2, cx),
            x87::mul_add(r2, c2, cy),
            x87::mul_add(r2, s1, cx),
            x87::mul_add(r2, c1, cy),
        ];
        let corners = ring_corners(masks)?;
        let corner_quad = quad(masks, corners, k)?.map(f32::from_bits);
        masks.vars.borrow_mut().ring_quad = corner_quad;
        span_quad(masks, corner_quad, effects)?;
    }
    Ok(())
}

/// `effects::span_ring`: shape `shape`'s (its number) ring (its quads): a run counted, the runs' place into its
/// rows; the angles' ends (`angle_ends` times 0.0175), their half and middle; for each quad its two
/// angles (the middle less the half times the cosine of the step's angle, and of the next),
/// sines and cosines into the table `angles` (then the angles over the sines), and
/// `span_ring_quad`; the angle on by pi over the count. Refuses a count of 0 or less or past
/// 0x80, and runs or spans past their room, before writing.
pub(super) fn span_ring(
    masks: &Masks,
    shape: usize,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let mut table = 0u32;
    {
        let (count, angle_ends) = {
            let vars = masks.vars.borrow();
            let s = &vars.shapes[shape];
            (s.count, s.angle_ends)
        };
        let n = count as i32;
        if n <= 0 || n > MOST_QUADS as i32 {
            return Err(BadInput("a ring of no quads or more than 0x80").into());
        }
        let corners = ring_corners(masks)?;
        let mut rows = Some(0u32);
        for k in 0..n as u32 {
            let q = quad(masks, corners, k)?;
            for t in [
                [q[0], q[1], q[2], q[3], q[4], q[5]],
                [q[4], q[5], q[6], q[7], q[0], q[1]],
            ] {
                rows = match (rows, triangle_rows(t, effects)) {
                    (Some(a), Some(b)) => Some(a + b),
                    _ => None,
                };
            }
        }
        spans_fit(masks, 2 * n as u32, rows)?;
        {
            let mut vars = masks.vars.borrow_mut();
            vars.run_count = vars.run_count.wrapping_add(1);
            vars.shapes[shape].rows = vars.runs;
        }
        let scales: [u32; 3] = effects
            .setup
            .mask_constants
            .borrow()
            .ring_scales
            .map(f32::to_bits);
        let ends = [
            x87::mul(angle_ends[0], scales[0]),
            x87::mul(angle_ends[1], scales[1]),
        ];
        let (half, middle) = x87::half_and_middle(ends[1], ends[0], scales[2]);
        let mut angle = 0u32;
        let step = bh::stf(bh::int(n).pi_over());
        {
            let mut vars = masks.vars.borrow_mut();
            vars.ring_ends = ends;
            vars.ring_half = half;
            vars.ring_middle = middle;
            vars.ring_angle = angle;
            vars.ring_step = step;
        }
        for k in 0..n as u32 {
            let a1 = x87::less_cosine(angle, None::<f32>, half, middle);
            let a2 = x87::less_cosine(angle, Some(step), half, middle);
            masks.vars.borrow_mut().ring_angles = [a1, a2];
            for (o, a) in [(0, a1), (8, a2)] {
                angle_put(masks, shape, table + o + 4, x87::cosine(a, false).to_bits())?;
                angle_put(masks, shape, table + o, x87::sine(a).to_bits())?;
            }
            angle_put(masks, shape, table, a1.to_bits())?;
            angle_put(masks, shape, table + 8, a2.to_bits())?;
            table += 8;
            span_ring_quad(masks, shape, k, effects)?;
            angle = x87::add(angle, step).to_bits();
            masks.vars.borrow_mut().ring_angle = angle;
        }
    }
    Ok(())
}
