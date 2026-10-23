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
//! Caller: `drive::race_mode_cars` (0x436758).
//! Refuses a start count (`dword_6051B4`) above 0x58 (the start tables' room) or a start
//! table car past the eight blocks, before anything; the calls' refusals mid-way (the
//! cars, the starts' variables and the work car put back first; the calls' scratch is
//! not).
//! Shadow mode runs the original alone (`drive::start_car_on_line` does, for
//! `race::reset_car_motion`'s calls into the game).
//! Self-test: a copy whose 24 calls go to stand-ins (`install::stand_ins`), against the
//! Rust version with the same stand-ins: every call with its registers, the registers
//! left and the cars' and starts' memory compared; all put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::math::x87;
use crate::race::car_starts::Part;
use crate::service::harness::{Region, capture, restore};
use rc2k_core::drive::pace_cars::{
    ahead_of_lead, ease_jump, ease_step, first_starts, gap_start, gaps, passed, push_apart,
    wanted_gap,
};

const LOAD: u32 = 0;
const STORE: u32 = 1;
const START: u32 = 2;
const TAKEN: u32 = 3;
const CLEAR: u32 = 4;
const BEHIND: u32 = 5;
const AHEAD: u32 = 6;

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

/// The memory put back on a refusal mid-way (and compared by the test).
pub fn regions() -> Vec<Region> {
    let a = |x: u32| game_addr(rva(x as usize)) as u32;
    let mut r = vec![
        (a(0x70F3EC), 8 * 0xFE0),
        (a(0x719FD8), 0xFE0),
        (a(0x6051A0), 0x40),
        (a(CURRENT), 4),
        (a(0x6365E4), 0xC),
    ];
    r.extend(crate::race::start_place::place_regions());
    r
}

/// One car moved: its slot `from_slot` emptied into the other's (`shift`), a new car at
/// start `start` with `dist` checkpoints on.
fn put_car(
    r: &mut Regs,
    part: &mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>,
    start: u32,
    slot: u32,
    number: u32,
    dist: u32,
    speed: u32,
) -> Result<(), Refused> {
    p(number, start);
    p(0x6051A0, start);
    r.ebx = start;
    r.eax = g(CARS_BY_START + 4 * start);
    p(CURRENT, r.eax);
    p(slot, r.eax);
    part(LOAD, r)?;
    p(0x6051B0, speed);
    r.ebx = dist;
    part(START, r)?;
    part(STORE, r)
}

/// The Rust core, the calls through `part` (`PLACES`).
pub unsafe fn start_cars_behind(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    if let Some(why) = refusal() {
        return Err(BadInput(why).into());
    }
    let all = regions();
    let saved = capture(&all);
    let mut r = *entry;
    let result = unsafe { run(&mut r, part) };
    match result {
        Ok(()) => Ok(r),
        Err(e) => {
            restore(&all, &saved);
            Err(e)
        }
    }
}

