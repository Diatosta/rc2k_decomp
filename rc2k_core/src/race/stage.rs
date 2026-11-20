//! The stage's loading: its number in file names, the cars' numbers from the cars' data text,
//! tweaked stages, the course's records grouped by kind, the texture swap, the difficulty and
//! the textures' records.

use crate::Invalid;
use crate::strings::bytes::Bytes;

/// A number's two digits as file names hold them (tens and units, each + '0').
pub fn two_digits(n: u8) -> [u8; 2] {
    [n / 10 + b'0', n % 10 + b'0']
}

/// A digit of the text at `at` (below '0' wraps within the byte); None past 9 or the text.
fn digit<B: Bytes + ?Sized>(text: &B, at: usize) -> Option<u32> {
    let d = text.byte(at)?.wrapping_sub(b'0') as u32;
    (d < 10).then_some(d)
}

/// The `places` digits (2 or 3) at `at` as a number; None if any is not a digit (no read past
/// the first non-digit).
pub(crate) fn parse_digits<B: Bytes + ?Sized>(text: &B, at: usize, places: usize) -> Option<u32> {
    (0..places).try_fold(0u32, |n, k| Some(n * 10 + digit(text, at.wrapping_add(k))?))
}

/// The most numbers a stage's line gives.
pub const MOST_NUMBERS: usize = 0x100;

/// What reading the car numbers gives.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CarNumbers {
    /// The stage's numbers (their sum is the stage's car total).
    pub numbers: Vec<u32>,
    /// The place reached in the text.
    pub place: u32,
    /// The last number parsed, 0 for a failed one; None when none was.
    pub last: Option<u32>,
}

/// In the text's first `len` bytes each 'l' or 'L' followed by two digits (0 for none) is a
/// stage's line; on `stage`'s, from three places on, three-digit numbers four apart until the
/// text's end or a failed number. Refuses more than `MOST_NUMBERS`.
pub fn stage_car_numbers<B: Bytes + ?Sized>(
    text: &B,
    len: u32,
    stage: u32,
) -> Result<CarNumbers, Invalid> {
    let mut place = 0u32;
    let mut last = None;
    let mut numbers = Vec::new();
    let found = 'look: loop {
        loop {
            let c = text.byte(place as usize);
            if c == Some(b'l') || c == Some(b'L') {
                break;
            }
            place += 1;
            if place >= len {
                break 'look false;
            }
        }
        place += 1;
        let n = parse_digits(text, place as usize, 2).unwrap_or(0);
        last = Some(n);
        if n == stage {
            break true;
        }
        place += 1;
        if place >= len {
            break false;
        }
    };
    if found {
        place += 3;
        while place < len {
            match parse_digits(text, place as usize, 3) {
                None => {
                    last = Some(0);
                    break;
                }
                Some(n) => {
                    if numbers.len() == MOST_NUMBERS {
                        return Err(Invalid("more numbers than the table holds"));
                    }
                    last = Some(n);
                    numbers.push(n);
                    place += 4;
                }
            }
        }
    }
    Ok(CarNumbers {
        numbers,
        place,
        last,
    })
}

/// The stages that are tweaked.
pub(crate) const TWEAKED_STAGES: [u32; 3] = [0x15, 0x3E, 0x36];

/// Whether the stage is tweaked.
pub(crate) fn stage_tweaked(stage: u32) -> bool {
    TWEAKED_STAGES.contains(&stage)
}

/// Which stages swap which texture kind for which.
pub(crate) const TEXTURE_SWAPS: [([u32; 3], u32, u32); 4] = [
    ([0x20, 0x22, 0x24], 0x2F82, 0x2FD0),
    ([0x34, 0x36, 0x38], 0x2F91, 0x2FE0),
    ([0x3D, 0x3F, 0x41], 0x2F72, 0x2FD2),
    ([0x3E, 0x40, 0x42], 0x2F71, 0x2FD1),
];
/// The texture kinds searched for the old kind.
pub(crate) const SWAP_KINDS: u32 = 0xFF;
/// The places of the swapped list.
pub(crate) const SWAPPED_PLACES: u32 = 0x40;

/// How `swap_texture` ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Swap {
    /// Not a stage of `TEXTURE_SWAPS`.
    NotThisStage,
    /// The old kind is not among the first 0xFF.
    NotFound,
    /// The kind at `kind` swapped; the new kind found in the list at `at`.
    Listed { kind: u32, at: u32 },
    /// The kind at `kind` swapped; the new kind >> 4 put in the list's free place `at`.
    Added { kind: u32, at: u32 },
    /// The kind at `kind` swapped; the list had no free place.
    Full { kind: u32 },
}

