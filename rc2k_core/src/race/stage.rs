//! The stage's loading: its number in the file names, the cars' numbers from the cars'
//! data text, the stages tweaked, and the course's records grouped by kind (converted
//! 2026-10-21 from `race::set_cdat_number` 0x4456DD, `set_course_number` 0x43944F,
//! `parse_2_digits` 0x437A25, `parse_3_digits` 0x4379EB, `read_stage_car_numbers`
//! 0x437971, `tweak_stage` 0x439AAB and `group_records_by_kind` 0x421E42; the texture
//! swap converted 2026-10-21 from `race::swap_stage_texture` 0x439B03).

use crate::Invalid;
use crate::strings::bytes::Bytes;

/// `race::set_cdat_number` / `race::set_course_number`: a number's two digits as the
/// file names hold them (`div 10` of the low byte, each + '0').
pub fn two_digits(n: u8) -> [u8; 2] {
    [n / 10 + b'0', n % 10 + b'0']
}

/// A digit of the text at `at` (`movzx`, `sub` of the low byte: below '0' wraps within
/// the byte), None past 9 or past the text.
fn digit<B: Bytes + ?Sized>(text: &B, at: usize) -> Option<u32> {
    let d = text.byte(at)?.wrapping_sub(b'0') as u32;
    (d < 10).then_some(d)
}

/// `race::parse_2_digits` (`places` 2) / `race::parse_3_digits` (3): the digits at `at`
/// as a number, each tested in order (none read past the first that is not a digit);
/// None with any not a digit.
pub fn parse_digits<B: Bytes + ?Sized>(text: &B, at: usize, places: usize) -> Option<u32> {
    (0..places).try_fold(0u32, |n, k| Some(n * 10 + digit(text, at.wrapping_add(k))?))
}

/// The most numbers a stage's line gives (`g_stageCarNumbers`).
pub const MOST_NUMBERS: usize = 0x100;

/// What `read_stage_car_numbers` reads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CarNumbers {
    /// The stage's numbers (`g_stageCarNumbers`; their sum is `g_stageCarTotal`).
    pub numbers: Vec<u32>,
    /// The place reached (ecx).
    pub place: u32,
    /// The last number parsed, 0 for a failed one (eax); None when none was.
    pub last: Option<u32>,
}

/// `race::read_stage_car_numbers`: in the text's first `len` bytes each 'l' or 'L'
/// followed by two digits (0 for none) is a stage's line; on `stage`'s, from three places
/// on, three-digit numbers four apart, up to the text's end or a failed number. Refuses
/// more than `MOST_NUMBERS`.
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

/// The stages `race::tweak_stage` tweaks (three dwords from `dword_60A564` 0xFF).
pub const TWEAKED_STAGES: [u32; 3] = [0x15, 0x3E, 0x36];

/// `race::tweak_stage`: whether the stage is tweaked.
pub fn stage_tweaked(stage: u32) -> bool {
    TWEAKED_STAGES.contains(&stage)
}

/// Which stages swap which texture kind for which (`race::swap_stage_texture`).
pub const TEXTURE_SWAPS: [([u32; 3], u32, u32); 4] = [
    ([0x20, 0x22, 0x24], 0x2F82, 0x2FD0),
    ([0x34, 0x36, 0x38], 0x2F91, 0x2FE0),
    ([0x3D, 0x3F, 0x41], 0x2F72, 0x2FD2),
    ([0x3E, 0x40, 0x42], 0x2F71, 0x2FD1),
];
/// The texture kinds searched (`g_textureKinds`).
pub const SWAP_KINDS: u32 = 0xFF;
/// The places of the swapped list (`dword_60A67C`).
pub const SWAPPED_PLACES: u32 = 0x40;

/// How `swap_texture` ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Swap {
    /// Not a stage of `TEXTURE_SWAPS`.
    NotThisStage,
    /// The old kind not among the first 0xFF.
    NotFound,
    /// The kind at `kind` swapped; the new kind found in the list at `at`.
    Listed { kind: u32, at: u32 },
    /// The kind at `kind` swapped; the new kind >> 4 put in the list's free place `at`.
    Added { kind: u32, at: u32 },
    /// The kind at `kind` swapped; the list had no free place.
    Full { kind: u32 },
}

/// `race::swap_stage_texture`: the pair tried last (old, new: the last of `TEXTURE_SWAPS`
/// when the stage is none of them) and what was done. The first texture kind (of the
/// first 0xFF, through `kinds`) that is the old one becomes the new one; then, unless the
/// new kind is in the swapped list (`swapped`, each place asked in order), it goes >> 4
/// into the list's first free place (the test made on the kind before the shift: section
/// 16).
pub fn swap_texture(
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

/// The kinds the course's records come in (`g_kindFirstRecords`, `g_kindLastRecords`).
pub const RECORD_KINDS: usize = 0x100;

/// A course record's kind from its first dword: its low four bits, + bits 11 .. 14 times
/// 16.
pub fn record_kind(word: u32) -> u32 {
    (word & 0xF) + (((word >> 11) & 0xF) << 4)
}

/// The record `group_by_kind` looked at last.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Looked {
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
    pub looked: Option<Looked>,
}

/// `race::group_records_by_kind`: `records` put in order of kind, each kind's in the
/// order met, by swapping each one found (from the next place on) into the next place.
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
