//! `physics::damage_wear` (0x4ABEDF): the wear of this step (`physics/damage_model.rs`).
//!
//! The clock into `DamageModel.clock`; table lerps of the speed shares into `DamageModel.engine_speed_wear_factor` ..; the
//! heat, damper, brake-fade and engine-heat gauges raised and lowered by their rates times
//! the shares and `wear_step`, each held 0 ..= `gauge_top`; the hit figures `DamageModel.roughness_wear_base` ..
//! `DamageModel.suspension_roughness_wear`; and `DamageModel.wear_figure` from the table `DamageModel.wear_table[0]` (row `DamageModel.tyre_kind`, two
//! columns through `DamageModel.wear_columns`, as the original indexes them) blended by
//! `DamageModel.surface_blend`, times `DamageModel.wear_figure_scale`.
//!
//! Caller: `physics::damage_step`.

use crate::errors::BadInput;
use crate::physics::car_store::Car;
use crate::physics::damage_gauges::raise;
use crate::physics::damage_model::{store, value};
use crate::physics::flow::{hold_low, table_lerp};
use crate::physics::store::DamageModel;
use crate::session::PlainFloat;

/// The wear of this step: the gauges raised or lowered by their rates times the shares,
/// the lerps of the speed shares, and the hit figures.
pub fn steps(car: &Car, damage: &DamageModel) -> Result<(), BadInput> {
    damage.clock.set(car.noise_word.get());
    table_lerp(
        &damage.engine_speed_share,
        &damage.lerp_tables[0],
        &damage.gauge_low,
        &damage.engine_speed_wear_factor,
    );
    for k in 0..2 {
        raise(damage, &car.brake_heat[k], |g| {
            add_wear(damage, &damage.wear_rates[3], &damage.brake_share, g, g);
        });
    }
    for k in 0..2 {
        raise(damage, &car.brake_heat[k], |g| {
            subtract_rate(damage, &damage.wear_rates[4], g, g);
        });
    }
    let extrapolated = [
        &damage.drive_a_extrapolated_share,
        &damage.drive_b_extrapolated_share,
    ];
    for (damper, share) in car.damper_wear.iter().zip(extrapolated) {
        raise(damage, damper, |g| {
            add_wear(damage, &damage.wear_rates[5], share, g, g);
        });
    }
    for k in 0..2 {
        raise(damage, &car.damper_wear[k], |g| {
            subtract_rate(damage, &damage.wear_rates[6], g, g);
        });
    }
    raise(damage, &car.brake_fade, |g| {
        add_wear(damage, &damage.wear_rates[7], &damage.on_surface_5, g, g);
    });
    raise(damage, &car.brake_fade, |g| {
        subtract_rate(damage, &damage.wear_rates[8], g, g);
    });
    raise(damage, &car.brake_fade, |g| {
        subtract_wear(damage, &damage.wear_rates[9], &damage.brake_share, g, g);
    });
    table_lerp(
        &car.cooling_damage,
        &damage.lerp_tables[1],
        &damage.gauge_low,
        &damage.cooling_capacity,
    );
    lerp_gap(damage);
    hold_low(&damage.overheat_excess, &damage.gauge_low);
    raise(damage, &car.engine_heat, |g| {
        add_wear(damage, &damage.wear_rates[0], &damage.overheat_excess, g, g);
    });
    raise(damage, &car.engine_heat, |g| {
        subtract_rate(damage, &damage.wear_rates[1], g, g);
    });
    raise(damage, &car.engine_heat, |g| {
        subtract_wear(damage, &damage.wear_rates[2], &damage.speed_share, g, g);
    });
    table_lerp(
        &car.stiffness_mean,
        &damage.lerp_tables_b[2],
        &damage.gauge_low,
        &damage.stiffness_body_factor,
    );
    square_into(&damage.stiffness_body_factor, &damage.stiffness_body_factor);
    table_lerp(
        &car.stiffness_mean,
        &damage.lerp_tables_b[0],
        &damage.gauge_low,
        &damage.stiffness_suspension_factor,
    );
    square_into(
        &damage.stiffness_suspension_factor,
        &damage.stiffness_suspension_factor,
    );
    table_lerp(
        &car.ride_height_mean,
        &damage.lerp_tables_b[1],
        &damage.gauge_low,
        &damage.ride_height_body_factor,
    );
    square_into(
        &damage.ride_height_body_factor,
        &damage.ride_height_body_factor,
    );
    table_lerp(
        &damage.drive_a_slip_share,
        &damage.lerp_tables[3],
        &damage.gauge_low,
        &damage.front_slip_wear_factor,
    );
    table_lerp(
        &damage.drive_b_slip_share,
        &damage.lerp_tables[3],
        &damage.gauge_low,
        &damage.rear_slip_wear_factor,
    );
    hit_figures(damage);
    // 0x4AC7C9: two figures of the table `DamageModel.wear_table` (row `DamageModel.tyre_kind`), blended by
    // `DamageModel.surface_blend`, x `DamageModel.wear_figure_scale`. Their columns are the surface's pair of
    // `DamageModel.wear_columns`: the first column, then the second. The original reads
    // the second at the first column's pair instead of the surface's (fixed: see
    // wiki/Fixed-bugs.md); its weight, `surface_blend`, is 0 on every shipped stage.
    let tyre_kind = damage.tyre_kind.get();
    let pair = &damage.wear_columns[(damage.surface.get() & 0xF) as usize];
    let first = damage.wear_figure_entry(tyre_kind, pair[0].get())?;
    let second = damage.wear_figure_entry(tyre_kind, pair[1].get())?;
    blend_wear_figures(first as i32, damage, second as i32);
    Ok(())
}

