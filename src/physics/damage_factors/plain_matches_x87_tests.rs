use super::*;
use crate::physics::damage_model::proof::prove;

#[test]
fn shares_and_wheels() {
    prove("scale_car_figures", |s| {
        scale_car_figures(&s.car, &mut s.damage);
    });
    prove("wheel_stiffness_from_share", |s| {
        let (d, c) = (&s.damage, &s.car);
        wheel_stiffness_from_share(d, d.wheel_stiffness_lerp[1], &c.wheel_stiffness.float_at(1));
    });
    prove("wheel_ride_height_from_share", |s| {
        wheel_ride_height_from_share(&s.damage, &s.car.wheel_ride_height.float_at(2));
    });
    prove("fail_wheel", |s| {
        fail_wheel(
            &s.damage,
            &s.car.wheel_stiffness.float_at(3),
            &s.car.wheel_ride_height.float_at(3),
        );
    });
    prove("means_and_engine_speed", |s| {
        s.car.drive_kind.set(s.car.drive_kind.get() & 3);
        means_and_engine_speed(&s.car, &mut s.damage);
    });
}

#[test]
fn speed_shares() {
    prove("engine_speed_share_step", |s| {
        engine_speed_share_step(&s.car, &mut s.damage);
    });
    prove("speed_share_step", |s| {
        speed_share_step(&s.car, &mut s.damage);
    });
    prove("drive_a_slip_step", |s| {
        drive_a_slip_step(&s.car, &mut s.damage);
    });
    prove("drive_extrapolation_step", |s| {
        let (c, d) = (&s.car, &mut s.damage);
        d.drive_b_slip_share = drive_extrapolation_step(c, d, &c.drive_speed_a, &c.drive_speed_a);
    });
    prove(
        "drive_extrapolation_step with a separate carried figure",
        |s| {
            let (c, d) = (&s.car, &mut s.damage);
            d.drive_a_extrapolated_share =
                drive_extrapolation_step(c, d, &c.drive_speed_b, &c.drive_speed_a);
        },
    );
    prove("finish_shares", |s| finish_shares(&mut s.damage));
    prove("brake_share_step", |s| {
        brake_share_step(&s.car, &mut s.damage);
    });
}

#[test]
fn engine_limit_hit_and_loads() {
    prove("engine_over_limit", |s| {
        s.damage.scratch = f32::from_bits(u32::from(engine_over_limit(&s.car, &s.damage)));
    });
    prove("hit_size_and_dive", |s| {
        hit_size_and_dive(&s.car, &mut s.damage);
    });
    prove("square_dive", |s| square_dive(&mut s.damage));
    prove("blend_wheel_loads", |s| blend_wheel_loads(&mut s.damage));
    // No wheel just landed: the loads are zero, the constants are the model's own.
    let still = |s: &mut crate::physics::damage_model::proof::State| {
        s.damage.wheel_loads = [0.0; 4];
        s.damage.quarter = 0.25;
        s.damage.half = 0.5;
    };
    prove("blend_wheel_loads at rest", |s| {
        still(s);
        blend_wheel_loads(&mut s.damage);
    });
    prove("the touch-down load", |s| {
        let (c, d) = (&s.car, &mut s.damage);
        d.wheel_loads[0] = (value(&c.wheel_squeeze[0]) * figure(d.scratch)) as f32;
    });
}
