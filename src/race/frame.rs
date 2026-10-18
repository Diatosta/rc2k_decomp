//! The race's frame outside the physics (`sub_444D46`'s callees, IDA 2026-10-17): the
//! players' states, the cars' counters, the view cars loaded into the work copy. Here
//! its leaves.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::race::objects::CAR_BLOCKS_RVA;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at};

const ONE_CHOSEN_RVA: usize = rva(0x52A910); // dword_52A910
const CHOSEN_CAR_RVA: usize = rva(0x52A920); // dword_52A920
pub const PLAYERS_RVA: usize = rva(0x63DB94); // dword_63DB94

fn block(n: u32) -> u32 {
    unsafe { game_read(CAR_BLOCKS_RVA + 4 * n as usize) }
}

/// `race::players_all_out` (0x445506): carry when every player is out (+0xB0 not 0): the
/// chosen car's with `dword_52A910` 1, else each of the `dword_63DB94` players' (none: no
/// carry). Every register kept. Refuses, before anything, a chosen car past 7 or more
/// than 8 players (the original reads past the table).
pub unsafe fn players_all_out() -> Result<bool, Refused> {
    unsafe {
        let out = |n: u32| ((block(n) + 0xB0) as *const u32).read_unaligned() != 0;
        if game_read::<u32>(ONE_CHOSEN_RVA) == 1 {
            let n: u32 = game_read(CHOSEN_CAR_RVA);
            if n > 7 {
                return Err(BadInput("a chosen car past the eight").into());
            }
            return Ok(out(n));
        }
        let players: u32 = game_read(PLAYERS_RVA);
        if players > 8 {
            return Err(BadInput("more than eight players").into());
        }
        if players == 0 {
            return Ok(false);
        }
        Ok((0..players).all(out))
    }
}

/// `race::set_car_counter` (0x43A96C): block esi's +0x71C eax, +0x730 0, +0x72C 0x12C.
pub unsafe fn set_car_counter(at: u32, value: u32) {
    unsafe {
        ((at + 0x71C) as *mut u32).write_unaligned(value);
        ((at + 0x730) as *mut u32).write_unaligned(0);
        ((at + 0x72C) as *mut u32).write_unaligned(0x12C);
    }
}

/// `race::set_both_counters` (0x43A987): `set_car_counter` of the first two blocks (by
/// their fixed addresses), each with eax.
pub unsafe fn set_both_counters(value: u32) {
    for b in [0x70F3EC, 0x7103CC] {
        unsafe { set_car_counter(game_addr(rva(b)) as u32, value) };
    }
}

pub fn counter_regions(b: u32) -> Vec<Region> {
    vec![(b + 0x71C, 0x18)]
}

pub fn both_counter_regions() -> Vec<Region> {
    [0x70F3EC, 0x7103CC]
        .iter()
        .flat_map(|&b| counter_regions(game_addr(rva(b)) as u32))
        .collect()
}

/// The view cars' numbers: `dword_63DBA8`, `dword_63DBAC`, `dword_63DBA0`.
pub const VIEW_CARS: [usize; 3] = [rva(0x63DBA8), rva(0x63DBAC), rva(0x63DBA0)];

/// `race::load_car_a` / `_b` / `_c` (0x45422F, 0x454241, 0x454277): `g_currentCar` the
/// view car's number, `drive::load_car`. Leaves the registers as it does.
pub unsafe fn load_view_car(n: usize) -> Result<Left, Refused> {
    unsafe { with_view_car(n, || crate::drive::load_car::load_car()) }
}

/// `g_currentCar` the view car's number, then `then`; on a refusal `g_currentCar` is put
/// back, so a refusing call has written nothing.
unsafe fn with_view_car(
    n: usize,
    then: impl FnOnce() -> Result<Left, Refused>,
) -> Result<Left, Refused> {
    unsafe {
        let current = crate::drive::car::CURRENT_CAR_RVA;
        let was: u32 = game_read(current);
        game_write(current, game_read::<u32>(VIEW_CARS[n]));
        let result = then();
        if result.is_err() {
            game_write(current, was);
        }
        result
    }
}

