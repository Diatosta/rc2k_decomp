//! `drive::start_cars_behind` (0x43703E): with `run_kind` 2 (by `drive::race_mode_cars`), two
//! computer cars kept near the lead car (car block 0), put back on the line behind or
//! ahead of it as it passes them (`CarsAndReplay.car_blocks.cars[0].mode2_behind` counts the moves; `CarsAndReplay.car_blocks.cars[0].pace_cars[0]` and
//! `CarsAndReplay.car_blocks.cars[0].pace_cars[1]` are their cars, `CarsAndReplay.car_blocks.cars[0].pace_starts[0]` / `54` their start numbers):
//! - once (`CarsAndReplay.car_blocks.cars[0].pace_placed`): the two set at starts `start_place` - 2 and - 1 (cars from
//!   `arcade_car_kinds`), 80 and 40 checkpoints on (`drive::start_car_on_line`);
//! - the gaps by `CarsAndReplay.car_blocks.cars[0].mode2_place` (held to 0 .. `arcade_cars` - 1 by the signs) and the
//!   start's `arcade_car_numbers` (low four bits): 0x2C / 0x28 + it (`pace_pass_gap`, `D8`,
//!   `DC` their sum);
//! - a car the lead has gone `pace_pass_gap` past moved behind it (- `pace_back_gap`), one
//!   too far ahead of it moved ahead (+), when `drive::checkpoint_taken` allows; each
//!   moved car's start values cleared (`drive::clear_start_state`);
//! - unless `CarsAndReplay.car_blocks.cars[0].finish_flag` is set: the cars ahead and behind counted into `CarsAndReplay.car_blocks.cars[0].mode2_place`,
//!   their least gaps (`drive::note_gap_ahead` / `behind`, from -1), the wanted gaps
//!   (0x70FA14, 0x70FA18; held to `pace_back_gap` .. `pace_gap_sum`), eased into 0x70FA1C,
//!   0x70FA20 (times 0x14A; jumps past 1000, else 1/64 of the way);
//! - the four cars' +0x674 0, and for the two when within 8 checkpoints of each other (not
//!   0) a push apart of 8 less the gap (times `pace_push_scale`, as floats).
//!
//! These are the arcade race's opponents (run kind 2 = "arcade" in the developers' mode list, and
//! the only kind `race::set_up_arcade_cars` and `race::panel_neighbours` act on): the game lists up
//! to 89 opponents by start number (`arcade_cars`), but only these two are simulated at a time, one
//! kept behind the player and one ahead, and each is put back at the line behind or ahead of the
//! player when passed or left far behind. The HUD's gaps to the neighbours (`race::panel_neighbours`)
//! are the eased wanted gaps (`pace_gaps_eased`), not measured times.
//!
//! Caller: `drive::race_mode_cars` (0x436758). Refuses a start count (`arcade_cars`) above 0x58
//! (the start tables' room) or a start table car past the eight blocks, before anything; the calls'
//! refusals mid-way (the cars, the starts' variables and the work car put back first; the calls'
//! scratch is not).

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::physics::car_store::Car;
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;
use crate::session::Dword;
use rc2k_core::drive::pace_cars::{
    ahead_of_lead, ease_jump, ease_step, first_starts, gap_start, gaps, passed, push_apart,
    wanted_gap,
};

/// The block of the car in start slot `slot`, or the refusal for none.
fn car(shared: &RaceShared, slot: u32) -> Result<&'static Car, BadInput> {
    shared.race_car_or(slot as usize, "a car's block")
}

/// `arcade_car_kinds`[k]: the car of each start. The table holds 89; the refusal below reads
/// one past it (the start count's 0x58 + 1), the next variable, as the original.
fn car_by_start(shared: &RaceShared, k: u32) -> Result<u32, BadInput> {
    crate::session::read_on_in(
        shared.tables(),
        &shared.tables().arcade_car_kinds,
        k as usize,
        "the start tables",
    )
}

