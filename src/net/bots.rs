//! The network lobby's computer drivers ("bots"): the host's setting as the lobby shows it, the
//! race's bot cars (kinds, numbers, the computer driver's pace) and the bots' results for the
//! lobby's table. The rules are in `rc2k_core::net::bots`.

use crate::errors::Refused;
use crate::net::owner::Net;
use crate::race::owner::Race;
use rc2k_core::net::{bots, record};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU8, Ordering::Relaxed};

/// The host's setting as its record says (0 when no host is listed).
pub fn host_setting(net: &Net) -> u32 {
    (0..net.race_slot.players())
        .map(|n| net.vars.player_block_or_end(n))
        .find(|block| block.record.state().is_host())
        .map_or(0, |block| {
            bots::setting(block.record.head.get(record::DIRECTION))
        })
}

/// The host's car kind (0 when no host is listed).
fn host_kind(net: &Net) -> u32 {
    (0..net.race_slot.players())
        .map(|n| net.vars.player_block_or_end(n))
        .find(|block| block.record.state().is_host())
        .map_or(0, |block| u32::from(block.record.head.get(record::CAR)))
}

/// The bots a race of the listed players would have.
pub fn listed_count(net: &Net) -> u32 {
    bots::count(host_setting(net), net.race_slot.players())
}

/// The host presses the bots' button: the next setting, published at once with the record.
pub fn cycle_own_setting(net: &Net) -> Result<(), Refused> {
    let held = net.vars.own.head.get(record::DIRECTION);
    let next = bots::next_setting(bots::setting(held));
    net.vars
        .own
        .head
        .set(record::DIRECTION, bots::with_setting(held, next));
    crate::net::list_players::list_players_regs(net)
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
pub fn make_drivers(race: &Race, net: &Net) {
    let (players, count) = (net.race_slot.players(), net.race_slot.bots());
    for k in 0..count {
        let car = &race.car_blocks().cars[(players + k) as usize];
        car.driver_kind.set(2);
        car.start_share.set(bots::pace_share(k));
    }
}

/// A bot's result of the races so far.
#[derive(Clone, Copy, Default)]
struct BotResult {
    /// The last race's time, 0 before one or when the bot did not finish.
    time: u32,
    total: u32,
}

static RESULTS: Mutex<[BotResult; 8]> = Mutex::new([BotResult { time: 0, total: 0 }; 8]);

/// The bots' results forgotten (a new session).
pub fn reset_results() {
    if let Ok(mut r) = RESULTS.lock() {
        *r = [BotResult { time: 0, total: 0 }; 8];
    }
}

/// A race ends: each bot's time (when it has finished) is noted, and added to its total.
pub fn note_results(race: &Race, net: &Net) {
    let Ok(mut results) = RESULTS.lock() else {
        return;
    };
    for k in 0..net.race_slot.bots() {
        let car = &race.car_blocks().cars[(net.race_slot.players() + k) as usize];
        let r = &mut results[k as usize];
        if car.finish_flag.get() != 0 {
            r.time = car.stage_time.get();
            r.total = r.total.wrapping_add(r.time);
        } else {
            r.time = 0;
        }
    }
}

/// The bot `k`'s result (the last race's time, the total).
pub fn result(k: u32) -> (u32, u32) {
    RESULTS
        .lock()
        .ok()
        .and_then(|r| r.get(k as usize).map(|r| (r.time, r.total)))
        .unwrap_or((0, 0))
}

/// The sequence byte the host's last bot message had.
static SEQUENCE: AtomicU8 = AtomicU8::new(0);

/// The next sequence byte for the bots' messages.
pub fn next_sequence() -> u8 {
    let next = rc2k_core::net::inbox::next_sequence(SEQUENCE.load(Relaxed));
    SEQUENCE.store(next, Relaxed);
    next
}
