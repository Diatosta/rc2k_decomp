//! `physics::save_car_state` (0x43DBD5): the counterpart of `physics::restore_saved_car`:
//! `dword_614738` = 0; `dword_71A10C` .. `114` into `dword_6141E0` .. `E8`;
//! `g_crashMatrix` (0x40 bytes) into `unk_6141EC`; `dword_71A238` dwords of
//! `dword_71ADC8` (the probe points' faces) into `dword_6D2814`.
//!
//! In:  nothing
//! Out: eax = `dword_71A114`, ecx 0, esi = 0x71ADC8 + 4 n, edi = 0x6D2814 + 4 n; no flags
//!      read
//!
//! Caller: `sub_43B635` (0x43B719), every physics step. Refuses a count above 0x40
//! (unsigned; `dword_6D2814` has 0x40 before `dword_6D2914`), before writing.

use crate::errors::Refused;
use crate::physics::restore_saved_car::SavedCar;
use rc2k_core::physics::saved_car::{CarState, save_count};

fn at(a: u32) -> u32 {
    crate::pointers::read_or_end::<u32>(a, "physics::save_car_state's operand")
}

/// The pieces at these places, `n` faces.
pub fn read_state(place: u32, matrix: u32, list: u32, n: usize) -> CarState {
    CarState {
        place: std::array::from_fn(|k| at(place + 4 * k as u32)),
        matrix: std::array::from_fn(|k| at(matrix + 4 * k as u32)),
        faces: (0..n as u32).map(|k| at(list + 4 * k)).collect(),
    }
}

/// The pieces written at these places.
pub fn write_state(s: &CarState, place: u32, matrix: u32, list: u32) {
    put_dwords(place, &s.place);
    put_dwords(matrix, &s.matrix);
    put_dwords(list, &s.faces);
}

fn put_dwords(to: u32, v: &[u32]) {
    for (k, x) in v.iter().enumerate() {
        crate::pointers::write_or_end::<u32>(
            to + 4 * k as u32,
            *x,
            "physics::save_car_state's operand",
        );
    }
}

/// The Rust core on the variables at `p` (`restore_saved_car`'s, the other way).
// Converted: `rc2k_core::physics::saved_car::save_count`.
pub unsafe fn save_car_state_at(p: &SavedCar) -> Result<(), Refused> {
    unsafe {
        let n = save_count(at(p.count))?;
        (crate::pointers::host(p.counter) as *mut u32).write_unaligned(0);
        let state = read_state(p.place, p.matrix, p.list, n);
        write_state(&state, p.saved_place, p.saved_matrix, p.saved_list);
        Ok(())
    }
}

/// The Rust version.
pub unsafe fn save_car_state() -> Result<(), Refused> {
    unsafe { save_car_state_at(&SavedCar::game()) }
}
