//! A list of the test's for the self-tests of the installer's list helpers: eight items
//! whose kinds, marks and texts a case sets, with their choices, values and edits.
//!
//! Memory: the list at +0 (0x18 + 8 x 0x10), the items' choice tables at +0x100 (0x20
//! each), chosen bytes at +0x200, edit records at +0x210 (8 each), edit texts at +0x300
//! (0x40 each), the backup and spare after.

use crate::install::installer::{rb, wb, wd, ww};
use crate::service::harness::Region;

pub const LEN: usize = 0x600;
pub const ITEMS: u32 = 8;

pub struct TestList(Box<[u8; LEN]>);

impl Default for TestList {
    fn default() -> Self {
        Self::new()
    }
}

impl TestList {
    pub fn new() -> Self {
        Self(Box::new([0xAA; LEN]))
    }

    pub fn base(&self) -> u32 {
        self.0.as_ptr() as u32
    }

    pub fn item(&self, n: u32) -> u32 {
        self.base() + 0x18 + 0x10 * n
    }

    pub fn region(&self) -> Region {
        (self.base(), LEN)
    }

    /// Everything 0xAA, then the list: `flags`, `kinds` (one byte a item: kind and marks),
    /// each item's +1 `next` (or its choices for kind 2), text id 0x100 + n; kind 2 with
    /// choices whose marks are `marks` (bit n: 0x40) and chosen byte `chosen`; kind 3 a
    /// value `chosen` in 10 .. 200; kind 4 an edit of `edit` flags, at most 0x20 bytes,
    /// font kind 1, width `width`, holding `text`.
    #[allow(clippy::too_many_arguments)] // one test list's whole description
    pub fn set(
        &self,
        flags: u8,
        kinds: &[u8],
        next: u8,
        marks: u32,
        chosen: u8,
        edit: u8,
        width: u16,
        text: &[u8],
    ) {
        let b = self.base();
        for n in 0..LEN as u32 {
            wb(b + n, 0xAA);
        }
        wb(b, flags);
        wb(b + 1, kinds.len() as u8);
        ww(b + 2, 0x99);
        wd(b + 4, 0);
        for n in 8..0x18 {
            wb(b + n, 0);
        }
        for (n, &kind) in kinds.iter().enumerate() {
            let n = n as u32;
            let item = self.item(n);
            wb(item, kind);
            wb(item + 1, next);
            ww(item + 2, 0x100 + n as u16);
            wd(item + 4, 0);
            match kind & 0xF {
                2 => {
                    let table = b + 0x100 + 0x20 * n;
                    wd(item + 8, b + 0x200 + n);
                    wd(item + 0xC, table);
                    wb(b + 0x200 + n, chosen);
                    for c in 0..0x10u32 {
                        ww(table + 2 * c, 0x200 + c as u16);
                        let mark = if marks >> c & 1 != 0 { 0x40 } else { 0 };
                        wb(table + 2 * c + 1, mark | 2);
                    }
                }
                3 => {
                    wd(item + 8, b + 0x200 + n);
                    wb(item + 0xC, 10);
                    wb(item + 0xD, 200);
                    wb(item + 0xE, 0);
                    wb(item + 0xF, 0);
                    wb(b + 0x200 + n, chosen);
                }
                4 => {
                    let record = b + 0x210 + 8 * n;
                    let at = b + 0x300 + 0x40 * (n % 4);
                    wd(item + 8, at);
                    wd(item + 0xC, record);
                    wb(record, edit);
                    wb(record + 1, 0x20);
                    ww(record + 2, 0x4E0);
                    wb(record + 4, 1);
                    wb(record + 5, 0);
                    ww(record + 6, width);
                    for (k, &c) in text.iter().enumerate() {
                        wb(at + k as u32, c);
                    }
                    wb(at + text.len() as u32, 0);
                }
                _ => {
                    wd(item + 8, 0);
                    wd(item + 0xC, 0);
                }
            }
        }
    }

    /// Sets an item's +8 (an action, say).
    pub fn set_action(&self, n: u32, action: u32) {
        wd(self.item(n) + 8, action);
    }

    pub fn byte(&self, offset: u32) -> u8 {
        rb(self.base() + offset)
    }
}
