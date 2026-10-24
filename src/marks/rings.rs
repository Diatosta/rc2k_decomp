//! `make_masks`' rings (IDA, 2026-10-17): a shape's ring of `n` quads, each from two
//! entries of a corner table (`dword_82E422`: 0x18 bytes an entry), spanned into the mask's
//! runs; its angles kept in a second table at esi.

use crate::errors::{BadInput, Refused};
use crate::marks::masks::{RUN_COUNT_RVA, RUNS_RVA};
use crate::marks::spans::{span_quad, spans_fit, triangle_rows};
use crate::marks::store::{Slot, game_addr, game_read, game_write};
use crate::math::x87;

pub const CORNERS_RVA: Slot = Slot::at(0x82E422); // g_marksRingCorners: the corner table in use
const SCALES_RVA: Slot = Slot::at(0x89C9B0); // flt_89C9B0, flt_89C9B4, flt_89C9B8
/// The ring's floats: 0x833D82 the two angles' ends .. 0x833DAE the step.
const ENDS_RVA: Slot = Slot::at(0x833D82); // flt_833D82, flt_833D86
const HALF_RVA: Slot = Slot::at(0x833D8E); // flt_833D8E
const MIDDLE_RVA: Slot = Slot::at(0x833D8A); // flt_833D8A
const ANGLES_RVA: Slot = Slot::at(0x833D92); // flt_833D92, flt_833D96
const TURNS_RVA: Slot = Slot::at(0x833D9A); // flt_833D9A .. flt_833DA6
pub const QUAD_RVA: Slot = Slot::at(0x833D5E); // g_marksRingQuad
const ANGLE_RVA: Slot = Slot::at(0x833DAA); // dword_833DAA
const STEP_RVA: Slot = Slot::at(0x833DAE); // flt_833DAE
pub const MOST_QUADS: u32 = 0x80;
const ENTRY: u32 = 0x18;

unsafe fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

unsafe fn put(at: u32, v: u32) {
    unsafe { (at as *mut u32).write_unaligned(v) }
}

/// The quad of corner entries k and k + 1: k's +0, +4, k + 1's +0, +4, k + 1's +0xC, +0x10,
/// k's +0xC, +0x10.
unsafe fn quad(corners: u32, k: u32) -> [u32; 8] {
    let e = corners.wrapping_add(k * ENTRY);
    unsafe { [0, 4, 0x18, 0x1C, 0x24, 0x28, 0xC, 0x10].map(|o| get(e + o)) }
}

/// `marks::span_ring_quad` (0x4BE592; it takes the quad's number from its
/// caller's stack): the two angles' sines and minus cosines into 0x833D9A .., the quad they
/// give by the shape's radii (+0x1C, +0x20) and middle (+0xC, +0x10) into
/// `g_marksRingQuad`, then all eight of its floats replaced by corner entries k and k + 1
/// (section 16), and that spanned (`span_quad`). `s` the shape.
unsafe fn span_ring_quad(s: u32, k: u32) -> Result<(), Refused> {
    unsafe {
        let [a1, a2]: [u32; 2] = game_read(ANGLES_RVA);
        let turns = [
            x87::sine(a1),
            x87::cosine(a1, true),
            x87::sine(a2),
            x87::cosine(a2, true),
        ];
        game_write(TURNS_RVA, turns);
        let (r1, r2, cx, cy) = (get(s + 0x1C), get(s + 0x20), get(s + 0xC), get(s + 0x10));
        let [s1, c1, s2, c2] = turns;
        game_write(
            QUAD_RVA,
            [
                x87::mul_add(r1, s1, cx),
                x87::mul_add(r1, c1, cy),
                x87::mul_add(r1, s2, cx),
                x87::mul_add(r1, c2, cy),
                x87::mul_add(r2, s2, cx),
                x87::mul_add(r2, c2, cy),
                x87::mul_add(r2, s1, cx),
                x87::mul_add(r2, c1, cy),
            ],
        );
        let corners: u32 = game_read(CORNERS_RVA);
        game_write(QUAD_RVA, quad(corners, k));
        span_quad(game_addr(QUAD_RVA) as u32)?;
    }
    Ok(())
}

/// `marks::span_ring` (0x4BE469): shape `s`'s ring (+4 quads): a run counted, the runs'
/// place into +8; the angles' ends (+0x14, +0x18 times 0.0175), their half and middle; for
/// each quad its two angles (the middle less the half times the cosine of the step's
/// angle, and of the next), sines and cosines into the table at `angles` (then the angles
/// over the sines), and `span_ring_quad`; the angle on by pi over the count. Refuses a
/// count of 0 or less or past 0x80 (the original's loop tests after counting), and runs or
/// spans past their room, before writing.
pub unsafe fn span_ring(s: u32, angles: u32) -> Result<(), Refused> {
    let mut table = angles;
    unsafe {
        let n = get(s + 4) as i32;
        if n <= 0 || n > MOST_QUADS as i32 {
            return Err(BadInput("a ring of no quads or more than 0x80").into());
        }
        let corners: u32 = game_read(CORNERS_RVA);
        let mut rows = Some(0u32);
        for k in 0..n as u32 {
            let q = quad(corners, k);
            for t in [
                [q[0], q[1], q[2], q[3], q[4], q[5]],
                [q[4], q[5], q[6], q[7], q[0], q[1]],
            ] {
                rows = match (rows, triangle_rows(t)) {
                    (Some(a), Some(b)) => Some(a + b),
                    _ => None,
                };
            }
        }
        spans_fit(2 * n as u32, rows)?;
        game_write(
            RUN_COUNT_RVA,
            game_read::<u32>(RUN_COUNT_RVA).wrapping_add(1),
        );
        put(s + 8, game_read(RUNS_RVA));
        let scales: [u32; 3] = game_read(SCALES_RVA);
        let ends = [
            x87::mul(get(s + 0x14), scales[0]),
            x87::mul(get(s + 0x18), scales[1]),
        ];
        game_write(ENDS_RVA, ends);
        let (half, middle) = x87::half_and_middle(ends[1], ends[0], scales[2]);
        game_write(HALF_RVA, half);
        game_write(MIDDLE_RVA, middle);
        let mut angle = 0u32;
        game_write(ANGLE_RVA, angle);
        let step = x87::pi_over(n);
        game_write(STEP_RVA, step);
        for k in 0..n as u32 {
            let a1 = x87::less_cosine(angle, None, half, middle);
            let a2 = x87::less_cosine(angle, Some(step), half, middle);
            game_write(ANGLES_RVA, [a1, a2]);
            for (o, a) in [(0, a1), (8, a2)] {
                put(table + o + 4, x87::cosine(a, false));
                put(table + o, x87::sine(a));
            }
            put(table, a1);
            put(table + 8, a2);
            table += 8;
            span_ring_quad(s, k)?;
            angle = x87::add(angle, step);
            game_write(ANGLE_RVA, angle);
        }
    }
    Ok(())
}
