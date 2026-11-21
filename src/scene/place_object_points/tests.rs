use super::*;
use crate::test_support::vectors::{Differential, Kind, differential};

#[test]
fn moved_matches_the_assembly() {
    // dword 0 the fraction, 1 .. 4 the point, 4 .. 7 the other, 7 .. 10 the whole numbers,
    // 10 the fraction as stored
    let test = Differential {
        name: "scene.place_object_points.moved",
        cases: 8000,
        seed: 18,
        kinds: [Kind::Exact; 11],
        fill: &|rng, m| {
            m.fill(0xAAAA_AAAA);
            m[0] = match rng.below(3) {
                0 => rng.unit().to_bits(),
                1 => (rng.unit() * 2.0).to_bits(),
                _ => rng.fl(),
            };
            for d in m.iter_mut().skip(1).take(6) {
                *d = if rng.below(2) == 0 {
                    rng.int()
                } else {
                    rng.small() as u32
                };
            }
        },
        plain: &|c| {
            let whole = |k: usize| [k, k + 1, k + 2].map(|n| c[n].get() as i32);
            let (out, stored) = moved_at(f32::from_bits(c[0].get()), whole(1), whole(4));
            for (k, v) in out.into_iter().enumerate() {
                c[7 + k].set(v as u32);
            }
            c[10].set(stored);
        },
    };
    differential(test);
}

#[test]
fn turn_matches_the_assembly() {
    // dwords 0 .. 3 the normal, 4 .. 19 the matrix, 20 .. 28 the record
    let test = Differential {
        name: "scene.place_object_points.turn",
        cases: 8000,
        seed: 19,
        kinds: [Kind::Exact; 28],
        fill: &|rng, m| {
            m.fill(0xAAAA_AAAA);
            let moderate = rng.below(2) == 0;
            for d in m.iter_mut().take(3) {
                *d = if moderate { rng.moderate() } else { rng.fl() };
            }
            for d in m.iter_mut().skip(4).take(15) {
                *d = if moderate { rng.moderate() } else { rng.fl() };
            }
        },
        plain: &|c| {
            let f = |k: usize| f32::from_bits(c[k].get());
            let m: [f32; 16] = std::array::from_fn(|k| f(4 + k));
            let out = turn_normal([f(0), f(1), f(2)], &m);
            for (k, v) in out.into_iter().enumerate() {
                c[24 + k].set(v);
            }
        },
    };
    differential(test);
}
