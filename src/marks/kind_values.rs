//! `marks::init_marks`' steps 8 and 9 (IDA, 2026-10-16): `marks::set_kind_values_1`
//! (0x4C7322) and `marks::set_kind_values_2` (0x4C7FA4), the same code on two sets of
//! tables: the values of the marks' kind (`g_marksKind`) from tables of 0x22 floats and
//! of 0x22 three-float entries.

use crate::errors::{BadInput, Refused};
use crate::marks::store::{Slot, game_read, game_write};
use crate::math::x87;

pub const KIND_RVA: Slot = Slot::at(0x769232); // g_marksKind
pub const KINDS: u32 = 0x22;

/// One set's addresses (RVAs), in the order the code uses them.
pub struct KindTables {
    /// Tables of 0x22 dwords: `count` (copied), `size`, `divisor`, `low`, `high`, `start`,
    /// `middle`, `end`, `spread`, `extra`.
    pub tables: [Slot; 10],
    /// Tables of 0x22 three-float entries: `to`, `from`, `copied`.
    pub triples: [Slot; 3],
    /// Constants: 0x7C, 0x80 .. 0xB0 after the set's base (14 floats).
    pub constants: [Slot; 14],
    /// Out: `count`, `size`, `ratio`, `high`, `low` (in that order of writing), `start`,
    /// `step`, `middle`, `slope`, `spread`, `extra`.
    pub outs: [Slot; 11],
    /// Out: the place's three floats (z, y, x as written), then the copied three, a 0,
    /// and the two differences.
    pub place: [Slot; 3],
    pub copied: Slot,
    /// `divisor` copied here too (the first set only).
    pub divisor_copy: Option<Slot>,
}

pub const FIRST: KindTables = KindTables {
    tables: [
        Slot::at(0x933B40),
        Slot::at(0x9339A8),
        Slot::at(0x933A30),
        Slot::at(0x933DE8),
        Slot::at(0x933D60),
        Slot::at(0x933BC8),
        Slot::at(0x933C50),
        Slot::at(0x933CD8),
        Slot::at(0x933AB8),
        Slot::at(0x933920),
    ],
    triples: [Slot::at(0x933788), Slot::at(0x9335F0), Slot::at(0x933458)],
    constants: [
        Slot::at(0x933E7C),
        Slot::at(0x933E80),
        Slot::at(0x933E84),
        Slot::at(0x933E88),
        Slot::at(0x933E8C),
        Slot::at(0x933E90),
        Slot::at(0x933E94),
        Slot::at(0x933E98),
        Slot::at(0x933E9C),
        Slot::at(0x933EA0),
        Slot::at(0x933EA4),
        Slot::at(0x933EA8),
        Slot::at(0x933EAC),
        Slot::at(0x933EB0),
    ],
    outs: [
        Slot::at(0x933248),
        Slot::at(0x933250),
        Slot::at(0x93324C),
        Slot::at(0x93325C),
        Slot::at(0x933260),
        Slot::at(0x933254),
        Slot::at(0x933258),
        Slot::at(0x933264),
        Slot::at(0x933268),
        Slot::at(0x93326C),
        Slot::at(0x933E70),
    ],
    place: [Slot::at(0x9331B4), Slot::at(0x9331B8), Slot::at(0x9331BC)],
    copied: Slot::at(0x933200),
    divisor_copy: Some(Slot::at(0x933E74)),
};

pub const SECOND: KindTables = KindTables {
    tables: [
        Slot::at(0x934848),
        Slot::at(0x9346B0),
        Slot::at(0x934738),
        Slot::at(0x934AF0),
        Slot::at(0x934A68),
        Slot::at(0x9348D0),
        Slot::at(0x934958),
        Slot::at(0x9349E0),
        Slot::at(0x9347C0),
        Slot::at(0x934628),
    ],
    triples: [Slot::at(0x934490), Slot::at(0x9342F8), Slot::at(0x934160)],
    constants: [
        Slot::at(0x934B7C),
        Slot::at(0x934B80),
        Slot::at(0x934B84),
        Slot::at(0x934B88),
        Slot::at(0x934B8C),
        Slot::at(0x934B90),
        Slot::at(0x934B94),
        Slot::at(0x934B98),
        Slot::at(0x934B9C),
        Slot::at(0x934BA0),
        Slot::at(0x934BA4),
        Slot::at(0x934BA8),
        Slot::at(0x934BAC),
        Slot::at(0x934BB0),
    ],
    outs: [
        Slot::at(0x933F64),
        Slot::at(0x933F6C),
        Slot::at(0x933F68),
        Slot::at(0x933F78),
        Slot::at(0x933F7C),
        Slot::at(0x933F70),
        Slot::at(0x933F74),
        Slot::at(0x933F80),
        Slot::at(0x933F84),
        Slot::at(0x933F88),
        Slot::at(0x934B78),
    ],
    place: [Slot::at(0x933ED0), Slot::at(0x933ED4), Slot::at(0x933ED8)],
    copied: Slot::at(0x933F1C),
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
