//! The cars' parts made ready to draw: the switch bytes of a car's marks and lamps, their count,
//! and the axles' places.

use crate::Invalid;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PartState {
    /// Non-zero when the car's lights are on.
    pub on: u32,
    /// The marks' mask.
    pub marks_mask: u32,
    /// The lamps' mask.
    pub lamps_mask: u32,
    /// Non-zero when the view is inside the car.
    pub inside: u32,
    /// The car's mode (2 and 3 set `mode_on`).
    pub mode: u32,
    /// -1 (all bits set) sets the reversing pair of switches.
    pub reversing: u32,
    /// The six parts' broken state.
    pub broken: [u32; 6],
    /// Bit 9: the six marks are counted.
    pub flags: u32,
}

/// The switches made: the marks' and mode's flags, the switch bytes, and the marks' count
/// before scaling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MarkSwitches {
    /// Non-zero when the marks are on.
    pub marks_on: u32,
    /// Non-zero for car modes 2 and 3.
    pub mode_on: u32,
    pub bytes: [u8; SWITCH_BYTES],
    /// How many marks are counted (before scaling into the byte count).
    pub count: u8,
}

/// The switch bytes' first offset in the car block, and their number.
pub const FIRST_SWITCH: u32 = 0xAC0;
pub const SWITCH_BYTES: usize = 0x11;

/// The switches of the car part matrices (the broken parts' bytes are cut twice: same as once).
pub fn mark_switches(p: &PartState) -> MarkSwitches {
    let mut b = [0u8; SWITCH_BYTES];
    let at = |o: u32| (o - FIRST_SWITCH) as usize;
    let lamps = (p.lamps_mask & p.on) as u8;
    b[at(0xAC8)] = lamps;
    b[at(0xAC9)] = lamps;
    for o in 0xAC0..=0xAC3 {
        b[at(o)] = p.on as u8;
    }
    let marks = (p.on & p.marks_mask) as u8;
    for o in 0xACA..=0xACF {
        b[at(o)] = marks;
    }
    let inside = u8::from(p.inside != 0);
    for o in [0xAC4, 0xAC5, 0xAD0] {
        b[at(o)] = inside;
    }
    let reversing = u8::from(p.reversing == u32::MAX);
    b[at(0xAC6)] = reversing;
    b[at(0xAC7)] = reversing;
    let mut cut = |o: u32, broken: u32| b[at(o)] &= (broken ^ 1) as u8;
    cut(0xAC0, p.broken[0]);
    cut(0xAC1, p.broken[1]);
    for _ in 0..2 {
        for o in 0xACA..=0xACF {
            cut(o, p.broken[4]);
        }
        for o in [0xAC8, 0xAC2, 0xAC4, 0xAC6] {
            cut(o, p.broken[2]);
        }
        for o in [0xAC9, 0xAC3, 0xAC5, 0xAC7] {
            cut(o, p.broken[3]);
        }
        cut(0xAD0, p.broken[5]);
    }
    let mut count = b[at(0xAC0)].wrapping_add(b[at(0xAC1)]);
    count = count.wrapping_add(count);
    count = count.wrapping_add(count);
    if p.flags & (1 << 9) != 0 {
        for o in 0xACA..=0xACF {
            count = count.wrapping_add(b[at(o)]);
        }
    }
    MarkSwitches {
        marks_on: p.marks_mask & p.on,
        mode_on: u32::from(matches!(p.mode, 2 | 3)),
        bytes: b,
        count,
    }
}

/// The marks' byte count: count x 0x20, 0 when the wear is not below 100.0 or the lights are off.
pub fn marks_bytes(count: u8, slow: bool, on: u32) -> u32 {
    if !slow || on == 0 {
        0
    } else {
        count as u32 * 0x20
    }
}

/// The steering: the top half of the car's steering word.
pub fn steering(word: u32) -> u32 {
    word >> 16
}

/// An axle value x 0x10000 / 1000; None where the division would overflow.
pub fn per_thousand(v: i64) -> Option<u32> {
    let q = v * 0x1_0000 / 1000;
    (i32::MIN as i64..=i32::MAX as i64)
        .contains(&q)
        .then_some(q as u32)
}

