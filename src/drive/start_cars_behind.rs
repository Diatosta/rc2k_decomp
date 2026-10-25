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
use crate::math::x87;
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::numbers_store::NUMBERS;
use crate::race::tables_store::RACE_TABLES;
use crate::race::work_store::RACE_WORK;
use crate::session::Dword;
use rc2k_core::drive::pace_cars::{
    ahead_of_lead, ease_jump, ease_step, first_starts, gap_start, gaps, passed, push_apart,
    wanted_gap,
};

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// `dword_604ECC`[k]: the car of each start. The table holds 89; the refusal below reads
/// one past it (the start count's 0x58 + 1), the next variable, as the original.
fn car_by_start(k: u32) -> u32 {
    match RACE_TABLES.arcade_car_kinds.get(k as usize) {
        Some(car) => car.get(),
        None => unsafe {
            ((RACE_TABLES.arcade_car_kinds[0].addr() + 4 * k as usize) as *const u32)
                .read_unaligned()
        },
    }
}

fn block(car: u32) -> u32 {
    RACE_WORK.car_block(car as usize)
}

/// The refusals worked out before anything.
fn refusal() -> Option<&'static str> {
    let lead = &CAR_BLOCKS.cars[0];
    let count = RACE_TABLES.arcade_cars.get();
    if count > 0x58 {
        return Some("a start count above 0x58 (the start tables' room)");
    }
    for k in 0..=count + 1 {
        if car_by_start(k) > 7 {
            return Some("a start table car past the eight blocks");
        }
    }
    if lead.pace_cars[0].get() > 7 || lead.pace_cars[1].get() > 7 {
        return Some("a kept car past the eight blocks");
    }
    None
}

/// One car moved: its slot `from_slot` emptied into the other's (`shift`), a new car at
/// start `start` with `dist` checkpoints on.
unsafe fn put_car(
    start: u32,
    slot: &Dword,
    number: &Dword,
    dist: u32,
    speed: u32,
) -> Result<(), Refused> {
    number.set(start);
    RACE_TABLES.start_slot.set(start);
    let car = car_by_start(start);
    RACE_WORK.current_car.set(car);
    slot.set(car);
    unsafe {
        crate::drive::load_car::load_car()?;
        RACE_TABLES.start_speed.set_bits(speed);
        crate::drive::start_car_on_line::start_car_on_line(dist)?;
        crate::drive::store_car::store_car()?;
    }
    Ok(())
}