// ---- the arithmetic ----

/// `to = rate * share * wear_step + from`.
fn add_wear(
    m: &DamageModel,
    rate: &PlainFloat,
    share: &PlainFloat,
    from: &PlainFloat,
    to: &PlainFloat,
) {
    store(
        to,
        value(rate) * value(share) * value(&m.wear_step) + value(from),
    );
}

/// `to = from - rate * wear_step`.
fn subtract_rate(m: &DamageModel, rate: &PlainFloat, from: &PlainFloat, to: &PlainFloat) {
    store(to, -(value(rate) * value(&m.wear_step)) + value(from));
}

/// `to = from - rate * share * wear_step`.
fn subtract_wear(
    m: &DamageModel,
    rate: &PlainFloat,
    share: &PlainFloat,
    from: &PlainFloat,
    to: &PlainFloat,
) {
    store(
        to,
        -(value(rate) * value(share) * value(&m.wear_step)) + value(from),
    );
}

/// How far the engine's speed share is past the cooling capacity.
fn lerp_gap(m: &DamageModel) {
    store(
        &m.overheat_excess,
        value(&m.engine_speed_share) - value(&m.cooling_capacity),
    );
}

/// `to = from * from`.
fn square_into(from: &PlainFloat, to: &PlainFloat) {
    let x = value(from);
    store(to, x * x);
}

/// The three hit figures, each the product of the surface wear, the speed share and the
/// wheels-down switch with further factors; the third is the second times the suspension's.
fn hit_figures(m: &DamageModel) {
    let base = value(&m.point_surface_copy) * value(&m.speed_share) * value(&m.figures[32]);
    store(
        &m.body_roughness_wear,
        base * value(&m.stiffness_body_factor)
            * value(&m.ride_height_body_factor)
            * value(&m.wheels_down),
    );
    let roughness = base * value(&m.wheels_down);
    store(&m.roughness_wear_base, roughness);
    store(
        &m.suspension_roughness_wear,
        roughness * value(&m.stiffness_suspension_factor),
    );
}

