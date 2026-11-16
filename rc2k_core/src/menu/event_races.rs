//! An event's races: the computer drivers' times shaken by lot (eight get time added, one to four
//! drop out), the standings before the race, then the players' races by the event mode, the event
//! ended when every result is -1. Its steps, on `EventRaces`.

/// The drivers the lot picks from (the first 0xB2).
pub const PICKED: u32 = 0xB2;
/// The time added to eight drivers: 0x4844 and a random number under 3000.
pub const ADDED_FROM: u32 = 0x4844;
pub const ADDED_BELOW: u32 = 3000;
/// How many drivers by lot get time added.
pub const SLOWED: u32 = 8;
/// A driver still running (+0x24) and one dropped out, with its times (+0xC, +0x10).
pub const RUNNING: u32 = 1;
pub const DROPPED: u32 = 4;
pub const DROPPED_TIME: u32 = 0x36EE8A;
/// The race's results: none, and left.
pub const NONE: u32 = u32::MAX;
pub const QUIT: u32 = 0xFFFF_FFFE;
/// Player +0xD0's value for one taking part.
pub const TAKES_PART: u32 = 1;
/// The fourth player's time and +0xD0 when all four dropped in mode 0xD.
pub const FOURTH_TIME: u32 = 0x36EE80;
pub const FOURTH_ENTRY: u32 = 6;
/// The cheat that keeps the race's switch on: bit 0x12.
pub const CHEAT: u32 = 1 << 0x12;

/// The players racing in an event mode, and whether the fourth's dropping out is noted.
pub fn players_of(mode: u32) -> Option<(u32, bool)> {
    match mode {
        0xB | 0x10 => Some((2, false)),
        0xC | 0x11 => Some((3, false)),
        0xD => Some((4, true)),
        0x12 => Some((4, false)),
        _ => None,
    }
}

/// The variables.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Var {
    /// The event goes on.
    GoesOn,
    /// `RaceState.settings.direction`, the cheat bits.
    CheatSwitch,
    Cheats,
    /// `ScreenFlow.kept`, `RaceState.settings.kept`.
    Kept,
    RaceKept,
    /// `Event.current_player`.
    Current,
    /// `RaceState.results.results[0].result`, `RaceState.results.results[1].result`.
    Result,
    Result2,
}

/// The steps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    CpuTimes,
    ResetPlayerTimes,
    StandingsFromDrivers,
    SortByTime,
    CopySummary,
    /// `RaceState.settings.rivals` the summary table, `RaceState.settings.stage` the track pack
    /// entry's +0x34.
    RaceInfo,
    /// `race::single_rally_race` (kind 1); `next` when another player's race follows
    /// (1 from `menu::player_entered`; 0 otherwise).
    Race {
        next: bool,
    },
    ReloadFrontEnd,
    ClearServiceSpent,
}

pub trait EventRaces {
    type Error;
    fn get(&mut self, v: Var) -> u32;
    fn set(&mut self, v: Var, value: u32);
    fn step(&mut self, s: Step) -> Result<(), Self::Error>;
    /// A random number below `n`.
    fn below(&mut self, n: u32) -> Result<u32, Self::Error>;
    /// Driver `k` (of the picked) gets `added` on its two times.
    fn slow(&mut self, k: u32, added: u32);
    /// Whether driver `k` still runs.
    fn running(&mut self, k: u32) -> bool;
    /// The driver last asked about drops out.
    fn drop_out(&mut self);
    /// Whether player `i` takes part.
    fn takes_part(&mut self, i: u32) -> bool;
    /// The fourth player's dropping out noted.
    fn fourth_out(&mut self);
}

/// The players' races of the modes 0xB .. 0xD, 0x10 .. 0x12.
fn players_races<E: EventRaces>(e: &mut E, n: u32, note_fourth: bool) -> Result<(), E::Error> {
    let mut kept = 0u32;
    for i in 0..n {
        e.set(Var::Current, i);
        if !e.takes_part(i) {
            if i == 0 {
                kept = NONE;
            } else {
                e.set(Var::Result, NONE);
            }
        } else {
            let next = i + 1 < n && e.takes_part(i + 1);
            e.step(Step::Race { next })?;
            if e.get(Var::Result) == QUIT {
                e.step(Step::ResetPlayerTimes)?;
                e.step(Step::ReloadFrontEnd)?;
                return Ok(());
            }
            if i == 0 {
                kept = e.get(Var::Result);
            }
        }
        if i > 0 {
            kept &= e.get(Var::Result);
        }
    }
    if kept == NONE {
        if note_fourth {
            e.fourth_out();
        }
        e.set(Var::GoesOn, 0);
    }
    Ok(())
}

