use super::*;
use crate::math::x87_mem::Base;
use crate::test_support::vectors::{Differential, Kind, differential, put_t80};

/// The angle (an 80-bit value, dwords 0 .. 3), then the nine floats at dword 4. The sine
/// and cosine are libm's: one float ulp apart is allowed.
fn turn_test(name: &str, seed: u64, stores: Stores) {
    let test = Differential {
        name,
        cases: 6000,
        seed,
        kinds: [Kind::Ulp1; 13],
        fill: &|rng, m| {
            m.fill(0xAAAA_AAAA);
            put_t80(m, 0, rng.angle());
            m[2] &= 0xFFFF;
            m[3..].fill(0xAAAA_AAAA);
        },
        plain: &|c| turn(stores, Base::tbyte(c, 0), &c[3..12]),
    };
    differential(test);
}

#[test]
fn turn_x_matches_the_assembly() {
    turn_test("scene.matrix3.turn_x", 7, TURN_X);
}

#[test]
fn turn_y_matches_the_assembly() {
    turn_test("scene.matrix3.turn_y", 8, TURN_Y);
}

#[test]
fn turn_z_matches_the_assembly() {
    turn_test("scene.matrix3.turn_z", 9, TURN_Z);
}

#[test]
fn extended_and_rounded_matches_the_assembly() {
    // a float, its 80-bit value, the 80-bit value rounded back
    let test = Differential {
        name: "scene.matrix3.extended_and_rounded",
        cases: 8000,
        seed: 10,
        kinds: [Kind::Exact; 8],
        fill: &|rng, m| {
            m.fill(0xAAAA_AAAA);
            m[0] = rng.fl();
            put_t80(m, 16, rng.fl());
        },
        plain: &|c| {
            Base::set_tbyte(c, 4, extended(c[0].get()));
            c[7].set(rounded(Base::tbyte(c, 16)));
        },
    };
    differential(test);
}
