//! `physics::put_back_motion_vectors` (0x43BD8B): the three dwords at `dword_614BCC` ..
//! back into the work car's motion (`dword_71A170` .., `g_car` +0x198) and those at
//! `dword_614BDC` .. into its spin (`dword_71A15C` .., +0x184), eax pushed around; then
//! `physics::car_frame_motion`. The undoing of `physics::keep_motion_vectors`.
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B8F1, before `sub_43F7AC`).

use crate::physics::car_frame_motion::car_frame_motion_at;
use crate::physics::car_store::CAR;
use crate::physics::tables_store::PHYSICS;

/// Where the vectors are: the game's or a test's.
#[derive(Clone, Copy)]
pub struct MotionPlaces {
    pub kept_motion: u32,
    pub kept_spin: u32,
    pub motion: u32,
    pub spin: u32,
    pub local: u32,
    pub inverse: u32,
}

impl MotionPlaces {
    pub fn game() -> Self {
        Self {
            kept_motion: (PHYSICS.kept_vectors[0].addr() as u32),
            kept_spin: (PHYSICS.kept_vectors[4].addr() as u32),
            motion: (CAR.velocity[0].addr() as u32),
            spin: (CAR.spin[0].addr() as u32),
            local: (CAR.local_velocity[0].addr() as u32),
            inverse: CAR.car_inverse_matrix[0].addr() as u32,
        }
    }
}

/// The Rust core.
pub unsafe fn put_back_motion_vectors_at(p: &MotionPlaces) {
    unsafe {
        for k in 0..3usize {
            let m = (p.kept_motion as *const u32).add(k).read_unaligned();
            (p.motion as *mut u32).add(k).write_unaligned(m);
        }
        for k in 0..3usize {
            let s = (p.kept_spin as *const u32).add(k).read_unaligned();
            (p.spin as *mut u32).add(k).write_unaligned(s);
        }
        car_frame_motion_at(p.motion, p.inverse, p.local);
    }
}

/// The Rust version.
pub unsafe fn put_back_motion_vectors() {
    unsafe { put_back_motion_vectors_at(&MotionPlaces::game()) }
}
