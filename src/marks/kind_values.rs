//! `marks::init_marks`' steps 8 and 9 (IDA, 2026-10-16): `marks::set_kind_values_1`
//! (0x4C7322) and `marks::set_kind_values_2` (0x4C7FA4), the same code on two sets of
//! tables: the values of the marks' kind (`g_marksKind`) from tables of 0x22 floats and
//! of 0x22 three-float entries.

use crate::errors::{BadInput, Refused};
use crate::marks::kind_store::{
    CONSTS_1, CONSTS_2, EXTRA_2, EXTRAS_1, KindConsts, QUAD_1, QUAD_2, QuadWork, TABLES_1, TABLES_2,
};
use crate::math::x87;
use crate::session::{Dword, Float};

pub const KINDS: u32 = 0x22;

/// One set's variables, in the order the code uses them (the typed statics of
/// `marks::kind_store`).
pub struct KindSet {
    /// Tables of 0x22 three-float entries: `to`, `from`, `copied`.
    pub triples: [&'static [[Float; 3]; KINDS as usize]; 3],
    /// Tables of 0x22 floats: `size`, `divisor`, `low`, `high`, `start`, `middle`, `end`,
    /// `spread`, `extra`.
    pub values: [&'static [Float; KINDS as usize]; 9],
    /// The table of 0x22 dwords: `count`.
    pub count: &'static [Dword; KINDS as usize],
    /// The constants.
    pub consts: &'static KindConsts,
    /// The quad the values are put in.
    pub quad: &'static QuadWork,
    /// Out: `extra`.
    pub extra: &'static Float,
    /// `divisor` copied here too (the first set only).
    pub divisor_copy: Option<&'static Dword>,
}

pub static FIRST: KindSet = KindSet {
    triples: [&TABLES_1.to, &TABLES_1.from, &TABLES_1.copied],
    values: [
        &TABLES_1.size,
        &TABLES_1.divisor,
        &TABLES_1.low,
        &TABLES_1.high,
        &TABLES_1.start,
        &TABLES_1.middle,
        &TABLES_1.end,
        &TABLES_1.spread,
        &TABLES_1.extra,
    ],
    count: &TABLES_1.count,
    consts: &CONSTS_1,
    quad: &QUAD_1,
    extra: &EXTRAS_1.extra,
    divisor_copy: Some(&EXTRAS_1.divisor_copy),
};

pub static SECOND: KindSet = KindSet {
    triples: [&TABLES_2.to, &TABLES_2.from, &TABLES_2.copied],
    values: [
        &TABLES_2.size,
        &TABLES_2.divisor,
        &TABLES_2.low,
        &TABLES_2.high,
        &TABLES_2.start,
        &TABLES_2.middle,
        &TABLES_2.end,
        &TABLES_2.spread,
        &TABLES_2.extra,
    ],
    count: &TABLES_2.count,
    consts: &CONSTS_2,
    quad: &QUAD_2,
    extra: &EXTRA_2.extra,
    divisor_copy: None,
};

/// The kind's values of one set. Refuses a kind past the tables (0x22).
pub unsafe fn set_kind_values(k: &KindSet) -> Result<(), Refused> {
    let kind: u32 = MARKS_KIND.kind.bits();
    if kind >= KINDS {
        return Err(BadInput("the marks' kind is past the tables (0x22)").into());
    }
    let n = kind as usize;
    let q = k.quad;
    let c = k.consts;
    let [size, divisor, low, high, start, middle, end, spread, extra] =
        k.values.map(|table| table[n].bits());
    let count = k.count[n].get();
    q.corner.set(count);
    let (scaled, ratio) = x87::mul_and_div(size, c.size_k.bits(), divisor);
    q.size.set_bits(scaled);
    q.ratio.set_bits(ratio);
    q.low.set_bits(low);
    q.high.set_bits(high);
    q.start.set_bits(x87::mul(start, c.start_k.bits()));
    q.step
        .set_bits(x87::sub_div_mul(middle, start, low, c.step_k.bits()));
    q.middle.set_bits(x87::mul(middle, c.middle_k.bits()));
    q.slope.set_bits(x87::diff_ratio_mul(
        end,
        middle,
        high,
        low,
        c.slope_k.bits(),
    ));
    q.spread.set_bits(x87::mul(spread, c.spread_k.bits()));
    k.extra.set_bits(x87::mul(extra, c.extra_k.bits()));
    if let Some(copy) = k.divisor_copy {
        copy.set(divisor);
    }
    let triple = |t: usize, part: usize| k.triples[t][n][part].bits();
    let lerp = |part: usize, less: Option<u32>| {
        x87::lerp_scaled_less(
            triple(0, part),
            triple(1, part),
            c.place_k[2 * part].bits(),
            c.place_k[2 * part + 1].bits(),
            less,
        )
    };
    q.place[0].set_bits(lerp(2, None));
    q.place[1].set_bits(lerp(1, Some(c.place_less.bits())));
    q.place[2].set_bits(lerp(0, None));
    // The first side is the kind's `copied` triple; the second side's first float is the 0
    // that goes with it (the original wrote the four as one), then the two differences.
    q.sides[0][0].set_bits(triple(2, 0));
    q.sides[0][1].set_bits(triple(2, 1));
    q.sides[0][2].set_bits(triple(2, 2));
    q.sides[1][0].set_bits(0);
    q.sides[1][1].set_bits(x87::sub(triple(0, 1), triple(1, 1)));
    q.sides[1][2].set_bits(x87::negated_sub(triple(0, 0), triple(1, 0)));
    Ok(())
}

/// The marks' kind (`marks::kind_values`) (IDA 0x769232 .. 0x769236, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct MarksKind {
    /// 0x769232: `g_marksKind`: the kind of the marks (the surface / weather: 0 .. 0x21), set at `marks::init_marks`.
    pub kind: Dword,
}

pub static MARKS_KIND: MarksKind = MarksKind {
    kind: Dword::new(0x0),
};

impl MarksKind {
    pub const START: usize = 0x769232;
    pub const LEN: usize = 0x4;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        &MARKS_KIND as *const MarksKind as usize
    }
}

#[cfg(test)]
mod marks_kind_tests {
    use super::*;

    #[test]
    fn marks_kind_starts_as_the_games() {
        use core::mem::offset_of;
        assert_eq!(size_of::<MarksKind>(), MarksKind::LEN);
        let p = &MARKS_KIND as *const MarksKind as *const u8;
        let ours: Vec<u8> = (0..size_of::<MarksKind>())
            .map(|i| unsafe { p.add(i).read() })
            .collect();
        assert!(
            ours == crate::moved::image_bytes(MarksKind::START, MarksKind::LEN),
            "MARKS_KIND starts differently from the image"
        );
        assert_eq!(offset_of!(MarksKind, kind), 0x769232 - MarksKind::START);
    }
}