/// The wear figure: the two table figures blended by `surface_blend`, times the scale.
fn blend_wear_figures(first: i32, m: &DamageModel, second: i32) {
    let blend = value(&m.surface_blend);
    store(
        &m.wear_figure,
        (f64::from(first) * (1.0 - blend) + f64::from(second) * blend)
            * value(&m.wear_figure_scale),
    );
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::physics::store::laid;

    /// The damage model as the test's buffer holds it (the buffer's base plus the RVA of
    /// 0x75B340).
    fn dm(base: usize) -> &'static DamageModel {
        unsafe {
            &*(crate::test_support::vectors::block_vectors::at(base + 0x35B340)
                as *const DamageModel)
        }
    }
    use crate::scene::x87_mem::view;
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse_at};

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "damage_wear line 36",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (laid!(0x35B390), In::F),
                            (laid!(0x35B3B0), In::F),
                            (laid!(0x35B584), In::F),
                        ],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A284, In::F)],
                        writes: &[(0x31A284, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    add_wear(
                        dm(a[0]),
                        &dm(a[0]).wear_rates[3],
                        &dm(a[0]).brake_share,
                        &crate::race::owner::race().shared.car().brake_heat[0],
                        &crate::race::owner::race().shared.car().brake_heat[0],
                    );
                },
            },
            Block {
                name: "damage_wear line 47",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (laid!(0x35B390), In::F),
                            (laid!(0x35B3B0), In::F),
                            (laid!(0x35B584), In::F),
                        ],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A288, In::F)],
                        writes: &[(0x31A288, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    add_wear(
                        dm(a[0]),
                        &dm(a[0]).wear_rates[3],
                        &dm(a[0]).brake_share,
                        &crate::race::owner::race().shared.car().brake_heat[1],
                        &crate::race::owner::race().shared.car().brake_heat[1],
                    );
                },
            },
            Block {
                name: "damage_wear line 58",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(laid!(0x35B390), In::F), (laid!(0x35B588), In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A284, In::F)],
                        writes: &[(0x31A284, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    subtract_rate(
                        dm(a[0]),
                        &dm(a[0]).wear_rates[4],
                        &crate::race::owner::race().shared.car().brake_heat[0],
                        &crate::race::owner::race().shared.car().brake_heat[0],
                    );
                },
            },
            Block {
                name: "damage_wear line 69",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(laid!(0x35B390), In::F), (laid!(0x35B588), In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A288, In::F)],
                        writes: &[(0x31A288, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    subtract_rate(
                        dm(a[0]),
                        &dm(a[0]).wear_rates[4],
                        &crate::race::owner::race().shared.car().brake_heat[1],
                        &crate::race::owner::race().shared.car().brake_heat[1],
                    );
                },
            },
            Block {
                name: "damage_wear line 80",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (laid!(0x35B390), In::F),
                            (laid!(0x35B3A8), In::F),
                            (laid!(0x35B58C), In::F),
                        ],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A27C, In::F)],
                        writes: &[(0x31A27C, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    add_wear(
                        dm(a[0]),
                        &dm(a[0]).wear_rates[5],
                        &dm(a[0]).drive_a_extrapolated_share,
                        &crate::race::owner::race().shared.car().damper_wear[0],
                        &crate::race::owner::race().shared.car().damper_wear[0],
                    );
                },
            },
            Block {
                name: "damage_wear line 91",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (laid!(0x35B390), In::F),
                            (laid!(0x35B3AC), In::F),
                            (laid!(0x35B58C), In::F),
                        ],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A280, In::F)],
                        writes: &[(0x31A280, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    add_wear(
                        dm(a[0]),
                        &dm(a[0]).wear_rates[5],
                        &dm(a[0]).drive_b_extrapolated_share,
                        &crate::race::owner::race().shared.car().damper_wear[1],
                        &crate::race::owner::race().shared.car().damper_wear[1],
                    );
                },
            },
            Block {
                name: "damage_wear line 102",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(laid!(0x35B390), In::F), (laid!(0x35B590), In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A27C, In::F)],
                        writes: &[(0x31A27C, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    subtract_rate(
                        dm(a[0]),
                        &dm(a[0]).wear_rates[6],
                        &crate::race::owner::race().shared.car().damper_wear[0],
                        &crate::race::owner::race().shared.car().damper_wear[0],
                    );
                },
            },
            Block {
                name: "damage_wear line 113",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(laid!(0x35B390), In::F), (laid!(0x35B590), In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A280, In::F)],
                        writes: &[(0x31A280, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    subtract_rate(
                        dm(a[0]),
                        &dm(a[0]).wear_rates[6],
                        &crate::race::owner::race().shared.car().damper_wear[1],
                        &crate::race::owner::race().shared.car().damper_wear[1],
                    );
                },
            },
            Block {
                name: "damage_wear line 124",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (laid!(0x35B390), In::F),
                            (laid!(0x35B3B4), In::F),
                            (laid!(0x35B594), In::F),
                        ],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A278, In::F)],
                        writes: &[(0x31A278, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    add_wear(
                        dm(a[0]),
                        &dm(a[0]).wear_rates[7],
                        &dm(a[0]).on_surface_5,
                        &crate::race::owner::race().shared.car().brake_fade,
                        &crate::race::owner::race().shared.car().brake_fade,
                    );
                },
            },
            Block {
                name: "damage_wear line 135",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(laid!(0x35B390), In::F), (laid!(0x35B598), In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A278, In::F)],
                        writes: &[(0x31A278, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    subtract_rate(
                        dm(a[0]),
                        &dm(a[0]).wear_rates[8],
                        &crate::race::owner::race().shared.car().brake_fade,
                        &crate::race::owner::race().shared.car().brake_fade,
                    );
                },
            },
            Block {
                name: "damage_wear line 146",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (laid!(0x35B390), In::F),
                            (laid!(0x35B3B0), In::F),
                            (laid!(0x35B59C), In::F),
                        ],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A278, In::F)],
                        writes: &[(0x31A278, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    subtract_wear(
                        dm(a[0]),
                        &dm(a[0]).wear_rates[9],
                        &dm(a[0]).brake_share,
                        &crate::race::owner::race().shared.car().brake_fade,
                        &crate::race::owner::race().shared.car().brake_fade,
                    );
                },
            },
            Block {
                name: "damage_wear line 159",
                operands: vec![Operand::Buffer {
                    reads: &[(laid!(0x35B398), In::F), (laid!(0x35B460), In::F)],
                    writes: &[(laid!(0x35B464), Out::F)],
                }],
                ulps: 0,
                plain: |a| lerp_gap(dm(a[0])),
            },
            Block {
                name: "damage_wear line 166",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (laid!(0x35B390), In::F),
                            (laid!(0x35B464), In::F),
                            (laid!(0x35B578), In::F),
                        ],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A28C, In::F)],
                        writes: &[(0x31A28C, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    add_wear(
                        dm(a[0]),
                        &dm(a[0]).wear_rates[0],
                        &dm(a[0]).overheat_excess,
                        &crate::race::owner::race().shared.car().engine_heat,
                        &crate::race::owner::race().shared.car().engine_heat,
                    );
                },
            },
            Block {
                name: "damage_wear line 177",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(laid!(0x35B390), In::F), (laid!(0x35B57C), In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A28C, In::F)],
                        writes: &[(0x31A28C, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    subtract_rate(
                        dm(a[0]),
                        &dm(a[0]).wear_rates[1],
                        &crate::race::owner::race().shared.car().engine_heat,
                        &crate::race::owner::race().shared.car().engine_heat,
                    );
                },
            },
            Block {
                name: "damage_wear line 188",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (laid!(0x35B390), In::F),
                            (laid!(0x35B39C), In::F),
                            (laid!(0x35B580), In::F),
                        ],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x31A28C, In::F)],
                        writes: &[(0x31A28C, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    subtract_wear(
                        dm(a[0]),
                        &dm(a[0]).wear_rates[2],
                        &dm(a[0]).speed_share,
                        &crate::race::owner::race().shared.car().engine_heat,
                        &crate::race::owner::race().shared.car().engine_heat,
                    );
                },
            },
            Block {
                name: "damage_wear line 201",
                operands: vec![Operand::Buffer {
                    reads: &[(laid!(0x35B44C), In::F)],
                    writes: &[(laid!(0x35B44C), Out::F)],
                }],
                ulps: 0,
                plain: |a| {
                    square_into(
                        &dm(a[0]).stiffness_body_factor,
                        &dm(a[0]).stiffness_body_factor,
                    );
                },
            },
            Block {
                name: "damage_wear line 208",
                operands: vec![Operand::Buffer {
                    reads: &[(laid!(0x35B448), In::F)],
                    writes: &[(laid!(0x35B448), Out::F)],
                }],
                ulps: 0,
                plain: |a| {
                    square_into(
                        &dm(a[0]).stiffness_suspension_factor,
                        &dm(a[0]).stiffness_suspension_factor,
                    );
                },
            },
            Block {
                name: "damage_wear line 215",
                operands: vec![Operand::Buffer {
                    reads: &[(laid!(0x35B468), In::F)],
                    writes: &[(laid!(0x35B468), Out::F)],
                }],
                ulps: 0,
                plain: |a| {
                    square_into(
                        &dm(a[0]).ride_height_body_factor,
                        &dm(a[0]).ride_height_body_factor,
                    );
                },
            },
            Block {
                name: "damage_wear line 223",
                operands: vec![Operand::Buffer {
                    reads: &[
                        (laid!(0x35B39C), In::F),
                        (laid!(0x35B3E0), In::F),
                        (laid!(0x35B3F4), In::F),
                        (laid!(0x35B448), In::F),
                        (laid!(0x35B44C), In::F),
                        (laid!(0x35B468), In::F),
                        (laid!(0x35B6F8), In::F),
                    ],
                    writes: &[
                        (laid!(0x35B450), Out::F),
                        (laid!(0x35B454), Out::F),
                        (laid!(0x35B458), Out::F),
                    ],
                }],
                ulps: 0,
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
                        reads: &[(laid!(0x35B3F0), In::F), (laid!(0x35B8BC), In::F)],
                        writes: &[(laid!(0x35B484), Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(0, In::I)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    blend_wear_figures(*view::<i32>(a[0], 0), dm(a[1]), *view::<i32>(a[2], 0));
                },
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        // The plain code rounds to nearest, as the game always runs the damage model.
        sparse_at("physics.damage_wear", &blocks(), 1200, 100, &[0x027F]);
    }
}

