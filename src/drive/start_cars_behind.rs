//! `drive::start_cars_behind` (0x43703E): with `dword_62389C` 2 (by `sub_436758`), two
//! computer cars kept near the lead car (car block 0), put back on the line behind or
//! ahead of it as it passes them (`dword_70FA44` counts the moves; `dword_70FA48` and
//! `dword_70FA4C` are their cars, `dword_70FA50` / `54` their start numbers):
//! - once (`dword_70FA58`): the two set at starts `dword_6051BC` - 2 and - 1 (cars from
//!   `dword_604ECC`), 80 and 40 checkpoints on (`drive::start_car_on_line`);
//! - the gaps by `dword_70FA40` (held to 0 .. `dword_6051B4` - 1 by the signs) and the
//!   start's `dword_605034` (low four bits): 0x2C / 0x28 + it (`dword_6051D4`, `D8`,
//!   `DC` their sum);
//! - a car the lead has gone `dword_6051D4` past moved behind it (- `dword_6051D8`), one
//!   too far ahead of it moved ahead (+), when `drive::checkpoint_taken` allows; each
//!   moved car's start values cleared (`drive::clear_start_state`);
//! - unless `dword_70F494` is set: the cars ahead and behind counted into `dword_70FA40`,
//!   their least gaps (`drive::note_gap_ahead` / `behind`, from -1), the wanted gaps
//!   (0x70FA14, 0x70FA18; held to `dword_6051D8` .. `dword_6051DC`), eased into 0x70FA1C,
//!   0x70FA20 (times 0x14A; jumps past 1000, else 1/64 of the way);
//! - the four cars' +0x674 0, and for the two when within 8 checkpoints of each other (not
//!   0) a push apart of 8 less the gap (times `flt_604EC4`, as floats).
//!
//! By the look of it a time trial's pace cars (a reading).
//!
//! In:  nothing
//! Out: the registers as the path leaves them (through its calls); no flags read
//!
//! Caller: `drive::race_mode_cars` (0x436758). Refuses a start count (`dword_6051B4`) above
//! 0x58 (the start tables' room) or a start table car past the eight blocks, before
//! anything; the calls' refusals mid-way (the cars, the starts' variables and the work car
//! put back first; the calls' scratch is not).

use crate::errors::{BadInput, Refused};
use crate::framework::{game_read, game_write, rva};
use crate::math::x87;
use rc2k_core::drive::pace_cars::{
    ahead_of_lead, ease_jump, ease_step, first_starts, gap_start, gaps, passed, push_apart,
    wanted_gap,
};

fn g(x: u32) -> u32 {
    unsafe { game_read(rva(x as usize)) }
}

fn p(x: u32, v: u32) {
    unsafe { game_write(rva(x as usize), v) }
}

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

const COUNT: u32 = 0x6051B4; // the starts
const CARS_BY_START: u32 = 0x604ECC; // dword_604ECC
const LEAD: u32 = 0x70F484; // car block 0's checkpoint
const CURRENT: u32 = 0x71B3F8; // g_currentCar

fn block(car: u32) -> u32 {
    g(0x71B3B8 + 4 * car)
}

/// The refusals worked out before anything.
fn refusal() -> Option<&'static str> {
    let count = g(COUNT);
    if count > 0x58 {
        return Some("a start count above 0x58 (the start tables' room)");
    }
    for k in 0..=count + 1 {
        if g(CARS_BY_START + 4 * k) > 7 {
            return Some("a start table car past the eight blocks");
        }
    }
    if g(0x70FA48) > 7 || g(0x70FA4C) > 7 {
        return Some("a kept car past the eight blocks");
    }
    None
}

/// One car moved: its slot `from_slot` emptied into the other's (`shift`), a new car at
/// start `start` with `dist` checkpoints on.
unsafe fn put_car(
    start: u32,
    slot: u32,
    number: u32,
    dist: u32,
    speed: u32,
) -> Result<(), Refused> {
    p(number, start);
    p(0x6051A0, start);
    let car = g(CARS_BY_START + 4 * start);
    p(CURRENT, car);
    p(slot, car);
    unsafe {
        crate::drive::load_car::load_car()?;
        p(0x6051B0, speed);
        crate::drive::start_car_on_line::start_car_on_line(dist)?;
        crate::drive::store_car::store_car()?;
    }
    Ok(())
}

/// A car taken off the line: made the work car, its start values cleared, stored back.
unsafe fn clear_car(car: u32) -> Result<(), Refused> {
    p(CURRENT, car);
    unsafe {
        crate::drive::load_car::load_car()?;
        crate::drive::clear_start_state::clear_start_state();
        crate::drive::store_car::store_car()?;
    }
    Ok(())
}

