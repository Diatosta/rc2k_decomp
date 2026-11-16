//! The dot product of two vectors, x x' + (y y' + z z'), left unrounded on the x87 stack. The
//! same operations as `scene::dot_product3`, whose version (`scene::vector3::dot_product3`) it
//! uses.

use crate::math::cells::{Cells, load};
use crate::scene::vector3::*;

/// The product as it is left in st(0).
pub fn dot_product(a: &(impl Cells + ?Sized), b: &(impl Cells + ?Sized)) -> [u8; 10] {
    dot_product3(load(a), load(b))
}
