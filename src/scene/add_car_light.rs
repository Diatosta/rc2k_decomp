//! `scene::add_car_light` (0x44F29F): what one car's lamps add to the light on an
//! object.
//!
//! Takes: the car's block
//!
//! Nothing unless the block's dword +0x84 is set. The place being lit
//! (`LightState.lit_place[0]` ..) less the car's (+0x134 ..), through the car's matrix (+0x9CC;
//! `scene::rotate_point`), as whole numbers in `LightState.turned_light[0]` ..: the place as the
//! car sees it, x across, y up, z ahead. Then three amounts, each held to 0 .. 255:
//! - 0x100 less a quarter of ((z - 0x200000) squared + (2x) squared) times
//!   `Numbers.one_over_65536` twice: a cone ahead of the car;
//! - one the original works out and then overwrites (((z - 0x100000) squared + x
//!   squared) times `Numbers.one_over_65536`, stored to `g_scratch`, which nothing read: not kept);
//! - (0x100000 less y without its sign) over 0x1000: how near the car's height.
//!
//! The third times twice the first over 256, times the block's dword +0xABC over 256, held to
//! 255, picks a colour from the table `LightTables.car_light_table[0]`, which is added to `g_lightColour` byte by
//! byte without overflow. The x87's arithmetic in `X` (plain Rust; the
//! original's instructions' results are recorded vectors in the tests (`data/vectors`)). Callers: `scene::add_car_lights`,
//! twice. Takes the car's block as a typed reference.

use crate::scene::vertex_light::plus;
use crate::scene::x87_mem::{Base, cells, fild, fistp, fld};

// ---- Rust version ----

/// Three whole numbers through a matrix and back to whole numbers: `fild` three times,
/// `scene::rotate_point`, `fistp` three times.
fn turned(from: [u32; 3], matrix: &[u32]) -> [u32; 3] {
    let mut w = [from[0], from[1], from[2], 0, 0, 0];
    turned_at(cells(&mut w), matrix);
    [w[3], w[4], w[5]]
}

fn turned_at(w: impl Base, m: impl Base) {
    let t0 = fild(w, 0x8);
    let t1 = fild(w, 0x4);
    let t2 = fild(w, 0x0);
    let t3 = fld(m, 0x0);
    let t4 = t3 * t2;
    let t5 = fld(m, 0x10);
    let t6 = t5 * t1;
    let t7 = fld(m, 0x20);
    let t8 = t7 * t0;
    let t9 = t6 + t8;
    let t10 = t4 + t9;
    let t11 = fld(m, 0x30);
    let t12 = t10 + t11;
    let t13 = fld(m, 0x4);
    let t14 = t13 * t2;
    let t15 = fld(m, 0x14);
    let t16 = t15 * t1;
    let t17 = fld(m, 0x24);
    let t18 = t17 * t0;
    let t19 = t16 + t18;
    let t20 = t14 + t19;
    let t21 = fld(m, 0x34);
    let t22 = t20 + t21;
    let t23 = fld(m, 0x8);
    let t24 = t2 * t23;
    let t25 = fld(m, 0x18);
    let t26 = t1 * t25;
    let t27 = fld(m, 0x28);
    let t28 = t0 * t27;
    let t29 = t28 + t26;
    let t30 = t24 + t29;
    let t31 = fld(m, 0x38);
    let t32 = t30 + t31;
    fistp(w, 0xC, t12);
    fistp(w, 0x10, t22);
    fistp(w, 0x14, t32);
}

/// `fild a; fmul st, st; fild b; fmul st, st; faddp; fmul k` (once or twice) `; fistp`.
fn squares(a: u32, b: u32, factor: f32, twice: bool) -> u32 {
    let (v, mut out) = ([a, b, factor.to_bits()], [0u32]);
    if twice {
        squares_twice(&v[..], cells(&mut out));
    } else {
        squares_once(&v[..], cells(&mut out));
    }
    out[0]
}

