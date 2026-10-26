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
//! `sub_45EFF3`, `sub_45F703`, under `sub_4454D2` (most followed by `sub_43F1A8`). Refuses
//! a count above 0x7C (unsigned: the list would run past the work copy's end, 0x71AFB8),
//! before writing.

use crate::errors::Refused;
use crate::framework::{game_addr, rva};
use crate::physics::car_store::CAR;
use crate::physics::contact_store::CONTACT_WORK;
use crate::physics::tables_store::PHYSICS;
use rc2k_core::physics::saved_car::{restore_count, restored};

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
        let _a = |x: usize| game_addr(rva(x)) as u32;
        Self {
            counter: (PHYSICS.clear_place_count.addr() as u32),
            saved_place: (PHYSICS.saved_place[0].addr() as u32),
            place: (CAR.position[0].addr() as u32),
            saved_matrix: (PHYSICS.saved_matrix[0].addr() as u32),
            matrix: (CAR.crash_matrix[0].addr() as u32),
            saved_list: (CONTACT_WORK.probe_saved_faces[0].addr() as u32),
            list: (CAR.probe_point_face[0].addr() as u32),
            count: (CAR.probe_count.addr() as u32),
        }
    }

    fn at(a: u32) -> u32 {
        crate::pointers::read_or_end::<u32>(a, "physics::restore_saved_car's operand")
    }
}

/// The Rust core on the variables at `p`.
// Converted: `rc2k_core::physics::saved_car::{restore_count, restored}`.
pub unsafe fn restore_saved_car_at(p: &SavedCar) -> Result<(), Refused> {
    use crate::physics::save_car_state::{read_state, write_state};
    unsafe {
        let n = restore_count(SavedCar::at(p.count))?;
        let c = restored(SavedCar::at(p.counter));
        (crate::pointers::host(p.counter) as *mut u32).write_unaligned(c);
        let state = read_state(p.saved_place, p.saved_matrix, p.saved_list, n);
        write_state(&state, p.place, p.matrix, p.list);
        Ok(())
    }
}

/// The Rust version.
pub unsafe fn restore_saved_car() -> Result<(), Refused> {
    unsafe { restore_saved_car_at(&SavedCar::game()) }
}
