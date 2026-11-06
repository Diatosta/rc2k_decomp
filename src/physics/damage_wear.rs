//! `physics::damage_wear` (0x4ABEDF): the wear of this step (`physics/damage_model.rs`).
//!
//! The clock into `dword_75B4A0`; table lerps of the speed shares into `flt_75B45C` ..; the
//! heat, damper, brake-fade and engine-heat gauges raised and lowered by their rates times
//! the shares and `wear_step`, each held 0 ..= `gauge_top`; the hit figures `flt_75B450` ..
//! `flt_75B458`; and `flt_75B484` from the table `dword_75B73C` (row `dword_75B3E8`, two
//! columns through `dword_75B8C0` / `75B8C4`, as the original indexes them) blended by
//! `dword_75B3F0`, times `flt_75B8BC`.
//!
//! Caller: `physics::damage_step`.

use crate::math::x87_float::{X, int, ld, st};
use crate::physics::car_store::CAR;
use crate::physics::damage_gauges::raise;
use crate::physics::flow::{hold_low, table_lerp};
use crate::physics::store::DamageModel;
use crate::pointers::GamePtr;
use crate::session::Cell32;
use core::mem::offset_of;

/// The wear of this step: the gauges raised or lowered by their rates times the shares,
/// the lerps of the speed shares, and the hit figures.
pub fn steps() {
    crate::race::owner::race()
        .physics
        .damage
        .clock
        .set_bits(CAR.noise_word.get());
    table_lerp(
        &crate::race::owner::race().physics.damage.engine_speed_share,
        &crate::race::owner::race().physics.damage.lerp_tables[0],
        &crate::race::owner::race().physics.damage.gauge_low,
        &crate::race::owner::race().physics.damage.flt_75b45c,
    );
    for k in 0..2 {
        raise(&CAR.brake_heat[k], |g| {
            add_wear(
                &crate::race::owner::race().physics.damage,
                &crate::race::owner::race().physics.damage.wear_rates[3],
                &crate::race::owner::race().physics.damage.brake_share,
                g,
                g,
            );
        });
    }
    for k in 0..2 {
        raise(&CAR.brake_heat[k], |g| {
            subtract_rate(
                &crate::race::owner::race().physics.damage,
                &crate::race::owner::race().physics.damage.wear_rates[4],
                g,
                g,
            );
        });
    }
    let extrapolated = [
        &crate::race::owner::race()
            .physics
            .damage
            .drive_a_extrapolated_share,
        &crate::race::owner::race()
            .physics
            .damage
            .drive_b_extrapolated_share,
    ];
    for (damper, share) in CAR.damper_wear.iter().zip(extrapolated) {
        raise(damper, |g| {
            add_wear(
                &crate::race::owner::race().physics.damage,
                &crate::race::owner::race().physics.damage.wear_rates[5],
                share,
                g,
                g,
            );
        });
    }
    for k in 0..2 {
        raise(&CAR.damper_wear[k], |g| {
            subtract_rate(
                &crate::race::owner::race().physics.damage,
                &crate::race::owner::race().physics.damage.wear_rates[6],
                g,
                g,
            );
        });
    }
    raise(&CAR.brake_fade, |g| {
        add_wear(
            &crate::race::owner::race().physics.damage,
            &crate::race::owner::race().physics.damage.wear_rates[7],
            &crate::race::owner::race().physics.damage.flt_75b3b4,
            g,
            g,
        );
    });
    raise(&CAR.brake_fade, |g| {
        subtract_rate(
            &crate::race::owner::race().physics.damage,
            &crate::race::owner::race().physics.damage.wear_rates[8],
            g,
            g,
        );
    });
    raise(&CAR.brake_fade, |g| {
        subtract_wear(
            &crate::race::owner::race().physics.damage,
            &crate::race::owner::race().physics.damage.wear_rates[9],
            &crate::race::owner::race().physics.damage.brake_share,
            g,
            g,
        );
    });
    table_lerp(
        &CAR.cooling_damage,
        &crate::race::owner::race().physics.damage.lerp_tables[1],
        &crate::race::owner::race().physics.damage.gauge_low,
        &crate::race::owner::race().physics.damage.flt_75b460,
    );
    lerp_gap(&crate::race::owner::race().physics.damage);
    hold_low(
        &crate::race::owner::race().physics.damage.flt_75b464,
        &crate::race::owner::race().physics.damage.gauge_low,
    );
    raise(&CAR.engine_heat, |g| {
        add_wear(
            &crate::race::owner::race().physics.damage,
            &crate::race::owner::race().physics.damage.wear_rates[0],
            &crate::race::owner::race().physics.damage.flt_75b464,
            g,
            g,
        );
    });
    raise(&CAR.engine_heat, |g| {
        subtract_rate(
            &crate::race::owner::race().physics.damage,
            &crate::race::owner::race().physics.damage.wear_rates[1],
            g,
            g,
        );
    });
    raise(&CAR.engine_heat, |g| {
        subtract_wear(
            &crate::race::owner::race().physics.damage,
            &crate::race::owner::race().physics.damage.wear_rates[2],
            &crate::race::owner::race().physics.damage.speed_share,
            g,
            g,
        );
    });
    table_lerp(
        &CAR.stiffness_mean,
        &crate::race::owner::race().physics.damage.lerp_tables_b[2],
        &crate::race::owner::race().physics.damage.gauge_low,
        &crate::race::owner::race().physics.damage.flt_75b44c,
    );
    square_into(
        &crate::race::owner::race().physics.damage.flt_75b44c,
        &crate::race::owner::race().physics.damage.flt_75b44c,
    );
    table_lerp(
        &CAR.stiffness_mean,
        &crate::race::owner::race().physics.damage.lerp_tables_b[0],
        &crate::race::owner::race().physics.damage.gauge_low,
        &crate::race::owner::race().physics.damage.flt_75b448,
    );
    square_into(
        &crate::race::owner::race().physics.damage.flt_75b448,
        &crate::race::owner::race().physics.damage.flt_75b448,
    );
    table_lerp(
        &CAR.ride_height_mean,
        &crate::race::owner::race().physics.damage.lerp_tables_b[1],
        &crate::race::owner::race().physics.damage.gauge_low,
        &crate::race::owner::race().physics.damage.flt_75b468,
    );
    square_into(
        &crate::race::owner::race().physics.damage.flt_75b468,
        &crate::race::owner::race().physics.damage.flt_75b468,
    );
    table_lerp(
        &crate::race::owner::race().physics.damage.drive_a_slip_share,
        &crate::race::owner::race().physics.damage.lerp_tables[3],
        &crate::race::owner::race().physics.damage.gauge_low,
        &crate::race::owner::race().physics.damage.flt_75b46c,
    );
    table_lerp(
        &crate::race::owner::race().physics.damage.drive_b_slip_share,
        &crate::race::owner::race().physics.damage.lerp_tables[3],
        &crate::race::owner::race().physics.damage.gauge_low,
        &crate::race::owner::race().physics.damage.flt_75b470,
    );
    hit_figures(&crate::race::owner::race().physics.damage);
    // 0x4AC7C9: two figures of the table `dword_75B73C` (row `dword_75B3E8`, columns
    // from `dword_75B8C0` by `dword_75B3EC` & 0xF, then from `dword_75B8C4` by that
    // column, as the original), blended by `dword_75B3F0`, x `flt_75B8BC`.
    let row = crate::race::owner::race()
        .physics
        .damage
        .dword_75b3e8
        .bits()
        << 6;
    // The original runs on through the store, whatever the columns hold.
    let store = GamePtr::to(&crate::race::owner::race().physics.damage);
    let at = |field: usize, n: u32| store.offset((field as u32).wrapping_add(n));
    let read = |at: GamePtr| {
        at.read::<u32>("a car block's field")
            .unwrap_or_else(|e| crate::fatal::raise::end("a car block's field", e))
    };
    let wear = offset_of!(DamageModel, wear_table);
    let column = read(at(
        offset_of!(DamageModel, dword_75b8c0),
        (crate::race::owner::race()
            .physics
            .damage
            .dword_75b3ec
            .bits()
            & 0xF)
            .wrapping_mul(8),
    ));
    let first = read(at(wear, row.wrapping_add(column.wrapping_mul(4))));
    let column = read(at(
        offset_of!(DamageModel, dword_75b8c4),
        column.wrapping_mul(8),
    ));
    let second = read(at(wear, row.wrapping_add(column.wrapping_mul(4))));
    blend_wear_figures(first, &crate::race::owner::race().physics.damage, second);
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + o4] = (([m + o0] * [m + o1]) * [m + 0x35B390]) + [rc + o3]`
fn add_wear(
    m: &DamageModel,
    o0: &impl Cell32,
    o1: &impl Cell32,
    o3: &impl crate::session::Cell32,
    o4: &impl crate::session::Cell32,
) {
    let t0 = ld(o0.bits());
    let t1 = ld(o1.bits());
    let t2 = t0 * t1;
    let t3 = ld(m.wear_step.bits());
    let t4 = t2 * t3;
    let t5 = ld(o3.bits());
    let t6 = t4 + t5;
    o4.set_bits(st(t6));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + o3] = -([m + o0] * [m + 0x35B390]) + [rc + o2]`
