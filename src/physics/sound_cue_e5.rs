//! `physics::sound_cue_e5` (0x45D028): when the current car's entry in `dword_641680` is
//! not -1, its sound object (`g_raceSoundObjects[g_currentCar & 7]`) gets +0x24C 0xE5,
//! +0x250 0 and +0x10 1 (a reading: a sound to play). All in `pusha` / `popa`.
//!
//! In:  nothing
//! Out: every register kept; no flags read (the caller returns)
//!
//! Caller: `sub_45EDB5` (its tail). Refuses (before anything) a current car past 7, or a
//! null sound object when written.

use crate::errors::Refused;
use crate::framework::{game_addr, rva};
use crate::race::frame_store::FRAME_WORK;
use crate::race::work_store::RACE_WORK;
use rc2k_core::physics::listed_cars::{CUE_E5, cue_e5_object};

fn at(a: u32) -> u32 {
    crate::pointers::read_or_end::<u32>(a, "physics::sound_cue_e5's operand")
}

fn put(a: u32, v: u32) {
    crate::pointers::write_or_end::<u32>(a, v, "physics::sound_cue_e5's operand")
}

/// The core: the car number, the listed table, the sound objects at these places.
// Converted: `rc2k_core::physics::listed_cars::{cue_e5_object, CUE_E5}`.
pub fn sound_cue_e5_at(current: u32, listed: u32, objects: u32) -> Result<(), Refused> {
    let object = cue_e5_object(
        at(current),
        |car| at(listed + 4 * car),
        |slot| at(objects + 4 * slot),
    )?;
    if let Some(object) = object {
        for (offset, v) in CUE_E5 {
            put(object + offset, v);
        }
    }
    Ok(())
}

/// The Rust version.
pub fn sound_cue_e5() -> Result<(), Refused> {
    let _g = |a: usize| game_addr(rva(a));
    sound_cue_e5_at(
        RACE_WORK.current_car.addr(),
        FRAME_WORK.listed_car_sounds[0].addr(),
        crate::pointers::game_address(crate::sound::store::RACE_SOUNDS.table.as_ptr()),
    )
}
