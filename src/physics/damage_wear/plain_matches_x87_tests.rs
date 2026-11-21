use super::*;
use crate::physics::damage_model::proof::prove;

#[test]
fn wear_additions_and_subtractions() {
    prove("add_wear", |s| {
        let (d, g) = (&s.damage, &s.car.brake_heat.float_at(0));
        g.set(add_wear(d, d.wear_rates[3], d.brake_share, g.get()));
    });
    prove("add_wear into another cell", |s| {
        let d = &mut s.damage;
        d.scratch = add_wear(
            d,
            d.wear_rates[0],
            d.overheat_excess,
            s.car.engine_heat.get(),
        );
    });
    prove("subtract_rate", |s| {
        let (d, g) = (&s.damage, &s.car.engine_heat);
        g.set(subtract_rate(d, d.wear_rates[1], g.get()));
    });
    prove("subtract_wear", |s| {
        let (d, g) = (&s.damage, &s.car.brake_fade);
        g.set(subtract_wear(d, d.wear_rates[9], d.brake_share, g.get()));
    });
}

#[test]
fn gap_square_and_figures() {
    prove("lerp_gap", |s| lerp_gap(&mut s.damage));
    prove("square_into", |s| {
        square_into(&mut s.damage.stiffness_body_factor);
    });
    prove("square_into another cell", |s| {
        let mut x = s.damage.stiffness_body_factor;
        square_into(&mut x);
        s.damage.scratch = x;
    });
    prove("hit_figures", |s| hit_figures(&mut s.damage));
    prove("blend_wear_figures", |s| {
        let (f, g) = (s.damage.figures[1].to_bits(), s.damage.figures[2].to_bits());
        blend_wear_figures(f as i32, &mut s.damage, g as i32);
    });
}
