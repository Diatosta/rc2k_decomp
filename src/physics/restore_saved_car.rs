//! `physics::restore_saved_car` (0x43DC25): puts a saved state back into the work copy:
//! `dword_614738` + 1; the three dwords `dword_6141E0` .. `E8` into `dword_71A10C` ..
//! `114`; the 0x40 bytes at `unk_6141EC` into `g_crashMatrix` (0x71A08C); and
//! `dword_71A238` dwords from `dword_6D2814` into `dword_71ADC8` (the copy's +0xDF0).
//! What the pieces are (a place, the car's matrix, a list) is a reading.
//!
//! In:  nothing
//! Out: eax = `dword_6141E8`, ecx = 0, esi = 0x6D2814 + 4 n, edi = 0x71ADC8 + 4 n (n the
//!      count); flags the `add`'s (no caller reads them)
//!
//! Callers (7): `sub_43BDD1`, `sub_45D737`, `sub_45D7B6` (twice), `sub_45E3BF`,
//! `sub_45EFF3`, `sub_45F703`, under `sub_4454D2` (most followed by `sub_43F1A8`).
//! Refuses a count above 0x7C (unsigned: the list would run past the work copy's end,
//! 0x71AFB8), before writing.
//! Self-test: a copy of the original whose twelve references are the test's, counts 0 ..
//! 0x7C; the refusal.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{ECX, EDI, ESI, Regs, game_addr, rva};
use crate::screens::harness::Left;
use crate::service::harness::{Region, state_entry};
use rc2k_core::physics::saved_car::{MOST_LISTED, restore_count, restored};

/// The variables' addresses: the game's, or a test's.
#[derive(Clone, Copy)]
pub struct SavedCar {
    pub counter: u32,
    pub saved_place: u32,
    pub place: u32,
    pub saved_matrix: u32,
    pub matrix: u32,
    pub saved_list: u32,
    pub list: u32,
    pub count: u32,
}

impl SavedCar {
    pub fn game() -> Self {
        let a = |x: usize| game_addr(rva(x)) as u32;
        Self {
            counter: a(0x614738),
            saved_place: a(0x6141E0),
            place: a(0x71A10C),
            saved_matrix: a(0x6141EC),
            matrix: a(0x71A08C),
            saved_list: a(0x6D2814),
            list: a(0x71ADC8),
            count: a(0x71A238),
        }
    }

    /// What it writes, for a count of `n`.
    pub fn regions(&self, n: u32) -> Vec<Region> {
        vec![
            (self.counter, 4),
            (self.place, 12),
            (self.matrix, 0x40),
            (self.list, 4 * n.min(MOST_LISTED) as usize),
        ]
    }

    unsafe fn at(a: u32) -> u32 {
        unsafe { (a as *const u32).read_unaligned() }
    }
}

/// The Rust core on the variables at `p`.
// Converted: `rc2k_core::physics::saved_car::{restore_count, restored}`.
pub unsafe fn restore_saved_car_at(p: &SavedCar) -> Result<Left, Refused> {
    use crate::physics::save_car_state::{read_state, write_state};
    unsafe {
        let n = restore_count(SavedCar::at(p.count))?;
        let c = restored(SavedCar::at(p.counter));
        (p.counter as *mut u32).write_unaligned(c);
        let state = read_state(p.saved_place, p.saved_matrix, p.saved_list, n);
        write_state(&state, p.place, p.matrix, p.list);
        let n = n as u32;
        Ok(Left::nothing()
            .with(crate::framework::EAX, state.place[2])
            .with(ECX, 0)
            .with(ESI, p.saved_list + 4 * n)
            .with(EDI, p.list + 4 * n))
    }
}

/// The Rust version.
pub unsafe fn restore_saved_car() -> Result<Left, Refused> {
    unsafe { restore_saved_car_at(&SavedCar::game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { restore_saved_car() }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
