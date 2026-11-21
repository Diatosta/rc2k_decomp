//! Drawing between two ticks: the values a frame shows are blended from the last two ticks'
//! (`prev`, `cur`) by how far the frame's time is past the last tick. The simulation never sees
//! them; a frame at a tick shows that tick (`alpha` 1) or, a tick behind, the one before it
//! (`alpha` 0).
//!
//! Words are the cells' bits: whole numbers (places in 16.16, wrapping), angles (a turn is
//! 2^32, the shorter way round) and floats.

/// The simulation's tick in milliseconds.
pub const TICK_MS: u32 = 10;

/// A car or camera that moved further than this between two ticks (in 16.16, 8 units) was put
/// somewhere (a reset, a camera cut): its frame shows the new tick, not a glide across.
pub const JUMP: u32 = 8 << 16;

/// An angle that turned further than this between two ticks (a quarter turn) was set, not
/// turned.
pub const JUMP_ANGLE: u32 = 1 << 30;

/// Matrix entries further apart than this between two ticks (a rotation of about 60 degrees)
/// were set, not turned.
pub(crate) const JUMP_ENTRY: f32 = 1.0;

/// How far a frame is between the last two ticks (0: the one before, 1: the last), from the
/// time since the last tick in milliseconds (`rest`, 0 .. 10): the picture shown is a tick
/// behind, so that it lies between two ticks the simulation has made. A `rest` past the tick
/// (a clock gone wrong) shows the last tick.
pub fn alpha(rest: u32) -> f32 {
    if rest >= TICK_MS {
        1.0
    } else {
        rest as f32 / TICK_MS as f32
    }
}

/// The whole number between `prev` and `cur` (as signed, wrapping) at `alpha`.
pub fn word(prev: u32, cur: u32, alpha: f32) -> u32 {
    let delta = cur.wrapping_sub(prev) as i32;
    prev.wrapping_add((f64::from(delta) * f64::from(alpha)).round() as i32 as u32)
}

/// Whether two whole numbers lie within `limit` of each other (as signed, wrapping).
pub fn near(prev: u32, cur: u32, limit: u32) -> bool {
    cur.wrapping_sub(prev).wrapping_add(limit) <= limit.saturating_mul(2)
}

/// The float between `prev` and `cur` (bits in, bits out).
pub fn float(prev: u32, cur: u32, alpha: f32) -> u32 {
    let (a, b) = (f32::from_bits(prev), f32::from_bits(cur));
    if !a.is_finite() || !b.is_finite() {
        return cur;
    }
    (a + (b - a) * alpha).to_bits()
}

/// Whether two matrices' entries are all within `JUMP_ENTRY` of each other (and finite).
pub fn matrices_near(prev: &[u32], cur: &[u32]) -> bool {
    prev.iter().zip(cur).all(|(&a, &b)| {
        let (a, b) = (f32::from_bits(a), f32::from_bits(b));
        a.is_finite() && b.is_finite() && (b - a).abs() <= JUMP_ENTRY
    })
}

/// A cosmetic effect that spawns once a frame spawns at most once a tick instead, so that a
/// faster screen gets no more of it: with `last` the clock (in milliseconds) of the last spawn
/// and `now` the frame's, the clock to keep if a spawn is due, none if the frame is
/// within a tick of the last. Frames a tick or more apart (a screen of a hundred frames a second
/// or fewer) each spawn, as before; the spawns' times keep their rhythm, a tick apart. A clock
/// far from `last` (the start of a race, a load) starts anew.
pub fn spawn_due(last: u32, now: u32) -> Option<u32> {
    let since = now.wrapping_sub(last);
    if since < TICK_MS {
        None
    } else if since < 10 * TICK_MS {
        Some(last.wrapping_add(since / TICK_MS * TICK_MS))
    } else {
        Some(now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alpha_is_the_rest_of_the_tick() {
        assert_eq!(alpha(0), 0.0);
        assert_eq!(alpha(5), 0.5);
        assert_eq!(alpha(10), 1.0);
        assert_eq!(alpha(4_000_000_000), 1.0);
    }

    #[test]
    fn words_go_the_short_way_round() {
        assert_eq!(word(100, 200, 0.5), 150);
        assert_eq!(word(100, 200, 0.0), 100);
        assert_eq!(word(100, 200, 1.0), 200);
        assert_eq!(word(200, 100, 0.25), 175);
        // An angle across the wrap of a turn.
        assert_eq!(word(0xFFFF_FF00, 0x100, 0.5), 0);
        assert_eq!(word(5u32.wrapping_neg(), 5, 0.5), 0);
    }

    #[test]
    fn a_spawn_is_due_once_a_tick() {
        // A hundred frames a second or fewer: every frame.
        assert_eq!(spawn_due(1000, 1010), Some(1010));
        assert_eq!(spawn_due(1000, 1017), Some(1010));
        assert_eq!(spawn_due(1010, 1033), Some(1030));
        // A faster screen: the frames between are left out.
        assert_eq!(spawn_due(1000, 1004), None);
        assert_eq!(spawn_due(1000, 1009), None);
        // A start, or a clock gone backwards.
        assert_eq!(spawn_due(0, 5_000_000), Some(5_000_000));
        assert_eq!(spawn_due(5000, 4000), Some(4000));
    }

    #[test]
    fn jumps_are_told_from_steps() {
        assert!(near(1000, 1000 + JUMP, JUMP));
        assert!(!near(1000, 1001 + JUMP, JUMP));
        assert!(near(1000, 1000u32.wrapping_sub(JUMP), JUMP));
        assert!(near(5, 0xFFFF_FFFF, JUMP));
        assert!(!near(0, 0x4000_0000, JUMP));
        assert!(near(0, 0x3FFF_FFFF, JUMP_ANGLE));
        assert!(!near(0, 0x4000_0001, JUMP_ANGLE));
    }

    #[test]
    fn floats_blend_and_nan_shows_the_tick() {
        let half = float(1.0f32.to_bits(), 3.0f32.to_bits(), 0.5);
        assert_eq!(f32::from_bits(half), 2.0);
        assert_eq!(
            float(f32::NAN.to_bits(), 3.0f32.to_bits(), 0.5),
            3.0f32.to_bits()
        );
    }

    #[test]
    fn matrices_set_apart_are_not_blended() {
        let a = [0.0f32.to_bits(); 4];
        let b = [0.5f32.to_bits(); 4];
        let c = [1.5f32.to_bits(); 4];
        assert!(matrices_near(&a, &b));
        assert!(!matrices_near(&a, &c));
    }
}