fn subtract_rate(
    m: &DamageModel,
    o0: &impl Cell32,
    o2: &impl crate::session::Cell32,
    o3: &impl crate::session::Cell32,
) {
    let t0 = ld(o0.bits());
    let t1 = ld(m.wear_step.bits());
    let t2 = t0 * t1;
    let t3 = -t2;
    let t4 = ld(o2.bits());
    let t5 = t3 + t4;
    o3.set_bits(st(t5));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + o4] = -(([m + o0] * [m + o1]) * [m + 0x35B390]) + [rc + o3]`
fn subtract_wear(
    m: &DamageModel,
    o0: &impl Cell32,
    o1: &impl Cell32,
    o3: &impl crate::session::Cell32,
    o4: &impl crate::session::Cell32,
) {
    let t0 = ld(o0.bits());
    let t1 = ld(o1.bits());
    let t2 = t0 * t1;
    let t3 = ld(m.wear_step.bits());
    let t4 = t2 * t3;
    let t5 = -t4;
    let t6 = ld(o3.bits());
    let t7 = t5 + t6;
    o4.set_bits(st(t7));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B464] = [m + 0x35B398] - [m + 0x35B460]`
fn lerp_gap(m: &DamageModel) {
    let t0 = ld(m.engine_speed_share.bits());
    let t1 = ld(m.flt_75b460.bits());
    let t2 = t0 - t1;
    m.flt_75b464.set_bits(st(t2));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + o1] = [m + o0] * [m + o0]`
fn square_into(o0: &impl Cell32, o1: &impl Cell32) {
    let t0 = ld(o0.bits());
    let t1 = t0 * t0;
    o1.set_bits(st(t1));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B454] = (((([m + 0x35B3E0] * [m + 0x35B39C]) * [m + 0x35B6F8]) * [m + 0x35B44C]) * [m + 0x35B468]) * [m + 0x35B3F4]`
