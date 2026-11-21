use super::*;
use crate::math::x87_mem::view;
use crate::memory::session::PlainFloat;
use crate::physics::store::laid;
use crate::test_support::own::race;

/// The damage model as the test's buffer holds it.
fn dm(base: usize) -> &'static mut DamageModel {
    crate::physics::store::model_in_test_buffer(base)
}
use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse_at};

fn blocks() -> Vec<Block> {
    vec![
        Block {
            name: "damage_gauges line 45",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B38C), In::F), (laid!(0x35B5D0), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A290, In::F)],
                    writes: &[(0x31A290, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race().shared.car().engine_damage;
                g.set(add_product(
                    dm(a[0]).gauge_rates[0],
                    dm(a[0]).gauge_step,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 55",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B38C), In::F), (laid!(0x35B5D4), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A294, In::F)],
                    writes: &[(0x31A294, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race().shared.car().exhaust_damage;
                g.set(add_product(
                    dm(a[0]).gauge_rates[1],
                    dm(a[0]).gauge_step,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 65",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B38C), In::F), (laid!(0x35B5D8), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A29C, In::F)],
                    writes: &[(0x31A29C, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race().shared.car().cooling_damage;
                g.set(add_product(
                    dm(a[0]).gauge_rates[2],
                    dm(a[0]).gauge_step,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 75",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B38C), In::F), (laid!(0x35B5DC), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2A0, In::F)],
                    writes: &[(0x31A2A0, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race().shared.car().clutch_damage;
                g.set(add_product(
                    dm(a[0]).gauge_rates[3],
                    dm(a[0]).gauge_step,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 85",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B38C), In::F), (laid!(0x35B5E0), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2A4, In::F)],
                    writes: &[(0x31A2A4, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .differential_damage;
                g.set(add_product(
                    dm(a[0]).gauge_rates[4],
                    dm(a[0]).gauge_step,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 95",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B38C), In::F), (laid!(0x35B5E4), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2E8, In::F)],
                    writes: &[(0x31A2E8, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .electrics_damage;
                g.set(add_product(
                    dm(a[0]).gauge_rates[5],
                    dm(a[0]).gauge_step,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 105",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B38C), In::F), (laid!(0x35B5E8), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2EC, In::F)],
                    writes: &[(0x31A2EC, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .light_damage
                    .float_at(0);
                g.set(add_product(
                    dm(a[0]).gauge_rates[6],
                    dm(a[0]).gauge_step,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 115",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B38C), In::F), (laid!(0x35B5E8), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2F0, In::F)],
                    writes: &[(0x31A2F0, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .light_damage
                    .float_at(1);
                g.set(add_product(
                    dm(a[0]).gauge_rates[6],
                    dm(a[0]).gauge_step,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 125",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B38C), In::F), (laid!(0x35B5E8), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2F4, In::F)],
                    writes: &[(0x31A2F4, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .light_damage
                    .float_at(2);
                g.set(add_product(
                    dm(a[0]).gauge_rates[6],
                    dm(a[0]).gauge_step,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 135",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B38C), In::F), (laid!(0x35B5E8), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2F8, In::F)],
                    writes: &[(0x31A2F8, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .light_damage
                    .float_at(3);
                g.set(add_product(
                    dm(a[0]).gauge_rates[6],
                    dm(a[0]).gauge_step,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 145",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B38C), In::F), (laid!(0x35B5E8), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2FC, In::F)],
                    writes: &[(0x31A2FC, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .light_damage
                    .float_at(4);
                g.set(add_product(
                    dm(a[0]).gauge_rates[6],
                    dm(a[0]).gauge_step,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 155",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B38C), In::F), (laid!(0x35B5E8), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A300, In::F)],
                    writes: &[(0x31A300, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .light_damage
                    .float_at(5);
                g.set(add_product(
                    dm(a[0]).gauge_rates[6],
                    dm(a[0]).gauge_step,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 165",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B38C), In::F), (laid!(0x35B5EC), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A308, In::F)],
                    writes: &[(0x31A308, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .steering_damage;
                g.set(add_product(
                    dm(a[0]).gauge_rates[7],
                    dm(a[0]).gauge_step,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 175",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B38C), In::F), (laid!(0x35B5F0), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A30C, In::F)],
                    writes: &[(0x31A30C, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .suspension_damage
                    .float_at(0);
                g.set(add_product(
                    dm(a[0]).gauge_rates[8],
                    dm(a[0]).gauge_step,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 185",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B38C), In::F), (laid!(0x35B5F0), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A310, In::F)],
                    writes: &[(0x31A310, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .suspension_damage
                    .float_at(1);
                g.set(add_product(
                    dm(a[0]).gauge_rates[8],
                    dm(a[0]).gauge_step,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 195",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B38C), In::F), (laid!(0x35B5F0), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A314, In::F)],
                    writes: &[(0x31A314, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .suspension_damage
                    .float_at(2);
                g.set(add_product(
                    dm(a[0]).gauge_rates[8],
                    dm(a[0]).gauge_step,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 205",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B38C), In::F), (laid!(0x35B5F0), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A318, In::F)],
                    writes: &[(0x31A318, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .suspension_damage
                    .float_at(3);
                g.set(add_product(
                    dm(a[0]).gauge_rates[8],
                    dm(a[0]).gauge_step,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 215",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B38C), In::F),
                        (laid!(0x35B398), In::F),
                        (laid!(0x35B3C0), In::F),
                        (laid!(0x35B5A4), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A298, In::F)],
                    writes: &[(0x31A298, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race().shared.car().turbo_damage;
                g.set(add_scaled_triple(
                    dm(a[0]),
                    dm(a[0]).wear_rates[11],
                    dm(a[0]).engine_over_limit,
                    dm(a[0]).engine_speed_share,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 229",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B370), In::F),
                        (laid!(0x35B398), In::F),
                        (laid!(0x35B5A8), In::F),
                    ],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[(0, In::F)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[(0, In::F)],
                    writes: &[(0, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                gear_wear_from_speed(
                    dm(a[0]),
                    view::<PlainFloat>(a[1], 0).get(),
                    view::<PlainFloat>(a[2], 0),
                );
            },
        },
        Block {
            name: "damage_gauges line 239",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B38C), In::F),
                        (laid!(0x35B3B0), In::F),
                        (laid!(0x35B400), In::F),
                        (laid!(0x35B5F4), In::F),
                    ],
                    writes: &[(laid!(0x35B43C), Out::F)],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A320, In::F)],
                    writes: &[(0x31A320, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| brake_wear_step(race().shared.car(), dm(a[0])),
        },
        Block {
            name: "damage_gauges line 254",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B38C), In::F),
                        (laid!(0x35B3B0), In::F),
                        (laid!(0x35B43C), In::F),
                        (laid!(0x35B5F4), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A324, In::F)],
                    writes: &[(0x31A324, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race().shared.car().brake_wear_b;
                g.set(add_scaled_triple(
                    dm(a[0]),
                    dm(a[0]).gauge_rates[9],
                    dm(a[0]).brake_balance_complement,
                    dm(a[0]).brake_share,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 266",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B38C), In::F),
                        (laid!(0x35B3B4), In::F),
                        (laid!(0x35B5F8), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2E8, In::F)],
                    writes: &[(0x31A2E8, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .electrics_damage;
                g.set(add_triple_product(
                    dm(a[0]).gauge_rates[10],
                    dm(a[0]).on_surface_5,
                    dm(a[0]).gauge_step,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 277",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B38C), In::F),
                        (laid!(0x35B3B4), In::F),
                        (laid!(0x35B5FC), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2EC, In::F)],
                    writes: &[(0x31A2EC, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .light_damage
                    .float_at(0);
                g.set(add_triple_product(
                    dm(a[0]).gauge_rates[11],
                    dm(a[0]).on_surface_5,
                    dm(a[0]).gauge_step,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 288",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B38C), In::F),
                        (laid!(0x35B3B4), In::F),
                        (laid!(0x35B5FC), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2F0, In::F)],
                    writes: &[(0x31A2F0, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .light_damage
                    .float_at(1);
                g.set(add_triple_product(
                    dm(a[0]).gauge_rates[11],
                    dm(a[0]).on_surface_5,
                    dm(a[0]).gauge_step,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 299",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B38C), In::F),
                        (laid!(0x35B3B4), In::F),
                        (laid!(0x35B5FC), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2F4, In::F)],
                    writes: &[(0x31A2F4, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .light_damage
                    .float_at(2);
                g.set(add_triple_product(
                    dm(a[0]).gauge_rates[11],
                    dm(a[0]).on_surface_5,
                    dm(a[0]).gauge_step,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 310",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B38C), In::F),
                        (laid!(0x35B3B4), In::F),
                        (laid!(0x35B5FC), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2F8, In::F)],
                    writes: &[(0x31A2F8, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .light_damage
                    .float_at(3);
                g.set(add_triple_product(
                    dm(a[0]).gauge_rates[11],
                    dm(a[0]).on_surface_5,
                    dm(a[0]).gauge_step,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 321",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B38C), In::F),
                        (laid!(0x35B3B4), In::F),
                        (laid!(0x35B5FC), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2FC, In::F)],
                    writes: &[(0x31A2FC, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .light_damage
                    .float_at(4);
                g.set(add_triple_product(
                    dm(a[0]).gauge_rates[11],
                    dm(a[0]).on_surface_5,
                    dm(a[0]).gauge_step,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 332",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B38C), In::F),
                        (laid!(0x35B3B4), In::F),
                        (laid!(0x35B5FC), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A300, In::F)],
                    writes: &[(0x31A300, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .light_damage
                    .float_at(5);
                g.set(add_triple_product(
                    dm(a[0]).gauge_rates[11],
                    dm(a[0]).on_surface_5,
                    dm(a[0]).gauge_step,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 343",
            operands: vec![Operand::Buffer {
                reads: &[
                    (laid!(0x35B34C), In::F),
                    (laid!(0x35B380), In::F),
                    (laid!(0x35B39C), In::F),
                    (laid!(0x35B3F4), In::F),
                    (laid!(0x35B484), In::F),
                ],
                writes: &[(laid!(0x35B488), Out::F), (laid!(0x35B48C), Out::F)],
            }],
            ulps: 0,
            plain: |a| part_hit_figures(dm(a[0])),
        },
        Block {
            name: "damage_gauges line 355",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B488), In::F), (laid!(0x35B48C), In::F)],
                    writes: &[(laid!(0x35B488), Out::F), (laid!(0x35B48C), Out::F)],
                },
                Operand::Buffer {
                    reads: &[(0, In::F)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[(0, In::F)],
                    writes: &[],
                },
            ],
            ulps: 0,
            plain: |a| {
                scale_part_hit_figures(
                    dm(a[0]),
                    view::<PlainFloat>(a[1], 0).get(),
                    view::<PlainFloat>(a[2], 0).get(),
                );
            },
        },
        Block {
            name: "damage_gauges line 365",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B370), In::F),
                        (laid!(0x35B46C), In::F),
                        (laid!(0x35B488), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A32C, In::F)],
                    writes: &[(0x31A32C, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .tyre_wears
                    .float_at(0);
                g.set(add_triple_product(
                    dm(a[0]).front_slip_wear_factor,
                    dm(a[0]).front_tyre_wear_rate,
                    dm(a[0]).hundredth,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 376",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B370), In::F),
                        (laid!(0x35B46C), In::F),
                        (laid!(0x35B488), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A330, In::F)],
                    writes: &[(0x31A330, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .tyre_wears
                    .float_at(1);
                g.set(add_triple_product(
                    dm(a[0]).front_slip_wear_factor,
                    dm(a[0]).front_tyre_wear_rate,
                    dm(a[0]).hundredth,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 387",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B370), In::F),
                        (laid!(0x35B470), In::F),
                        (laid!(0x35B48C), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A334, In::F)],
                    writes: &[(0x31A334, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .tyre_wears
                    .float_at(2);
                g.set(add_triple_product(
                    dm(a[0]).rear_slip_wear_factor,
                    dm(a[0]).rear_tyre_wear_rate,
                    dm(a[0]).hundredth,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 398",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B370), In::F),
                        (laid!(0x35B470), In::F),
                        (laid!(0x35B48C), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A338, In::F)],
                    writes: &[(0x31A338, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .tyre_wears
                    .float_at(3);
                g.set(add_triple_product(
                    dm(a[0]).rear_slip_wear_factor,
                    dm(a[0]).rear_tyre_wear_rate,
                    dm(a[0]).hundredth,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 409",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B370), In::F),
                        (laid!(0x35B45C), In::F),
                        (laid!(0x35B604), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A290, In::F)],
                    writes: &[(0x31A290, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race().shared.car().engine_damage;
                g.set(add_triple_product(
                    dm(a[0]).engine_speed_wear_factor,
                    dm(a[0]).gauge_rates[13],
                    dm(a[0]).hundredth,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 420",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B370), In::F),
                        (laid!(0x35B45C), In::F),
                        (laid!(0x35B600), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A29C, In::F)],
                    writes: &[(0x31A29C, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race().shared.car().cooling_damage;
                g.set(add_triple_product(
                    dm(a[0]).engine_speed_wear_factor,
                    dm(a[0]).gauge_rates[12],
                    dm(a[0]).hundredth,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 432",
            operands: vec![
                Operand::RaceBlock {
                    reads: &[(0x31A28C, In::F), (0x31A290, In::F)],
                    writes: &[(0x31A290, Out::F)],
                },
                Operand::Buffer {
                    reads: &[(laid!(0x35B370), In::F), (laid!(0x35B608), In::F)],
                    writes: &[],
                },
            ],
            ulps: 0,
            plain: |a| engine_damage_carry(race().shared.car(), dm(a[1])),
        },
        Block {
            name: "damage_gauges line 443",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B370), In::F),
                        (laid!(0x35B464), In::F),
                        (laid!(0x35B60C), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A29C, In::F)],
                    writes: &[(0x31A29C, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race().shared.car().cooling_damage;
                g.set(add_triple_product(
                    dm(a[0]).overheat_excess,
                    dm(a[0]).gauge_rates[15],
                    dm(a[0]).hundredth,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 456",
            operands: vec![
                Operand::RaceBlock {
                    reads: &[(0x31A2A0, In::F)],
                    writes: &[(0x31A2A0, Out::F)],
                },
                Operand::Buffer {
                    reads: &[(laid!(0x35B610), In::F)],
                    writes: &[],
                },
            ],
            ulps: 0,
            plain: |a| add_rate_to_clutch(race().shared.car(), dm(a[1]).gauge_rates[16]),
        },
        Block {
            name: "damage_gauges line 465",
            operands: vec![
                Operand::RaceBlock {
                    reads: &[(0x31A2A0, In::F)],
                    writes: &[(0x31A2A0, Out::F)],
                },
                Operand::Buffer {
                    reads: &[(laid!(0x35B614), In::F)],
                    writes: &[],
                },
            ],
            ulps: 0,
            plain: |a| add_rate_to_clutch(race().shared.car(), dm(a[1]).gauge_rates[17]),
        },
        Block {
            name: "damage_gauges line 476",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0, In::F)],
                    writes: &[(0, Out::F)],
                },
                Operand::Buffer {
                    reads: &[(laid!(0x35B618), In::F)],
                    writes: &[],
                },
            ],
            ulps: 0,
            plain: |a| {
                add_rate_to_gauge(view::<PlainFloat>(a[0], 0), dm(a[1]).gauge_rates[18]);
            },
        },
        Block {
            name: "damage_gauges line 486",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0, In::F)],
                    writes: &[(0, Out::F)],
                },
                Operand::Buffer {
                    reads: &[(laid!(0x35B61C), In::F)],
                    writes: &[],
                },
            ],
            ulps: 0,
            plain: |a| {
                add_rate_to_gauge(view::<PlainFloat>(a[0], 0), dm(a[1]).gauge_rates[19]);
            },
        },
        Block {
            name: "damage_gauges line 493",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B370), In::F),
                        (laid!(0x35B450), In::F),
                        (laid!(0x35B670), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2D0, In::F)],
                    writes: &[(0x31A2D0, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .wheel_hit_damage
                    .float_at(0);
                g.set(add_triple_product(
                    dm(a[0]).roughness_wear_base,
                    dm(a[0]).wheel_roughness_rate,
                    dm(a[0]).hundredth,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 504",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B370), In::F),
                        (laid!(0x35B450), In::F),
                        (laid!(0x35B670), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2D4, In::F)],
                    writes: &[(0x31A2D4, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .wheel_hit_damage
                    .float_at(1);
                g.set(add_triple_product(
                    dm(a[0]).roughness_wear_base,
                    dm(a[0]).wheel_roughness_rate,
                    dm(a[0]).hundredth,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 515",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B370), In::F),
                        (laid!(0x35B450), In::F),
                        (laid!(0x35B670), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2D8, In::F)],
                    writes: &[(0x31A2D8, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .wheel_hit_damage
                    .float_at(2);
                g.set(add_triple_product(
                    dm(a[0]).roughness_wear_base,
                    dm(a[0]).wheel_roughness_rate,
                    dm(a[0]).hundredth,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 526",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B370), In::F),
                        (laid!(0x35B450), In::F),
                        (laid!(0x35B670), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2DC, In::F)],
                    writes: &[(0x31A2DC, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .wheel_hit_damage
                    .float_at(3);
                g.set(add_triple_product(
                    dm(a[0]).roughness_wear_base,
                    dm(a[0]).wheel_roughness_rate,
                    dm(a[0]).hundredth,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 537",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B370), In::F),
                        (laid!(0x35B458), In::F),
                        (laid!(0x35B678), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A30C, In::F)],
                    writes: &[(0x31A30C, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .suspension_damage
                    .float_at(0);
                g.set(add_triple_product(
                    dm(a[0]).suspension_roughness_wear,
                    dm(a[0]).figures[0],
                    dm(a[0]).hundredth,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 548",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B370), In::F),
                        (laid!(0x35B458), In::F),
                        (laid!(0x35B678), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A310, In::F)],
                    writes: &[(0x31A310, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .suspension_damage
                    .float_at(1);
                g.set(add_triple_product(
                    dm(a[0]).suspension_roughness_wear,
                    dm(a[0]).figures[0],
                    dm(a[0]).hundredth,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 559",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B370), In::F),
                        (laid!(0x35B458), In::F),
                        (laid!(0x35B678), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A314, In::F)],
                    writes: &[(0x31A314, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .suspension_damage
                    .float_at(2);
                g.set(add_triple_product(
                    dm(a[0]).suspension_roughness_wear,
                    dm(a[0]).figures[0],
                    dm(a[0]).hundredth,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 570",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B370), In::F),
                        (laid!(0x35B458), In::F),
                        (laid!(0x35B678), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A318, In::F)],
                    writes: &[(0x31A318, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .suspension_damage
                    .float_at(3);
                g.set(add_triple_product(
                    dm(a[0]).suspension_roughness_wear,
                    dm(a[0]).figures[0],
                    dm(a[0]).hundredth,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 581",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B370), In::F),
                        (laid!(0x35B454), In::F),
                        (laid!(0x35B67C), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2E0, In::F)],
                    writes: &[(0x31A2E0, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .body_hit_damage;
                g.set(add_triple_product(
                    dm(a[0]).body_roughness_wear,
                    dm(a[0]).figures[1],
                    dm(a[0]).hundredth,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 592",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B370), In::F),
                        (laid!(0x35B454), In::F),
                        (laid!(0x35B680), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A308, In::F)],
                    writes: &[(0x31A308, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .steering_damage;
                g.set(add_triple_product(
                    dm(a[0]).body_roughness_wear,
                    dm(a[0]).figures[2],
                    dm(a[0]).hundredth,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 603",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B370), In::F),
                        (laid!(0x35B454), In::F),
                        (laid!(0x35B684), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A290, In::F)],
                    writes: &[(0x31A290, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race().shared.car().engine_damage;
                g.set(add_triple_product(
                    dm(a[0]).body_roughness_wear,
                    dm(a[0]).figures[3],
                    dm(a[0]).hundredth,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 614",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B370), In::F),
                        (laid!(0x35B454), In::F),
                        (laid!(0x35B688), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A294, In::F)],
                    writes: &[(0x31A294, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race().shared.car().exhaust_damage;
                g.set(add_triple_product(
                    dm(a[0]).body_roughness_wear,
                    dm(a[0]).figures[4],
                    dm(a[0]).hundredth,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 625",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B370), In::F),
                        (laid!(0x35B454), In::F),
                        (laid!(0x35B68C), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2A8, In::F)],
                    writes: &[(0x31A2A8, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .gear_wear
                    .float_at(0);
                g.set(add_triple_product(
                    dm(a[0]).body_roughness_wear,
                    dm(a[0]).figures[5],
                    dm(a[0]).hundredth,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 636",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B370), In::F),
                        (laid!(0x35B454), In::F),
                        (laid!(0x35B68C), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2B0, In::F)],
                    writes: &[(0x31A2B0, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .gear_wear
                    .float_at(2);
                g.set(add_triple_product(
                    dm(a[0]).body_roughness_wear,
                    dm(a[0]).figures[5],
                    dm(a[0]).hundredth,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 647",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B370), In::F),
                        (laid!(0x35B454), In::F),
                        (laid!(0x35B68C), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2B4, In::F)],
                    writes: &[(0x31A2B4, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .gear_wear
                    .float_at(3);
                g.set(add_triple_product(
                    dm(a[0]).body_roughness_wear,
                    dm(a[0]).figures[5],
                    dm(a[0]).hundredth,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 658",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B370), In::F),
                        (laid!(0x35B454), In::F),
                        (laid!(0x35B68C), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2B8, In::F)],
                    writes: &[(0x31A2B8, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .gear_wear
                    .float_at(4);
                g.set(add_triple_product(
                    dm(a[0]).body_roughness_wear,
                    dm(a[0]).figures[5],
                    dm(a[0]).hundredth,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 669",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B370), In::F),
                        (laid!(0x35B454), In::F),
                        (laid!(0x35B68C), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2BC, In::F)],
                    writes: &[(0x31A2BC, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .gear_wear
                    .float_at(5);
                g.set(add_triple_product(
                    dm(a[0]).body_roughness_wear,
                    dm(a[0]).figures[5],
                    dm(a[0]).hundredth,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 680",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B370), In::F),
                        (laid!(0x35B454), In::F),
                        (laid!(0x35B68C), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2C0, In::F)],
                    writes: &[(0x31A2C0, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .gear_wear
                    .float_at(6);
                g.set(add_triple_product(
                    dm(a[0]).body_roughness_wear,
                    dm(a[0]).figures[5],
                    dm(a[0]).hundredth,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 691",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B370), In::F),
                        (laid!(0x35B454), In::F),
                        (laid!(0x35B68C), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2C4, In::F)],
                    writes: &[(0x31A2C4, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .gear_wear
                    .float_at(7);
                g.set(add_triple_product(
                    dm(a[0]).body_roughness_wear,
                    dm(a[0]).figures[5],
                    dm(a[0]).hundredth,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 702",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B370), In::F),
                        (laid!(0x35B454), In::F),
                        (laid!(0x35B68C), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2C8, In::F)],
                    writes: &[(0x31A2C8, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .gear_wear
                    .float_at(8);
                g.set(add_triple_product(
                    dm(a[0]).body_roughness_wear,
                    dm(a[0]).figures[5],
                    dm(a[0]).hundredth,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 713",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B370), In::F),
                        (laid!(0x35B454), In::F),
                        (laid!(0x35B690), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2A4, In::F)],
                    writes: &[(0x31A2A4, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .differential_damage;
                g.set(add_triple_product(
                    dm(a[0]).body_roughness_wear,
                    dm(a[0]).figures[6],
                    dm(a[0]).hundredth,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 725",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6AC), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2E0, In::F)],
                    writes: &[(0x31A2E0, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .body_hit_damage;
                g.set(add_product(
                    dm(a[0]).hit_size,
                    dm(a[0]).figures[13],
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 737",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F)],
                    writes: &[(laid!(0x35B3C8), Out::F)],
                },
                Operand::RaceBlock {
                    reads: &[(0x31C708, In::F)],
                    writes: &[],
                },
            ],
            ulps: 0,
            plain: |a| scale_hit_size(race().shared.work.get(), dm(a[0])),
        },
        Block {
            name: "damage_gauges line 745",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6D8), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2EC, In::F)],
                    writes: &[(0x31A2EC, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .light_damage
                    .float_at(0);
                g.set(add_product(
                    dm(a[0]).hit_size,
                    dm(a[0]).figures[24],
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 757",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6D8), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2F0, In::F)],
                    writes: &[(0x31A2F0, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .light_damage
                    .float_at(1);
                g.set(add_product(
                    dm(a[0]).hit_size,
                    dm(a[0]).figures[24],
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 769",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6D8), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2F4, In::F)],
                    writes: &[(0x31A2F4, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .light_damage
                    .float_at(2);
                g.set(add_product(
                    dm(a[0]).hit_size,
                    dm(a[0]).figures[24],
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 781",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6D8), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2F8, In::F)],
                    writes: &[(0x31A2F8, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .light_damage
                    .float_at(3);
                g.set(add_product(
                    dm(a[0]).hit_size,
                    dm(a[0]).figures[24],
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 793",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6DC), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2FC, In::F)],
                    writes: &[(0x31A2FC, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .light_damage
                    .float_at(4);
                g.set(add_product(
                    dm(a[0]).hit_size,
                    dm(a[0]).figures[25],
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 805",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6DC), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2FC, In::F)],
                    writes: &[(0x31A2FC, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .light_damage
                    .float_at(4);
                g.set(add_product(
                    dm(a[0]).hit_size,
                    dm(a[0]).figures[25],
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 817",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6DC), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A300, In::F)],
                    writes: &[(0x31A300, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .light_damage
                    .float_at(5);
                g.set(add_product(
                    dm(a[0]).hit_size,
                    dm(a[0]).figures[25],
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 829",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6DC), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A300, In::F)],
                    writes: &[(0x31A300, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .light_damage
                    .float_at(5);
                g.set(add_product(
                    dm(a[0]).hit_size,
                    dm(a[0]).figures[25],
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 840",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6D4), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A308, In::F)],
                    writes: &[(0x31A308, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .steering_damage;
                g.set(add_product(
                    dm(a[0]).hit_size,
                    dm(a[0]).figures[23],
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 850",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6D0), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2E8, In::F)],
                    writes: &[(0x31A2E8, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .electrics_damage;
                g.set(add_product(
                    dm(a[0]).hit_size,
                    dm(a[0]).figures[22],
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 860",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6BC), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A298, In::F)],
                    writes: &[(0x31A298, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race().shared.car().turbo_damage;
                g.set(add_product(
                    dm(a[0]).hit_size,
                    dm(a[0]).figures[17],
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 870",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6C0), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2A0, In::F)],
                    writes: &[(0x31A2A0, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race().shared.car().clutch_damage;
                g.set(add_product(
                    dm(a[0]).hit_size,
                    dm(a[0]).figures[18],
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 880",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6C4), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2A4, In::F)],
                    writes: &[(0x31A2A4, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .differential_damage;
                g.set(add_product(
                    dm(a[0]).hit_size,
                    dm(a[0]).figures[19],
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 890",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6C8), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2A8, In::F)],
                    writes: &[(0x31A2A8, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .gear_wear
                    .float_at(0);
                g.set(add_product(
                    dm(a[0]).hit_size,
                    dm(a[0]).figures[20],
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 900",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6C8), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2AC, In::F)],
                    writes: &[(0x31A2AC, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .gear_wear
                    .float_at(1);
                g.set(add_product(
                    dm(a[0]).hit_size,
                    dm(a[0]).figures[20],
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 910",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6C8), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2B0, In::F)],
                    writes: &[(0x31A2B0, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .gear_wear
                    .float_at(2);
                g.set(add_product(
                    dm(a[0]).hit_size,
                    dm(a[0]).figures[20],
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 920",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6C8), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2B4, In::F)],
                    writes: &[(0x31A2B4, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .gear_wear
                    .float_at(3);
                g.set(add_product(
                    dm(a[0]).hit_size,
                    dm(a[0]).figures[20],
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 930",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6C8), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2B8, In::F)],
                    writes: &[(0x31A2B8, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .gear_wear
                    .float_at(4);
                g.set(add_product(
                    dm(a[0]).hit_size,
                    dm(a[0]).figures[20],
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 940",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6C8), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2BC, In::F)],
                    writes: &[(0x31A2BC, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .gear_wear
                    .float_at(5);
                g.set(add_product(
                    dm(a[0]).hit_size,
                    dm(a[0]).figures[20],
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 950",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6C8), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2C0, In::F)],
                    writes: &[(0x31A2C0, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .gear_wear
                    .float_at(6);
                g.set(add_product(
                    dm(a[0]).hit_size,
                    dm(a[0]).figures[20],
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 960",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6C8), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2C4, In::F)],
                    writes: &[(0x31A2C4, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .gear_wear
                    .float_at(7);
                g.set(add_product(
                    dm(a[0]).hit_size,
                    dm(a[0]).figures[20],
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 970",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6C8), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2C8, In::F)],
                    writes: &[(0x31A2C8, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .gear_wear
                    .float_at(8);
                g.set(add_product(
                    dm(a[0]).hit_size,
                    dm(a[0]).figures[20],
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 982",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6CC), In::F)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[(0, In::F)],
                    writes: &[(0, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| add_hit_to_gear_gauge(dm(a[0]), view::<PlainFloat>(a[1], 0)),
        },
        Block {
            name: "damage_gauges line 991",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6E8), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2D0, In::F)],
                    writes: &[(0x31A2D0, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .wheel_hit_damage
                    .float_at(0);
                g.set(add_product(
                    dm(a[0]).hit_size,
                    dm(a[0]).figures[28],
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 1003",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6E8), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2D4, In::F)],
                    writes: &[(0x31A2D4, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .wheel_hit_damage
                    .float_at(1);
                g.set(add_product(
                    dm(a[0]).hit_size,
                    dm(a[0]).figures[28],
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 1015",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6E8), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2D8, In::F)],
                    writes: &[(0x31A2D8, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .wheel_hit_damage
                    .float_at(2);
                g.set(add_product(
                    dm(a[0]).hit_size,
                    dm(a[0]).figures[28],
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 1027",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6E8), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A2DC, In::F)],
                    writes: &[(0x31A2DC, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .wheel_hit_damage
                    .float_at(3);
                g.set(add_product(
                    dm(a[0]).hit_size,
                    dm(a[0]).figures[28],
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 1039",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6F0), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A30C, In::F)],
                    writes: &[(0x31A30C, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .suspension_damage
                    .float_at(0);
                g.set(add_product(
                    dm(a[0]).hit_size,
                    dm(a[0]).figures[30],
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 1051",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6F0), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A310, In::F)],
                    writes: &[(0x31A310, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .suspension_damage
                    .float_at(1);
                g.set(add_product(
                    dm(a[0]).hit_size,
                    dm(a[0]).figures[30],
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 1063",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6F0), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A314, In::F)],
                    writes: &[(0x31A314, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .suspension_damage
                    .float_at(2);
                g.set(add_product(
                    dm(a[0]).hit_size,
                    dm(a[0]).figures[30],
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 1075",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6F0), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A318, In::F)],
                    writes: &[(0x31A318, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .suspension_damage
                    .float_at(3);
                g.set(add_product(
                    dm(a[0]).hit_size,
                    dm(a[0]).figures[30],
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 1087",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6E4), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A29C, In::F)],
                    writes: &[(0x31A29C, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race().shared.car().cooling_damage;
                g.set(add_product(
                    dm(a[0]).hit_size,
                    dm(a[0]).figures[27],
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 1099",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6E4), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A29C, In::F)],
                    writes: &[(0x31A29C, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race().shared.car().cooling_damage;
                g.set(add_product(
                    dm(a[0]).hit_size,
                    dm(a[0]).figures[27],
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 1111",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6E0), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A290, In::F)],
                    writes: &[(0x31A290, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race().shared.car().engine_damage;
                g.set(add_product(
                    dm(a[0]).hit_size,
                    dm(a[0]).figures[26],
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 1123",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6E0), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A290, In::F)],
                    writes: &[(0x31A290, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race().shared.car().engine_damage;
                g.set(add_product(
                    dm(a[0]).hit_size,
                    dm(a[0]).figures[26],
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 1135",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6EC), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A320, In::F)],
                    writes: &[(0x31A320, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race().shared.car().brake_wear_a;
                g.set(add_product(
                    dm(a[0]).hit_size,
                    dm(a[0]).figures[29],
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 1147",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6EC), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A320, In::F)],
                    writes: &[(0x31A320, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race().shared.car().brake_wear_a;
                g.set(add_product(
                    dm(a[0]).hit_size,
                    dm(a[0]).figures[29],
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 1159",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6EC), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A324, In::F)],
                    writes: &[(0x31A324, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race().shared.car().brake_wear_b;
                g.set(add_product(
                    dm(a[0]).hit_size,
                    dm(a[0]).figures[29],
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 1171",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B3C8), In::F), (laid!(0x35B6EC), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A324, In::F)],
                    writes: &[(0x31A324, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race().shared.car().brake_wear_b;
                g.set(add_product(
                    dm(a[0]).hit_size,
                    dm(a[0]).figures[29],
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 1183",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B370), In::F),
                        (laid!(0x35B3CC), In::F),
                        (laid!(0x35B6B0), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A30C, In::F)],
                    writes: &[(0x31A30C, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .suspension_damage
                    .float_at(0);
                g.set(add_triple_product(
                    dm(a[0]).wheel_loads[0],
                    dm(a[0]).figures[14],
                    dm(a[0]).hundredth,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 1194",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B370), In::F),
                        (laid!(0x35B3D0), In::F),
                        (laid!(0x35B6B0), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A310, In::F)],
                    writes: &[(0x31A310, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .suspension_damage
                    .float_at(1);
                g.set(add_triple_product(
                    dm(a[0]).wheel_loads[1],
                    dm(a[0]).figures[14],
                    dm(a[0]).hundredth,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 1205",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B370), In::F),
                        (laid!(0x35B3D4), In::F),
                        (laid!(0x35B6B0), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A314, In::F)],
                    writes: &[(0x31A314, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .suspension_damage
                    .float_at(2);
                g.set(add_triple_product(
                    dm(a[0]).wheel_loads[2],
                    dm(a[0]).figures[14],
                    dm(a[0]).hundredth,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 1216",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B370), In::F),
                        (laid!(0x35B3D8), In::F),
                        (laid!(0x35B6B0), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A318, In::F)],
                    writes: &[(0x31A318, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .suspension_damage
                    .float_at(3);
                g.set(add_triple_product(
                    dm(a[0]).wheel_loads[3],
                    dm(a[0]).figures[14],
                    dm(a[0]).hundredth,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 1227",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B370), In::F),
                        (laid!(0x35B3DC), In::F),
                        (laid!(0x35B6B4), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A294, In::F)],
                    writes: &[(0x31A294, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race().shared.car().exhaust_damage;
                g.set(add_triple_product(
                    dm(a[0]).wheel_load_mean,
                    dm(a[0]).figures[15],
                    dm(a[0]).hundredth,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 1238",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B370), In::F),
                        (laid!(0x35B3DC), In::F),
                        (laid!(0x35B6B8), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A308, In::F)],
                    writes: &[(0x31A308, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .steering_damage;
                g.set(add_triple_product(
                    dm(a[0]).wheel_load_mean,
                    dm(a[0]).figures[16],
                    dm(a[0]).hundredth,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_gauges line 1249",
            operands: vec![
                Operand::RaceBlock {
                    reads: &[
                        (0x31A2A8, In::F),
                        (0x31A2B0, In::F),
                        (0x31A2B4, In::F),
                        (0x31A2B8, In::F),
                        (0x31A2BC, In::F),
                        (0x31A2C0, In::F),
                        (0x31A2C4, In::F),
                        (0x31A2C8, In::F),
                        (0x31A2D0, In::F),
                        (0x31A2D4, In::F),
                        (0x31A2D8, In::F),
                        (0x31A2DC, In::F),
                        (0x31A2E0, In::F),
                        (0x31A2EC, In::F),
                        (0x31A2F0, In::F),
                        (0x31A2F4, In::F),
                        (0x31A2F8, In::F),
                        (0x31A2FC, In::F),
                        (0x31A300, In::F),
                        (0x31A30C, In::F),
                        (0x31A310, In::F),
                        (0x31A314, In::F),
                        (0x31A318, In::F),
                        (0x31A320, In::F),
                        (0x31A324, In::F),
                        (0x31A32C, In::F),
                        (0x31A330, In::F),
                        (0x31A334, In::F),
                        (0x31A338, In::F),
                        (0x31A420, In::F),
                        (0x31A424, In::F),
                        (0x31A428, In::F),
                        (0x31A42C, In::F),
                    ],
                    writes: &[
                        (0x31A2CC, Out::F),
                        (0x31A2E4, Out::F),
                        (0x31A304, Out::F),
                        (0x31A31C, Out::F),
                        (0x31A328, Out::F),
                        (0x31A33C, Out::F),
                        (0x31A430, Out::F),
                    ],
                },
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B344), In::F),
                        (laid!(0x35B360), In::F),
                        (laid!(0x35B364), In::F),
                        (laid!(0x35B368), In::F),
                        (laid!(0x35B440), In::F),
                        (laid!(0x35B444), In::F),
                    ],
                    writes: &[(laid!(0x35B440), Out::F), (laid!(0x35B444), Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| gauge_totals(race().shared.car(), dm(a[1])),
        },
    ]
}

#[test]
fn x87_blocks_matches_the_assembly() {
    // The plain code rounds to nearest, the mode the damage model always runs in.
    sparse_at("physics.damage_gauges", &blocks(), 900, 100, &[0x027F]);
}
