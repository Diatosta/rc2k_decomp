//! Vectors of three floats held as their bits (`[u32; 3]`), with the steps of the cell versions
//! (`scene::cross_product`, `dot_product`, `scale_vector`, `vector_length`, the parts product,
//! `turn_by_inverse` and `turn_by_car_matrix`) taking and returning values: the contact work's
//! vectors go through these.

use crate::math::cells::{Cells, load};
use crate::math::matrix3::{extended, rounded_value};
use crate::math::x87;
use crate::math::x87_float::{ld, st};
use crate::scene::rotate_point::rotate_point;
use crate::scene::vector3::dot_product3;

/// `b` x `c`, each part made from the two products' difference and stored as a float.
pub fn cross(b: [u32; 3], c: [u32; 3]) -> [u32; 3] {
    let part = |x: usize, y: usize| x87::product_difference(c[x], b[y], b[x], c[y]).to_bits();
    [part(2, 1), part(0, 2), part(1, 0)]
}

/// The dot product x x' + (y y' + z z'), left unrounded (as 80 bits).
pub fn dot(a: [u32; 3], b: [u32; 3]) -> [u8; 10] {
    dot_product3(a, b)
}

/// The vector times the 80-bit `factor`, each part stored as a float.
pub fn scale(v: [u32; 3], factor: [u8; 10]) -> [u32; 3] {
    x87::scale_by_80(v, factor)
}

/// The length, sqrt(x x + (y y + z z)), left unrounded (as 80 bits).
pub fn length(v: [u32; 3]) -> [u8; 10] {
    x87::length_80(v)
}

/// The parts of `a` times those of `b`, each stored as a float.
pub fn multiply(a: [u32; 3], b: [u32; 3]) -> [u32; 3] {
    std::array::from_fn(|k| x87::mul(a[k], b[k]).to_bits())
}

/// The parts of `a` plus those of `b` (each loaded and stored as the x87 does).
pub fn add(a: [u32; 3], b: [u32; 3]) -> [u32; 3] {
    std::array::from_fn(|k| st(ld(a[k]) + ld(b[k])))
}

/// The parts negated.
pub fn negate(v: [u32; 3]) -> [u32; 3] {
    v.map(|part| crate::physics::flow::negated(f32::from_bits(part)).to_bits())
}

/// The point `v` turned and placed by the matrix `m`, each part stored as a float.
pub fn turn(m: &(impl Cells + ?Sized), v: [u32; 3]) -> [u32; 3] {
    let point = v.map(extended);
    rotate_point(m, point).map(|e| rounded_value(e).to_bits())
}

/// The first three cells' bits.
pub fn bits_of(cells: &(impl Cells + ?Sized)) -> [u32; 3] {
    load(cells)
}

/// `v` raised to `limit` when below it (or unordered), as the loaded limit.
pub fn hold_low(v: u32, limit: u32) -> u32 {
    if crate::physics::flow::below_or_unordered(f32::from_bits(v), f32::from_bits(limit)) {
        x87::loaded(limit).to_bits()
    } else {
        v
    }
}