/// `race::store_car_a` / `_b` / `_c` (0x454253, 0x454265, 0x454289): `g_currentCar` the
/// view car's number, `drive::store_car`. Leaves the registers as it does.
pub unsafe fn store_view_car(n: usize) -> Result<Left, Refused> {
    unsafe { with_view_car(n, || crate::drive::store_car::store_car()) }
}

/// What a view car's store writes: that car's block (none when it is null: refused) and
/// `g_currentCar`.
pub fn store_regions(n: usize) -> Vec<Region> {
    let car: u32 = unsafe { game_read(VIEW_CARS[n]) };
    let mut regions = vec![at(crate::drive::car::CURRENT_CAR_RVA)];
    let b = block(car & 7);
    if b != 0 {
        regions.push((b, crate::drive::car::CAR_LEN));
    }
    regions
}

/// What a view car's load writes: the work copy and `g_currentCar`.
pub fn load_regions() -> Vec<Region> {
    vec![
        (
            game_addr(crate::drive::car::CAR_RVA) as u32,
            crate::drive::car::CAR_LEN,
        ),
        at(crate::drive::car::CURRENT_CAR_RVA),
    ]
}

pub type Part<'a> = &'a mut dyn FnMut(u32, &mut crate::framework::Regs) -> Result<(), Refused>;

/// `race::race_keys`' sound calls by place: 1 `sound::send_command_8` (0x409BB4), 2
/// `sound::set_music_volume` (0x40F83E). Place 0 is not used (the tests' stand-in 0 gives
/// eax).
pub const KEY_SOUND_CALLEES: [(u32, usize); 2] = [(1, 0x409BB4), (2, 0x40F83E)];

const VIEW_MODE_RVA: usize = rva(0x4F6B79); // byte_4F6B79: the second view's keys (a reading)
const NEXT_VIEW_KEY_RVA: usize = rva(0x4F6D17); // byte_4F6D17
const SPLIT_KEY_RVA: usize = rva(0x4F6D18); // byte_4F6D18
const DOWN_KEY_RVA: usize = rva(0x4F6D19); // byte_4F6D19
const UP_KEY_RVA: usize = rva(0x4F6D1A); // byte_4F6D1A
const SHARED_KEY_RVA: usize = rva(0x4F6D1B); // byte_4F6D1B
pub const SPLIT_RVA: usize = rva(0x6238A4); // dword_6238A4
pub const MASTER_VOLUME_RVA: usize = rva(0x4F82D8); // g_masterVolume
pub const MUSIC_VOLUME_RVA: usize = rva(0x518100); // dword_518100
const OTHER_VOLUME_RVA: usize = rva(0x75568C); // dword_75568C (read by the music "up" step)
pub const VIEW_BLOCKS: [usize; 2] = [rva(0x63DBB0), rva(0x63DBB4)]; // off_63DBB0, off_63DBB4
const VIEW_COUNTERS_RVA: usize = rva(0x6138D4); // dword_6138D4[7]: by camera view
/// The keys that flip a bit of the view block's +0x220: (key, bit, counter when the bit
/// is now clear, when set), in the order tested.
pub const BIT_KEYS: [(usize, u32, u32, u32); 7] = [
    (0x4F6D34, 0x10, 0x117, 0x118),
    (0x4F6D1E, 0x08, 0x119, 0x11A),
    (0x4F6D1D, 0x80, 0x11B, 0x11C),
    (0x4F6D20, 0x01, 0x11D, 0x11E),
    (0x4F6D1F, 0x04, 0x11F, 0x120),
    (0x4F6D1C, 0x02, 0x121, 0x122),
    (0x4F6D33, 0x20, 0x123, 0x124),
];
/// Every key byte the function reads (IDA addresses).
pub const KEY_BYTES: [usize; 13] = [
    0x4F6B79, 0x4F6D17, 0x4F6D18, 0x4F6D19, 0x4F6D1A, 0x4F6D1B, 0x4F6D1C, 0x4F6D1D, 0x4F6D1E,
    0x4F6D1F, 0x4F6D20, 0x4F6D33, 0x4F6D34,
];

fn pressed(key: usize) -> bool {
    unsafe { game_read::<u8>(key) != 0 }
}

