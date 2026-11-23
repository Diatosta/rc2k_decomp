//! The Discord status from the game's state: read at the end of each frame into a small key,
//! and only when the key changed made into a status and handed to `platform::discord`.
//!
//! - Racing: `Course::in_race`. The event: a network race (`race_slot`), a replay being played
//!   (`replay_mode` 2), the arcade (race kind 2, its level from `Arcade`), else by the event's
//!   kind (`event_kind`: 0x14 .. 0x17 the time trial, 9 .. 0x13 a championship; 0x18, 8: the
//!   single rally) and, for the one-car races of a split screen, its type (`event_type`: 3 time
//!   trial, 4 and 5 the championships).
//! - The stage: the track entry whose stage number is the race's (`stage_number`), its name's
//!   text number in English.
//! - The car: the first player's car number's description, its name's text number in English
//!   (left out in a network race and a replay, which do not run on the player's own choice).

use crate::owners::Owners;
use rc2k_core::discord::{Event, Situation, activity, car_name, stage_name};
use std::sync::Mutex;

/// What the status is made from.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Key {
    racing: bool,
    replay: bool,
    /// The players of a network race, 0 outside one.
    network: u32,
    run_kind: u32,
    event_type: u32,
    event_kind: u32,
    stage: u32,
    car: u32,
    level: u32,
}

/// The key last seen and the second its race began.
static LAST: Mutex<Option<(Key, u64)>> = Mutex::new(None);

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| since.as_secs())
}

fn key_of(o: Owners<'static>) -> Key {
    let race = o.race;
    if !race.course.in_race() {
        return Key {
            racing: false,
            replay: false,
            network: 0,
            run_kind: 0,
            event_type: 0,
            event_kind: 0,
            stage: 0,
            car: 0,
            level: 0,
        };
    }
    let event = o.menus.screens.event.read();
    Key {
        racing: true,
        replay: race.course.replay_mode() == 2,
        network: if o.net.race_slot.is_on() {
            o.net.race_slot.players().max(1)
        } else {
            0
        },
        run_kind: race.shared.vars.get().run_kind.get(),
        event_type: event.event_type,
        event_kind: event.event_kind,
        stage: race.shared.vars.get().stage_number.get(),
        car: crate::menu::player::player(o.menus, 0).map_or(u32::MAX, |p| p.get().car_number),
        level: o.menus.play.arcade.read().level,
    }
}

fn event_of(key: &Key) -> Event {
    if key.network != 0 {
        return Event::Network {
            players: key.network,
        };
    }
    if key.replay {
        return Event::Replay;
    }
    if key.run_kind == 2 {
        return Event::Arcade {
            level: key.level + 1,
        };
    }
    let by_type = || match key.event_type {
        3 => Event::TimeTrial,
        4 => Event::Championship,
        5 => Event::A8Championship,
        _ => Event::SingleRally,
    };
    match key.event_kind {
        0x14..=0x17 => Event::TimeTrial,
        9..=0x13 if key.event_type == 5 => Event::A8Championship,
        9..=0x13 => Event::Championship,
        0x18 | 8 => Event::SingleRally,
        _ => by_type(),
    }
}

fn situation_of(o: Owners<'static>, key: &Key) -> Situation {
    if !key.racing {
        return Situation::Menus;
    }
    let stage = o
        .system
        .records()
        .stage_tables
        .trak_pak
        .tracks
        .iter()
        .find(|track| track.stage.get() == key.stage)
        .and_then(|track| stage_name(track.name.get()));
    let car = if key.network != 0 || key.replay {
        None
    } else {
        o.menus
            .cars
            .table
            .read()
            .descriptions
            .iter()
            .find(|d| d.car == key.car)
            .and_then(|d| car_name(d.name))
    };
    Situation::Racing {
        event: event_of(key),
        car,
        stage,
    }
}

/// A run with no window the player sees (the checks' copies, `SDL_VIDEODRIVER=offscreen`) or the
/// child that renders a replay to a video: it leaves the player's Discord status alone.
fn unseen_run() -> bool {
    static UNSEEN: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *UNSEEN.get_or_init(|| {
        std::env::var("SDL_VIDEODRIVER")
            .is_ok_and(|driver| driver.eq_ignore_ascii_case("offscreen"))
    }) || crate::video::active()
}

/// Called at the end of every frame: hands the status over when what it is made from changed.
pub fn tick(o: Owners<'static>) {
    if !crate::platform::discord::wanted() || unseen_run() {
        return;
    }
    let key = key_of(o);
    let mut last = LAST
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let start = match *last {
        Some((seen, _)) if seen == key => return,
        // The race's time runs on through what changes inside it (the arcade's next stage
        // starts a new one).
        Some((seen, began)) if seen.racing && key.racing && seen.stage == key.stage => began,
        _ => now(),
    };
    *last = Some((key, start));
    drop(last);
    let situation = situation_of(o, &key);
    crate::platform::discord::post(Some(activity(&situation, start)));
}
