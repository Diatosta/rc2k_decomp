//! `physics::apply_torque` (0x45FB82): one force record (esi: the force at +0, the point
//! it acts at at +0xC) turning a car (edi, its block): the car's spin (+0x184 .. +0x18C)
//! copied to `dword_6D30B4` .. and turned into the car's frame
//! (`physics::turn_by_inverse_at`); the cross product of the record's two vectors
//! (`scene::cross_product`, into `flt_614274` ..); each part of the spin less that part
//! times the step `flt_61388C` times `flt_613FA8 + 4k` (`fld c; fmul; fmul; fchs; fadd;
//! fstp`); turned back into the world (`physics::turn_by_matrix_at`) and stored as the
//! spin. All in `pusha` / `popa` (eax pushed besides). A reading: the force's twist on
//! the car, `flt_613FA8` the inverse inertia.
//!
//! In:  esi the record, edi the car block
//! Out: every register kept; no flags read
//!
//! Caller: `physics::apply_torques` (in a loop over records).

use crate::framework::{game_addr, game_read, rva};
use crate::math::x87_run::neg_product2_plus;
use crate::physics::turn_by_inverse_at::turn_by_inverse_at;
use crate::physics::turn_by_matrix_at::turn_by_matrix_at;
use crate::scene::view_matrix::cross_product;

const WORK_RVA: usize = rva(0x6D30B4); // dword_6D30B4
const CROSS_RVA: usize = rva(0x614274); // flt_614274
const STEP_RVA: usize = rva(0x61388C); // flt_61388C
const INERTIA_RVA: usize = rva(0x613FA8); // flt_613FA8
/// Where a car block's spin is.
pub const SPIN_OFFSET: u32 = 0x184;

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The Rust core: the record at `rec`, the car at `car`, the work vector at `work`, the
/// cross product at `cross`.
pub unsafe fn apply_torque_at(rec: u32, car: u32, work: u32, cross: u32) {
    unsafe {
        for k in 0..3 {
            put(work + 4 * k, at(car + SPIN_OFFSET + 4 * k));
        }
        turn_by_inverse_at(work, car);
        cross_product(rec as usize, rec as usize + 0xC, cross as usize);
        let step = game_read::<u32>(STEP_RVA);
        for k in 0..3 {
            let inertia = game_read::<u32>(INERTIA_RVA + 4 * k as usize);
            let w = work + 4 * k;
            put(
                w,
                neg_product2_plus(at(cross + 4 * k), step, inertia, at(w)),
            );
        }
        turn_by_matrix_at(work, car);
        for k in 0..3 {
            put(car + SPIN_OFFSET + 4 * k, at(work + 4 * k));
        }
    }
}

/// The Rust version (the exact one, for `physics::apply_torques`).
pub unsafe fn apply_torque(rec: u32, car: u32) {
    unsafe {
        apply_torque_at(
            rec,
            car,
            game_addr(WORK_RVA) as u32,
            game_addr(CROSS_RVA) as u32,
        )
    }
}
