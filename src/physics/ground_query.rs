//! What a ground search (`physics::ground`) reads and writes, as references: the physics'
//! own fields (`GroundQuery::of_physics`, scattered over the ground work, the ground probe
//! and the course work as the original had them) or a drawing's cells (`GroundCells`, frame
//! plan F4: the shadow's and the course view's searches no longer write the physics' work;
//! held races showed that nothing in the physics read what they left there).

use crate::physics::ground_probe_store::GroundProbe;
use crate::physics::ground_store::GroundWork;
use crate::race::course_store::CourseWork;
use crate::session::{Dword, Float};

/// The search's fields (the physics' names in brackets).
#[derive(Clone, Copy)]
pub struct GroundQuery<'a> {
    /// The place asked for (`ground_x`, `ground_z`).
    pub x: &'a Dword,
    pub z: &'a Dword,
    /// The ground's height under it (`ground_height`), 1 when a face was found
    /// (`ground_found`), 1 once `face_under_point` found one (`face_under_point_done`).
    pub height: &'a Dword,
    pub found: &'a Dword,
    pub done: &'a Dword,
    /// The face's three corner point numbers (`face_corner_numbers`), their places
    /// (`face_corners`), the point copied (`face_point_x`, `face_point_z`) and the
    /// differences (`corner_x_differences`, `corner_y_differences`, `corner_z_differences`).
    pub corner_numbers: &'a [Dword; 3],
    pub corners: &'a [Dword; 9],
    pub point_x: &'a Dword,
    pub point_z: &'a Dword,
    pub x_differences: &'a [Dword; 3],
    pub y_differences: &'a [Dword; 2],
    pub z_differences: &'a [Dword; 3],
    /// `physics::face_shares`' outputs (`face_share_a` .. `face_share_scale`, `plane_f`,
    /// `plane_g`, `share_s`, `share_t`).
    pub share_a: &'a Float,
    pub share_b: &'a Float,
    pub share_c: &'a Float,
    pub share_scale: &'a Float,
    pub plane_f: &'a Float,
    pub plane_g: &'a Float,
    pub share_s: &'a Dword,
    pub share_t: &'a Dword,
    /// The edge test's 64-bit product (`GroundProbe.product`) and the caller's mark
    /// (`GroundProbe.settle_mark`, 0x40 or 0x80).
    pub product: &'a [Dword; 2],
    pub settle_mark: &'a Dword,
    /// The course work's search fields: the face tested (`ground_number`), the face found
    /// (`ground_slot`), its half (`ground_half`) and corners (`ground_corners`), the face
    /// tried first (`ground_face_last`).
    pub number: &'a Dword,
    pub slot: &'a Dword,
    pub half: &'a Dword,
    pub face_corners: &'a [Dword; 3],
    pub last: &'a Dword,
}

impl<'a> GroundQuery<'a> {
    /// The physics' own fields.
    pub fn of_physics(g: &'a GroundWork, p: &'a GroundProbe, c: &'a CourseWork) -> Self {
        GroundQuery {
            x: &g.ground_x,
            z: &g.ground_z,
            height: &g.ground_height,
            found: &g.ground_found,
            done: &g.face_under_point_done,
            corner_numbers: &g.face_corner_numbers,
            corners: &g.face_corners,
            point_x: &g.face_point_x,
            point_z: &g.face_point_z,
            x_differences: &g.corner_x_differences,
            y_differences: &g.corner_y_differences,
            z_differences: &g.corner_z_differences,
            share_a: &g.face_share_a,
            share_b: &g.face_share_b,
            share_c: &g.face_share_c,
            share_scale: &g.face_share_scale,
            plane_f: &g.plane_f,
            plane_g: &g.plane_g,
            share_s: &g.share_s,
            share_t: &g.share_t,
            product: &p.product,
            settle_mark: &p.settle_mark,
            number: &c.ground_number,
            slot: &c.ground_slot,
            half: &c.ground_half,
            face_corners: &c.ground_corners,
            last: &c.ground_face_last,
        }
    }
}

/// A drawing's own search fields (see `GroundQuery`), all 0 to begin with.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct GroundCells {
    x: Dword,
    z: Dword,
    height: Dword,
    found: Dword,
    done: Dword,
    corner_numbers: [Dword; 3],
    corners: [Dword; 9],
    point_x: Dword,
    point_z: Dword,
    x_differences: [Dword; 3],
    y_differences: [Dword; 2],
    z_differences: [Dword; 3],
    share_a: Float,
    share_b: Float,
    share_c: Float,
    share_scale: Float,
    plane_f: Float,
    plane_g: Float,
    share_s: Dword,
    share_t: Dword,
    product: [Dword; 2],
    settle_mark: Dword,
    number: Dword,
    slot: Dword,
    half: Dword,
    face_corners: [Dword; 3],
    last: Dword,
}

impl GroundCells {
    pub fn new() -> Self {
        let d = || Dword::new(0);
        let f = || Float::new(0.0);
        GroundCells {
            x: d(),
            z: d(),
            height: d(),
            found: d(),
            done: d(),
            corner_numbers: std::array::from_fn(|_| d()),
            corners: std::array::from_fn(|_| d()),
            point_x: d(),
            point_z: d(),
            x_differences: std::array::from_fn(|_| d()),
            y_differences: std::array::from_fn(|_| d()),
            z_differences: std::array::from_fn(|_| d()),
            share_a: f(),
            share_b: f(),
            share_c: f(),
            share_scale: f(),
            plane_f: f(),
            plane_g: f(),
            share_s: d(),
            share_t: d(),
            product: std::array::from_fn(|_| d()),
            settle_mark: d(),
            number: d(),
            slot: d(),
            half: d(),
            face_corners: std::array::from_fn(|_| d()),
            last: d(),
        }
    }

    /// The query on these cells.
    pub fn query(&self) -> GroundQuery<'_> {
        GroundQuery {
            x: &self.x,
            z: &self.z,
            height: &self.height,
            found: &self.found,
            done: &self.done,
            corner_numbers: &self.corner_numbers,
            corners: &self.corners,
            point_x: &self.point_x,
            point_z: &self.point_z,
            x_differences: &self.x_differences,
            y_differences: &self.y_differences,
            z_differences: &self.z_differences,
            share_a: &self.share_a,
            share_b: &self.share_b,
            share_c: &self.share_c,
            share_scale: &self.share_scale,
            plane_f: &self.plane_f,
            plane_g: &self.plane_g,
            share_s: &self.share_s,
            share_t: &self.share_t,
            product: &self.product,
            settle_mark: &self.settle_mark,
            number: &self.number,
            slot: &self.slot,
            half: &self.half,
            face_corners: &self.face_corners,
            last: &self.last,
        }
    }
}
