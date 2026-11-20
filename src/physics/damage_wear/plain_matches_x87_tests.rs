use super::*;
use crate::physics::damage_model::proof::{State, prove};

fn d(s: &State) -> &DamageModel {
    &s.damage
}

#[test]
fn wear_additions_and_subtractions() {
    prove("add_wear", |s| {
        let (d, g) = (&s.damage, &s.car.brake_heat[0]);
        add_wear(d, &d.wear_rates[3], &d.brake_share, g, g);
    });
    prove("add_wear into another cell", |s| {
        let d = &s.damage;
        let from = &s.car.engine_heat;
        add_wear(d, &d.wear_rates[0], &d.overheat_excess, from, &d.scratch);
    });
    prove("subtract_rate", |s| {
        let (d, g) = (&s.damage, &s.car.engine_heat);
        subtract_rate(d, &d.wear_rates[1], g, g);
    });
    prove("subtract_wear", |s| {
        let (d, g) = (&s.damage, &s.car.brake_fade);
        subtract_wear(d, &d.wear_rates[9], &d.brake_share, g, g);
    });
}

#[test]
fn gap_square_and_figures() {
    prove("lerp_gap", |s| lerp_gap(&s.damage));
    prove("square_into", |s| {
        square_into(&d(s).stiffness_body_factor, &d(s).stiffness_body_factor);
    });
    prove("square_into another cell", |s| {
        square_into(&d(s).stiffness_body_factor, &d(s).scratch);
    });
    prove("hit_figures", |s| hit_figures(&s.damage));
    prove("blend_wear_figures", |s| {
        let (f, g) = (s.damage.figures[1].bits(), s.damage.figures[2].bits());
        blend_wear_figures(f as i32, &s.damage, g as i32);
    });
}
