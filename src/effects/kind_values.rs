//! `effects::init_marks`' steps 8 and 9 (IDA): `effects::set_kind_values_1`
//! (0x4C7322) and `effects::set_kind_values_2` (0x4C7FA4), the same code on two sets of
//! tables: the values of the marks' kind (`g_marksKind`) from tables of 0x22 floats and
//! of 0x22 three-float entries.

use crate::effects::kind_store::{CONSTS_1, CONSTS_2, KindConsts, QuadWork, TABLES_1, TABLES_2};
use crate::effects::owner::Effects;
#[cfg(test)]
use crate::effects::owner::effects;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::math::x87_float as bh;
use crate::session::{PlainDword, PlainFloat};

pub const KINDS: u32 = 0x22;

/// One set's variables, in the order the code uses them (the typed statics of
/// `effects::kind_store`).
pub struct KindSet<'a> {
    /// Tables of 0x22 three-float entries: `to`, `from`, `copied`.
    pub triples: [&'a [[f32; 3]; KINDS as usize]; 3],
    /// Tables of 0x22 floats: `size`, `divisor`, `low`, `high`, `start`, `middle`, `end`,
    /// `spread`, `extra`.
    pub values: [&'a [f32; KINDS as usize]; 9],
    /// The table of 0x22 dwords: `count`.
    pub count: &'a [u32; KINDS as usize],
    /// The constants.
    pub consts: &'a KindConsts,
    /// The quad the values are put in.
    pub quad: &'a QuadWork,
    /// Out: `extra`.
    pub extra: &'a PlainFloat,
    /// `divisor` copied here too (the first set only).
    pub divisor_copy: Option<&'a PlainFloat>,
}

pub fn first(effects: &Effects) -> KindSet<'_> {
    KindSet {
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
        quad: &effects.kinds.overlay_quad,
        extra: &effects.kinds.overlay_extras.extra,
        divisor_copy: Some(&effects.kinds.overlay_extras.divisor_copy),
    }
}

pub fn second(effects: &Effects) -> KindSet<'_> {
    KindSet {
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
        quad: &effects.kinds.shadow_quad,
        extra: &effects.kinds.shadow_extra.extra,
        divisor_copy: None,
    }
}

/// The kind's values of one set. Refuses a kind past the tables (0x22).
pub fn set_kind_values(k: &KindSet<'_>, effects: &Effects) -> Result<(), Refused> {
    let kind: u32 = effects.kinds.marks_kind.kind.get();
    if kind >= KINDS {
        return Err(BadInput("the marks' kind is past the tables (0x22)").into());
    }
    let n = kind as usize;
    let q = k.quad;
    let c = k.consts;
    let [size, divisor, low, high, start, middle, end, spread, extra] =
        k.values.map(|table| table[n]);
    let count = k.count[n];
    q.corner.set(count);
    let (scaled, ratio) = x87::mul_and_div(size, c.size_k, divisor);
    q.size.set(scaled);
    q.ratio.set(ratio);
    q.low.set(low);
    q.high.set(high);
    q.start.set(x87::mul(start, c.start_k));
    q.step.set(bh::stf(
        (bh::ld(middle) - bh::ld(start)) / bh::ld(low) * bh::ld(c.step_k),
    ));
    q.middle.set(x87::mul(middle, c.middle_k));
    q.slope.set(bh::stf(
        (bh::ld(end) - bh::ld(middle)) / (bh::ld(high) - bh::ld(low)) * bh::ld(c.slope_k),
    ));
    q.spread.set(x87::mul(spread, c.spread_k));
    k.extra.set(x87::mul(extra, c.extra_k));
    if let Some(copy) = k.divisor_copy {
        copy.set(divisor);
    }
    let triple = |t: usize, part: usize| k.triples[t][n][part].to_bits();
    let lerp = |part: usize, less: Option<u32>| {
        x87::lerp_scaled_less(
            triple(0, part),
            triple(1, part),
            c.place_k[2 * part],
            c.place_k[2 * part + 1],
            less,
        )
    };
    q.place[0].set(lerp(2, None));
    q.place[1].set(lerp(1, Some(c.place_less.to_bits())));
    q.place[2].set(lerp(0, None));
    // The first side is the kind's `copied` triple; the second side's first float is the 0
    // that goes with it (the original wrote the four as one), then the two differences.
    q.sides[0][0].set_bits(triple(2, 0));
    q.sides[0][1].set_bits(triple(2, 1));
    q.sides[0][2].set_bits(triple(2, 2));
    q.sides[1][0].set(0.0);
    q.sides[1][1].set(x87::sub(triple(0, 1), triple(1, 1)));
    q.sides[1][2].set(x87::negated(x87::sub(triple(0, 0), triple(1, 0))));
    Ok(())
}

/// The marks' kind (`effects::kind_values`) (IDA 0x769232 .. 0x769236).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct MarksKind {
    /// 0x769232: `g_marksKind`: the kind of the marks (the surface / weather: 0 .. 0x21), set at `effects::init_marks`.
    pub kind: PlainDword,
}

impl MarksKind {
    pub const fn new() -> Self {
        Self {
            kind: PlainDword::new(0x0),
        }
    }
}

impl MarksKind {
    pub const START: usize = 0x769232;
    pub const LEN: usize = 0x4;
}

#[cfg(test)]
mod marks_kind_tests {
    use super::*;

    #[test]
    fn marks_kind_starts_as_the_games() {
        use core::mem::offset_of;
        assert_eq!(size_of::<MarksKind>(), MarksKind::LEN);
        let p = (&raw const effects().kinds.marks_kind).cast::<u8>();
        let ours: Vec<u8> = (0..size_of::<MarksKind>())
            .map(|i| unsafe { p.add(i).read() })
            .collect();
        assert!(
            ours == crate::test_support::first_values::bytes(
                "effects.owner.EFFECTS.kinds.marks_kind/4"
            ),
            "MARKS_KIND starts differently from the image"
        );
        assert_eq!(offset_of!(MarksKind, kind), 0x0);
    }
}