#[cfg(test)]
mod plain_matches_x87 {
    use super::*;
    use crate::physics::damage_model::proof::{State, prove, wear};

    fn d(s: &State) -> &DamageModel {
        &s.damage
    }

    #[test]
    fn wear_additions_and_subtractions() {
        prove(
            "add_wear",
            |s| {
                let (d, g) = (&s.damage, &s.car.brake_heat[0]);
                wear::add_wear(d, &d.wear_rates[3], &d.brake_share, g, g);
            },
            |s| {
                let (d, g) = (&s.damage, &s.car.brake_heat[0]);
                add_wear(d, &d.wear_rates[3], &d.brake_share, g, g);
            },
        );
        prove(
            "add_wear into another cell",
            |s| {
                let d = &s.damage;
                let from = &s.car.engine_heat;
                wear::add_wear(d, &d.wear_rates[0], &d.overheat_excess, from, &d.scratch);
            },
            |s| {
                let d = &s.damage;
                let from = &s.car.engine_heat;
                add_wear(d, &d.wear_rates[0], &d.overheat_excess, from, &d.scratch);
            },
        );
        prove(
            "subtract_rate",
            |s| {
                let (d, g) = (&s.damage, &s.car.engine_heat);
                wear::subtract_rate(d, &d.wear_rates[1], g, g);
            },
            |s| {
                let (d, g) = (&s.damage, &s.car.engine_heat);
                subtract_rate(d, &d.wear_rates[1], g, g);
            },
        );
        prove(
            "subtract_wear",
            |s| {
                let (d, g) = (&s.damage, &s.car.brake_fade);
                wear::subtract_wear(d, &d.wear_rates[9], &d.brake_share, g, g);
            },
            |s| {
                let (d, g) = (&s.damage, &s.car.brake_fade);
                subtract_wear(d, &d.wear_rates[9], &d.brake_share, g, g);
            },
        );
    }

    #[test]
    fn gap_square_and_figures() {
        prove(
            "lerp_gap",
            |s| wear::lerp_gap(&s.damage),
            |s| lerp_gap(&s.damage),
        );
        prove(
            "square_into",
            |s| wear::square_into(&d(s).stiffness_body_factor, &d(s).stiffness_body_factor),
            |s| square_into(&d(s).stiffness_body_factor, &d(s).stiffness_body_factor),
        );
        prove(
            "square_into another cell",
            |s| wear::square_into(&d(s).stiffness_body_factor, &d(s).scratch),
            |s| square_into(&d(s).stiffness_body_factor, &d(s).scratch),
        );
        prove(
            "hit_figures",
            |s| wear::hit_figures(&s.damage),
            |s| hit_figures(&s.damage),
        );
        prove(
            "blend_wear_figures",
            |s| {
                let (f, g) = (s.damage.figures[1].bits(), s.damage.figures[2].bits());
                wear::blend_wear_figures(f, &s.damage, g);
            },
            |s| {
                let (f, g) = (s.damage.figures[1].bits(), s.damage.figures[2].bits());
                blend_wear_figures(f as i32, &s.damage, g as i32);
            },
        );
    }
}
