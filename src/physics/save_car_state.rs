//! `physics::save_car_state` (0x43DBD5): the counterpart of `physics::restore_saved_car`:
//! `dword_614738` = 0; `dword_71A10C` .. `114` into `dword_6141E0` .. `E8`;
//! `g_crashMatrix` (0x40 bytes) into `unk_6141EC`; `dword_71A238` dwords of
//! `dword_71ADC8` (the probe points' faces) into `dword_6D2814`.
//!
//! In:  nothing
//! Out: eax = `dword_71A114`, ecx 0, esi = 0x71ADC8 + 4 n, edi = 0x6D2814 + 4 n; no flags
//!      read
//!
//! Caller: `sub_43B635` (0x43B719), every physics step.
//! Refuses a count above 0x40 (unsigned; `dword_6D2814` has 0x40 before
//! `dword_6D2914`), before writing.
//! Self-test: a copy of the original whose twelve references are the test's, counts 0 ..
//! 0x40; the refusal.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, ECX, EDI, ESI, Regs};
use crate::physics::restore_saved_car::SavedCar;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use rc2k_core::physics::saved_car::{CarState, save_count};

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The pieces at these places, `n` faces.
pub unsafe fn read_state(place: u32, matrix: u32, list: u32, n: usize) -> CarState {
    unsafe {
        CarState {
            place: std::array::from_fn(|k| at(place + 4 * k as u32)),
            matrix: std::array::from_fn(|k| at(matrix + 4 * k as u32)),
            faces: (0..n as u32).map(|k| at(list + 4 * k)).collect(),
        }
    }
}

/// The pieces written at these places.
pub unsafe fn write_state(s: &CarState, place: u32, matrix: u32, list: u32) {
    unsafe {
        put_dwords(place, &s.place);
        put_dwords(matrix, &s.matrix);
        put_dwords(list, &s.faces);
    }
}

unsafe fn put_dwords(to: u32, v: &[u32]) {
    for (k, x) in v.iter().enumerate() {
        unsafe { ((to + 4 * k as u32) as *mut u32).write_unaligned(*x) };
    }
}

/// The Rust core on the variables at `p` (`restore_saved_car`'s, the other way).
// Converted: `rc2k_core::physics::saved_car::save_count`.
pub unsafe fn save_car_state_at(p: &SavedCar) -> Result<Left, Refused> {
    unsafe {
        let n = save_count(at(p.count))?;
        (p.counter as *mut u32).write_unaligned(0);
        let state = read_state(p.place, p.matrix, p.list, n);
        write_state(&state, p.saved_place, p.saved_matrix, p.saved_list);
        let n = n as u32;
        Ok(Left::nothing()
            .with(EAX, state.place[2])
            .with(ECX, 0)
            .with(ESI, p.list + 4 * n)
            .with(EDI, p.saved_list + 4 * n))
    }
}

/// The Rust version.
pub unsafe fn save_car_state() -> Result<Left, Refused> {
    unsafe { save_car_state_at(&SavedCar::game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { save_car_state() }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
