//! `effects::place_snow_view`: `effects::place_rain_view` on other variables (the point (0, 0,
//! 0x80000), the matrix at `SnowState.view_matrix[0]`, the place `SnowState.view_place[0]`, the
//! height `SnowLight.height`, out at `SnowState.view[0]`, the copy at `SnowState.matrix_copy[0]`),
//! then `SnowState.cleared[0]` .. `SnowState.cleared[7]` 0 and the four floats of the snow's SSE
//! work set to 0x80000.
//!
//! It shares no variable with `place_rain_view`.

use crate::effects::place_rain_view::{ViewPlace, place_view_with};
use crate::effects::rain_store::Weather;
use crate::math::x87;

pub(super) fn place_snow_view(weather: &Weather) {
    let height = weather.snow_light.borrow().height;
    let mut snow = weather.snow.borrow_mut();
    let s = &mut *snow;
    place_view_with(ViewPlace {
        point: &mut s.point,
        start_z: 0x8_0000,
        place: &s.view_place,
        matrix: &s.view_matrix,
        out: &mut s.view,
        height,
        matrix_copy: &mut s.matrix_copy,
    });
    s.cleared = [0; 8];
    s.filled = [x87::from_int(s.point[2] as i32); 4];
}
