//! `marks::init_marks`' steps 8 and 9 (IDA, 2026-10-16): `marks::set_kind_values_1`
//! (0x4C7322) and `marks::set_kind_values_2` (0x4C7FA4), the same code on two sets of
//! tables: the values of the marks' kind (`g_marksKind`) from tables of 0x22 floats and
//! of 0x22 three-float entries.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::math::x87;
use crate::service::floats::{FLOAT, Floats, Tolerance, at_test_precisions, check_state_floats};

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

/// The kind's values of one set. eax, ebx, ecx are left the copied three, ebp the kind
/// times 12. Refuses a kind past the tables (0x22).
pub unsafe fn set_kind_values(entry: &Regs, k: &KindTables) -> Result<Regs, Refused> {
    let mut g = *entry;
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
        g.ebp = kind * 12;
        let triple =
            |n: usize, part: usize| -> u32 { game_read(k.triples[n] + g.ebp as usize + 4 * part) };
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
        g.eax = triple(2, 0);
        g.ebx = triple(2, 1);
        g.ecx = triple(2, 2);
        game_write(k.copied, [g.eax, g.ebx, g.ecx, 0]);
        game_write(k.copied + 16, x87::sub(triple(0, 1), triple(1, 1)));
        game_write(k.copied + 20, x87::negated_sub(triple(0, 0), triple(1, 0)));
    }
    Ok(g)
}

/// The converted version (plain floats): the hooks'. The x87 version above stays for
/// `marks::init_marks` (unconverted).
// Converted: `rc2k_core::marks::kind_values::values`
pub unsafe fn set_kind_values_plain(entry: &Regs, k: &KindTables) -> Result<Regs, Refused> {
    use rc2k_core::marks::kind_values::{Row, Triples, kind_entry, values};
    unsafe {
        let kind = kind_entry(game_read(KIND_RVA))?;
        let t = |n: usize| -> u32 { game_read(k.tables[n] + 4 * kind) };
        let f = |n: usize| f32::from_bits(t(n));
        let row = Row {
            count: t(0),
            size: f(1),
            divisor: f(2),
            low: t(3),
            high: t(4),
            start: f(5),
            middle: f(6),
            end: f(7),
            spread: f(8),
            extra: f(9),
        };
        let triple = |n: usize| -> [u32; 3] { game_read(k.triples[n] + 12 * kind) };
        let triples = Triples {
            to: triple(0).map(f32::from_bits),
            from: triple(1).map(f32::from_bits),
            copied: triple(2),
        };
        let c = k.constants.map(|a| f32::from_bits(game_read(a)));
        let v = values(&row, &triples, &c);
        let outs = [
            v.count,
            v.size.to_bits(),
            v.ratio.to_bits(),
            v.low,
            v.high,
            v.start.to_bits(),
            v.step.to_bits(),
            v.middle.to_bits(),
            v.slope.to_bits(),
            v.spread.to_bits(),
            v.extra.to_bits(),
        ];
        for (&to, v) in k.outs.iter().zip(outs) {
            game_write(to, v);
        }
        if let Some(copy) = k.divisor_copy {
            game_write(copy, t(2));
        }
        for (&to, p) in k.place.iter().zip(v.place) {
            game_write(to, p.to_bits());
        }
        let [eax, ebx, ecx] = triples.copied;
        game_write(k.copied, [eax, ebx, ecx, 0]);
        game_write(k.copied + 16, v.differences.map(f32::to_bits));
        Ok(Regs {
            eax,
            ebx,
            ecx,
            ebp: kind as u32 * 12,
            ..*entry
        })
    }
}

/// The float places of a set's writes: all of them but the dwords copied.
pub fn kind_floats(k: &KindTables) -> Vec<(crate::service::harness::Region, Tolerance)> {
    use crate::service::harness::at;
    let mut r: Vec<_> = [1, 2, 5, 6, 7, 8, 9, 10]
        .iter()
        .map(|&n| (at(k.outs[n]), FLOAT))
        .collect();
    r.extend(k.place.iter().map(|&a| (at(a), FLOAT)));
    r.push(((game_addr(k.copied) as u32 + 16, 8), FLOAT));
    r
}

/// What `set_kind_values` writes for a set.
pub fn kind_regions(k: &KindTables) -> Vec<crate::service::harness::Region> {
    use crate::service::harness::at;
    let mut r: Vec<_> = k.outs.iter().chain(&k.place).map(|&a| at(a)).collect();
    r.push((game_addr(k.copied) as u32, 24));
    r.extend(k.divisor_copy.map(at));
    r
}

/// The tables a test fills (0x22 floats each, 0x22 x 12 bytes each).
pub fn kind_inputs(k: &KindTables) -> Vec<crate::service::harness::Region> {
    let mut r: Vec<_> = k
        .tables
        .iter()
        .map(|&a| (game_addr(a) as u32, 4 * KINDS as usize))
        .collect();
    r.extend(
        k.triples
            .iter()
            .map(|&a| (game_addr(a) as u32, 12 * KINDS as usize)),
    );
    r.push(crate::service::harness::at(KIND_RVA));
    r
}

/// The self-test of either set: its tables filled with floats of the test's (each table's
/// own, so no difference is 0), every kind compared at both test precisions (exact at
/// double), then 0x22 refused.
pub fn test_kinds(
    t: &mut crate::framework::SelfTest,
    name: &'static str,
    k: &KindTables,
    call_original: crate::screens::harness::CallOriginal,
    rust: crate::service::harness::Rust,
) {
    use crate::service::harness::{capture, restore};
    let regions = kind_regions(k);
    let inputs = kind_inputs(k);
    let (saved, saved_inputs) = (capture(&regions), capture(&inputs));
    let places = kind_floats(k);
    let tolerant = Floats {
        memory: &places,
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, held| {
        for kind in 0..KINDS {
            check_state_floats(
                t,
                name,
                &format!("control word {word:#x}, kind {kind:#x}"),
                call_original,
                Regs::SENTINEL,
                &|| unsafe {
                    for &(a, n) in &regions {
                        std::ptr::write_bytes(a as *mut u8, 0xAA, n);
                    }
                    for (m, &(a, n)) in inputs[..inputs.len() - 1].iter().enumerate() {
                        for i in 0..n / 4 {
                            let v = 1.5 + m as f32 * 3.7 + i as f32 * (0.31 + m as f32 * 0.05);
                            ((a as usize + 4 * i) as *mut f32).write_unaligned(v);
                        }
                    }
                    game_write(KIND_RVA, kind);
                },
                &regions,
                held,
                rust,
            );
        }
    });
    unsafe { game_write(KIND_RVA, KINDS) };
    t.check(
        unsafe { set_kind_values_plain(&Regs::SENTINEL, k) }.is_err(),
        || "kind 0x22: the Rust version went on".into(),
    );
    restore(&inputs, &saved_inputs);
    restore(&regions, &saved);
}