/// `menu::run_event_races` (its checks made by the caller): `mode` the event mode,
/// `reset_only` `MenuPages.reset_times` 1.
pub fn run_event_races<E: EventRaces>(
    e: &mut E,
    mode: u32,
    reset_only: bool,
) -> Result<(), E::Error> {
    e.set(Var::GoesOn, 1);
    let cheat = e.get(Var::Cheats) & CHEAT != 0;
    e.set(Var::CheatSwitch, u32::from(cheat));
    e.set(Var::Kept, 0);
    e.step(Step::CpuTimes)?;
    for _ in 0..SLOWED {
        let k = e.below(PICKED)?;
        let added = e.below(ADDED_BELOW)?.wrapping_add(ADDED_FROM);
        e.slow(k, added);
    }
    let out = e.below(4)? + 1;
    for _ in 0..out {
        loop {
            let k = e.below(PICKED)?;
            if e.running(k) {
                break;
            }
        }
        e.drop_out();
    }
    if reset_only {
        e.step(Step::ResetPlayerTimes)?;
    } else {
        e.step(Step::StandingsFromDrivers)?;
        e.step(Step::SortByTime)?;
        e.step(Step::CopySummary)?;
        e.step(Step::RaceInfo)?;
        match mode {
            9 | 0xA | 0xE | 0xF => {
                e.set(Var::Current, 0);
                let kept = mode == 0xA || mode == 0xF;
                if kept {
                    let v = e.get(Var::Kept);
                    e.set(Var::RaceKept, v);
                }
                e.step(Step::Race { next: false })?;
                if mode == 0xA {
                    let v = e.get(Var::Kept);
                    e.set(Var::RaceKept, v);
                }
                let ended = if kept {
                    let both = e.get(Var::Result) & e.get(Var::Result2);
                    both == NONE
                } else {
                    e.get(Var::Result) == NONE
                };
                if ended {
                    e.set(Var::GoesOn, 0);
                }
            }
            _ => {
                if let Some((n, note_fourth)) = players_of(mode) {
                    players_races(e, n, note_fourth)?;
                }
            }
        }
    }
    e.step(Step::ClearServiceSpent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Log {
        steps: Vec<String>,
        draws: Vec<u32>,
        results: Vec<u32>,
        running: Vec<bool>,
        parts: Vec<bool>,
        vars: std::collections::HashMap<String, u32>,
    }

    impl EventRaces for Log {
        type Error = ();
        fn get(&mut self, v: Var) -> u32 {
            *self.vars.get(&format!("{v:?}")).unwrap_or(&0)
        }
        fn set(&mut self, v: Var, value: u32) {
            self.steps.push(format!("{v:?}={value}"));
            self.vars.insert(format!("{v:?}"), value);
        }
        fn step(&mut self, s: Step) -> Result<(), ()> {
            self.steps.push(format!("{s:?}"));
            if let Step::Race { .. } = s {
                let r = self.results.remove(0);
                self.vars.insert("Result".into(), r);
            }
            Ok(())
        }
        fn below(&mut self, n: u32) -> Result<u32, ()> {
            Ok(self.draws.pop().unwrap_or(0) % n)
        }
        fn slow(&mut self, k: u32, added: u32) {
            self.steps.push(format!("slow {k} {added}"));
        }
        fn running(&mut self, _: u32) -> bool {
            self.running.remove(0)
        }
        fn drop_out(&mut self) {
            self.steps.push("out".into());
        }
        fn takes_part(&mut self, i: u32) -> bool {
            self.parts[i as usize]
        }
        fn fourth_out(&mut self) {
            self.steps.push("fourth".into());
        }
    }

    #[test]
    fn lot_and_players() {
        // Draws popped from the end: eight pairs, the count (1 => two out), then the
        // drop-outs' picks (the first not running).
        let mut draws = vec![5, 7, 9];
        draws.push(1);
        for k in 0..8 {
            draws.extend([100 + k, k]);
        }
        let mut log = Log {
            draws,
            running: vec![false, true, true],
            parts: vec![true, false, true, true],
            results: vec![NONE; 3],
            ..Log::default()
        };
        run_event_races(&mut log, 0xD, false).unwrap();
        assert!(log.steps.contains(&format!("slow 0 {}", 100 + ADDED_FROM)));
        assert_eq!(log.steps.iter().filter(|s| *s == "out").count(), 2);
        // Player 1 sits out: player 0's race is not followed by one, player 2's is.
        assert_eq!(
            log.steps
                .iter()
                .filter(|s| s.starts_with("Race "))
                .collect::<Vec<_>>(),
            [
                "Race { next: false }",
                "Race { next: true }",
                "Race { next: false }"
            ]
        );
        assert!(log.steps.contains(&"fourth".to_string()));
        assert!(log.steps.contains(&"GoesOn=0".to_string()));
        assert_eq!(log.steps.last().unwrap(), "ClearServiceSpent");

        let mut log = Log {
            running: vec![true; 4],
            results: vec![3],
            ..Log::default()
        };
        run_event_races(&mut log, 0xA, false).unwrap();
        assert_eq!(log.steps.iter().filter(|s| *s == "RaceKept=0").count(), 2);
        assert!(!log.steps.contains(&"GoesOn=0".to_string()));
        assert_eq!(players_of(0x11), Some((3, false)));
    }
}