/// Twice the factor.
fn squares_twice(v: impl Base, o: impl Base) {
    let t0 = fild(v, 0x0);
    let t1 = t0 * t0;
    let t2 = fild(v, 0x4);
    let t3 = t2 * t2;
    let t4 = t1 + t3;
    let t5 = fld(v, 0x8);
    let t6 = t4 * t5;
    let t7 = fld(v, 0x8);
    let t8 = t6 * t7;
    fistp(o, 0x0, t8);
}

/// Once the factor.
fn squares_once(v: impl Base, o: impl Base) {
    let t0 = fild(v, 0x0);
    let t1 = t0 * t0;
    let t2 = fild(v, 0x4);
    let t3 = t2 * t2;
    let t4 = t1 + t3;
    let t5 = fld(v, 0x8);
    let t6 = t4 * t5;
    fistp(o, 0x0, t6);
}

/// Held to 0 ..= 255 by the signs of the differences, as the game compares.
fn held(value: i32) -> u32 {
    let value = value.max(0);
    if value.wrapping_sub(0xFF) < 0 {
        value as u32
    } else {
        0xFF
    }
}

pub fn add_car_light(
    course: &crate::race::owner_course::Course,
    car: &crate::race::frame_view::CarView,
    marks_count: u32,
    scene: &crate::scene::owner::Scene,
    light: &crate::scene::object_light::ObjectLight,
    system: &crate::system::owner::System,
) {
    if car.lights_on.get() == 0 {
        return;
    }
    let place: [u32; 3] = crate::session::read_bits(&light.lit_place);
    let mut from = [0u32; 3];
    for k in 0..3 {
        from[k] = place[k].wrapping_sub(car.position[k].get());
    }
    let matrix = crate::session::read_bits(&car.car_inverse_matrix);
    let seen = turned(from, &matrix);
    crate::session::write_bits(&light.turned_light, seen);
    let [x, y, z] = seen;
    let factor = course.numbers.one_over_65536.get();
    let ahead = squares(z.wrapping_sub(0x20_0000), x.wrapping_add(x), factor, true);
    let cone = held(0x100i32.wrapping_sub((ahead >> 2) as i32));
    // The original also worked out a second sum of squares, stored to `g_scratch` and
    // never read: not kept.
    let height = if (y as i32) < 0 { y.wrapping_neg() } else { y };
    let level = held((0x10_0000u32.wrapping_sub(height) as i32) >> 12);
    let amount = (level.wrapping_mul(cone * 2) >> 8).wrapping_mul(marks_count) >> 8;
    let colour: u32 = scene.light_tables.car_light_table[held(amount as i32) as usize].get();
    let mmx = system.session.machine.has_mmx.get() != 0;
    light
        .light_colour
        .set(plus(colour, light.light_colour.get(), mmx));
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::test_support::vectors::{Differential, Kind, differential};

    #[test]
    fn turned_matches_the_assembly() {
        // dwords 0 .. 3 the place, 3 .. 6 the result, 8 .. 23 the matrix
        let test = Differential {
            name: "scene.add_car_light.turned",
            cases: 8000,
            seed: 16,
            kinds: [Kind::Exact; 23],
            fill: &|rng, m| {
                m.fill(0xAAAA_AAAA);
                for d in m.iter_mut().take(3) {
                    *d = rng.int();
                }
                let moderate = rng.below(2) == 0;
                for d in m.iter_mut().skip(8) {
                    *d = if moderate { rng.moderate() } else { rng.fl() };
                }
            },
            plain: &|c| turned_at(c, &c[8..]),
        };
        differential(test);
    }

    #[test]
    fn squares_matches_the_assembly() {
        // dwords 0 .. 3 the two whole numbers and the factor, 3 the result of the first
        // way, 4 of the second
        let test = Differential {
            name: "scene.add_car_light.squares",
            cases: 8000,
            seed: 17,
            kinds: [Kind::Exact; 5],
            fill: &|rng, m| {
                m[0] = rng.int();
                m[1] = rng.int();
                m[2] = if rng.below(2) == 0 {
                    rng.moderate()
                } else {
                    rng.fl()
                };
                m[3] = 0xAAAA_AAAA;
                m[4] = 0xAAAA_AAAA;
            },
            plain: &|c| {
                squares_twice(c, &c[3..]);
                squares_once(c, &c[4..]);
            },
        };
        differential(test);
    }
}