/// The refusals worked out before anything.
fn refusal(race: &Race) -> Result<Option<&'static str>, BadInput> {
    let lead = &race.car_blocks().cars[0];
    let shared = &race.shared;
    let count = shared.tables().arcade_cars.get();
    if count > 0x58 {
        return Ok(Some("a start count above 0x58 (the start tables' room)"));
    }
    for k in 0..=count + 1 {
        if car_by_start(shared, k)? > 7 {
            return Ok(Some("a start table car past the eight blocks"));
        }
    }
    if lead.pace_cars[0].get() > 7 || lead.pace_cars[1].get() > 7 {
        return Ok(Some("a kept car past the eight blocks"));
    }
    Ok(None)
}

/// One car moved: its slot `from_slot` emptied into the other's (`shift`), a new car at
/// start `start` with `dist` checkpoints on.
fn put_car(
    race: &Race,
    start: u32,
    slot: &Dword,
    number: &Dword,
    dist: u32,
    speed: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    number.set(start);
    race.shared.tables().start_slot.set(start);
    let car = car_by_start(&race.shared, start)?;
    race.shared.work.current_car.set(car);
    slot.set(car);
    crate::drive::load_car::load_car(&race.shared)?;
    race.shared.tables().start_speed.set_bits(speed);
    crate::drive::start_car_on_line::start_car_on_line(race, dist, scene)?;
    crate::drive::store_car::store_car(&race.shared)?;
    Ok(())
}

/// A car taken off the line: made the work car, its start values cleared, stored back.
fn clear_car(shared: &RaceShared, car: u32) -> Result<(), Refused> {
    shared.work.current_car.set(car);
    crate::drive::load_car::load_car(shared)?;
    crate::drive::clear_start_state::clear_start_state(shared);
    crate::drive::store_car::store_car(shared)?;
    Ok(())
}

