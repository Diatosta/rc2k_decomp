//! The network lobby's computer drivers ("bots"): the host's setting as the lobby shows it, the
//! race's bot cars (kinds, numbers, the computer driver's pace) and the bots' results for the
//! lobby's table. The rules are in `rc2k_core::net::bots`.

use crate::errors::Refused;
use crate::net::owner::Net;
use crate::race::owner::Race;
use rc2k_core::net::{bots, record};
use std::sync::atomic::{AtomicU8, Ordering::Relaxed};

/// The host's setting as its record says (0 when no host is listed).
pub(super) fn host_setting(net: &Net) -> u32 {
    (0..net.race_slot.players())
        .map(|n| net.vars.player_block_or_end(n))
        .find(|block| block.record.state().is_host())
        .map_or(0, |block| {
            bots::setting(block.record.get().byte(record::DIRECTION))
        })
}

/// The host's direction byte (0 when no host is listed).
fn host_direction(net: &Net) -> u8 {
    (0..net.race_slot.players())
        .map(|n| net.vars.player_block_or_end(n))
        .find(|block| block.record.state().is_host())
        .map_or(0, |block| block.record.get().byte(record::DIRECTION))
}

/// Whether the host has the bots start spread on the course like the arcade's opponents.
pub(super) fn host_arcade_start(net: &Net) -> bool {
    bots::arcade_start(host_direction(net))
}

/// The host's car kind (0 when no host is listed).
fn host_kind(net: &Net) -> u32 {
    (0..net.race_slot.players())
        .map(|n| net.vars.player_block_or_end(n))
        .find(|block| block.record.state().is_host())
        .map_or(0, |block| u32::from(block.record.get().byte(record::CAR)))
}

/// The bots a race of the listed players would have.
pub(super) fn listed_count(net: &Net) -> u32 {
    bots::count(host_setting(net), net.race_slot.players())
}

/// The host presses the bots' button: the next setting, published at once with the record.
pub(super) fn cycle_own_setting(net: &Net) -> Result<(), Refused> {
    let held = net.vars.own.get().byte(record::DIRECTION);
    let next = bots::next_setting(bots::setting(held));
    net.vars
        .own
        .update(|r| r.set_byte(record::DIRECTION, bots::with_setting(held, next)));
    crate::net::list_players::refresh_players(net)
}

/// The host presses the way-to-start button: grid, then arcade, published with the record.
pub(super) fn toggle_own_start(net: &Net) -> Result<(), Refused> {
    let held = net.vars.own.get().byte(record::DIRECTION);
    net.vars.own.update(|r| {
        r.set_byte(
            record::DIRECTION,
            bots::with_arcade_start(held, !bots::arcade_start(held)),
        );
    });
    crate::net::list_players::refresh_players(net)
}

/// The kinds the bots drive, as every machine works them out from the host's car.
pub fn kinds(race: &Race, net: &Net, count: u32) -> Result<Vec<u32>, Refused> {
    let first = host_kind(net);
    let candidates = crate::race::race_cars::opponent_kinds(&race.shared, first)?;
    Ok(bots::kinds(&candidates, first, count))
}

/// The race number the bot `k` carries (the arcade's table of numbers).
pub fn number(race: &Race, k: u32) -> u32 {
    crate::drive::start_car_on_line::car_number(&race.shared, k as usize).unwrap_or(k + 1)
}

/// Gives the race's bots their driver: the computer drives them (as it drives the arcade's
/// cars) with a pace of their own. Called when the race's cars have been set up.
pub fn make_drivers(
    race: &Race,
    net: &Net,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let (players, count) = (net.race_slot.players(), net.race_slot.bots());
    if count != 0 && host_arcade_start(net) {
        place_like_arcade(race, scene, players, count)?;
    }
    for k in 0..count {
        let car = &race.car_blocks().cars[(players + k) as usize];
        car.driver_kind.set(2);
        car.start_share.set(bots::pace_share(k));
    }
    Ok(())
}

/// Puts the bots where `bots::arcade_places` says, on the line of checkpoints, instead of on
/// the grid. Every machine works from the players' places on the grid, which are the same.
fn place_like_arcade(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    players: u32,
    count: u32,
) -> Result<(), Refused> {
    let shared = &race.shared;
    let checkpoints: Vec<u32> = race.car_blocks().cars[..players as usize]
        .iter()
        .map(|car| car.checkpoint.get())
        .collect();
    let places = bots::arcade_places(
        &checkpoints,
        count,
        shared.vars.get().checkpoint_count.get(),
        race.physics.split_tables().split_colour_count,
    );
    let c = crate::physics::step_inputs::StepInputs::of(race).reset();
    for (k, place) in places.into_iter().enumerate() {
        shared.work_mut().current_car = players + k as u32;
        crate::drive::load_car::load_car(shared)?;
        scene.line_and_shake_mut().line_place = place;
        scene.line_and_shake_mut().line_fraction = 0;
        crate::race::start_place::place_car_on_line(shared, &race.course, scene)?;
        let nearest =
            crate::race::start_place::nearest_checkpoint(c.physics, c.course, shared, scene)?;
        let car = shared.car();
        for cell in [&car.checkpoint, &car.reset_checkpoint, &car.last_pace_call] {
            cell.set(nearest);
        }
        let checks = rc2k_core::race::start::start_checks(
            crate::race::checkpoints_store::check_count(shared.checkpoints(), nearest as usize),
        );
        car.splits_passed.set(checks);
        car.splits_recorded.set(checks);
        crate::race::car_starts::reset_car_motion(&c, scene)?;
        crate::drive::store_car::store_car(shared)?;
    }
    Ok(())
}

/// The sequence byte the host's last bot message had.
static SEQUENCE: AtomicU8 = AtomicU8::new(0);

/// The next sequence byte for the bots' messages.
pub fn next_sequence() -> u8 {
    let next = rc2k_core::net::inbox::next_sequence(SEQUENCE.load(Relaxed));
    SEQUENCE.store(next, Relaxed);
    next
}
