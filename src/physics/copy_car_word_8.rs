//! `physics::copy_car_word_8` (0x440C66): `g_car` +8 (`dword_719FE0`) into
//! `dword_6140F0`, and `dword_6140EC` 0 (in the gearbox: by the look of it the moment a
//! gear change began; a reading).
//!
//! In:  nothing
//! Out: eax the word; the rest kept; no flags read
//!
//! Caller: `sub_43C8AC`.

use crate::physics::car_store::CAR;
use crate::physics::tables_store::PHYSICS;

unsafe fn core(v: &[u32]) {
    unsafe {
        let w = (crate::pointers::host(v[0]) as *const u32).read_unaligned();
        (crate::pointers::host(v[1]) as *mut u32).write_unaligned(w);
        (crate::pointers::host(v[2]) as *mut u32).write_unaligned(0);
    }
}

fn game() -> Vec<u32> {
    vec![
        CAR.step_clock.addr(),
        PHYSICS.time_marks[1].addr(),
        PHYSICS.time_marks[0].addr(),
    ]
}

/// The Rust version.
pub unsafe fn copy_car_word_8() {
    unsafe { core(&game()) }
}