/// A volume step: `(v + step) & !3`, held to 0 ..= 0x40.
pub fn volume_step(v: u32, step: i32) -> u32 {
    ((v.wrapping_add(step as u32) & !3) as i32).clamp(0, 0x40) as u32
}

/// `race::race_keys` (0x43A649): the race's view and sound keys (see its header). Gives
/// the registers it leaves.
pub unsafe fn race_keys(
    entry: &crate::framework::Regs,
    part: Part,
) -> Result<crate::framework::Regs, Refused> {
    unsafe {
        let mut r = *entry;
        crate::drive::choose_shown_cars::choose_shown_cars()?.apply(&mut r);
        r.esi = game_read(VIEW_BLOCKS[0]);
        let second = pressed(VIEW_MODE_RVA);
        if second {
            r.esi = game_read(VIEW_BLOCKS[1]);
        }
        if pressed(NEXT_VIEW_KEY_RVA) {
            crate::drive::next_camera_view::next_camera_view(r.esi).apply(&mut r);
            r.ebx = ((r.esi + 0x688) as *const u32).read_unaligned();
            if r.ebx < 7 {
                r.eax = game_read(VIEW_COUNTERS_RVA + 4 * r.ebx as usize);
                set_car_counter(r.esi, r.eax);
            }
            return Ok(r);
        }
        if pressed(SPLIT_KEY_RVA) {
            if game_read::<u32>(PLAYERS_RVA) == 1 {
                return Ok(r);
            }
            let split = game_read::<u32>(SPLIT_RVA) ^ 3;
            game_write(SPLIT_RVA, split);
            r.eax = if split == 1 { 0x127 } else { 0x128 };
            set_both_counters(r.eax);
            return Ok(r);
        }
        let step = if pressed(DOWN_KEY_RVA) {
            Some((-4, 0))
        } else if pressed(UP_KEY_RVA) {
            Some((4, 1))
        } else {
            None
        };
        if let Some((step, n)) = step {
            if second {
                // The "up" step reads `dword_75568C`, not the music's volume (see bugs).
                let from = if n == 0 {
                    MUSIC_VOLUME_RVA
                } else {
                    OTHER_VOLUME_RVA
                };
                set_both_counters(0x115 + n);
                r.eax = volume_step(game_read(from), step);
                game_write(MUSIC_VOLUME_RVA, r.eax);
                part(2, &mut r)?;
            } else {
                set_both_counters(0x113 + n);
                r.eax = volume_step(game_read(MASTER_VOLUME_RVA), step);
                game_write(MASTER_VOLUME_RVA, r.eax);
                part(1, &mut r)?;
            }
            return Ok(r);
        }
        for (key, bit, clear, set) in BIT_KEYS {
            if pressed(rva(key)) {
                let flags = (r.esi + 0x220) as *mut u32;
                flags.write_unaligned(flags.read_unaligned() ^ bit);
                r.eax = if flags.read_unaligned() & bit != 0 {
                    set
                } else {
                    clear
                };
                set_car_counter(r.esi, r.eax);
                return Ok(r);
            }
        }
        r.esi = game_read(VIEW_BLOCKS[0]);
        if pressed(SHARED_KEY_RVA) {
            let flags = (r.esi + 0x220) as *mut u32;
            flags.write_unaligned(flags.read_unaligned() ^ 0x40);
            r.eax = if flags.read_unaligned() & 0x40 != 0 {
                0x126
            } else {
                0x125
            };
            set_both_counters(r.eax);
        }
        Ok(r)
    }
}

/// `race_keys`' sound calls in the game: the Rust versions, sent as the originals send.
pub fn key_sound_part(place: u32, r: &mut crate::framework::Regs) -> Result<(), Refused> {
    use crate::sound::queue::{Queue, send_command, sent};
    use crate::sound::wait_for_commands::{Game, wait_for_commands};
    unsafe {
        match place {
            // `pusha`, `push 8`, `send_command`, `pop eax`, `popa`.
            1 => send_command(&Queue::game(), 8, sent(r)),
            _ => {
                if let Some(with) = crate::sound::set_music_volume::set_music_volume(r) {
                    send_command(&Queue::game(), 0xB, sent(&with));
                    wait_for_commands(&mut Game(sent(&with)));
                }
            }
        }
    }
    Ok(())
}

