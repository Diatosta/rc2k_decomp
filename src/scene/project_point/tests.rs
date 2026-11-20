use super::*;
use crate::test_support::vectors::{Differential, Kind, differential};

#[test]
fn rotate_matches_the_assembly() {
    // dwords 0 .. 3 the point, 4 .. 19 the matrix, 20 .. 28 the record
    let test = Differential {
        name: "scene.project_point.rotate",
        cases: 8000,
        seed: 13,
        kinds: [Kind::Float; 28],
        fill: &|rng, m| {
            m.fill(0xAAAA_AAAA);
            for d in m.iter_mut().take(3) {
                *d = rng.int();
            }
            let moderate = rng.below(2) == 0;
            for d in m.iter_mut().skip(4).take(15) {
                *d = if moderate { rng.moderate() } else { rng.fl() };
            }
        },
        plain: &|c| {
            let m: [f32; 16] = std::array::from_fn(|k| f32::from_bits(c[4 + k].get()));
            let place = rotate_at([0, 1, 2].map(|k| c[k].get() as i32), &m);
            for (k, v) in place.into_iter().enumerate() {
                c[24 + k].set(v.to_bits());
            }
        },
    };
    differential(test);
}

/// The window's values and the place in the view of a test's block.
fn screen_of(c: &[std::cell::Cell<u32>]) -> (Screen, [f32; 3]) {
    let f = |k: usize| f32::from_bits(c[k].get());
    let screen = Screen {
        scale: f(0),
        centre: [f(1), f(2)],
        flat_depth: c[3].get() as i32,
    };
    (screen, [f(12), f(13), f(14)])
}

/// The screen's x and y into the test's record (+0xC, +0x1C).
fn put_screen(c: &[std::cell::Cell<u32>], [x, y]: [u32; 2]) {
    c[11].set(x);
    c[15].set(y);
}

/// dwords 0 .. 4 the scale, centre x, centre y and whole number, 4 the checked word, 8 ..
/// 16 the record (+0x10 .. +0x18 the place in the view).
fn screen_fill(rng: &mut crate::test_support::vectors::Rng, m: &mut [u32; 16]) {
    m.fill(0xAAAA_AAAA);
    let moderate = rng.below(2) == 0;
    for d in m.iter_mut().take(3) {
        *d = if moderate { rng.moderate() } else { rng.fl() };
    }
    m[3] = rng.int();
    for d in m.iter_mut().skip(12).take(3) {
        *d = if moderate { rng.moderate() } else { rng.fl() };
    }
}

#[test]
fn screen_flat_matches_the_assembly() {
    let test = Differential {
        name: "scene.project_point.screen_flat",
        cases: 8000,
        seed: 14,
        kinds: [Kind::Float; 16],
        fill: &screen_fill,
        plain: &|c| {
            let (screen, view) = screen_of(c);
            put_screen(c, screen_flat(view, screen, &c[4..]));
        },
    };
    differential(test);
}

#[test]
fn screen_plain_matches_the_assembly() {
    let test = Differential {
        name: "scene.project_point.screen_plain",
        cases: 8000,
        seed: 15,
        kinds: [Kind::Float; 16],
        fill: &screen_fill,
        plain: &|c| {
            let (screen, view) = screen_of(c);
            put_screen(c, screen_plain(view, screen));
        },
    };
    differential(test);
}
