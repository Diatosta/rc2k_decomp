//! `effects::place_rain_view`: the point (0, 0, 0x20000) turned by the 3 x 3 matrix at
//! `RainState.view_matrix[0]` (`scene::transform_int_points`, in place at `RainState.point[0]`);
//! the place `RainState.view_place[0]` .. less it into `RainState.view[0]`, `RainState.view[1]`,
//! and the rain's height less its z plus `RainState.view_place[2]` into `RainState.view[2]` (each
//! rounded); the matrix's nine dwords copied to `RainState.matrix_copy[0]`; the point put back to
//! (0, 0, 0x20000).

use crate::effects::rain_store::{RainRules, RainState, Weather};
use crate::math::x87;
use crate::math::x87_float::{self as bh, Rounds};

/// One of the two near-copies' variables (`place_rain_view`'s and `place_snow_view`'s).
pub struct ViewPlace<'a> {
    pub point: &'a mut [u32; 3],
    pub start_z: u32,
    pub place: &'a [u32; 3],
    pub matrix: &'a [f32; 9],
    pub out: &'a mut [u32; 3],
    /// The height, read when the view's variables are made (nothing writes it between).
    pub height: f32,
    pub matrix_copy: &'a mut [f32; 9],
}

/// `place_rain_view`'s: the rain's, on its variables.
pub fn first<'a>(rain: &'a mut RainState, rules: &RainRules) -> ViewPlace<'a> {
    ViewPlace {
        point: &mut rain.point,
        start_z: 0x2_0000,
        place: &rain.view_place,
        matrix: &rain.view_matrix,
        out: &mut rain.view,
        height: rules.height,
        matrix_copy: &mut rain.matrix_copy,
    }
}

/// The point of three whole numbers turned by the matrix `m`, each rounded to a whole number as
/// made: x m0 + (y m1 + z m2), x m3 + (y m4 + z m5), y m7 + (x m6 + z m8).
fn turn_int_point(point: [u32; 3], m: &[f32; 9]) -> [u32; 3] {
    bh::at_rounding!(|r| {
        let int = |k: usize| r.int(point[k] as i32);
        let (x, y, z) = (int(0), int(1), int(2));
        let c = |k: usize| r.ld(m[k]);
        let first = x * c(0) + (y * c(1) + z * c(2));
        let second = x * c(3) + (y * c(4) + z * c(5));
        let third = y * c(7) + (x * c(6) + z * c(8));
        [first, second, third].map(|v| r.to_int(v) as u32)
    })
}

/// The shared body (see the header), on `v`'s variables.
pub(super) fn place_view_with(v: ViewPlace) {
    let start = [0, 0, v.start_z];
    *v.point = start;
    *v.point = turn_int_point(*v.point, v.matrix);
    let point = v.point.map(|c| c as i32);
    let place = v.place.map(|c| c as i32);
    *v.out = [
        x87::zero_less_int_plus_int(point[0], place[0]),
        x87::zero_less_int_plus_int(point[1], place[1]),
        bh::to_int(bh::ld(v.height.to_bits()) - bh::int(point[2]) + bh::int(place[2])),
    ]
    .map(|x| x as u32);
    *v.matrix_copy = *v.matrix;
    *v.point = start;
}

/// The rain's view placed.
pub(super) fn place_rain_view(weather: &Weather) {
    place_view_with(first(&mut weather.rain.borrow_mut(), &weather.rain_rules));
}