/// The pair tried last (old, new; the last of `TEXTURE_SWAPS` when the stage is none) and
/// what was done. The first kind (of the first 0xFF, via `kinds`) equal to the old one becomes
/// the new one; then, unless the new kind is in the swapped list, it goes >> 4 into the first
/// free place (the test uses the kind before the shift: kept as is).
pub(crate) fn swap_texture(
    stage: u32,
    kinds: impl Fn(u32) -> u32,
    swapped: impl Fn(u32) -> u32,
) -> ((u32, u32), Swap) {
    let mut pair = (0, 0);
    let mut found = false;
    for (stages, old, new) in TEXTURE_SWAPS {
        pair = (old, new);
        if stages.contains(&stage) {
            found = true;
            break;
        }
    }
    if !found {
        return (pair, Swap::NotThisStage);
    }
    let Some(kind) = (0..SWAP_KINDS).find(|&n| kinds(n) == pair.0) else {
        return (pair, Swap::NotFound);
    };
    if let Some(at) = (0..SWAPPED_PLACES).find(|&n| swapped(n) == pair.1) {
        return (pair, Swap::Listed { kind, at });
    }
    match (0..SWAPPED_PLACES).find(|&n| swapped(n) == 0) {
        Some(at) => (pair, Swap::Added { kind, at }),
        None => (pair, Swap::Full { kind }),
    }
}

/// The kinds the course's records come in.
pub(crate) const RECORD_KINDS: usize = 0x100;

/// A course record's kind: its low four bits plus bits 11 ..= 14 times 16.
pub fn record_kind(word: u32) -> u32 {
    (word & 0xF) + (((word >> 11) & 0xF) << 4)
}

/// The record `group_by_kind` looked at last.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Looked {
    /// The record's index.
    pub index: u32,
    /// Whether it was swapped into place.
    pub swapped: bool,
    /// Its first dword.
    pub word: u32,
}

/// What `group_by_kind` makes besides the order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grouped {
    /// Each kind's first record, -1 with none.
    pub first: Vec<u32>,
    /// Each kind's last record, -1 with none.
    pub last: Vec<u32>,
    /// The records placed (all of them).
    pub placed: u32,
    /// The record looked at last.
    pub looked: Option<Looked>,
}

/// Orders `records` by kind, each kind's in the order met, swapping each one found into the
/// next place.
pub fn group_by_kind<R>(records: &mut [R], word: impl Fn(&R) -> u32) -> Grouped {
    let count = records.len() as u32;
    let mut first = vec![0u32; RECORD_KINDS];
    let mut last = vec![0u32; RECORD_KINDS];
    let mut looked = None;
    let mut placed = 0u32;
    for kind in 0..RECORD_KINDS as u32 {
        first[kind as usize] = placed;
        let mut n = placed;
        while n != count {
            let w = word(&records[n as usize]);
            let swapped = record_kind(w) == kind;
            if swapped {
                records.swap(placed as usize, n as usize);
                placed += 1;
            }
            looked = Some(Looked {
                index: n,
                swapped,
                word: w,
            });
            n += 1;
        }
        let end = placed.wrapping_sub(1);
        last[kind as usize] = end;
        if (end.wrapping_sub(first[kind as usize]) as i32) < 0 {
            first[kind as usize] = u32::MAX;
            last[kind as usize] = u32::MAX;
        }
    }
    Grouped {
        first,
        last,
        placed,
        looked,
    }
}

/// Stages past the rallies' (from here up nothing is timed).
pub const UNTIMED_STAGES: u32 = 0x4F;

/// The rally of a stage: its first stage and its level (0 ..= 5); none from `UNTIMED_STAGES`
/// up (stages below 0x15 go with the first, their index wrapping below the table).
pub fn rally_of(stage: u32) -> Option<(u32, u32)> {
    match stage {
        _ if stage < 0x1D => Some((0x15, 0)),
        _ if stage < 0x27 => Some((0x1F, 3)),
        _ if stage < 0x31 => Some((0x29, 5)),
        _ if stage < 0x3B => Some((0x33, 1)),
        _ if stage < 0x45 => Some((0x3D, 4)),
        _ if stage < UNTIMED_STAGES => Some((0x47, 2)),
        _ => None,
    }
}

/// The difficulty index: the stage's place in its rally plus 6 x the rally's level.
pub fn difficulty(stage: u32, (first, level): (u32, u32)) -> u32 {
    stage.wrapping_sub(first).wrapping_add(6 * level)
}

/// The kind whose factor the cars' times take: the first car's held to 0 ..= 0x16.
pub fn timed_kind(kind: i32) -> u32 {
    let kind = kind.max(0);
    if kind.wrapping_sub(0x16) >= 0 {
        0x16
    } else {
        kind as u32
    }
}

/// A texture's record from its kind and the course info's record of the kind (+kind x 0x80);
/// bytes not written are left.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextureInfo {
    /// +0 ..= +0xC: the course record's first four dwords.
    pub first: [u32; 4],
    /// +0x10, +0x14, +0x18: the course record's +0x20, +0x10, +0x14.
    pub values: [u32; 3],
    /// +0x1C: the course record's byte +0x18.
    pub byte: u8,
    /// +0x20: the kind >> 4.
    pub group: u32,
    /// +0x24: the kind & 0xF.
    pub row: u32,
    /// +0x28: the group with bit 31 set.
    pub marked: u32,
    /// +0x2C ..= +0x2E: the course record's bytes +0x24 ..= +0x26.
    pub bytes: [u8; 3],
}