/// What `race_keys` may write: the eight car blocks, `choose_shown_cars`' variables, the
/// split, both volumes and `set_camera_view`'s step count.
pub fn key_regions() -> Vec<Region> {
    let mut v: Vec<Region> = vec![(game_addr(rva(0x70F3EC)) as u32, 8 * 0xFE0)];
    for a in [
        0x63DBA8, 0x63DBAC, 0x63DB98, 0x6238A4, 0x63DBB0, 0x63DBB4, 0x4F82D8, 0x518100, 0x719FE0,
    ] {
        v.push(at(rva(a)));
    }
    v.push((game_addr(rva(0x641680)) as u32, 0x20));
    v
}

/// `race::tyre_change`'s variables (IDA addresses): the copies `dword_5BEBE0` /
/// `dword_5BEBE4` and their sources `dword_5D1444` / `dword_5D2444`; car 0's (block at
/// 0x70F3EC) +0x268 "c1 tyrec" count `dword_70F654`, +0x26C the change's state
/// `dword_70F658`, +0x298 the tyre bits `dword_70F684`, +0x88 the time `dword_70F474`, and
/// +0x354 .. +0x360 `dword_70F740` .. `dword_70F74C` (one per bit 1 .. 4).
pub const TYRE_VARIABLES: [usize; 9] = [
    0x5BEBE0, 0x5BEBE4, 0x5D1444, 0x5D2444, 0x70F654, 0x70F658, 0x70F684, 0x70F474, 0x70F740,
];

/// `race::tyre_change` (0x444F7A), on the variables at `v` (`TYRE_VARIABLES`' order; the
/// last is the first of four dwords). Part 1 is `sub_41C1D6` (the change's gauge: eax 0
/// starts it, 1 runs it and gives the time when done, else 0; esi the input device).
/// Gives the registers it leaves (eax, esi).
pub unsafe fn tyre_change_at(
    entry: &crate::framework::Regs,
    v: &[u32],
    part: Part,
) -> Result<crate::framework::Regs, Refused> {
    let get = |k: usize| unsafe { (v[k] as *const u32).read_unaligned() };
    let put = |k: usize, x: u32| unsafe { (v[k] as *mut u32).write_unaligned(x) };
    let mut r = *entry;
    put(0, get(2));
    put(1, get(3));
    r.eax = get(6) & 0x1E;
    if r.eax != 0x1E && get(5) == 0 && get(4) >= 0x12C {
        r.eax = 0;
        r.esi = 0;
        part(1, &mut r)?;
        put(5, 1);
    }
    if get(5) != 1 {
        return Ok(r);
    }
    r.eax = 1;
    r.esi = 0;
    part(1, &mut r)?;
    if r.eax == 0 {
        return Ok(r);
    }
    put(7, get(7).wrapping_add(r.eax));
    put(5, 0xFFFF_FED3);
    for bit in 1..=4 {
        let bits = get(6);
        put(6, bits | 1 << bit);
        if bits & 1 << bit == 0 {
            unsafe { ((v[8] + 4 * (bit - 1)) as *mut u32).write_unaligned(0) };
        }
    }
    Ok(r)
}

pub fn tyre_variables() -> Vec<u32> {
    TYRE_VARIABLES
        .iter()
        .map(|&a| game_addr(rva(a)) as u32)
        .collect()
}

/// What `tyre_change` writes (with `sub_41C1D6` stood in for).
pub fn tyre_regions(v: &[u32]) -> Vec<Region> {
    [0usize, 1, 5, 6, 7]
        .iter()
        .map(|&k| (v[k], 4))
        .chain([(v[8], 0x10)])
        .collect()
}

/// `tyre_change`'s part in the game: `sub_41C1D6` (not ported: it draws through
/// DirectDraw) in the game.
pub fn tyre_part(place: u32, r: &mut crate::framework::Regs) -> Result<(), Refused> {
    let _ = place;
    unsafe { crate::framework::call_game(r, game_addr(rva(0x41C1D6))) };
    Ok(())
}