/// The five axle values from the model's record and the lift: (front x, rear x less lift,
/// height, front z, rear z less lift); None where a scaling would overflow.
pub fn axles(record: [u32; 5], lift: u32) -> Option<[u32; 5]> {
    let rec = |n: usize| record[n] as i32 as i64;
    Some([
        per_thousand((rec(2) as i32).wrapping_add(0xC3) as i64)?,
        per_thousand(rec(3))?.wrapping_sub(lift),
        per_thousand(rec(0))?,
        per_thousand((rec(1) as i32).wrapping_neg() as i64)?,
        per_thousand(rec(4))?.wrapping_sub(lift),
    ])
}

/// Refuses a car count outside 1 ..= 8 or a car without its block.
pub fn check_part_cars(count: u32, block: impl Fn(u32) -> u32) -> Result<(), Invalid> {
    if count == 0 || count > 8 || (0..count).any(|n| block(n) == 0) {
        return Err(Invalid(
            "no race cars, more than eight, or one without its block",
        ));
    }
    Ok(())
}

/// A car's parts are made ready only when they are shown.
pub fn parts_shown(shown: u32) -> bool {
    shown != 0
}

/// The matrices kept in the block: the four wheels', the car's turn and its place.
pub const KEPT_MATRICES: u32 = 6;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn switches() {
        let p = PartState {
            on: 0x1FF,
            marks_mask: 0xFFFF_FFFF,
            lamps_mask: 0x0F,
            inside: 3,
            mode: 3,
            reversing: u32::MAX,
            broken: [0, 1, 0, 1, 0, 0],
            flags: 1 << 9,
        };
        let s = mark_switches(&p);
        assert_eq!(s.marks_on, 0x1FF);
        assert_eq!(s.mode_on, 1);
        let at = |o: u32| s.bytes[(o - FIRST_SWITCH) as usize];
        // Whole (0) cuts with 1, keeping the low bit; broken (1) with 0.
        assert_eq!((at(0xAC0), at(0xAC1), at(0xAC2), at(0xAC3)), (1, 0, 1, 0));
        assert_eq!((at(0xAC8), at(0xAC9)), (1, 0));
        assert_eq!((at(0xAC4), at(0xAC5), at(0xAD0)), (1, 0, 1));
        assert_eq!((at(0xAC6), at(0xAC7)), (1, 0));
        assert_eq!(at(0xACA), 1);
        // (1 + 0) x 4, then the six marks' bytes.
        assert_eq!(s.count, 10);
        let s = mark_switches(&PartState {
            flags: 0,
            lamps_mask: 0,
            ..p
        });
        assert_eq!(s.count, 4);
        assert_eq!(s.bytes[(0xAC8 - FIRST_SWITCH) as usize], 0);
        assert_eq!(marks_bytes(3, true, 1), 0x60);
        assert_eq!(marks_bytes(3, false, 1), 0);
        assert_eq!(marks_bytes(3, true, 0), 0);
        assert_eq!(steering(0x1234_5678), 0x1234);
    }

    #[test]
    fn axles_and_cars() {
        assert_eq!(per_thousand(1000), Some(0x1_0000));
        assert_eq!(per_thousand(-1000), Some(0xFFFF_0000));
        assert_eq!(per_thousand(i32::MAX as i64), None);
        let a = axles([1000, 1000, 1000 - 0xC3, 2000, 3000], 0x100).unwrap();
        assert_eq!(
            a,
            [
                0x1_0000,
                0x2_0000 - 0x100,
                0x1_0000,
                0xFFFF_0000,
                0x3_0000 - 0x100
            ]
        );
        assert_eq!(axles([0, 0, 0, 0, 0x7FFF_FFFF], 0), None);
        assert!(check_part_cars(0, |_| 1).is_err());
        assert!(check_part_cars(9, |_| 1).is_err());
        assert!(check_part_cars(2, |n| n).is_err());
        assert!(check_part_cars(2, |_| 1).is_ok());
        assert!(parts_shown(1) && !parts_shown(0));
    }
}