/// A texture's record from its kind (`dword(offset)`, `byte(offset)` read the kind's record in
/// the course info, asked in a fixed order).
pub fn texture_info(
    kind: u32,
    mut dword: impl FnMut(u32) -> u32,
    mut byte: impl FnMut(u32) -> u8,
) -> TextureInfo {
    let group = kind >> 4;
    let values = [dword(0x20), dword(0x10), dword(0x14)];
    let b = byte(0x18);
    let bytes = [byte(0x24), byte(0x25), byte(0x26)];
    let first = [dword(0), dword(4), dword(8), dword(0xC)];
    TextureInfo {
        first,
        values,
        byte: b,
        group,
        row: kind & 0xF,
        marked: group | 0x8000_0000,
        bytes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn difficulties() {
        assert_eq!(rally_of(0x15), Some((0x15, 0)));
        assert_eq!(rally_of(0x26), Some((0x1F, 3)));
        assert_eq!(rally_of(0x4E), Some((0x47, 2)));
        assert_eq!(rally_of(0x4F), None);
        assert_eq!(difficulty(0x28, (0x1F, 3)), 9 + 18);
        assert_eq!(difficulty(0x10, (0x15, 0)), (-5i32) as u32);
        assert_eq!(timed_kind(-3), 0);
        assert_eq!(timed_kind(0x15), 0x15);
        assert_eq!(timed_kind(0x40), 0x16);
    }

    #[test]
    fn digits() {
        assert_eq!(two_digits(7), *b"07");
        assert_eq!(two_digits(255), [b'0' + 25, b'5']);
        let text = b"x12/9a345";
        assert_eq!(parse_digits(&text[..], 1, 2), Some(12));
        assert_eq!(parse_digits(&text[..], 2, 2), None);
        assert_eq!(parse_digits(&text[..], 6, 3), Some(345));
        assert_eq!(parse_digits(&text[..], 7, 3), None);
    }

    #[test]
    fn car_numbers() {
        let text = b"L01 100 200\r\nl02 300 400 x\r\n";
        let r = stage_car_numbers(&text[..], text.len() as u32, 2).unwrap();
        assert_eq!(r.numbers, [300, 400]);
        assert_eq!((r.place, r.last), (25, Some(0)));
        let r = stage_car_numbers(&text[..], text.len() as u32, 9).unwrap();
        assert!(r.numbers.is_empty());
        assert_eq!(r.last, Some(2));
        let r = stage_car_numbers(&b"abc"[..], 3, 1).unwrap();
        assert_eq!((r.place, r.last), (3, None));
        assert!(stage_tweaked(0x3E) && !stage_tweaked(0x3D));
    }

    #[test]
    fn texture_swap() {
        let kinds = |n: u32| if n == 5 { 0x2F82 } else { 0x1000 + n };
        assert_eq!(
            swap_texture(0x10, kinds, |_| unreachable!()),
            ((0x2F71, 0x2FD1), Swap::NotThisStage)
        );
        assert_eq!(
            swap_texture(0x22, |_| 0, |_| unreachable!()),
            ((0x2F82, 0x2FD0), Swap::NotFound)
        );
        let list = |n: u32| if n < 3 { 0x77 + n } else { 0 };
        assert_eq!(
            swap_texture(0x20, kinds, list).1,
            Swap::Added { kind: 5, at: 3 }
        );
        assert_eq!(
            swap_texture(0x20, kinds, |n| if n == 9 { 0x2FD0 } else { 1 }).1,
            Swap::Listed { kind: 5, at: 9 }
        );
        assert_eq!(swap_texture(0x20, kinds, |_| 1).1, Swap::Full { kind: 5 });
    }

    #[test]
    fn grouping() {
        assert_eq!(record_kind(0x7802 | 0x30), 0xF2);
        let mut r = [3u32, 1, 3, 0];
        let g = group_by_kind(&mut r, |&w| w);
        assert_eq!(r, [0, 1, 3, 3]);
        assert_eq!((g.first[0], g.last[0]), (0, 0));
        assert_eq!((g.first[3], g.last[3]), (2, 3));
        assert_eq!((g.first[2], g.last[2]), (u32::MAX, u32::MAX));
        assert_eq!(g.placed, 4);
        assert_eq!(
            g.looked,
            Some(Looked {
                index: 3,
                swapped: true,
                word: 3
            })
        );
        let mut none: [u32; 0] = [];
        assert_eq!(group_by_kind(&mut none, |&w| w).looked, None);
    }

    #[test]
    fn texture_records() {
        let t = texture_info(0x5A, |o| 0x1000 + o, |o| o as u8);
        assert_eq!(t.first, [0x1000, 0x1004, 0x1008, 0x100C]);
        assert_eq!(t.values, [0x1020, 0x1010, 0x1014]);
        assert_eq!((t.byte, t.bytes), (0x18, [0x24, 0x25, 0x26]));
        assert_eq!((t.group, t.row, t.marked), (5, 0xA, 0x8000_0005));
    }
}
