//! Moves the work car by a point's speed about its spin: the spin is copied to `lever_push_place`
//! and turned into the car's frame (`physics::turn_by_inverse`); crossed with the lever
//! `lever_push_vector` (into `lever_push_turned`), turned back (`physics::turn_by_car_matrix`),
//! scaled by the share `lever_share_left` and by `lever_share` (`physics::scale_vector`); its x and
//! z times 65536 and the step (`step_time`), rounded, go into `lever_pushes` and are added to the
//! car's x and z.

use crate::math::matrix3::extended;
use crate::math::x87::mul2_to_int;
use crate::physics::car_store::Car;
use crate::physics::scale_vector::scale_vector;
use crate::physics::tables_store::PhysicsTables;
use crate::physics::turn_by_car_matrix::turn_by_car_matrix;
use crate::physics::turn_by_inverse::turn_by_inverse;
use crate::race::numbers_store::Numbers;
use crate::scene::view_matrix::cross_product;

pub fn lever_push(car: &Car, tables: &PhysicsTables, numbers: &Numbers) {
    for k in 0..3 {
        tables.lever_push_place[k].set(car.spin[k].get());
    }
    turn_by_inverse(car, &tables.lever_push_place);
    cross_product(
        &tables.lever_push_vector,
        &tables.lever_push_place,
        &tables.lever_push_turned,
    );
    turn_by_car_matrix(car, &tables.lever_push_turned);
    for factor in [&tables.lever_share_left, &tables.lever_share] {
        let f = extended(factor.get());
        scale_vector(&tables.lever_push_turned, f);
    }
    for (from, int, place) in [
        (
            &tables.lever_push_turned[0],
            &tables.lever_pushes[0],
            &car.position[0],
        ),
        (
            &tables.lever_push_turned[2],
            &tables.lever_pushes[1],
            &car.position[2],
        ),
    ] {
        let v = mul2_to_int(from.get(), &numbers.k65536, &tables.step_time) as u32;
        int.set(v);
        place.set(place.get().wrapping_add(int.get()));
    }
}