/// `[m + 0x35B450] = (([m + 0x35B3E0] * [m + 0x35B39C]) * [m + 0x35B6F8]) * [m + 0x35B3F4]`
/// `[m + 0x35B458] = ((([m + 0x35B3E0] * [m + 0x35B39C]) * [m + 0x35B6F8]) * [m + 0x35B3F4]) * [m + 0x35B448]`
fn hit_figures(m: &DamageModel) {
    let t0 = ld(m.flt_75b3e0.bits());
    let t1 = ld(m.speed_share.bits());
    let t2 = t0 * t1;
    let t3 = ld(m.figures[32].bits());
    let t4 = t2 * t3;
    let t5 = ld(m.flt_75b44c.bits());
    let t6 = t4 * t5;
    let t7 = ld(m.flt_75b468.bits());
    let t8 = t6 * t7;
    let t9 = ld(m.wheels_down.bits());
    let t10 = t8 * t9;
    m.flt_75b454.set_bits(st(t10));
    let t11 = ld(m.flt_75b3e0.bits());
    let t12 = ld(m.speed_share.bits());
    let t13 = t11 * t12;
    let t14 = ld(m.figures[32].bits());
    let t15 = t13 * t14;
    let t16 = ld(m.wheels_down.bits());
    let t17 = t15 * t16;
    m.flt_75b450.set_bits(st(t17));
    let t18 = ld(m.flt_75b448.bits());
    let t19 = t17 * t18;
    m.flt_75b458.set_bits(st(t19));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[m + 0x35B484] = ((int [f + 0] * (1 - [m + 0x35B3F0])) + (int [s + 0] * [m + 0x35B3F0])) * [m + 0x35B8BC]`
fn blend_wear_figures(f: u32, m: &DamageModel, s: u32) {
    let t0 = int(f as i32);
    let t1 = X(1.0);
    let t2 = ld(m.dword_75b3f0.bits());
    let t3 = t1 - t2;
    let t4 = t0 * t3;
    let t5 = int(s as i32);
    let t6 = ld(m.dword_75b3f0.bits());
    let t7 = t5 * t6;
    let t8 = t4 + t7;
    let t9 = ld(m.flt_75b8bc.bits());
    let t10 = t8 * t9;
    m.flt_75b484.set_bits(st(t10));
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;

    /// The damage model as the test's buffer holds it (the buffer's base plus the RVA of
    /// 0x75B340).
    fn dm(base: usize) -> &'static DamageModel {
        unsafe { &*(crate::pointers::host_at(base + 0x35B340) as *const DamageModel) }
    }
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};
    use crate::scene::x87_mem::HostCell;

    // The original's instructions, one function per run.
    reference!(ref_0(m, rc) "fld dword ptr [{m} + 0x35B584]", "fmul dword ptr [{m} + 0x35B3B0]", "fmul dword ptr [{m} + 0x35B390]", "fadd dword ptr [{rc} + 0x31A284]", "fstp dword ptr [{rc} + 0x31A284]");
    reference!(ref_1(m, rc) "fld dword ptr [{m} + 0x35B584]", "fmul dword ptr [{m} + 0x35B3B0]", "fmul dword ptr [{m} + 0x35B390]", "fadd dword ptr [{rc} + 0x31A288]", "fstp dword ptr [{rc} + 0x31A288]");
    reference!(ref_2(m, rc) "fld dword ptr [{m} + 0x35B588]", "fmul dword ptr [{m} + 0x35B390]", ".byte 0xD9, 0xE0", "fadd dword ptr [{rc} + 0x31A284]", "fstp dword ptr [{rc} + 0x31A284]");
    reference!(ref_3(m, rc) "fld dword ptr [{m} + 0x35B588]", "fmul dword ptr [{m} + 0x35B390]", ".byte 0xD9, 0xE0", "fadd dword ptr [{rc} + 0x31A288]", "fstp dword ptr [{rc} + 0x31A288]");
    reference!(ref_4(m, rc) "fld dword ptr [{m} + 0x35B58C]", "fmul dword ptr [{m} + 0x35B3A8]", "fmul dword ptr [{m} + 0x35B390]", "fadd dword ptr [{rc} + 0x31A27C]", "fstp dword ptr [{rc} + 0x31A27C]");
    reference!(ref_5(m, rc) "fld dword ptr [{m} + 0x35B58C]", "fmul dword ptr [{m} + 0x35B3AC]", "fmul dword ptr [{m} + 0x35B390]", "fadd dword ptr [{rc} + 0x31A280]", "fstp dword ptr [{rc} + 0x31A280]");
    reference!(ref_6(m, rc) "fld dword ptr [{m} + 0x35B590]", "fmul dword ptr [{m} + 0x35B390]", ".byte 0xD9, 0xE0", "fadd dword ptr [{rc} + 0x31A27C]", "fstp dword ptr [{rc} + 0x31A27C]");
    reference!(ref_7(m, rc) "fld dword ptr [{m} + 0x35B590]", "fmul dword ptr [{m} + 0x35B390]", ".byte 0xD9, 0xE0", "fadd dword ptr [{rc} + 0x31A280]", "fstp dword ptr [{rc} + 0x31A280]");
    reference!(ref_8(m, rc) "fld dword ptr [{m} + 0x35B594]", "fmul dword ptr [{m} + 0x35B3B4]", "fmul dword ptr [{m} + 0x35B390]", "fadd dword ptr [{rc} + 0x31A278]", "fstp dword ptr [{rc} + 0x31A278]");
    reference!(ref_9(m, rc) "fld dword ptr [{m} + 0x35B598]", "fmul dword ptr [{m} + 0x35B390]", ".byte 0xD9, 0xE0", "fadd dword ptr [{rc} + 0x31A278]", "fstp dword ptr [{rc} + 0x31A278]");
    reference!(ref_10(m, rc) "fld dword ptr [{m} + 0x35B59C]", "fmul dword ptr [{m} + 0x35B3B0]", "fmul dword ptr [{m} + 0x35B390]", ".byte 0xD9, 0xE0", "fadd dword ptr [{rc} + 0x31A278]", "fstp dword ptr [{rc} + 0x31A278]");
    reference!(ref_11(m) "fld dword ptr [{m} + 0x35B398]", "fsub dword ptr [{m} + 0x35B460]", "fstp dword ptr [{m} + 0x35B464]");
    reference!(ref_12(m, rc) "fld dword ptr [{m} + 0x35B578]", "fmul dword ptr [{m} + 0x35B464]", "fmul dword ptr [{m} + 0x35B390]", "fadd dword ptr [{rc} + 0x31A28C]", "fstp dword ptr [{rc} + 0x31A28C]");
    reference!(ref_13(m, rc) "fld dword ptr [{m} + 0x35B57C]", "fmul dword ptr [{m} + 0x35B390]", ".byte 0xD9, 0xE0", "fadd dword ptr [{rc} + 0x31A28C]", "fstp dword ptr [{rc} + 0x31A28C]");
    reference!(ref_14(m, rc) "fld dword ptr [{m} + 0x35B580]", "fmul dword ptr [{m} + 0x35B39C]", "fmul dword ptr [{m} + 0x35B390]", ".byte 0xD9, 0xE0", "fadd dword ptr [{rc} + 0x31A28C]", "fstp dword ptr [{rc} + 0x31A28C]");
    reference!(ref_15(m) "fld dword ptr [{m} + 0x35B44C]", ".byte 0xD8, 0xC8", "fstp dword ptr [{m} + 0x35B44C]");
    reference!(ref_16(m) "fld dword ptr [{m} + 0x35B448]", ".byte 0xD8, 0xC8", "fstp dword ptr [{m} + 0x35B448]");
    reference!(ref_17(m) "fld dword ptr [{m} + 0x35B468]", ".byte 0xD8, 0xC8", "fstp dword ptr [{m} + 0x35B468]");
    reference!(ref_18(m) "fld dword ptr [{m} + 0x35B3E0]", "fmul dword ptr [{m} + 0x35B39C]", "fmul dword ptr [{m} + 0x35B6F8]", "fmul dword ptr [{m} + 0x35B44C]", "fmul dword ptr [{m} + 0x35B468]", "fmul dword ptr [{m} + 0x35B3F4]", "fstp dword ptr [{m} + 0x35B454]", "fld dword ptr [{m} + 0x35B3E0]", "fmul dword ptr [{m} + 0x35B39C]", "fmul dword ptr [{m} + 0x35B6F8]", "fmul dword ptr [{m} + 0x35B3F4]", "fst dword ptr [{m} + 0x35B450]", "fmul dword ptr [{m} + 0x35B448]", "fstp dword ptr [{m} + 0x35B458]");
    reference!(ref_19(f, m, s) "fild dword ptr [{f}]", ".byte 0xD9, 0xE8", "fsub dword ptr [{m} + 0x35B3F0]", ".byte 0xDE, 0xC9", "fild dword ptr [{s}]", "fld dword ptr [{m} + 0x35B3F0]", ".byte 0xDE, 0xC9", ".byte 0xDE, 0xC1", "fmul dword ptr [{m} + 0x35B8BC]", "fstp dword ptr [{m} + 0x35B484]");

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "damage_wear line 36",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B390, In::F), (0x35B3B0, In::F), (0x35B584, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A284, In::F)],
                        writes: &[(0x31A284, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_0(a[0], a[1]) },
                plain: |a| {
                    add_wear(
                        dm(a[0]),
                        &dm(a[0]).wear_rates[3],
                        &dm(a[0]).brake_share,
                        &CAR.brake_heat[0],
                        &CAR.brake_heat[0],
                    );
                },
            },
            Block {
                name: "damage_wear line 47",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B390, In::F), (0x35B3B0, In::F), (0x35B584, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A288, In::F)],
                        writes: &[(0x31A288, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_1(a[0], a[1]) },
                plain: |a| {
                    add_wear(
                        dm(a[0]),
                        &dm(a[0]).wear_rates[3],
                        &dm(a[0]).brake_share,
                        &CAR.brake_heat[1],
                        &CAR.brake_heat[1],
                    );
                },
            },
            Block {
                name: "damage_wear line 58",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B390, In::F), (0x35B588, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A284, In::F)],
                        writes: &[(0x31A284, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_2(a[0], a[1]) },
                plain: |a| {
                    subtract_rate(
                        dm(a[0]),
                        &dm(a[0]).wear_rates[4],
                        &CAR.brake_heat[0],
                        &CAR.brake_heat[0],
                    );
                },
            },
            Block {
                name: "damage_wear line 69",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B390, In::F), (0x35B588, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A288, In::F)],
                        writes: &[(0x31A288, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_3(a[0], a[1]) },
                plain: |a| {
                    subtract_rate(
                        dm(a[0]),
                        &dm(a[0]).wear_rates[4],
                        &CAR.brake_heat[1],
                        &CAR.brake_heat[1],
                    );
                },
            },
            Block {
                name: "damage_wear line 80",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B390, In::F), (0x35B3A8, In::F), (0x35B58C, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A27C, In::F)],
                        writes: &[(0x31A27C, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_4(a[0], a[1]) },
                plain: |a| {
                    add_wear(
                        dm(a[0]),
                        &dm(a[0]).wear_rates[5],
                        &dm(a[0]).drive_a_extrapolated_share,
                        &CAR.damper_wear[0],
                        &CAR.damper_wear[0],
                    );
                },
            },
            Block {
                name: "damage_wear line 91",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B390, In::F), (0x35B3AC, In::F), (0x35B58C, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A280, In::F)],
                        writes: &[(0x31A280, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_5(a[0], a[1]) },
                plain: |a| {
                    add_wear(
                        dm(a[0]),
                        &dm(a[0]).wear_rates[5],
                        &dm(a[0]).drive_b_extrapolated_share,
                        &CAR.damper_wear[1],
                        &CAR.damper_wear[1],
                    );
                },
            },
            Block {
                name: "damage_wear line 102",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B390, In::F), (0x35B590, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A27C, In::F)],
                        writes: &[(0x31A27C, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_6(a[0], a[1]) },
                plain: |a| {
                    subtract_rate(
                        dm(a[0]),
                        &dm(a[0]).wear_rates[6],
                        &CAR.damper_wear[0],
                        &CAR.damper_wear[0],
                    );
                },
            },
            Block {
                name: "damage_wear line 113",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B390, In::F), (0x35B590, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A280, In::F)],
                        writes: &[(0x31A280, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_7(a[0], a[1]) },
                plain: |a| {
                    subtract_rate(
                        dm(a[0]),
                        &dm(a[0]).wear_rates[6],
                        &CAR.damper_wear[1],
                        &CAR.damper_wear[1],
                    );
                },
            },
            Block {
                name: "damage_wear line 124",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B390, In::F), (0x35B3B4, In::F), (0x35B594, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A278, In::F)],
                        writes: &[(0x31A278, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_8(a[0], a[1]) },
                plain: |a| {
                    add_wear(
                        dm(a[0]),
                        &dm(a[0]).wear_rates[7],
                        &dm(a[0]).flt_75b3b4,
                        &CAR.brake_fade,
                        &CAR.brake_fade,
                    );
                },
            },
            Block {
                name: "damage_wear line 135",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B390, In::F), (0x35B598, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A278, In::F)],
                        writes: &[(0x31A278, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_9(a[0], a[1]) },
                plain: |a| {
                    subtract_rate(
                        dm(a[0]),
                        &dm(a[0]).wear_rates[8],
                        &CAR.brake_fade,
                        &CAR.brake_fade,
                    );
                },
            },
            Block {
                name: "damage_wear line 146",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B390, In::F), (0x35B3B0, In::F), (0x35B59C, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A278, In::F)],
                        writes: &[(0x31A278, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_10(a[0], a[1]) },
                plain: |a| {
                    subtract_wear(
                        dm(a[0]),
                        &dm(a[0]).wear_rates[9],
                        &dm(a[0]).brake_share,
                        &CAR.brake_fade,
                        &CAR.brake_fade,
                    );
                },
            },
            Block {
                name: "damage_wear line 159",
                operands: vec![Operand::Buffer {
                    reads: &[(0x35B398, In::F), (0x35B460, In::F)],
                    writes: &[(0x35B464, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_11(a[0]) },
                plain: |a| lerp_gap(dm(a[0])),
            },
            Block {
                name: "damage_wear line 166",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B390, In::F), (0x35B464, In::F), (0x35B578, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A28C, In::F)],
                        writes: &[(0x31A28C, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_12(a[0], a[1]) },
                plain: |a| {
                    add_wear(
                        dm(a[0]),
                        &dm(a[0]).wear_rates[0],
                        &dm(a[0]).flt_75b464,
                        &CAR.engine_heat,
                        &CAR.engine_heat,
                    );
                },
            },
            Block {
                name: "damage_wear line 177",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B390, In::F), (0x35B57C, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A28C, In::F)],
                        writes: &[(0x31A28C, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_13(a[0], a[1]) },
                plain: |a| {
                    subtract_rate(
                        dm(a[0]),
                        &dm(a[0]).wear_rates[1],
                        &CAR.engine_heat,
                        &CAR.engine_heat,
                    );
                },
            },
            Block {
                name: "damage_wear line 188",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x35B390, In::F), (0x35B39C, In::F), (0x35B580, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A28C, In::F)],
                        writes: &[(0x31A28C, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_14(a[0], a[1]) },
                plain: |a| {
                    subtract_wear(
                        dm(a[0]),
                        &dm(a[0]).wear_rates[2],
                        &dm(a[0]).speed_share,
                        &CAR.engine_heat,
                        &CAR.engine_heat,
                    );
                },
            },
            Block {
                name: "damage_wear line 201",
                operands: vec![Operand::Buffer {
                    reads: &[(0x35B44C, In::F)],
                    writes: &[(0x35B44C, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_15(a[0]) },
                plain: |a| square_into(&dm(a[0]).flt_75b44c, &dm(a[0]).flt_75b44c),
            },
            Block {
                name: "damage_wear line 208",
                operands: vec![Operand::Buffer {
                    reads: &[(0x35B448, In::F)],
                    writes: &[(0x35B448, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_16(a[0]) },
                plain: |a| square_into(&dm(a[0]).flt_75b448, &dm(a[0]).flt_75b448),
            },
            Block {
                name: "damage_wear line 215",
                operands: vec![Operand::Buffer {
                    reads: &[(0x35B468, In::F)],
                    writes: &[(0x35B468, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_17(a[0]) },
                plain: |a| square_into(&dm(a[0]).flt_75b468, &dm(a[0]).flt_75b468),
            },
            Block {
                name: "damage_wear line 223",
                operands: vec![Operand::Buffer {
                    reads: &[
                        (0x35B39C, In::F),
                        (0x35B3E0, In::F),
                        (0x35B3F4, In::F),
                        (0x35B448, In::F),
                        (0x35B44C, In::F),
                        (0x35B468, In::F),
                        (0x35B6F8, In::F),
                    ],
                    writes: &[(0x35B450, Out::F), (0x35B454, Out::F), (0x35B458, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_18(a[0]) },
                plain: |a| hit_figures(dm(a[0])),
            },
            Block {
                name: "damage_wear line 252",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::I)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x35B3F0, In::F), (0x35B8BC, In::F)],
                        writes: &[(0x35B484, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(0, In::I)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_19(a[0], a[1], a[2]) },
                plain: |a| {
                    blend_wear_figures(HostCell(a[0]).bits(), dm(a[1]), HostCell(a[2]).bits());
                },
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
