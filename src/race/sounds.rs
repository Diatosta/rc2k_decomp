//! The race's sound objects woken at the loading (IDA, 2026-10-15): `off_512A10`, eight
//! pointers to the cars' sound objects (`sound::load_race_sounds`), each with a flag at +8.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read, game_write, rva};

pub const OBJECTS_RVA: usize = rva(0x512A10); // off_512A10
pub const CARS_RVA: usize = rva(0x63DB90); // dword_63DB90: the cars in the race
pub const COUNT_RVA: usize = rva(0x514F7C); // dword_514F7C: the sound objects in use
pub const STARTED_RVA: usize = rva(0x5124E4); // dword_5124E4: 1 once started
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

/// `race::start_race_sounds` (0x40D592): `sound::start_race_clock` (part 0), and unless
/// `dword_514F7C` is 0 it is held to 1 .. 8 and that many objects woken; then
/// `sound::let_sounds_play` (part 1) and `dword_5124E4` 1. Refuses an object missing.
pub unsafe fn start_race_sounds(
    entry: &Regs,
    part: &mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>,
) -> Result<(), Refused> {
    if !objects_there() {
        return Err(BadInput("a race sound object missing").into());
    }
    let mut g = *entry;
    unsafe {
        part(0, &mut g)?;
        let count: u32 = game_read(COUNT_RVA);
        if count != 0 {
            let count = count.clamp(1, OBJECTS);
            game_write(COUNT_RVA, count);
            for n in 0..count {
                wake(n);
            }
        }
        part(1, &mut g)?;
        game_write(STARTED_RVA, 1u32);
    }
    Ok(())
}

pub fn game_part(place: u32, _: &mut Regs) -> Result<(), Refused> {
    unsafe {
        if place == 0 {
            crate::sound::race_clock::start_race_clock(crate::system::tick_count::tick_count());
        } else {
            crate::sound::let_sounds_play::let_sounds_play();
        }
    }
    Ok(())
}

/// `race::wake_car_sounds` (0x45D255): nothing while `dword_71C3E0` is set; else each car's
/// object (`dword_63DB90` of them, the table taken round its eight) woken, then
/// `race::start_race_sounds` (part 2). Leaves ebx the count and edi the last object when it
/// wakes any (the original has no `pusha`). Refuses 0x10000 cars or more (the original
/// counts on to them), an object missing.
pub unsafe fn wake_car_sounds(
    entry: &Regs,
    part: &mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>,
) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        if game_read::<u32>(REPLAY_RVA) != 0 {
            return Ok(g);
        }
        let cars: u32 = game_read(CARS_RVA);
        if cars >= MOST_CARS {
            return Err(BadInput("0x10000 cars or more").into());
        }
        if !objects_there() {
            return Err(BadInput("a race sound object missing").into());
        }
        if cars != 0 {
            for n in 0..cars {
                wake(n);
            }
            g.ebx = cars;
            g.edi = object(cars - 1);
        }
        part(2, &mut g)?;
    }
    Ok(g)
}

/// `wake_car_sounds`' part in the game: `race::start_race_sounds` in Rust.
pub fn game_wake_part(_: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe { start_race_sounds(r, &mut game_part) }
}
