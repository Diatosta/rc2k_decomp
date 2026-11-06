//! `physics::lever_push` (0x43DEF4): the work car's spin (`dword_71A15C` .., `g_car`
//! +0x184) copied to `dword_614B64` .. and turned into the car's frame
//! (`physics::turn_by_inverse`); crossed with the lever `flt_614B44` .. (`scene::cross_product`
//! into `flt_614B54` ..), turned back (`physics::turn_by_car_matrix`), scaled by the share
//! `flt_614C48` and by `flt_614768` (`physics::scale_vector`, each factor then stored in
//! `dword_62400C`); its x and z times 65536 (`flt_5C5D98`) and the step (`flt_61388C`),
//! `fistp` into `dword_6140E4` / `dword_6140E8` and added to the car's x / z
//! (`dword_71A10C` / `dword_71A114`). A reading: the car moved by a point's speed about
//! its spin.
//!
//! In:  nothing
//! Out: eax the z push; ebx 0x614B44, ecx 0x614B64, edx and esi 0x614B54; the rest kept;
//!      no flags read
//!
//! Caller: `physics::lever_pushes` (twice).

use crate::math::x87_run::mul2_to_int;
use crate::physics::scale_vector::scale_vector;
use crate::physics::turn_by_car_matrix::turn_by_car_matrix;
use crate::physics::turn_by_inverse::turn_by_inverse;
use crate::race::owner::Race;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::owner::scene;
use crate::scene::view_matrix::cross_product;

/// The Rust version.
pub fn lever_push(race: &Race) {
    for k in 0..3 {
        race.shared.physics_tables.lever_push_place[k].set(race.shared.car.spin[k].bits());
    }
    turn_by_inverse(&race.shared, &race.shared.physics_tables.lever_push_place);
    cross_product(
        &race.shared.physics_tables.lever_push_vector,
        &race.shared.physics_tables.lever_push_place,
        &race.shared.physics_tables.lever_push_turned,
    );
    turn_by_car_matrix(&race.shared, &race.shared.physics_tables.lever_push_turned);
    for factor in [
        &race.shared.physics_tables.lever_share_left,
        &race.shared.physics_tables.lever_share,
    ] {
        let f = extended(factor.bits());
        scale_vector(&race.shared.physics_tables.lever_push_turned, f);
        scene().drawing.scene.scratch.set(rounded(f));
    }
    for (from, int, place) in [
        (
            &race.shared.physics_tables.lever_push_turned[0],
            &race.shared.physics_tables.lever_pushes[0],
            &race.shared.car.position[0],
        ),
        (
            &race.shared.physics_tables.lever_push_turned[2],
            &race.shared.physics_tables.lever_pushes[1],
            &race.shared.car.position[2],
        ),
    ] {
        let v = mul2_to_int(
            from.get(),
            race.course.numbers.k65536.bits(),
            race.shared.physics_tables.step_time.bits(),
        ) as u32;
        int.set(v);
        place.set(place.get().wrapping_add(int.get()));
    }
}
