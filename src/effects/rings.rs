//! `make_masks`' rings (IDA, 2026-10-17): a shape's ring of `n` quads, each from two
//! entries of a corner table (`dword_82E422`, here the table's number: 0x18 bytes an entry), spanned into the mask's
//! runs; its angles kept in a second table at esi.

use crate::effects::mask_constants::MASK_CONSTANTS;
use crate::effects::mask_vars::MASKS;
use crate::effects::spans::{span_quad, spans_fit, triangle_rows};
use crate::errors::{BadInput, Refused};
use crate::pointers::GamePtr;
use crate::session::Bytes;

use crate::math::x87;
use crate::session::{read_bits, write_bits};

/// The ring's floats are `MASKS.ring_*` (0x833D82 the two angles' ends .. 0x833DAE the step).
pub const MOST_QUADS: u32 = 0x80;
const ENTRY: u32 = 0x18;

fn get(at: u32) -> Result<u32, BadInput> {
    GamePtr::new(at).read("a game field")
}

fn put(at: u32, v: u32) -> Result<(), BadInput> {
    GamePtr::new(at).write(v, "a game field")
}

/// The corner table in use (`g_marksRingCorners` holds its number, 0 for A and 1 for B).
fn ring_corners() -> Result<&'static Bytes<3096>, BadInput> {
    crate::effects::corners::tables()
        .get(MASKS.ring_corners.get() as usize)
        .copied()
        .ok_or(BadInput("a ring corner table other than A and B"))
}

/// The dword at byte `at` of a corner table.
fn corner_get(corners: &Bytes<3096>, at: u32) -> Result<u32, BadInput> {
    let bytes = corners.read_at(at as usize, 4, "the ring corners")?;
    Ok(u32::from_le_bytes(bytes.try_into().expect("four bytes")))
}

/// `v` put at byte `at` of a ring angle table.
fn angle_put(angles: &Bytes<1032>, at: u32, v: u32) -> Result<(), BadInput> {
    angles.write_at(at as usize, &v.to_le_bytes(), "the ring angles")
}

/// The quad of corner entries k and k + 1: k's +0, +4, k + 1's +0, +4, k + 1's +0xC, +0x10,
/// k's +0xC, +0x10.
fn quad(corners: &Bytes<3096>, k: u32) -> Result<[u32; 8], Refused> {
    Ok({
        let e = k * ENTRY;
        crate::effects::try_from_fn(|i__| {
            let o = [0, 4, 0x18, 0x1C, 0x24, 0x28, 0xC, 0x10][i__];
            Ok(corner_get(corners, e + o)?)
        })?
    })
}

/// `effects::span_ring_quad` (0x4BE592; it takes the quad's number from its
/// caller's stack): the two angles' sines and minus cosines into 0x833D9A .., the quad they
/// give by the shape's radii (+0x1C, +0x20) and middle (+0xC, +0x10) into
/// `g_marksRingQuad`, then all eight of its floats replaced by corner entries k and k + 1
/// (section 16), and that spanned (`span_quad`). `s` the shape.
fn span_ring_quad(s: u32, k: u32) -> Result<(), Refused> {
    {
        let [a1, a2]: [u32; 2] = read_bits(&MASKS.ring_angles);
        let turns = [
            x87::sine(a1),
            x87::cosine(a1, true),
            x87::sine(a2),
            x87::cosine(a2, true),
        ];
        write_bits(&MASKS.ring_turns, turns);
        let (r1, r2, cx, cy) = (
            get(s + 0x1C)?,
            get(s + 0x20)?,
            get(s + 0xC)?,
            get(s + 0x10)?,
        );
        let [s1, c1, s2, c2] = turns;
        write_bits(
            &MASKS.ring_quad,
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
        let corners = ring_corners()?;
        write_bits(&MASKS.ring_quad, quad(corners, k)?);
        span_quad(crate::pointers::game_address(MASKS.ring_quad.as_ptr()))?;
    }
    Ok(())
}

/// `effects::span_ring` (0x4BE469): shape `s`'s ring (+4 quads): a run counted, the runs'
/// place into +8; the angles' ends (+0x14, +0x18 times 0.0175), their half and middle; for
/// each quad its two angles (the middle less the half times the cosine of the step's
/// angle, and of the next), sines and cosines into the table `angles` (then the angles
/// over the sines), and `span_ring_quad`; the angle on by pi over the count. Refuses a
/// count of 0 or less or past 0x80 (the original's loop tests after counting), and runs or
/// spans past their room, before writing.
pub fn span_ring(s: u32, angles: &Bytes<1032>) -> Result<(), Refused> {
    let mut table = 0u32;
    {
        let n = get(s + 4)? as i32;
        if n <= 0 || n > MOST_QUADS as i32 {
            return Err(BadInput("a ring of no quads or more than 0x80").into());
        }
        let corners = ring_corners()?;
        let mut rows = Some(0u32);
        for k in 0..n as u32 {
            let q = quad(corners, k)?;
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
        MASKS.run_count.set(MASKS.run_count.get().wrapping_add(1));
        put(s + 8, MASKS.runs.get())?;
        let scales: [u32; 3] = read_bits(&MASK_CONSTANTS.ring_scales);
        let ends = [
            x87::mul(get(s + 0x14)?, scales[0]),
            x87::mul(get(s + 0x18)?, scales[1]),
        ];
        write_bits(&MASKS.ring_ends, ends);
        let (half, middle) = x87::half_and_middle(ends[1], ends[0], scales[2]);
        MASKS.ring_half.set_bits(half);
        MASKS.ring_middle.set_bits(middle);
        let mut angle = 0u32;
        MASKS.ring_angle.set(angle);
        let step = x87::pi_over(n);
        MASKS.ring_step.set_bits(step);
        for k in 0..n as u32 {
            let a1 = x87::less_cosine(angle, None, half, middle);
            let a2 = x87::less_cosine(angle, Some(step), half, middle);
            write_bits(&MASKS.ring_angles, [a1, a2]);
            for (o, a) in [(0, a1), (8, a2)] {
                angle_put(angles, table + o + 4, x87::cosine(a, false))?;
                angle_put(angles, table + o, x87::sine(a))?;
            }
            angle_put(angles, table, a1)?;
            angle_put(angles, table + 8, a2)?;
            table += 8;
            span_ring_quad(s, k)?;
            angle = x87::add(angle, step);
            MASKS.ring_angle.set(angle);
        }
    }
    Ok(())
}
