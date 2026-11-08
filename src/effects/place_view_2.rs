//! `effects::place_view_2` (0x4C2AA5): `effects::place_view` on other variables (the point
//! (0, 0, 0x80000), the matrix at `dword_8D216C`, the place `dword_8D2160`, the height
//! `flt_8DB53C`, out at `dword_8D2214`, the copy at `dword_8D2220`), then `dword_8D20C0`
//! .. `dword_8D20DC` 0 and the four floats at `xmmword_8D20E0` 0x80000.
//!
//! See `marks/place_view.rs`; no variable shared with it. One caller, `effects::run_snow`.
//!
//! compared as `place_view`.

use crate::effects::place_view::{ViewPlace, place_view_with};
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

/// The Rust version (see the header).
pub fn place_view_2(weather: &Weather) -> Result<(), Refused> {
    let second = second(weather);
    let snow = &weather.snow;
    place_view_with(&second)?;
    for cell in &snow.cleared {
        cell.set(0);
    }
    let z = x87::from_int(snow.point[2].get() as i32);
    for cell in &snow.filled {
        cell.set_bits(z);
    }
    Ok(())
}
