//! `marks::place_view` (0x4C1CC8): the point (0, 0, 0x20000) turned by the 3 x 3 matrix at
//! `dword_8BC441` (`scene::transform_int_points`, in place at `dword_8BDD71`); the place
//! `dword_8BC435` .. less it into `dword_8BDD7D`, `dword_8BDD81`, and `flt_8C5FE1` less
//! its z plus `dword_8BC43D` into `dword_8BDD85` (each rounded); the matrix's nine
//! dwords copied to `dword_8BDD89`; the point put back to (0, 0, 0x20000).
//!
//! In:  nothing
//! Out: nothing (its caller reads no register after it)
//!
//! One caller, `sub_4C16F9` (the marks' frame).

use crate::errors::Refused;
use crate::marks::rain::RAIN;
use crate::math::x87;
use crate::session::{Dword, Float};

/// One of the two near-copies' variables (`place_view`'s and `place_view_2`'s), as addresses.
pub struct ViewPlace {
    pub point: &'static [Dword; 3],
    pub start_z: u32,
    pub place: &'static [Dword; 3],
    pub matrix: &'static [Float; 9],
    pub out: &'static [Dword; 3],
    pub height: &'static Float,
    pub matrix_copy: &'static [Float; 9],
}

/// `place_view`'s: the rain's.
pub fn first() -> ViewPlace {
    ViewPlace {
        point: &RAIN.point,
        start_z: 0x2_0000,
        place: &RAIN.view_place,
        matrix: &RAIN.view_matrix,
        out: &RAIN.view,
        height: &crate::marks::rain::RAIN_RULES.height,
        matrix_copy: &RAIN.matrix_copy,
    }
}

/// The shared body (see the header), on `v`'s variables.
pub fn place_view_with(v: &ViewPlace) -> Result<(), Refused> {
    let start = [0, 0, v.start_z];
    let set_point = |p: [u32; 3]| {
        for (cell, x) in v.point.iter().zip(p) {
            cell.set(x);
        }
    };
    {
        set_point(start);
        let point = v.point[0].addr() as u32;
        crate::marks::splines::transform(point, point, 1, v.matrix[0].addr() as u32)?;
        let point = v.point.each_ref().map(|c| c.get() as i32);
        let place = v.place.each_ref().map(|c| c.get() as i32);
        let out = [
            x87::zero_less_int_plus_int(point[0], place[0]),
            x87::zero_less_int_plus_int(point[1], place[1]),
            x87::less_int_plus_int(v.height.bits(), point[2], place[2]),
        ];
        for (cell, x) in v.out.iter().zip(out) {
            cell.set(x as u32);
        }
        for (to, from) in v.matrix_copy.iter().zip(v.matrix.iter()) {
            to.set_bits(from.bits());
        }
        set_point(start);
    }
    Ok(())
}

/// The Rust version (see the header). Refuses as `transform_int_points` (never, ecx 1).
pub fn place_view() -> Result<(), Refused> {
    place_view_with(&first())
}
