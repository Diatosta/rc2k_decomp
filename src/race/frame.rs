//! The race's frame outside the physics (`sub_444D46`'s callees, IDA 2026-10-17): the
//! players' states, the cars' counters, the view cars loaded into the work copy. Here
//! its leaves.

use crate::errors::Refused;
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::race::objects::CAR_BLOCKS_RVA;
use crate::screens::harness::Left;

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
    // Converted: `rc2k_core::race::cars::players_all_out`.
    use rc2k_core::race::cars::{Players, players_all_out};
    unsafe {
        let out = |n: u32| ((block(n) + 0xB0) as *const u32).read_unaligned() != 0;
        let players = if game_read::<u32>(ONE_CHOSEN_RVA) == 1 {
            Players::Chosen(game_read(CHOSEN_CAR_RVA))
        } else {
            Players::First(game_read(PLAYERS_RVA))
        };
        Ok(players_all_out(players, out)?)
    }
}

/// `race::set_car_counter` (0x43A96C): block esi's message (+0x71C eax, +0x72C 0x12C,
/// +0x730 0), written in the original's order (+0x71C, +0x730, +0x72C).
pub unsafe fn set_car_counter(at: u32, value: u32) {
    // Converted: `rc2k_core::race::cars::Message::new`.
    let m = rc2k_core::race::cars::Message::new(value);
    unsafe {
        ((at + 0x71C) as *mut u32).write_unaligned(m.id);
        ((at + 0x730) as *mut u32).write_unaligned(m.age);
        ((at + 0x72C) as *mut u32).write_unaligned(m.time_left);
    }
}

/// `race::set_both_counters` (0x43A987): `set_car_counter` of the first two blocks (by
/// their fixed addresses), each with eax.
pub unsafe fn set_both_counters(value: u32) {
    // Converted: through `set_car_counter` (`rc2k_core::race::cars::Message`).
    for b in [0x70F3EC, 0x7103CC] {
        unsafe { set_car_counter(game_addr(rva(b)) as u32, value) };
    }
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

pub type Part<'a> = &'a mut dyn FnMut(u32, &mut crate::framework::Regs) -> Result<(), Refused>;

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
/// The keys that flip a bit of the view block's +0x220 (`rc2k_core::race::keys::BIT_KEYS`
/// by place): their key bytes, in the order tested.
pub const BIT_KEYS: [usize; 7] = [
    0x4F6D34, 0x4F6D1E, 0x4F6D1D, 0x4F6D20, 0x4F6D1F, 0x4F6D1C, 0x4F6D33,
];
fn pressed(key: usize) -> bool {
    unsafe { game_read::<u8>(key) != 0 }
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
        // Converted: `rc2k_core::race::keys::{first_key, split_flip, volume_key,
        // volume_step, flip_bit}`.
        use rc2k_core::race::keys::{
            BIT_KEYS as BITS, COUNTED_VIEWS, Key, SHARED_BIT, first_key, flip_bit, split_flip,
            volume_key, volume_step,
        };
        let key = first_key(|k| {
            pressed(match k {
                Key::NextView => NEXT_VIEW_KEY_RVA,
                Key::Split => SPLIT_KEY_RVA,
                Key::VolumeDown => DOWN_KEY_RVA,
                Key::VolumeUp => UP_KEY_RVA,
                Key::Bit(n) => rva(BIT_KEYS[n]),
                Key::Shared => SHARED_KEY_RVA,
            })
        });
        let flip = |r: &mut crate::framework::Regs, bit: (u32, u32, u32)| {
            let flags = (r.esi + 0x220) as *mut u32;
            let (now, counter) = flip_bit(flags.read_unaligned(), bit);
            flags.write_unaligned(now);
            r.eax = counter;
        };
        match key {
            Some(Key::NextView) => {
                crate::drive::next_camera_view::next_camera_view(r.esi).apply(&mut r);
                r.ebx = ((r.esi + 0x688) as *const u32).read_unaligned();
                if r.ebx < COUNTED_VIEWS {
                    r.eax = game_read(VIEW_COUNTERS_RVA + 4 * r.ebx as usize);
                    set_car_counter(r.esi, r.eax);
                }
            }
            Some(Key::Split) => {
                if game_read::<u32>(PLAYERS_RVA) == 1 {
                    return Ok(r);
                }
                let (split, counter) = split_flip(game_read(SPLIT_RVA));
                game_write(SPLIT_RVA, split);
                r.eax = counter;
                set_both_counters(r.eax);
            }
            Some(k @ (Key::VolumeDown | Key::VolumeUp)) => {
                let up = k == Key::VolumeUp;
                let (step, counter) = volume_key(up, second);
                set_both_counters(counter);
                if second {
                    // The "up" step reads `dword_75568C`, not the music's volume (see bugs).
                    let from = if up {
                        OTHER_VOLUME_RVA
                    } else {
                        MUSIC_VOLUME_RVA
                    };
                    r.eax = volume_step(game_read(from), step);
                    game_write(MUSIC_VOLUME_RVA, r.eax);
                    part(2, &mut r)?;
                } else {
                    r.eax = volume_step(game_read(MASTER_VOLUME_RVA), step);
                    game_write(MASTER_VOLUME_RVA, r.eax);
                    part(1, &mut r)?;
                }
            }
            Some(Key::Bit(n)) => {
                flip(&mut r, BITS[n]);
                set_car_counter(r.esi, r.eax);
            }
            Some(Key::Shared) => {
                r.esi = game_read(VIEW_BLOCKS[0]);
                flip(&mut r, SHARED_BIT);
                set_both_counters(r.eax);
            }
            None => r.esi = game_read(VIEW_BLOCKS[0]),
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

/// `race::tyre_change`'s variables (IDA addresses): the copies `dword_5BEBE0` /
/// `dword_5BEBE4` and their sources `dword_5D1444` / `dword_5D2444`; car 0's (block at
/// 0x70F3EC) +0x268 "c1 tyrec" count `dword_70F654`, +0x26C the change's state
/// `dword_70F658`, +0x298 the tyre bits `dword_70F684`, +0x88 the time `dword_70F474`, and
/// +0x354 .. +0x360 `dword_70F740` .. `dword_70F74C` (one per bit 1 .. 4).
pub const TYRE_VARIABLES: [usize; 9] = [
    0x5BEBE0, 0x5BEBE4, 0x5D1444, 0x5D2444, 0x70F654, 0x70F658, 0x70F684, 0x70F474, 0x70F740,
];

/// `race::tyre_change` (0x444F7A), on the variables at `v` (`TYRE_VARIABLES`' order; the
/// last is the first of four dwords). Part 1 is `race::tyre_change_gauge` (eax 0
/// starts it, 1 runs it and gives the time when done, else 0; esi the input device).
/// Gives the registers it leaves (eax, esi).
pub unsafe fn tyre_change_at(
    entry: &crate::framework::Regs,
    v: &[u32],
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
        crate::race::frame::tyre_part(1, &mut r)?;
        put(5, 1);
    }
    if get(5) != 1 {
        return Ok(r);
    }
    r.eax = 1;
    r.esi = 0;
    crate::race::frame::tyre_part(1, &mut r)?;
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

/// `tyre_change`'s part in the game: `race::tyre_change_gauge`'s Rust version (eax its
/// result, the rest kept).
pub fn tyre_part(place: u32, r: &mut crate::framework::Regs) -> Result<(), Refused> {
    let _ = place;
    crate::race::tyre_change_gauge::tyre_change_gauge_in(
        r,
        &mut crate::race::tyre_change_gauge::gauge_part,
    )
}
