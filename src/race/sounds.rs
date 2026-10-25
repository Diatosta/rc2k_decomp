//! The race's sound objects woken at the loading (IDA, 2026-10-15): `off_512A10`, eight
//! pointers to the cars' sound objects (`sound::load_race_sounds`), each with a flag at +8.

use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::race::view_store::RACE_VIEW;
use crate::race::work_store::RACE_WORK;

pub const REPLAY_RVA: usize = rva(0x71C3E0); // dword_71C3E0
const OBJECTS: u32 = 8;
const MOST_CARS: u32 = 0x1_0000;

pub fn object(n: u32) -> u32 {
    crate::sound::store::RACE_SOUNDS.table[(n & 7) as usize].get()
}

unsafe fn wake(n: u32) {
    unsafe { crate::sound::store::SoundObject::at(object(n)) }
        .in_use
        .set(1);
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
        let count: u32 = crate::sound::store::RACE_SOUNDS.count.get();
        if count != 0 {
            let count = count.clamp(1, OBJECTS);
            crate::sound::store::RACE_SOUNDS.count.set(count);
            for n in 0..count {
                wake(n);
            }
        }
        crate::sound::let_sounds_play::let_sounds_play();
        crate::sound::store::RACE_CLOCK.started.set(1);
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
        if RACE_WORK.in_credits.bits() != 0 {
            return Ok(None);
        }
        let cars: u32 = RACE_VIEW.race_cars.bits();
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
