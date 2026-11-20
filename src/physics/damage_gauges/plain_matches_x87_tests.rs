use super::*;
use crate::physics::damage_model::proof::prove;

#[test]
fn products_and_triples() {
    prove("add_product", |s| {
        let (d, g) = (&s.damage, &s.car.engine_damage);
        g.set(add_product(d.gauge_rates[0], d.gauge_step, g.get()));
    });
    prove("add_scaled_triple", |s| {
        let (d, g) = (&s.damage, &s.car.turbo_damage);
        g.set(add_scaled_triple(
            d,
            d.wear_rates[11],
            d.engine_over_limit,
            d.engine_speed_share,
            g.get(),
        ));
    });
    prove("add_triple_product", |s| {
        let (d, g) = (&s.damage, &s.car.electrics_damage);
        g.set(add_triple_product(
            d.gauge_rates[10],
            d.on_surface_5,
            d.gauge_step,
            g.get(),
        ));
    });
    prove("gear_wear_from_speed", |s| {
        gear_wear_from_speed(&s.damage, s.damage.gear_factors[2], &s.car.gear_wear[2]);
    });
    prove("brake_wear_step", |s| {
        brake_wear_step(&s.car, &mut s.damage);
    });
    prove("engine_damage_carry", |s| {
        engine_damage_carry(&s.car, &s.damage);
    });
}

#[test]
fn tyre_and_hit_figures() {
    prove("part_hit_figures", |s| part_hit_figures(&mut s.damage));
    prove("scale_part_hit_figures", |s| {
        let (front, rear) = (s.damage.figures[3], s.damage.figures[4]);
        scale_part_hit_figures(&mut s.damage, front, rear);
    });
    prove("add_rate_to_clutch", |s| {
        add_rate_to_clutch(&s.car, s.damage.figures[5]);
    });
    prove("add_rate_to_gauge", |s| {
        add_rate_to_gauge(&s.car.tyre_wears[1], s.damage.figures[6]);
    });
    prove("scale_hit_size", |s| scale_hit_size(&s.work, &mut s.damage));
    prove("add_hit_to_gear_gauge", |s| {
        add_hit_to_gear_gauge(&s.damage, &s.car.gear_wear[4]);
    });
}

#[test]
fn totals() {
    prove("gauge_totals", |s| gauge_totals(&s.car, &mut s.damage));
}
