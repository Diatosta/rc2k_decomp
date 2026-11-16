//! The marks' kind's values: a kind's row of ten tables of 0x22 floats and of three tables of 0x22
//! three-float entries, made into the values the marks run on. As in `math::vector`, what the x87
//! keeps on its stack is an `f64`, rounded to `f32` where it is stored: the same bits at the
//! game's double precision. Values copied as dwords stay bits.

use crate::Invalid;

/// The kinds the tables hold.
pub const KINDS: u32 = 0x22;

/// A kind's row: `count`, `low`, `high` copied as dwords; the rest floats.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Row {
    pub count: u32,
    pub size: f32,
    pub divisor: f32,
    pub low: u32,
    pub high: u32,
    pub start: f32,
    pub middle: f32,
    pub end: f32,
    pub spread: f32,
    pub extra: f32,
}

/// A kind's three-float entries: the place's two ends, and three dwords copied.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Triples {
    pub to: [f32; 3],
    pub from: [f32; 3],
    pub copied: [u32; 3],
}

/// What is made (the set's outputs, in the order they are written).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Values {
    pub count: u32,
    pub size: f32,
    pub ratio: f32,
    pub low: u32,
    pub high: u32,
    pub start: f32,
    pub step: f32,
    pub middle: f32,
    pub slope: f32,
    pub spread: f32,
    pub extra: f32,
    /// The place's z, y, x as written.
    pub place: [f32; 3],
    /// The two differences: the ends' y, and their x negated.
    pub differences: [f32; 2],
}

/// The table entry of a kind; refuses one past the tables.
pub fn kind_entry(kind: u32) -> Result<usize, Invalid> {
    if kind >= KINDS {
        return Err(Invalid("the marks' kind is past the tables (0x22)"));
    }
    Ok(kind as usize)
}

/// The values of a kind's row and triples with the set's 14 constants:
/// - the size times c0, and that (unrounded) over the divisor;
/// - the start times c1; (the middle less the start) over `low` (as a float) times c2;
///   the middle times c3; (the end less the middle) over (`high` less `low`) times c4;
///   the spread times c5, the extra times c6;
/// - each part of the place: from + (to - from) c(7 + 2 part), times c(8 + 2 part); its
///   y less c13;
/// - the ends' y difference, and their x difference negated.
pub fn values(row: &Row, t: &Triples, c: &[f32; 14]) -> Values {
    let k = c.map(f64::from);
    let f = f64::from;
    let (low, high) = (
        f32::from_bits(row.low) as f64,
        f32::from_bits(row.high) as f64,
    );
    let scaled = f(row.size) * k[0];
    let part = |n: usize| (f(t.to[n]) - f(t.from[n])) * k[7 + 2 * n] + f(t.from[n]);
    Values {
        count: row.count,
        size: scaled as f32,
        ratio: (scaled / f(row.divisor)) as f32,
        low: row.low,
        high: row.high,
        start: (f(row.start) * k[1]) as f32,
        step: ((f(row.middle) - f(row.start)) / low * k[2]) as f32,
        middle: (f(row.middle) * k[3]) as f32,
        slope: ((f(row.end) - f(row.middle)) / (high - low) * k[4]) as f32,
        spread: (f(row.spread) * k[5]) as f32,
        extra: (f(row.extra) * k[6]) as f32,
        place: [
            (part(2) * k[12]) as f32,
            (part(1) * k[10] - k[13]) as f32,
            (part(0) * k[8]) as f32,
        ],
        differences: [
            (f(t.to[1]) - f(t.from[1])) as f32,
            (-(f(t.to[0]) - f(t.from[0]))) as f32,
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind() {
        assert!(kind_entry(KINDS).is_err());
        let row = Row {
            count: 7,
            size: 2.0,
            divisor: 8.0,
            low: 2.0f32.to_bits(),
            high: 4.0f32.to_bits(),
            start: 1.0,
            middle: 3.0,
            end: 7.0,
            spread: 5.0,
            extra: 6.0,
        };
        let t = Triples {
            to: [3.0, 5.0, 9.0],
            from: [1.0, 1.0, 1.0],
            copied: [1, 2, 3],
        };
        let mut c = [1.0f32; 14];
        c[0] = 3.0;
        c[13] = 0.5;
        let v = values(&row, &t, &c);
        assert_eq!((v.count, v.size, v.ratio), (7, 6.0, 0.75));
        assert_eq!((v.step, v.slope), (1.0, 2.0));
        assert_eq!(v.place, [9.0, 4.5, 3.0]);
        assert_eq!(v.differences, [4.0, -2.0]);
    }
}
