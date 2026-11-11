//! `effects::place_snow_view` (0x4C2AA5): `effects::place_rain_view` on other variables (the point
//! (0, 0, 0x80000), the matrix at `SnowState.view_matrix[0]`, the place `SnowState.view_place[0]`, the height
//! `SnowLight.height`, out at `SnowState.view[0]`, the copy at `SnowState.matrix_copy[0]`), then `SnowState.cleared[0]`
//! .. `SnowState.cleared[7]` 0 and the four floats at `xmmword_8D20E0` 0x80000.
//!
//! See `marks/place_rain_view.rs`; no variable shared with it. One caller, `effects::run_snow`.
//!
//! compared as `place_rain_view`.

use crate::effects::place_rain_view::{ViewPlace, place_view_with};
use crate::effects::rain::Weather;
use crate::errors::Refused;
use crate::math::x87;

/// The snow's view variables.
fn second(weather: &Weather) -> ViewPlace<'_> {
    let (s, l) = (&weather.snow, &weather.snow_light);
    ViewPlace {
        point: &s.point,
        start_z: 0x8_0000,
        place: &s.view_place,
        matrix: &s.view_matrix,
        out: &s.view,
        height: l.height.bits(),
        matrix_copy: &s.matrix_copy,
    }
}

pub fn place_snow_view(weather: &Weather) -> Result<(), Refused> {
    let second = second(weather);
    let snow = &weather.snow;
    place_view_with(&second)?;
    for cell in &snow.cleared {
        cell.set(0);
    }
    let z = x87::from_int(snow.point[2].get() as i32);
    for cell in &snow.filled {
        cell.set_bits(z.to_bits());
    }
    Ok(())
}
