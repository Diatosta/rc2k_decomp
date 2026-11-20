//! The work car's loose parts: parts are bits of one word; a part that comes off goes into one
//! of the eight race slots and flies with the car's motion.

use super::car_word::car_word_bits;
use super::x87::below;

/// The race slots.
pub const SLOTS: usize = 8;

/// The highest part number (the parts' tables hold 0x100).
pub const LAST_PART: u32 = 0xFF;

/// A damage figure that never breaks its part.
pub const UNBROKEN: u32 = 0x8000_0000;

/// The slot chosen for a part that comes off.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlotChoice {
    /// The slot's number.
    pub slot: usize,
    /// The slot number the search ends at, 8 when none was empty.
    pub search_end: u32,
    /// With no slot empty: the lowest age (a clock).
    pub lowest: Option<u32>,
}

/// The first empty slot (asked in order), else the one with the lowest age (unsigned, the
/// first of equals; asked only when none is empty). `None` when every age is -1 (callers
/// refuse it).
pub fn choose_slot(part: impl Fn(usize) -> u32, age: impl Fn(usize) -> u32) -> Option<SlotChoice> {
    if let Some(slot) = (0..SLOTS).find(|&c| part(c) == 0) {
        return Some(SlotChoice {
            slot,
            search_end: slot as u32,
            lowest: None,
        });
    }
    let mut lowest = u32::MAX;
    let mut best = None;
    for c in 0..SLOTS {
        let a = age(c);
        if a < lowest {
            lowest = a;
            best = Some(c);
        }
    }
    best.map(|slot| SlotChoice {
        slot,
        search_end: SLOTS as u32,
        lowest: Some(lowest),
    })
}

/// A bit of the parts' word taken (bit mod 32): (the bit's mask, the word with it cleared
/// when it was set).
pub fn take_bit(bits: u32, bit: u32) -> (u32, Option<u32>) {
    let mask = 1u32.wrapping_shl(bit & 0x1F);
    (mask, (bits & mask != 0).then_some(bits ^ mask))
}

/// The part a car's bit names: 0x50 + 0x14 x the car + the bit; `None` past `LAST_PART`.
pub fn part_number(car: u32, bit: u32) -> Option<u32> {
    let part = 0x14u32
        .wrapping_mul(car)
        .wrapping_add(0x50)
        .wrapping_add(bit);
    (part <= LAST_PART).then_some(part)
}

/// The part bits of the broken-part check, by damage figure.
pub const BROKEN_BITS: [u32; 7] = [9, 0xD, 0xE, 0x10, 0x11, 0xC, 0xF];

/// Whether damage figure `k` is checked: the first only when `switch` (asked only for it).
pub fn broken_checked(k: usize, switch: impl FnOnce() -> bool) -> bool {
    k != 0 || switch()
}

/// Whether a part breaks: its damage not `UNBROKEN` and not below its limit (unsigned); the
/// limit is asked only past the first test.
pub fn breaks(damage: u32, limit: impl FnOnce() -> u32) -> bool {
    damage != UNBROKEN && damage >= limit()
}

/// One of the four hit tests: the word's rotation and the part's bit.
pub type HitTest = (u8, u32);

/// The tests for a part that drops when hit.
pub const DROP_TESTS: [HitTest; 4] = [(5, 5), (9, 6), (0xE, 7), (0x19, 8)];

/// The tests for the sound of a part hit.
pub const SOUND_TESTS: [HitTest; 4] = [(5, 1), (9, 2), (0xE, 3), (0x19, 4)];

/// How the hit tests end.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hit {
    /// No test passed.
    None,
    /// Test `k` passed with its part's bit already clear.
    Gone(usize),
    /// Test `k` passed with its part's bit set: the part's step runs and the timer is set.
    Act(usize),
}

