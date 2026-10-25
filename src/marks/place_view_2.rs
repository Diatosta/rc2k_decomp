//! `marks::place_view_2` (0x4C2AA5): `marks::place_view` on other variables (the point
//! (0, 0, 0x80000), the matrix at `dword_8D216C`, the place `dword_8D2160`, the height
//! `flt_8DB53C`, out at `dword_8D2214`, the copy at `dword_8D2220`), then `dword_8D20C0`
//! .. `dword_8D20DC` 0 and the four floats at `xmmword_8D20E0` 0x80000.
//!
//! In:  nothing
//! Out: nothing (its caller reads no register after it)
//!
//! See `marks/place_view.rs`; no variable shared with it. One caller, `sub_4C25F6`.
//!
//! compared as `place_view`.

use crate::errors::Refused;
use crate::marks::place_view::{ViewPlace, place_view_with};
use crate::marks::snow::{SNOW, SNOW_LIGHT};
use crate::math::x87;

/// The snow's view variables.
fn second() -> ViewPlace {
    let (s, l) = (&SNOW, &SNOW_LIGHT);
    ViewPlace {
        point: s.point[0].addr(),
        start_z: 0x8_0000,
        place: s.view_place[0].addr(),
        matrix: s.view_matrix[0].addr(),
        out: s.view[0].addr(),
        height: l.height.addr(),
        matrix_copy: s.matrix_copy[0].addr(),
    }
}

/// The Rust version (see the header).
pub unsafe fn place_view_2() -> Result<(), Refused> {
    let second = second();
    unsafe { place_view_with(&second)? };
    for cell in &SNOW.cleared {
        cell.set(0);
    }
    let z = x87::from_int(SNOW.point[2].get() as i32);
    for cell in &SNOW.filled {
        cell.set_bits(z);
    }
    Ok(())
}