// Converted: `rc2k_core::drive::pace_cars` (the rules; the moves and calls stay here).
unsafe fn run(r: &mut Regs, part: Part) -> Result<(), Refused> {
    unsafe {
        if g(0x70FA58) == 0 {
            let [(a, a_dist), (b, b_dist)] = first_starts(g(0x6051BC));
            r.ebx = b;
            p(0x6051C4, a);
            p(0x6051C8, b);
            p(0x6051CC, a_dist);
            p(0x6051D0, b_dist);
            for (start, slot, dist) in [
                (0x6051C4, 0x70FA48, 0x6051CC),
                (0x6051C8, 0x70FA4C, 0x6051D0),
            ] {
                put_car(r, part, g(start), slot, 0x70FA50, g(dist), 0)?;
            }
            p(0x70FA58, 1);
        }
        let count = g(COUNT);
        r.eax = gap_start(g(0x70FA40), count);
        let word = g(0x605034 + 4 * r.eax);
        r.ebx = word & 0xF;
        let (pass, back, sum) = gaps(word);
        p(0x6051D4, pass);
        p(0x6051D8, back);
        r.eax = sum;
        p(0x6051DC, r.eax);
        'moves: {
            if g(0x70FA44) != 0 {
                r.ebx = g(0x70FA48);
                r.esi = block(r.ebx);
                r.eax = at(r.esi + 0x98).wrapping_sub(g(LEAD));
                if passed(r.eax, g(0x6051D4)) {
                    r.eax = g(LEAD).wrapping_sub(g(0x6051D8));
                    part(TAKEN, r)?;
                    if r.carry() {
                        break 'moves;
                    }
                    p(CURRENT, r.ebx);
                    part(LOAD, r)?;
                    part(CLEAR, r)?;
                    part(STORE, r)?;
                    r.eax = g(0x70FA4C);
                    p(0x70FA48, r.eax);
                    r.eax = g(0x70FA54);
                    p(0x70FA50, r.eax);
                    let start = g(COUNT).wrapping_sub(g(0x70FA44));
                    put_car(
                        r,
                        part,
                        start,
                        0x70FA4C,
                        0x70FA54,
                        g(0x6051D8).wrapping_neg(),
                        g(0x5C5D0C),
                    )?;
                    p(0x70FA44, g(0x70FA44).wrapping_sub(1));
                }
            }
            r.eax = g(COUNT).wrapping_sub(3);
            if g(0x70FA44) == r.eax {
                break 'moves;
            }
            r.eax = g(COUNT).wrapping_sub(2).wrapping_sub(g(0x6051C0));
            if g(0x70FA44) >= r.eax {
                break 'moves;
            }
            r.ebx = g(0x70FA4C);
            r.esi = block(r.ebx);
            r.eax = g(LEAD).wrapping_sub(at(r.esi + 0x98));
            if !passed(r.eax, g(0x6051D4)) {
                break 'moves;
            }
            r.eax = g(LEAD).wrapping_add(g(0x6051D8));
            part(TAKEN, r)?;
            if r.carry() {
                break 'moves;
            }
            p(CURRENT, r.ebx);
            part(LOAD, r)?;
            part(CLEAR, r)?;
            part(STORE, r)?;
            r.eax = g(0x70FA50);
            p(0x70FA54, r.eax);
            r.eax = g(0x70FA48);
            p(0x70FA4C, r.eax);
            let start = g(COUNT).wrapping_sub(3).wrapping_sub(g(0x70FA44));
            put_car(r, part, start, 0x70FA48, 0x70FA50, g(0x6051D8), g(0x5C5D0C))?;
            p(0x70FA44, g(0x70FA44).wrapping_add(1));
        }
        if g(0x70F494) == 0 {
            p(0x70FA14, u32::MAX);
            p(0x70FA18, u32::MAX);
            r.ebp = g(COUNT).wrapping_sub(g(0x70FA44));
            r.eax = g(LEAD);
            r.edx = g(0x70FA10);
            for slot in [0x70FA4C, 0x70FA48] {
                r.ebx = g(slot);
                r.esi = block(r.ebx);
                let esi = r.esi;
                if ahead_of_lead(r.eax, at(esi + 0x98), r.edx, || at(esi + 0x624)) {
                    part(AHEAD, r)?;
                } else {
                    r.ebp = r.ebp.wrapping_sub(1);
                    part(BEHIND, r)?;
                }
            }
            p(0x70FA40, r.ebp);
            let gap = |r: &mut Regs, other: u32| {
                r.eax = wanted_gap(g(0x6051DC), g(other), g(0x6051D8));
                r.eax
            };
            let mut to_ease = true;
            if g(0x70FA18) == u32::MAX {
                if g(0x70FA44) == 0 {
                    to_ease = false;
                } else {
                    let v = gap(r, 0x70FA14);
                    p(0x70FA18, v);
                }
            }
            if to_ease && g(0x70FA14) == u32::MAX && g(0x70FA40) != 1 {
                let v = gap(r, 0x70FA18);
                p(0x70FA14, v);
            }
            for (wanted, eased) in [(0x70FA18, 0x70FA20), (0x70FA14, 0x70FA1C)] {
                let jumped;
                (r.eax, jumped) = ease_jump(g(wanted), g(eased));
                if let Some(v) = jumped {
                    p(eased, v);
                }
            }
            for (wanted, eased) in [(0x70FA18, 0x70FA20), (0x70FA14, 0x70FA1C)] {
                let next;
                (r.eax, next) = ease_step(g(wanted), g(eased));
                p(eased, next);
            }
        }
        for v in [0x70FA60, 0x710A40, 0x711A20, 0x712A00] {
            p(v, 0);
        }
        r.ebx = g(0x70FA48);
        r.esi = block(r.ebx);
        r.ebx = g(0x70FA4C);
        r.edi = block(r.ebx);
        r.eax = at(r.esi + 0x98).wrapping_sub(at(r.edi + 0x98));
        p(0x70FA5C, r.eax);
        let Some(push) = push_apart(r.eax) else {
            return Ok(());
        };
        r.eax = push;
        let put = |a: u32, v: u32| (a as *mut u32).write_unaligned(v);
        put(r.esi + 0x674, r.eax);
        r.eax = r.eax.wrapping_neg();
        put(r.edi + 0x674, r.eax);
        let k = g(0x604EC4);
        for b in [r.esi, r.edi] {
            put(b + 0x674, x87::int_times(at(b + 0x674) as i32, k));
        }
        Ok(())
    }
}

/// The calls in the game: the Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            LOAD => crate::drive::load_car::load_car()?.apply(r),
            STORE => crate::drive::store_car::store_car()?.apply(r),
            START => {
                *r = crate::drive::start_car_on_line::start_car_on_line(
                    r,
                    &mut crate::race::car_starts::game_part,
                )?
            }
            TAKEN => crate::drive::checkpoint_taken::checkpoint_taken(r).apply(r),
            CLEAR => crate::drive::clear_start_state::clear_start_state(),
            BEHIND => crate::drive::note_gap_behind::note_gap_behind(r.esi).apply(r),
            _ => crate::drive::note_gap_ahead::note_gap_ahead(r.esi).apply(r),
        }
    }
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { start_cars_behind(r, &mut game_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}

// ---- the self-test's stand-ins ----