/// A car taken off the line: made the work car, its start values cleared, stored back.
unsafe fn clear_car(car: u32) -> Result<(), Refused> {
    RACE_WORK.current_car.set(car);
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
    // Block 0, the lead car: mode 2's pace-car state is kept in its fields.
    let lead = &CAR_BLOCKS.cars[0];
    unsafe {
        if lead.pace_placed.get() == 0 {
            let [(a, a_dist), (b, b_dist)] = first_starts(RACE_TABLES.start_place.get());
            RACE_TABLES.pace_start_a.set(a);
            RACE_TABLES.pace_start_b.set(b);
            RACE_TABLES.pace_distance_a.set(a_dist);
            RACE_TABLES.pace_distance_b.set(b_dist);
            for (start, slot, dist) in [
                (
                    &RACE_TABLES.pace_start_a,
                    &lead.pace_cars[0],
                    &RACE_TABLES.pace_distance_a,
                ),
                (
                    &RACE_TABLES.pace_start_b,
                    &lead.pace_cars[1],
                    &RACE_TABLES.pace_distance_b,
                ),
            ] {
                put_car(start.get(), slot, &lead.pace_starts[0], dist.get(), 0)?;
            }
            lead.pace_placed.set(1);
        }
        let count = RACE_TABLES.arcade_cars.get();
        let start = gap_start(lead.mode2_place.get(), count);
        let word = crate::drive::start_car_on_line::car_number(start as usize);
        let (pass, back, sum) = gaps(word);
        RACE_TABLES.pace_pass_gap.set(pass);
        RACE_TABLES.pace_back_gap.set(back);
        RACE_TABLES.pace_gap_sum.set(sum);
        'moves: {
            if lead.mode2_behind.get() != 0 {
                let car = lead.pace_cars[0].get();
                let gone = at(block(car) + 0x98).wrapping_sub(lead.checkpoint.get());
                if passed(gone, RACE_TABLES.pace_pass_gap.get()) {
                    let to = lead
                        .checkpoint
                        .get()
                        .wrapping_sub(RACE_TABLES.pace_back_gap.get());
                    if crate::drive::checkpoint_taken::checkpoint_taken(to) {
                        break 'moves;
                    }
                    clear_car(car)?;
                    lead.pace_cars[0].set(lead.pace_cars[1].get());
                    lead.pace_starts[0].set(lead.pace_starts[1].get());
                    let start = RACE_TABLES
                        .arcade_cars
                        .get()
                        .wrapping_sub(lead.mode2_behind.get());
                    put_car(
                        start,
                        &lead.pace_cars[1],
                        &lead.pace_starts[1],
                        RACE_TABLES.pace_back_gap.get().wrapping_neg(),
                        NUMBERS.twenty.bits(),
                    )?;
                    lead.mode2_behind
                        .set(lead.mode2_behind.get().wrapping_sub(1));
                }
            }
            if lead.mode2_behind.get() == RACE_TABLES.arcade_cars.get().wrapping_sub(3) {
                break 'moves;
            }
            if lead.mode2_behind.get()
                >= RACE_TABLES
                    .arcade_cars
                    .get()
                    .wrapping_sub(2)
                    .wrapping_sub(RACE_TABLES.best_place.get())
            {
                break 'moves;
            }
            let car = lead.pace_cars[1].get();
            let ahead = lead.checkpoint.get().wrapping_sub(at(block(car) + 0x98));
            if !passed(ahead, RACE_TABLES.pace_pass_gap.get()) {
                break 'moves;
            }
            let to = lead
                .checkpoint
                .get()
                .wrapping_add(RACE_TABLES.pace_back_gap.get());
            if crate::drive::checkpoint_taken::checkpoint_taken(to) {
                break 'moves;
            }
            clear_car(car)?;
            lead.pace_starts[1].set(lead.pace_starts[0].get());
            lead.pace_cars[1].set(lead.pace_cars[0].get());
            let start = RACE_TABLES
                .arcade_cars
                .get()
                .wrapping_sub(3)
                .wrapping_sub(lead.mode2_behind.get());
            put_car(
                start,
                &lead.pace_cars[0],
                &lead.pace_starts[0],
                RACE_TABLES.pace_back_gap.get(),
                NUMBERS.twenty.bits(),
            )?;
            lead.mode2_behind
                .set(lead.mode2_behind.get().wrapping_add(1));
        }
        if lead.finish_flag.get() == 0 {
            lead.pace_gaps[0].set(u32::MAX);
            lead.pace_gaps[1].set(u32::MAX);
            let mut behind = RACE_TABLES
                .arcade_cars
                .get()
                .wrapping_sub(lead.mode2_behind.get());
            let lead_checkpoint = lead.checkpoint.get();
            // edx: `dword_70FA10` for the first car, then the gap the first car's note
            // left (the original's register, read again for the second car).
            let mut edx = lead.checkpoint_offset[1].get();
            for slot in [&lead.pace_cars[1], &lead.pace_cars[0]] {
                let b = block(slot.get());
                edx = if ahead_of_lead(lead_checkpoint, at(b + 0x98), edx, || at(b + 0x624)) {
                    crate::drive::note_gap_ahead::note_gap_ahead(b)
                } else {
                    behind = behind.wrapping_sub(1);
                    crate::drive::note_gap_behind::note_gap_behind(b)
                };
            }
            lead.mode2_place.set(behind);
            let gap = |other: &Dword| {
                wanted_gap(
                    RACE_TABLES.pace_gap_sum.get(),
                    other.get(),
                    RACE_TABLES.pace_back_gap.get(),
                )
            };
            let mut to_ease = true;
            if lead.pace_gaps[1].get() == u32::MAX {
                if lead.mode2_behind.get() == 0 {
                    to_ease = false;
                } else {
                    lead.pace_gaps[1].set(gap(&lead.pace_gaps[0]));
                }
            }
            if to_ease && lead.pace_gaps[0].get() == u32::MAX && lead.mode2_place.get() != 1 {
                lead.pace_gaps[0].set(gap(&lead.pace_gaps[1]));
            }
            let pairs = || [1, 0].map(|k| (&lead.pace_gaps[k], &lead.pace_gaps_eased[k]));
            for (wanted, eased) in pairs() {
                if let (_, Some(v)) = ease_jump(wanted.get(), eased.get()) {
                    eased.set(v);
                }
            }
            for (wanted, eased) in pairs() {
                eased.set(ease_step(wanted.get(), eased.get()).1);
            }
        }
        for car in &CAR_BLOCKS.cars[..4] {
            car.pace_push.set_bits(0);
        }
        let first = block(lead.pace_cars[0].get());
        let second = block(lead.pace_cars[1].get());
        let apart = at(first + 0x98).wrapping_sub(at(second + 0x98));
        lead.pace_apart.set(apart);
        let Some(push) = push_apart(apart) else {
            return Ok(());
        };
        let put = |a: u32, v: u32| (a as *mut u32).write_unaligned(v);
        put(first + 0x674, push);
        put(second + 0x674, push.wrapping_neg());
        let k = RACE_TABLES.pace_push_scale.bits();
        for b in [first, second] {
            put(b + 0x674, x87::int_times(at(b + 0x674) as i32, k));
        }
        Ok(())
    }
}
