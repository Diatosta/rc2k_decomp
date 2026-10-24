//! `marks::init_marks`' steps 8 and 9 (IDA, 2026-10-16): `marks::set_kind_values_1`
//! (0x4C7322) and `marks::set_kind_values_2` (0x4C7FA4), the same code on two sets of
//! tables: the values of the marks' kind (`g_marksKind`) from tables of 0x22 floats and
//! of 0x22 three-float entries.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_read, game_write, rva};
use crate::math::x87;

pub const KIND_RVA: usize = rva(0x769232); // g_marksKind
pub const KINDS: u32 = 0x22;

/// One set's addresses (RVAs), in the order the code uses them.
pub struct KindTables {
    /// Tables of 0x22 dwords: `count` (copied), `size`, `divisor`, `low`, `high`, `start`,
    /// `middle`, `end`, `spread`, `extra`.
    pub tables: [usize; 10],
    /// Tables of 0x22 three-float entries: `to`, `from`, `copied`.
    pub triples: [usize; 3],
    /// Constants: 0x7C, 0x80 .. 0xB0 after the set's base (14 floats).
    pub constants: [usize; 14],
    /// Out: `count`, `size`, `ratio`, `high`, `low` (in that order of writing), `start`,
    /// `step`, `middle`, `slope`, `spread`, `extra`.
    pub outs: [usize; 11],
    /// Out: the place's three floats (z, y, x as written), then the copied three, a 0,
    /// and the two differences.
    pub place: [usize; 3],
    pub copied: usize,
    /// `divisor` copied here too (the first set only).
    pub divisor_copy: Option<usize>,
}

const fn r(a: usize) -> usize {
    rva(a)
}

pub const FIRST: KindTables = KindTables {
    tables: [
        r(0x933B40),
        r(0x9339A8),
        r(0x933A30),
        r(0x933DE8),
        r(0x933D60),
        r(0x933BC8),
        r(0x933C50),
        r(0x933CD8),
        r(0x933AB8),
        r(0x933920),
    ],
    triples: [r(0x933788), r(0x9335F0), r(0x933458)],
    constants: [
        r(0x933E7C),
        r(0x933E80),
        r(0x933E84),
        r(0x933E88),
        r(0x933E8C),
        r(0x933E90),
        r(0x933E94),
        r(0x933E98),
        r(0x933E9C),
        r(0x933EA0),
        r(0x933EA4),
        r(0x933EA8),
        r(0x933EAC),
        r(0x933EB0),
    ],
    outs: [
        r(0x933248),
        r(0x933250),
        r(0x93324C),
        r(0x93325C),
        r(0x933260),
        r(0x933254),
        r(0x933258),
        r(0x933264),
        r(0x933268),
        r(0x93326C),
        r(0x933E70),
    ],
    place: [r(0x9331B4), r(0x9331B8), r(0x9331BC)],
    copied: r(0x933200),
    divisor_copy: Some(r(0x933E74)),
};

pub const SECOND: KindTables = KindTables {
    tables: [
        r(0x934848),
        r(0x9346B0),
        r(0x934738),
        r(0x934AF0),
        r(0x934A68),
        r(0x9348D0),
        r(0x934958),
        r(0x9349E0),
        r(0x9347C0),
        r(0x934628),
    ],
    triples: [r(0x934490), r(0x9342F8), r(0x934160)],
    constants: [
        r(0x934B7C),
        r(0x934B80),
        r(0x934B84),
        r(0x934B88),
        r(0x934B8C),
        r(0x934B90),
        r(0x934B94),
        r(0x934B98),
        r(0x934B9C),
        r(0x934BA0),
        r(0x934BA4),
        r(0x934BA8),
        r(0x934BAC),
        r(0x934BB0),
    ],
    outs: [
        r(0x933F64),
        r(0x933F6C),
        r(0x933F68),
        r(0x933F78),
        r(0x933F7C),
        r(0x933F70),
        r(0x933F74),
        r(0x933F80),
        r(0x933F84),
        r(0x933F88),
        r(0x934B78),
    ],
    place: [r(0x933ED0), r(0x933ED4), r(0x933ED8)],
    copied: r(0x933F1C),
    divisor_copy: None,
};

/// The kind's values of one set. Refuses a kind past the tables (0x22).
pub unsafe fn set_kind_values(k: &KindTables) -> Result<(), Refused> {
    unsafe {
        let kind: u32 = game_read(KIND_RVA);
        if kind >= KINDS {
            return Err(BadInput("the marks' kind is past the tables (0x22)").into());
        }
        let t = |n: usize| -> u32 { game_read(k.tables[n] + 4 * kind as usize) };
        let c = |n: usize| -> u32 { game_read(k.constants[n]) };
        let put = |n: usize, v: u32| game_write(k.outs[n], v);
        let [
            count,
            size,
            divisor,
            low,
            high,
            start,
            middle,
            end,
            spread,
            extra,
        ] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9].map(t);
        put(0, count);
        let (scaled, ratio) = x87::mul_and_div(size, c(0), divisor);
        put(1, scaled);
        put(2, ratio);
        put(3, low);
        put(4, high);
        put(5, x87::mul(start, c(1)));
        put(6, x87::sub_div_mul(middle, start, low, c(2)));
        put(7, x87::mul(middle, c(3)));
        put(8, x87::diff_ratio_mul(end, middle, high, low, c(4)));
        put(9, x87::mul(spread, c(5)));
        put(10, x87::mul(extra, c(6)));
        if let Some(copy) = k.divisor_copy {
            game_write(copy, divisor);
        }
        let row = kind as usize * 12;
        let triple = |n: usize, part: usize| -> u32 { game_read(k.triples[n] + row + 4 * part) };
        let lerp = |part: usize, less: Option<u32>| {
            x87::lerp_scaled_less(
                triple(0, part),
                triple(1, part),
                c(7 + 2 * part),
                c(8 + 2 * part),
                less,
            )
        };
        game_write(k.place[0], lerp(2, None));
        game_write(k.place[1], lerp(1, Some(c(13))));
        game_write(k.place[2], lerp(0, None));
        game_write(k.copied, [triple(2, 0), triple(2, 1), triple(2, 2), 0]);
        game_write(k.copied + 16, x87::sub(triple(0, 1), triple(1, 1)));
        game_write(k.copied + 20, x87::negated_sub(triple(0, 0), triple(1, 0)));
    }
    Ok(())
}