/// The hit tests: (the last test run, how they end). A test passes when the car's word
/// rotated by its rotation has its low 16 bits 0 and its timer is not below the limit
/// (unordered counts as below); the inputs are asked only where needed.
pub fn hit_tests(
    tests: &[HitTest; 4],
    word: u32,
    timer: impl Fn(usize) -> f32,
    limit: impl Fn() -> f32,
    bits: impl FnOnce() -> u32,
) -> (usize, Hit) {
    for (k, &(rotation, bit)) in tests.iter().enumerate() {
        if !car_word_bits(word, rotation).carry {
            continue;
        }
        if below(timer(k) as f64, limit() as f64) {
            continue;
        }
        if bits() >> bit & 1 == 0 {
            return (k, Hit::Gone(k));
        }
        return (k, Hit::Act(k));
    }
    (tests.len() - 1, Hit::None)
}

/// The hit-check figure: 1, -1 with bits 5, 6, 1, 2 all set, 0 with bits 7, 8, 3, 4 also set;
/// the hit tests run only at 0.
pub fn parts_left(bits: u32) -> u32 {
    let all = |b: &[u32]| b.iter().all(|&b| bits >> b & 1 != 0);
    if !all(&[5, 6, 1, 2]) {
        1
    } else if !all(&[7, 8, 3, 4]) {
        u32::MAX
    } else {
        0
    }
}

/// The car kinds with a hit-sound table.
pub const SOUND_KINDS: u32 = 0x22;

/// Whether a car kind has sounds for its hit parts: below `SOUND_KINDS` and its entry not 0.
pub fn has_hit_sounds(kind: u32, entry: impl FnOnce(u32) -> u32) -> bool {
    kind < SOUND_KINDS && entry(kind) != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slots() {
        let c = choose_slot(|k| [1, 2, 0, 0, 5, 6, 7, 8][k], |_| unreachable!()).unwrap();
        assert_eq!((c.slot, c.search_end, c.lowest), (2, 2, None));
        let ages = [9u32, 4, 7, 4, 10, 11, 12, 13];
        let c = choose_slot(|_| 1, |k| ages[k]).unwrap();
        assert_eq!((c.slot, c.search_end, c.lowest), (1, 8, Some(4)));
        assert_eq!(choose_slot(|_| 1, |_| u32::MAX), None);
    }

    #[test]
    fn bits_and_parts() {
        assert_eq!(take_bit(0b1010, 1), (2, Some(0b1000)));
        assert_eq!(take_bit(0b1010, 2), (4, None));
        assert_eq!(take_bit(1, 32), (1, Some(0)));
        assert_eq!(part_number(0, 9), Some(0x59));
        assert_eq!(part_number(7, 0x11), Some(0x50 + 0x8C + 0x11));
        assert_eq!(part_number(8, 0x11), None);
        assert!(!broken_checked(0, || false));
        assert!(broken_checked(3, || unreachable!()));
        assert!(breaks(5, || 5));
        assert!(!breaks(4, || 5));
        assert!(!breaks(UNBROKEN, || unreachable!()));
    }

    #[test]
    fn hits() {
        let t = |k: usize| [50.0f32, 100.0, f32::NAN, 300.0][k];
        // Word 0: every test's word part is 0.
        assert_eq!(
            hit_tests(&DROP_TESTS, 0, t, || 100.0, || 0x1FF),
            (1, Hit::Act(1))
        );
        assert_eq!(
            hit_tests(&DROP_TESTS, 0, t, || 100.0, || 0x1BF),
            (1, Hit::Gone(1))
        );
        assert_eq!(
            hit_tests(&DROP_TESTS, u32::MAX, t, || 100.0, || 0),
            (3, Hit::None)
        );
        assert_eq!(parts_left(0), 1);
        assert_eq!(parts_left(0x66), u32::MAX);
        assert_eq!(parts_left(0x1FE), 0);
        assert!(has_hit_sounds(3, |_| 1));
        assert!(!has_hit_sounds(SOUND_KINDS, |_| unreachable!()));
    }
}
