use super::*;
use crate::test_support::vectors::{Differential, Kind, differential};

#[test]
fn normalise_matches_the_assembly() {
    let test = Differential {
        name: "scene.vector3.normalise",
        cases: 8000,
        seed: 4,
        kinds: [Kind::Float; 3],
        fill: &|rng, m| {
            let moderate = rng.below(2) == 0;
            for d in m.iter_mut() {
                *d = if moderate { rng.moderate() } else { rng.fl() };
            }
        },
        plain: &|c| normalise(c),
    };
    differential(test);
}

#[test]
fn dot_matches_the_assembly() {
    // a, b, the 80-bit result (3 dwords)
    let test = Differential {
        name: "scene.vector3.dot",
        cases: 8000,
        seed: 5,
        kinds: [Kind::Exact; 9],
        fill: &|rng, m| {
            let moderate = rng.below(2) == 0;
            for d in m.iter_mut().take(6) {
                *d = if moderate { rng.moderate() } else { rng.fl() };
            }
            m[6..].fill(0xAAAA_AAAA);
        },
        plain: &|c| dot(c, &c[3..], &c[6..]),
    };
    differential(test);
}

#[test]
fn cosine_matches_the_assembly() {
    // the angle, the float it is compared with (often the cosine itself), the status
    let test = Differential {
        name: "scene.vector3.cosine",
        cases: 8000,
        seed: 6,
        kinds: [Kind::Exact; 3],
        fill: &|rng, m| {
            m[0] = rng.angle();
            m[1] = match rng.below(3) {
                0 => rc2k_core::math::portable::cos(f32::from_bits(m[0]) as f64) as f32,
                1 => rng.unit(),
                _ => f32::from_bits(rng.fl()),
            }
            .to_bits();
            m[2] = 0;
        },
        plain: &|c| {
            cosine(c, &c[1..], &c[2..]);
            c[2].set(c[2].get() & 0x700);
        },
    };
    differential(test);
}
