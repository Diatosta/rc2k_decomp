//! `ground_height` = c + (a - c) `plane_f` + (b - c) `plane_g`, rounded to an integer (the control
//! word's rounding), with a, b, c the whole numbers `corners[4]`, `corners[7]`, `corners[1]` of
//! the search: by the look of it the height of a course face's plane under a point, from its
//! corners' heights and the point's shares (a reading; set by `physics::face_under_point`).

use crate::physics::ground_query::GroundSearch;

pub fn plane_height(q: &mut GroundSearch) {
    let h = crate::math::x87::plane_height(
        q.corners[4] as i32,
        q.corners[7] as i32,
        q.corners[1] as i32,
        q.plane_f,
        q.plane_g,
    );
    q.height = h as u32;
}
