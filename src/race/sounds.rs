//! The race's sound objects woken at the loading (IDA, 2026-10-15): `off_512A10`, eight
//! pointers to the cars' sound objects (`sound::load_race_sounds`), each with a flag at +8.

use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::sound::store::ClockSlot;
use crate::sound::store::ObjectsSlot;
use crate::store::{game_read, game_write};

pub const OBJECTS_RVA: ObjectsSlot = ObjectsSlot::at(0x512A10); // off_512A10
pub const CARS_RVA: usize = rva(0x63DB90); // dword_63DB90: the cars in the race
pub const COUNT_RVA: ObjectsSlot = ObjectsSlot::at(0x514F7C); // dword_514F7C: the sound objects in use
pub const STARTED_RVA: ClockSlot = ClockSlot::at(0x5124E4); // dword_5124E4: 1 once started
pub const REPLAY_RVA: usize = rva(0x71C3E0); // dword_71C3E0
const OBJECTS: u32 = 8;
const MOST_CARS: u32 = 0x1_0000;

pub fn object(n: u32) -> u32 {
    unsafe { game_read(OBJECTS_RVA + 4 * (n & 7) as usize) }
}

unsafe fn wake(n: u32) {
    unsafe { ((object(n) + 8) as *mut u32).write_unaligned(1) };
}

pub fn objects_there() -> bool {
    (0..OBJECTS).all(|n| object(n) != 0)
}

/// `race::start_race_sounds` (0x40D592): `sound::start_race_clock`, and unless
/// `dword_514F7C` is 0 it is held to 1 .. 8 and that many objects woken; then
/// `sound::let_sounds_play` and `dword_5124E4` 1. Refuses an object missing.
pub fn start_race_sounds() -> Result<(), Refused> {
    if !objects_there() {
        return Err(BadInput("a race sound object missing").into());
    }
    unsafe {
        crate::sound::race_clock::start_race_clock(crate::system::tick_count::tick_count());
        let count: u32 = game_read(COUNT_RVA);
        if count != 0 {
            let count = count.clamp(1, OBJECTS);
            game_write(COUNT_RVA, count);
            for n in 0..count {
                wake(n);
            }
        }
        crate::sound::let_sounds_play::let_sounds_play();
        game_write(STARTED_RVA, 1u32);
    }
    Ok(())
}

/// `race::wake_car_sounds` (0x45D255): nothing while `dword_71C3E0` is set; else each car's
/// object (`dword_63DB90` of them, the table taken round its eight) woken, then
/// `race::start_race_sounds`. Returns the count and the last object when it wakes any
/// (the original leaves them in ebx and edi: it has no `pusha`). Refuses 0x10000 cars or
/// more (the original counts on to them), an object missing.
pub fn wake_car_sounds() -> Result<Option<(u32, u32)>, Refused> {
    unsafe {
        if game_read::<u32>(REPLAY_RVA) != 0 {
            return Ok(None);
        }
        let cars: u32 = game_read(CARS_RVA);
        if cars >= MOST_CARS {
            return Err(BadInput("0x10000 cars or more").into());
        }
        if !objects_there() {
            return Err(BadInput("a race sound object missing").into());
        }
        let woken = (cars != 0).then(|| {
            for n in 0..cars {
                wake(n);
            }
            (cars, object(cars - 1))
        });
        start_race_sounds()?;
        Ok(woken)
    }
}
