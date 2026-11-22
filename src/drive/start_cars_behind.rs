//! With `run_kind` 2 (the arcade race, by `drive::race_mode_cars`), two computer cars kept near
//! the lead car (car block 0): put at their starts once (`pace_placed`), then moved back on the
//! line behind or ahead of it when it has passed them by `pace_pass_gap` or they are too far
//! ahead (when `drive::checkpoint_taken` allows), their start values cleared; the cars ahead and
//! behind counted, their least gaps noted and the wanted gaps eased (`pace_gaps_eased`: the HUD's
//! gaps to the neighbours, `race::panel_neighbours`, are these, not measured times); the four
//! cars' `pace_push` cleared and the two pushed apart when within 8 checkpoints. The game lists up
//! to 89 opponents (`arcade_cars`) but only these two are simulated at a time.
//!
//! Refuses a start count above 0x58 (the start tables' room) or a start table car past the eight
//! blocks, before anything; the calls' refusals mid-way (the cars, the starts' variables and the
//! work car put back first; the calls' scratch is not).

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::physics::car_store::Car;
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;
use rc2k_core::drive::pace_cars::{
    ahead_of_lead, ease_jump, ease_step, first_starts, gap_start, gaps, passed, push_apart,
    wanted_gap,
};

/// The block of the car in start slot `slot`, or the refusal for none.
fn car(shared: &RaceShared, slot: u32) -> Result<std::cell::Ref<'_, Car>, BadInput> {
    shared.race_car_or(slot as usize, "a car's block")
}

/// The lead car's block (block 0), to read.
fn lead(race: &Race) -> std::cell::Ref<'_, Car> {
    std::cell::Ref::map(race.car_blocks(), |blocks| &blocks.cars[0])
}

/// `change` on the lead car's block (block 0), nothing else borrowed meanwhile.
fn set_lead(race: &Race, change: impl FnOnce(&mut Car)) {
    change(&mut race.car_blocks_mut().cars[0]);
}

/// `arcade_car_kinds`[k]: the car of each start. The table holds 89; the refusal below reads
/// one past it (the start count's 0x58 + 1), the next variable `number_before_first`, as the
/// game does. A start further on (`put_car`'s is dynamic) reads on through the holder.
fn car_by_start(shared: &RaceShared, k: u32) -> Result<u32, BadInput> {
    let tables = shared.tables();
    match k as usize {
        n if n < tables.arcade_car_kinds.len() => Ok(tables.arcade_car_kinds[n].get()),
        n if n == tables.arcade_car_kinds.len() => Ok(tables.number_before_first.get()),
        n => tables.dword_from(crate::race::tables_store::Listed::ArcadeCarKinds, n),
    }
}

/// The refusals worked out before anything.
fn refusal(race: &Race) -> Result<Option<&'static str>, BadInput> {
    let lead = lead(race);
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
    if lead.pace_cars[0] > 7 || lead.pace_cars[1] > 7 {
        return Ok(Some("a kept car past the eight blocks"));
    }
    Ok(None)
}

/// One car moved: its slot `from_slot` emptied into the other's (`shift`), a new car at
/// start `start` with `dist` checkpoints on. The lead's `pace_cars[slot]` and `pace_starts[number]`
/// are set.
fn put_car(
    race: &Race,
    start: u32,
    slot: usize,
    number: usize,
    dist: u32,
    speed: f32,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    set_lead(race, |l| l.pace_starts[number] = start);
    race.shared.tables().start_slot.set(start);
    let car = car_by_start(&race.shared, start)?;
    race.shared.work_mut().current_car = car;
    set_lead(race, |l| l.pace_cars[slot] = car);
    crate::drive::load_car::load_car(&race.shared)?;
    race.shared.tables().start_speed.set(speed);
    crate::drive::start_car_on_line::start_car_on_line(race, dist, scene)?;
    crate::drive::store_car::store_car(&race.shared)?;
    Ok(())
}

