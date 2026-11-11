//! `effects::place_rain_view` (0x4C1CC8): the point (0, 0, 0x20000) turned by the 3 x 3 matrix at
//! `RainState.view_matrix[0]` (`scene::transform_int_points`, in place at `RainState.point[0]`); the place
//! `RainState.view_place[0]` .. less it into `RainState.view[0]`, `RainState.view[1]`, and `flt_8C5FE1` less
//! its z plus `RainState.view_place[2]` into `RainState.view[2]` (each rounded); the matrix's nine
//! dwords copied to `RainState.matrix_copy[0]`; the point put back to (0, 0, 0x20000).
//!
//! One caller, `effects::run_rain` (the marks' frame).

use crate::effects::rain::Weather;
use crate::errors::Refused;
use crate::math::x87;
use crate::math::x87_float as bh;
use crate::session::{PlainDword, PlainFloat};

/// One of the two near-copies' variables (`place_rain_view`'s and `place_snow_view`'s), as addresses.
pub struct ViewPlace<'a> {
    pub point: &'a [PlainDword; 3],
    pub start_z: u32,
    pub place: &'a [PlainDword; 3],
    pub matrix: &'a [PlainFloat; 9],
    pub out: &'a [PlainDword; 3],
    /// The height's bits, read when the view's variables are made (nothing writes it between).
    pub height: u32,
    pub matrix_copy: &'a [PlainFloat; 9],
}

/// `place_rain_view`'s: the rain's.
pub fn first(weather: &Weather) -> ViewPlace<'_> {
    let (rain, rules) = (&weather.rain, &weather.rain_rules);
    ViewPlace {
        point: &rain.point,
        start_z: 0x2_0000,
        place: &rain.view_place,
        matrix: &rain.view_matrix,
        out: &rain.view,
        height: rules.height.to_bits(),
        matrix_copy: &rain.matrix_copy,
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
        crate::scene::matrix3::transform_int_points(v.point, v.point, 1, v.matrix)?;
        let point = v.point.each_ref().map(|c| c.get() as i32);
        let place = v.place.each_ref().map(|c| c.get() as i32);
        let out = [
            x87::zero_less_int_plus_int(point[0], place[0]),
            x87::zero_less_int_plus_int(point[1], place[1]),
            bh::to_int(bh::ld(v.height) - bh::int(point[2]) + bh::int(place[2])),
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

/// Refuses as `transform_int_points` (never).
pub fn place_rain_view(weather: &Weather) -> Result<(), Refused> {
    place_view_with(&first(weather))
}
