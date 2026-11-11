//! The camera's place (from `physics::place_camera` 0x438E5A): the
//! first offset turned three times by the camera's angles (the whole-number turns of
//! `math::rotate_point_about_y*`, which stay theirs), then the second offset turned with the car
//! (its matrix, `g_crashMatrix`) and added with the car's place.
//!
//! Floats: each part / 65536 and x 65536 is exact; the turn is
//! `math::vector::rotate_point` in `f64` (at the game's double precision the x87's 80-bit
//! sums round as an `f64`'s; at extended a whole number may land one off at a rounding
//! edge).

use crate::math::geometry::fistp64;
use crate::math::vector::rotate_point;

/// The view mode (`split`) whose figures come from the second record
/// (`place_second`; else `place_usual`).
pub const SECOND_RECORD_MODE: u32 = 2;

/// The second offset (whole numbers, `scale` = 65536 a unit) turned by the car's matrix
/// and made whole again (`fistp`).
pub fn turned_offset(m: &[f32; 15], offset: [i32; 3], scale: f32) -> [i32; 3] {
    let k = scale as f64;
    rotate_point(m, offset.map(|v| v as f64 / k)).map(|v| fistp64(v * k))
}

/// The turned offset plus the car's place (`camera_car` ..), and the camera's place with it added.
pub fn add_offset(place: [u32; 3], turned: [i32; 3], car: [u32; 3]) -> ([u32; 3], [u32; 3]) {
    let add: [u32; 3] = std::array::from_fn(|k| (turned[k] as u32).wrapping_add(car[k]));
    (add, std::array::from_fn(|k| place[k].wrapping_add(add[k])))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offsets() {
        let mut m = [0.0f32; 15];
        // A quarter turn about y (the game's rows): x' = z, z' = -x.
        m[8] = 1.0;
        m[5] = 1.0;
        m[2] = -1.0;
        assert_eq!(
            turned_offset(&m, [0x10000, 5, 0x20000], 65536.0),
            [0x20000, 5, -0x10000]
        );
        m[12] = 0.5;
        assert_eq!(turned_offset(&m, [0, 0, 0], 65536.0), [0x8000, 0, 0]);
        let (add, place) = add_offset([10, 20, 30], [-1, 2, 3], [100, 200, 300]);
        assert_eq!(add, [99, 202, 303]);
        assert_eq!(place, [109, 222, 333]);
    }
}