/// The Rust version. The registers it leaves are read by no caller (liveness: the
/// callers' after-sets are kept or dead).
// Converted: `rc2k_core::drive::pace_cars` (the rules; the moves and calls stay here).
pub unsafe fn start_cars_behind() -> Result<(), Refused> {
    if let Some(why) = refusal() {
        return Err(BadInput(why).into());
    }
    unsafe {
        if g(0x70FA58) == 0 {
            let [(a, a_dist), (b, b_dist)] = first_starts(g(0x6051BC));
            p(0x6051C4, a);
            p(0x6051C8, b);
            p(0x6051CC, a_dist);
            p(0x6051D0, b_dist);
            for (start, slot, dist) in [
                (0x6051C4, 0x70FA48, 0x6051CC),
                (0x6051C8, 0x70FA4C, 0x6051D0),
            ] {
                put_car(g(start), slot, 0x70FA50, g(dist), 0)?;
            }
            p(0x70FA58, 1);
        }
        let count = g(COUNT);
        let start = gap_start(g(0x70FA40), count);
        let word = g(0x605034 + 4 * start);
        let (pass, back, sum) = gaps(word);
        p(0x6051D4, pass);
        p(0x6051D8, back);
        p(0x6051DC, sum);
        'moves: {
            if g(0x70FA44) != 0 {
                let car = g(0x70FA48);
                let gone = at(block(car) + 0x98).wrapping_sub(g(LEAD));
                if passed(gone, g(0x6051D4)) {
                    let to = g(LEAD).wrapping_sub(g(0x6051D8));
                    if crate::drive::checkpoint_taken::checkpoint_taken(to) {
                        break 'moves;
                    }
                    clear_car(car)?;
                    p(0x70FA48, g(0x70FA4C));
                    p(0x70FA50, g(0x70FA54));
                    let start = g(COUNT).wrapping_sub(g(0x70FA44));
                    put_car(
                        start,
                        0x70FA4C,
                        0x70FA54,
                        g(0x6051D8).wrapping_neg(),
                        g(0x5C5D0C),
                    )?;
                    p(0x70FA44, g(0x70FA44).wrapping_sub(1));
                }
            }
            if g(0x70FA44) == g(COUNT).wrapping_sub(3) {
                break 'moves;
            }
            if g(0x70FA44) >= g(COUNT).wrapping_sub(2).wrapping_sub(g(0x6051C0)) {
                break 'moves;
            }
            let car = g(0x70FA4C);
            let ahead = g(LEAD).wrapping_sub(at(block(car) + 0x98));
            if !passed(ahead, g(0x6051D4)) {
                break 'moves;
            }
            let to = g(LEAD).wrapping_add(g(0x6051D8));
            if crate::drive::checkpoint_taken::checkpoint_taken(to) {
                break 'moves;
            }
            clear_car(car)?;
            p(0x70FA54, g(0x70FA50));
            p(0x70FA4C, g(0x70FA48));
            let start = g(COUNT).wrapping_sub(3).wrapping_sub(g(0x70FA44));
            put_car(start, 0x70FA48, 0x70FA50, g(0x6051D8), g(0x5C5D0C))?;
            p(0x70FA44, g(0x70FA44).wrapping_add(1));
        }
        if g(0x70F494) == 0 {
            p(0x70FA14, u32::MAX);
            p(0x70FA18, u32::MAX);
            let mut behind = g(COUNT).wrapping_sub(g(0x70FA44));
            let lead = g(LEAD);
            // edx: `dword_70FA10` for the first car, then the gap the first car's note
            // left (the original's register, read again for the second car).
            let mut edx = g(0x70FA10);
            for slot in [0x70FA4C, 0x70FA48] {
                let b = block(g(slot));
                edx = if ahead_of_lead(lead, at(b + 0x98), edx, || at(b + 0x624)) {
                    crate::drive::note_gap_ahead::note_gap_ahead(b)
                } else {
                    behind = behind.wrapping_sub(1);
                    crate::drive::note_gap_behind::note_gap_behind(b)
                };
            }
            p(0x70FA40, behind);
            let gap = |other: u32| wanted_gap(g(0x6051DC), g(other), g(0x6051D8));
            let mut to_ease = true;
            if g(0x70FA18) == u32::MAX {
                if g(0x70FA44) == 0 {
                    to_ease = false;
                } else {
                    p(0x70FA18, gap(0x70FA14));
                }
            }
            if to_ease && g(0x70FA14) == u32::MAX && g(0x70FA40) != 1 {
                p(0x70FA14, gap(0x70FA18));
            }
            for (wanted, eased) in [(0x70FA18, 0x70FA20), (0x70FA14, 0x70FA1C)] {
                if let (_, Some(v)) = ease_jump(g(wanted), g(eased)) {
                    p(eased, v);
                }
            }
            for (wanted, eased) in [(0x70FA18, 0x70FA20), (0x70FA14, 0x70FA1C)] {
                p(eased, ease_step(g(wanted), g(eased)).1);
            }
        }
        for v in [0x70FA60, 0x710A40, 0x711A20, 0x712A00] {
            p(v, 0);
        }
        let first = block(g(0x70FA48));
        let second = block(g(0x70FA4C));
        let apart = at(first + 0x98).wrapping_sub(at(second + 0x98));
        p(0x70FA5C, apart);
        let Some(push) = push_apart(apart) else {
            return Ok(());
        };
        let put = |a: u32, v: u32| (a as *mut u32).write_unaligned(v);
        put(first + 0x674, push);
        put(second + 0x674, push.wrapping_neg());
        let k = g(0x604EC4);
        for b in [first, second] {
            put(b + 0x674, x87::int_times(at(b + 0x674) as i32, k));
        }
        Ok(())
    }
}
