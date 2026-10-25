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

/// A variable read at an address (the view's variables are the rain's typed ones or the
/// snow's, still in the marks' block: `ViewPlace` holds their addresses).
unsafe fn read_at<T: Copy>(address: usize) -> T {
    unsafe { (address as *const T).read_unaligned() }
}

/// A variable written at an address.
unsafe fn write_at<T: Copy>(address: usize, value: T) {
    unsafe { (address as *mut T).write_unaligned(value) }
}

/// One of the two near-copies' variables (`place_view`'s and `place_view_2`'s), as addresses.
pub struct ViewPlace {
    pub point: usize,
    pub start_z: u32,
    pub place: usize,
    pub matrix: usize,
    pub out: usize,
    pub height: usize,
    pub matrix_copy: usize,
}

/// `place_view`'s: the rain's.
pub fn first() -> ViewPlace {
    ViewPlace {
        point: RAIN.point[0].addr(),
        start_z: 0x2_0000,
        place: RAIN.view_place[0].addr(),
        matrix: RAIN.view_matrix[0].addr(),
        out: RAIN.view[0].addr(),
        height: crate::marks::rain::RAIN_RULES.height.addr(),
        matrix_copy: RAIN.matrix_copy[0].addr(),
    }
}

/// The shared body (see the header), on `v`'s variables.
pub unsafe fn place_view_with(v: &ViewPlace) -> Result<(), Refused> {
    let start = [0, 0, v.start_z];
    unsafe {
        write_at(v.point, start);
        let point = v.point as u32;
        crate::marks::splines::transform(point, point, 1, v.matrix as u32)?;
        let point: [i32; 3] = read_at(v.point);
        let place: [i32; 3] = read_at(v.place);
        write_at(
            v.out,
            [
                x87::zero_less_int_plus_int(point[0], place[0]),
                x87::zero_less_int_plus_int(point[1], place[1]),
                x87::less_int_plus_int(read_at(v.height), point[2], place[2]),
            ],
        );
        write_at(v.matrix_copy, read_at::<[u32; 9]>(v.matrix));
        write_at(v.point, start);
    }
    Ok(())
}

/// The Rust version (see the header). Refuses as `transform_int_points` (never, ecx 1).
pub unsafe fn place_view() -> Result<(), Refused> {
    unsafe { place_view_with(&first()) }
}
