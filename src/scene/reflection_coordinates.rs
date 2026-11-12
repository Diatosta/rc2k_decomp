//! `scene::reflection_coordinates` (0x45198C): where on the reflection's picture a
//! vertex of a shiny surface looks.
//!
//! In:  nothing; the object's reflection normal number is the number of the vertex's normal,
//!      its reflection point number the number of its point
//!
//! The normal's place in the view (record 0x1000 + (number & 0xFFF), +0x10 .. +0x18) goes to
//! the reflection work; the point's, negated, to the work vector, which is made one long and
//! copied: the direction from the point to the eye. `scene::reflect_vector`; the result, with
//! 1.0 added to its second part, is made one long; then each of its first and third parts,
//! times 0.5 plus the same, times 65536 times 256, rounded, is a coordinate.
//! Callers: `scene::cached_reflection_coordinates`.
//! Refuses when there are no records and what `math::normalise_vector` refuses.

use crate::errors::BadInput;
use crate::math::normalise_vector::normalise_vector;
use crate::math::x87;
use crate::scene::reflect_vector::reflect_vector;
use crate::scene::x87_mem::{Base, cells, fistp, fld, fstp};

// ---- Rust version ----

/// `fld a; fmul half; fadd half; fmul scale_a; fmul scale_b; fistp`.
fn coordinate(a: u32, half: f32, scales: [f32; 2]) -> i32 {
    let (v, mut out) = (
        [a, half.to_bits(), scales[0].to_bits(), scales[1].to_bits()],
        [0u32],
    );
    coordinate_at(&v[..], cells(&mut out));
    out[0] as i32
}

fn coordinate_at(v: impl Base, o: impl Base) {
    let t0 = fld(v, 0x0);
    let t1 = fld(v, 0x4);
    let t2 = t0 * t1;
    let t3 = fld(v, 0x4);
    let t4 = t2 + t3;
    let t5 = fld(v, 0x8);
    let t6 = t4 * t5;
    let t7 = fld(v, 0xC);
    let t8 = t6 * t7;
    fistp(o, 0x0, t8);
}

/// `fld a; fchs; fstp`.
fn negated(a: u32) -> u32 {
    let mut out = [0u32];
    negated_at(&[a][..], cells(&mut out));
    out[0]
}

fn negated_at(a: impl Base, o: impl Base) {
    let t0 = fld(a, 0x0);
    let t1 = -t0;
    fstp(o, 0x0, t1);
}

pub fn reflection_coordinates(
    race: &crate::race::owner::Race,
    part: &crate::workers::PartScratch,
    scene: &crate::scene::owner::Scene,
    object: &crate::scene::object_place::ObjectPlace,
    m: &crate::scene::view_matrices::ViewMatrices,
) -> Result<(), BadInput> {
    if !crate::scene::point_records::held(&race.course) {
        return Err(BadInput("no records for the points"));
    }
    let table = crate::scene::reading::PointRecords::new(&race.course, part)?;
    let normal = table.place(object.reflection_normal_number.get())?;
    let point = table.place(object.reflection_point_number.get().wrapping_neg())?;
    crate::session::write_bits(&m.reflect_vector, normal);
    crate::session::write_first(&m.work_vector, &point.map(negated));
    normalise_vector(race, scene, &m.work_vector)?;
    let to_eye: [u32; 3] = crate::session::read_first(&m.work_vector);
    crate::session::write_bits(&m.reflect_normal, to_eye);
    reflect_vector(m, &m.view_back_matrix);
    let mirrored: [u32; 3] = crate::session::read_bits(&m.reflect_vector);
    crate::session::write_first(
        &m.work_vector,
        &[
            x87::loaded(mirrored[0]).to_bits(),
            x87::add(mirrored[1], &race.course.numbers.one).to_bits(),
            x87::loaded(mirrored[2]).to_bits(),
        ],
    );
    normalise_vector(race, scene, &m.work_vector)?;
    let looks: [u32; 3] = crate::session::read_first(&m.work_vector);
    let half = race.course.numbers.half.get();
    let scales = [
        race.course.numbers.k65536.get(),
        race.course.numbers.k256.get(),
    ];
    crate::session::write_bits(
        &object.reflection_coordinates,
        [
            coordinate(looks[0], half, scales) as u32,
            coordinate(looks[2], half, scales) as u32,
        ],
    );
    Ok(())
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::test_support::vectors::{Differential, Kind, differential};

    #[test]
    fn coordinate_matches_the_assembly() {
        // dwords 0 .. 4 the four floats, 4 the whole number
        let test = Differential {
            name: "scene.reflection_coordinates.coordinate",
            cases: 8000,
            seed: 20,
            kinds: [Kind::Exact; 5],
            fill: &|rng, m| {
                m[0] = rng.unit().to_bits();
                m[1] = if rng.below(2) == 0 {
                    0.5f32.to_bits()
                } else {
                    rng.fl()
                };
                m[2] = if rng.below(2) == 0 {
                    65536.0f32.to_bits()
                } else {
                    rng.moderate()
                };
                m[3] = if rng.below(2) == 0 {
                    256.0f32.to_bits()
                } else {
                    rng.moderate()
                };
                m[4] = 0xAAAA_AAAA;
            },
            plain: &|c| coordinate_at(c, &c[4..]),
        };
        differential(test);
    }

    #[test]
    fn negated_matches_the_assembly() {
        let test = Differential {
            name: "scene.reflection_coordinates.negated",
            cases: 8000,
            seed: 21,
            kinds: [Kind::Exact; 2],
            fill: &|rng, m| {
                m[0] = rng.fl();
                m[1] = 0xAAAA_AAAA;
            },
            plain: &|c| negated_at(c, &c[1..]),
        };
        differential(test);
    }
}
