//! The self-tests of the installer's item helpers (`install::choice_back` .. `key_number`):
//! an item of the test's (kind, choices, the chosen byte, the choices' marks or the value's
//! bounds) in every combination that matters, the original against the Rust version.

use crate::errors::Refused;
use crate::framework::{Regs, SelfTest};
use crate::install::installer::{REPEATS, ZF, p, rb, wb, wd};
use crate::race::viewer::{CF, Saved, check_original, dword};
use crate::screens::harness::CallOriginal;

/// The test's memory: the item (0x10), the chosen byte (+0x10), the choices (+0x20, two
/// bytes each, the marks in the second).
pub struct Item(Box<[u8; 0x240]>);

impl Item {
    pub fn new() -> Self {
        Self(Box::new([0xAA; 0x240]))
    }

    pub fn base(&self) -> u32 {
        self.0.as_ptr() as u32
    }

    /// kind, choices; the chosen byte; the choices marked (bit n: choice n gets 0x40);
    /// for kind 3 the bounds (min, max) in +0xC, +0xD instead of a table.
    pub fn set(&self, kind: u8, count: u8, chosen: u8, marked: u64, bounds: Option<(u8, u8)>) {
        let b = self.base();
        for n in 0..0x240 {
            wb(b + n, 0xAA);
        }
        wb(b, kind);
        wb(b + 1, count);
        wd(b + 8, b + 0x10);
        wb(b + 0x10, chosen);
        match bounds {
            Some((lo, hi)) => {
                wb(b + 0xC, lo);
                wb(b + 0xD, hi);
                wb(b + 0xE, 0);
                wb(b + 0xF, 0);
            }
            None => wd(b + 0xC, b + 0x20),
        }
        for n in 0..0x100u32 {
            let mark = if n < 64 && marked >> n & 1 != 0 {
                0x40
            } else {
                0
            };
            wb(b + 0x20 + 2 * n + 1, mark);
        }
    }

    pub fn chosen(&self) -> u8 {
        rb(self.base() + 0x10)
    }
}

impl Default for Item {
    fn default() -> Self {
        Self::new()
    }
}

/// The cases every helper is run on: (what, kind, choices, chosen, marked, bounds,
/// repeats, al).
#[allow(clippy::type_complexity)] // the cases' description
pub fn cases() -> Vec<(String, u8, u8, u8, u64, Option<(u8, u8)>, u32, u8)> {
    let mut all = Vec::new();
    for kind in [2u8, 3, 0x22, 0x23, 0x12, 4, 0x42] {
        for (count, chosen, marked) in [
            (5u8, 0u8, 0u64),
            (5, 4, 0),
            (5, 2, 0b00110),
            (5, 2, 0b01010),
            (5, 0, 0b11110),
            (5, 0x7F, 0b11111),
            (5, 7, 0b00101),
            (1, 0, 0),
            (9, 0x80, 0b1_1111_1110),
            (9, 0x81, 0b1_0000_0000),
        ] {
            for (bounds, repeats) in [
                (Some((10u8, 200u8)), 1u32),
                (Some((10, 200)), 30),
                (Some((0, 255)), 300),
                (Some((50, 20)), 7),
            ] {
                if kind & 0xF == 2 && bounds != Some((10, 200)) {
                    continue; // the bounds only matter for kind 3
                }
                for al in [2u8, 5, 10] {
                    let bounds = if kind & 0xF == 3 { bounds } else { None };
                    all.push((
                        format!(
                            "kind {kind:#x}, {count} choices, chosen {chosen:#x}, marked \
                             {marked:#b}, bounds {bounds:?}, repeats {repeats}, al {al}"
                        ),
                        kind,
                        count,
                        chosen,
                        marked,
                        bounds,
                        repeats,
                        al,
                    ));
                }
            }
        }
    }
    all
}

/// Items with a table of choices (kind 2 in the low nibble), for the helpers that read one.
pub fn with_table(kind: u8) -> bool {
    kind & 0xF == 2
}

/// Items with a value's bounds (kind 3).
pub fn with_bounds(kind: u8) -> bool {
    kind & 0xF == 3
}

pub fn any(_: u8) -> bool {
    true
}

/// Runs `cases` (the kinds `wanted` lets through) on one helper: the item at esi, al given;
/// the registers, ZF and CF, the item's memory compared. A case the Rust version refuses
/// (its original never leaves it) is not run.
pub fn run(
    t: &mut SelfTest,
    call_original: CallOriginal,
    rust: &dyn Fn(&Regs) -> Result<Regs, Refused>,
    wanted: fn(u8) -> bool,
) {
    let item = Item::new();
    let saved = Saved::new(&[dword(REPEATS)]);
    for (what, kind, count, chosen, marked, bounds, repeats, al) in cases() {
        if !wanted(kind) {
            continue;
        }
        item.set(kind, count, chosen, marked, bounds);
        let before = Regs {
            eax: 0x1234_5600 | al as u32,
            // what `install::key_number` gives the number helpers
            ecx: (al as u32).wrapping_sub(2),
            esi: item.base(),
            ..Regs::SENTINEL
        };
        p(REPEATS, repeats);
        if rust(&before).is_err() {
            continue;
        }
        item.set(kind, count, chosen, marked, bounds);
        let regions = [(item.base(), 0x240usize)];
        check_original(
            t,
            &what,
            call_original,
            before,
            &regions,
            ZF | CF,
            &|| {
                item.set(kind, count, chosen, marked, bounds);
                p(REPEATS, repeats);
            },
            rust,
        );
    }
    drop(saved);
}