/// A car taken off the line: made the work car, its start values cleared, stored back.
fn clear_car(shared: &RaceShared, car: u32) -> Result<(), Refused> {
    shared.work_mut().current_car = car;
    crate::drive::load_car::load_car(shared)?;
    crate::drive::clear_start_state::clear_start_state(shared);
    crate::drive::store_car::store_car(shared)?;
    Ok(())
}

pub fn start_cars_behind(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let course = &race.course;
    if let Some(why) = refusal(race)? {
        return Err(BadInput(why).into());
    }
    // Block 0, the lead car: mode 2's pace-car state is kept in its fields.
    if lead(race).pace_placed == 0 {
        let [(a, a_dist), (b, b_dist)] = first_starts(race.shared.tables().start_place.get());
        race.shared.tables().pace_start_a.set(a);
        race.shared.tables().pace_start_b.set(b);
        race.shared.tables().pace_distance_a.set(a_dist);
        race.shared.tables().pace_distance_b.set(b_dist);
        for (start, slot, dist) in [
            (
                &race.shared.tables().pace_start_a,
                0,
                &race.shared.tables().pace_distance_a,
            ),
            (
                &race.shared.tables().pace_start_b,
                1,
                &race.shared.tables().pace_distance_b,
            ),
        ] {
            put_car(race, start.get(), slot, 0, dist.get(), 0.0, scene)?;
        }
        set_lead(race, |l| l.pace_placed = 1);
    }
    let count = race.shared.tables().arcade_cars.get();
    let start = gap_start(lead(race).mode2_place, count);
    let word = crate::drive::start_car_on_line::car_number(&race.shared, start as usize)?;
    let (pass, back, sum) = gaps(word);
    race.shared.tables().pace_pass_gap.set(pass);
    race.shared.tables().pace_back_gap.set(back);
    race.shared.tables().pace_gap_sum.set(sum);
    'moves: {
        if lead(race).mode2_behind != 0 {
            let (c, lead_checkpoint, behind) = {
                let l = lead(race);
                (l.pace_cars[0], l.checkpoint, l.mode2_behind)
            };
            let gone = car(&race.shared, c)?
                .checkpoint
                .wrapping_sub(lead_checkpoint);
            if passed(gone, race.shared.tables().pace_pass_gap.get()) {
                let to = lead_checkpoint.wrapping_sub(race.shared.tables().pace_back_gap.get());
                if crate::drive::checkpoint_taken::checkpoint_taken(race, to) {
                    break 'moves;
                }
                clear_car(&race.shared, c)?;
                set_lead(race, |l| {
                    l.pace_cars[0] = l.pace_cars[1];
                    l.pace_starts[0] = l.pace_starts[1];
                });
                let start = race.shared.tables().arcade_cars.get().wrapping_sub(behind);
                put_car(
                    race,
                    start,
                    1,
                    1,
                    race.shared.tables().pace_back_gap.get().wrapping_neg(),
                    course.numbers.twenty.get(),
                    scene,
                )?;
                set_lead(race, |l| l.mode2_behind = l.mode2_behind.wrapping_sub(1));
            }
        }
        let behind = lead(race).mode2_behind;
        if behind == race.shared.tables().arcade_cars.get().wrapping_sub(3) {
            break 'moves;
        }
        if behind
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
        let (c, lead_checkpoint) = {
            let l = lead(race);
            (l.pace_cars[1], l.checkpoint)
        };
        let ahead = lead_checkpoint.wrapping_sub(car(&race.shared, c)?.checkpoint);
        if !passed(ahead, race.shared.tables().pace_pass_gap.get()) {
            break 'moves;
        }
        let to = lead_checkpoint.wrapping_add(race.shared.tables().pace_back_gap.get());
        if crate::drive::checkpoint_taken::checkpoint_taken(race, to) {
            break 'moves;
        }
        clear_car(&race.shared, c)?;
        set_lead(race, |l| {
            l.pace_starts[1] = l.pace_starts[0];
            l.pace_cars[1] = l.pace_cars[0];
        });
        let start = race
            .shared
            .tables()
            .arcade_cars
            .get()
            .wrapping_sub(3)
            .wrapping_sub(behind);
        put_car(
            race,
            start,
            0,
            0,
            race.shared.tables().pace_back_gap.get(),
            course.numbers.twenty.get(),
            scene,
        )?;
        set_lead(race, |l| l.mode2_behind = l.mode2_behind.wrapping_add(1));
    }
    if lead(race).finish_flag == 0 {
        set_lead(race, |l| l.pace_gaps = [u32::MAX; 2]);
        let (mode2_behind, lead_checkpoint, mut carried_gap, pace_cars) = {
            let l = lead(race);
            // The carried gap: the lead's `checkpoint_offset[1]` for the first car, then the gap
            // the first car's note left (read again for the second car).
            (
                l.mode2_behind,
                l.checkpoint,
                l.checkpoint_offset[1],
                l.pace_cars,
            )
        };
        let mut behind = race
            .shared
            .tables()
            .arcade_cars
            .get()
            .wrapping_sub(mode2_behind);
        for slot in [pace_cars[1], pace_cars[0]] {
            let (b_checkpoint, b_offset) = {
                let b = car(&race.shared, slot)?;
                (b.checkpoint, b.checkpoint_offset[1])
            };
            let is_ahead = ahead_of_lead(lead_checkpoint, b_checkpoint, carried_gap, || b_offset);
            carried_gap = if is_ahead {
                crate::drive::note_gap_ahead::note_gap_ahead(race, b_checkpoint)
            } else {
                behind = behind.wrapping_sub(1);
                crate::drive::note_gap_behind::note_gap_behind(race, b_checkpoint)
            };
        }
        set_lead(race, |l| l.mode2_place = behind);
        let gap = |other: u32| {
            wanted_gap(
                race.shared.tables().pace_gap_sum.get(),
                other,
                race.shared.tables().pace_back_gap.get(),
            )
        };
        let mut l = race.car_blocks_mut();
        let l = &mut l.cars[0];
        let mut to_ease = true;
        if l.pace_gaps[1] == u32::MAX {
            if l.mode2_behind == 0 {
                to_ease = false;
            } else {
                l.pace_gaps[1] = gap(l.pace_gaps[0]);
            }
        }
        if to_ease && l.pace_gaps[0] == u32::MAX && l.mode2_place != 1 {
            l.pace_gaps[0] = gap(l.pace_gaps[1]);
        }
        for k in [1, 0] {
            if let (_, Some(v)) = ease_jump(l.pace_gaps[k], l.pace_gaps_eased[k]) {
                l.pace_gaps_eased[k] = v;
            }
        }
        for k in [1, 0] {
            l.pace_gaps_eased[k] = ease_step(l.pace_gaps[k], l.pace_gaps_eased[k]).1;
        }
    }
    for car in &mut race.car_blocks_mut().cars[..4] {
        car.pace_push = 0.0;
    }
    let (first_slot, second_slot) = {
        let l = lead(race);
        (l.pace_cars[0], l.pace_cars[1])
    };
    let apart = car(&race.shared, first_slot)?
        .checkpoint
        .wrapping_sub(car(&race.shared, second_slot)?.checkpoint);
    set_lead(race, |l| l.pace_apart = apart);
    let Some(push) = push_apart(apart) else {
        return Ok(());
    };
    // The whole numbers are stored first and each is turned into its float in place
    // (the two cars differ: a gap of 0 pushes nothing).
    let k = race.shared.tables().pace_push_scale.get();
    for (slot, push) in [(first_slot, push), (second_slot, push.wrapping_neg())] {
        race.shared
            .race_car_or_mut(slot as usize, "a car's block")?
            .pace_push = x87::int_times(push as i32, k);
    }
    Ok(())
}