// Converted: `rc2k_core::drive::pace_cars` (the rules; the moves and calls stay here).
pub fn start_cars_behind(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let course = &race.course;
    if let Some(why) = refusal(race)? {
        return Err(BadInput(why).into());
    }
    // Block 0, the lead car: mode 2's pace-car state is kept in its fields.
    let lead = &race.car_blocks().cars[0];
    if lead.pace_placed.get() == 0 {
        let [(a, a_dist), (b, b_dist)] = first_starts(race.shared.tables().start_place.get());
        race.shared.tables().pace_start_a.set(a);
        race.shared.tables().pace_start_b.set(b);
        race.shared.tables().pace_distance_a.set(a_dist);
        race.shared.tables().pace_distance_b.set(b_dist);
        for (start, slot, dist) in [
            (
                &race.shared.tables().pace_start_a,
                &lead.pace_cars[0],
                &race.shared.tables().pace_distance_a,
            ),
            (
                &race.shared.tables().pace_start_b,
                &lead.pace_cars[1],
                &race.shared.tables().pace_distance_b,
            ),
        ] {
            put_car(
                race,
                start.get(),
                slot,
                &lead.pace_starts[0],
                dist.get(),
                0,
                scene,
            )?;
        }
        lead.pace_placed.set(1);
    }
    let count = race.shared.tables().arcade_cars.get();
    let start = gap_start(lead.mode2_place.get(), count);
    let word = crate::drive::start_car_on_line::car_number(&race.shared, start as usize)?;
    let (pass, back, sum) = gaps(word);
    race.shared.tables().pace_pass_gap.set(pass);
    race.shared.tables().pace_back_gap.set(back);
    race.shared.tables().pace_gap_sum.set(sum);
    'moves: {
        if lead.mode2_behind.get() != 0 {
            let c = lead.pace_cars[0].get();
            let gone = car(&race.shared, c)?
                .checkpoint
                .get()
                .wrapping_sub(lead.checkpoint.get());
            if passed(gone, race.shared.tables().pace_pass_gap.get()) {
                let to = lead
                    .checkpoint
                    .get()
                    .wrapping_sub(race.shared.tables().pace_back_gap.get());
                if crate::drive::checkpoint_taken::checkpoint_taken(race, to) {
                    break 'moves;
                }
                clear_car(&race.shared, c)?;
                lead.pace_cars[0].set(lead.pace_cars[1].get());
                lead.pace_starts[0].set(lead.pace_starts[1].get());
                let start = race
                    .shared
                    .tables()
                    .arcade_cars
                    .get()
                    .wrapping_sub(lead.mode2_behind.get());
                put_car(
                    race,
                    start,
                    &lead.pace_cars[1],
                    &lead.pace_starts[1],
                    race.shared.tables().pace_back_gap.get().wrapping_neg(),
                    course.numbers.twenty.bits(),
                    scene,
                )?;
                lead.mode2_behind
                    .set(lead.mode2_behind.get().wrapping_sub(1));
            }
        }
        if lead.mode2_behind.get() == race.shared.tables().arcade_cars.get().wrapping_sub(3) {
            break 'moves;
        }
        if lead.mode2_behind.get()
            >= race
                .shared
                .tables()
                .arcade_cars
                .get()
                .wrapping_sub(2)
                .wrapping_sub(race.shared.tables().best_place.get())
        {
            break 'moves;
        }
        let c = lead.pace_cars[1].get();
        let ahead = lead
            .checkpoint
            .get()
            .wrapping_sub(car(&race.shared, c)?.checkpoint.get());
        if !passed(ahead, race.shared.tables().pace_pass_gap.get()) {
            break 'moves;
        }
        let to = lead
            .checkpoint
            .get()
            .wrapping_add(race.shared.tables().pace_back_gap.get());
        if crate::drive::checkpoint_taken::checkpoint_taken(race, to) {
            break 'moves;
        }
        clear_car(&race.shared, c)?;
        lead.pace_starts[1].set(lead.pace_starts[0].get());
        lead.pace_cars[1].set(lead.pace_cars[0].get());
        let start = race
            .shared
            .tables()
            .arcade_cars
            .get()
            .wrapping_sub(3)
            .wrapping_sub(lead.mode2_behind.get());
        put_car(
            race,
            start,
            &lead.pace_cars[0],
            &lead.pace_starts[0],
            race.shared.tables().pace_back_gap.get(),
            course.numbers.twenty.bits(),
            scene,
        )?;
        lead.mode2_behind
            .set(lead.mode2_behind.get().wrapping_add(1));
    }
    if lead.finish_flag.get() == 0 {
        lead.pace_gaps[0].set(u32::MAX);
        lead.pace_gaps[1].set(u32::MAX);
        let mut behind = race
            .shared
            .tables()
            .arcade_cars
            .get()
            .wrapping_sub(lead.mode2_behind.get());
        let lead_checkpoint = lead.checkpoint.get();
        // The carried gap: the lead's `checkpoint_offset[1]` for the first car, then the gap the
        // first car's note left (read again for the second car).
        let mut edx = lead.checkpoint_offset[1].get();
        for slot in [&lead.pace_cars[1], &lead.pace_cars[0]] {
            let b = car(&race.shared, slot.get())?;
            let is_ahead = ahead_of_lead(lead_checkpoint, b.checkpoint.get(), edx, || {
                b.checkpoint_offset[1].get()
            });
            edx = if is_ahead {
                crate::drive::note_gap_ahead::note_gap_ahead(race, b)
            } else {
                behind = behind.wrapping_sub(1);
                crate::drive::note_gap_behind::note_gap_behind(race, b)
            };
        }
        lead.mode2_place.set(behind);
        let gap = |other: &Dword| {
            wanted_gap(
                race.shared.tables().pace_gap_sum.get(),
                other.get(),
                race.shared.tables().pace_back_gap.get(),
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
    for car in &race.car_blocks().cars[..4] {
        car.pace_push.set_bits(0);
    }
    let first = car(&race.shared, lead.pace_cars[0].get())?;
    let second = car(&race.shared, lead.pace_cars[1].get())?;
    let apart = first.checkpoint.get().wrapping_sub(second.checkpoint.get());
    lead.pace_apart.set(apart);
    let Some(push) = push_apart(apart) else {
        return Ok(());
    };
    first.pace_push.set_bits(push);
    second.pace_push.set_bits(push.wrapping_neg());
    let k = race.shared.tables().pace_push_scale.get();
    for b in [first, second] {
        b.pace_push
            .set(x87::int_times(b.pace_push.bits() as i32, k));
    }
    Ok(())
}
