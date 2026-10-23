//! `saves::gather_savegame` (0x4AB427): copies the state a saved game holds into a buffer.
//!
//! In:  edi = where it goes (0x725C bytes)
//! Out: every register preserved
//! In this order:
//!   - seven dwords: `dword_755984`, `dword_755950`, the event's stage index and section
//!     (0x755958, 0x75595C), `dword_755988`, `dword_7556D0`, `dword_75AB40`;
//!   - the four players `off_75647C[0 .. 4]`, 0x28C bytes each;
//!   - 180 records of 0x94 bytes from `unk_73B674`.
//!
//! Before the cars are copied it also changes them: car 0 gets `dword_7554C0` at +0x14 and
//! `dword_7554C4` at +0x274, and every car gets the 0x20 bytes its pointer at +0x18 points to
//! copied to +0x124 (a name, by its size; a reading), so that the saved copy carries them.
//! What the seven dwords and the records are has not been worked out.
//! Refuses, before anything is written: a missing player or a car without that pointer.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::BadInput;
use crate::framework::{game_addr, game_read, rva};
use std::ptr::copy_nonoverlapping;

// ---- game data ----

/// The seven variables at the start, in the order they are saved.
pub const HEAD_RVAS: [usize; 7] = [
    rva(0x755984), // dword_755984
    rva(0x755950), // dword_755950
    rva(0x755958), // index: the event's stage in its section
    rva(0x75595C), // section
    rva(0x755988), // dword_755988
    rva(0x7556D0), // dword_7556D0
    rva(0x75AB40), // dword_75AB40
];
const PLAYERS_RVA: usize = rva(0x75647C); // g_players[4]
pub use rc2k_core::saves::savegame::{CAR_LEN, CARS, NAME_LEN, RECORDS_LEN};
pub const CAR0_A_RVA: usize = rva(0x7554C0); // dword_7554C0 -> car 0 +0x14
pub const CAR0_B_RVA: usize = rva(0x7554C4); // dword_7554C4 -> car 0 +0x274
pub use rc2k_core::saves::savegame::{NAME_AT as NAME_OFFSET, NAME_POINTER as NAME_PTR_OFFSET};
pub const RECORDS_RVA: usize = rva(0x73B674); // unk_73B674

/// The four players, if every one is there and has its pointer at +0x18.
pub fn cars() -> Result<[u32; CARS], BadInput> {
    let mut cars = [0u32; CARS];
    for (n, car) in cars.iter_mut().enumerate() {
        *car = unsafe { ((game_addr(PLAYERS_RVA) as *const u32).add(n)).read_unaligned() };
        if *car == 0 {
            return Err(BadInput(
                "a player is missing: the original reads through null",
            ));
        }
        if unsafe { ((*car + NAME_PTR_OFFSET) as *const u32).read_unaligned() } == 0 {
            return Err(BadInput(
                "a car has no pointer at +0x18: the original reads through null",
            ));
        }
    }
    Ok(cars)
}

// ---- Rust version ----

pub unsafe fn gather_savegame(buffer: *mut u8) -> Result<(), BadInput> {
    use rc2k_core::saves::savegame::{CAR0_A, CAR0_B, RECORDS_AT, car_at};
    // Converted: `rc2k_core::saves::savegame` (the state's layout).
    let cars = cars()?;
    unsafe {
        for (n, at) in HEAD_RVAS.into_iter().enumerate() {
            (buffer.add(4 * n) as *mut u32).write_unaligned(game_read(at));
        }
        ((cars[0] + CAR0_A) as *mut u32).write_unaligned(game_read(CAR0_A_RVA));
        ((cars[0] + CAR0_B) as *mut u32).write_unaligned(game_read(CAR0_B_RVA));
        for (n, car) in cars.into_iter().enumerate() {
            let name = ((car + NAME_PTR_OFFSET) as *const u32).read_unaligned();
            std::ptr::copy(name as *const u8, (car + NAME_OFFSET) as *mut u8, NAME_LEN);
            copy_nonoverlapping(car as *const u8, buffer.add(car_at(n)), CAR_LEN);
        }
        copy_nonoverlapping(
            game_addr(RECORDS_RVA) as *const u8,
            buffer.add(RECORDS_AT),
            RECORDS_LEN,
        );
    }
    Ok(())
}
