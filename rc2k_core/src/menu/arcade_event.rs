//! The arcade event: from the car choice on, a level after another, six stages a level; a failed
//! race retries the level from its first stage. Its steps, on `Arcade`.

use super::leaving::Switch;
use crate::Invalid;

/// The variables.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Var {
    /// The stage's race failed (a retry).
    Failed,
    /// The level's three totals.
    TotalA8,
    Total798,
    TotalB8,
    /// The stage's two figures.
    Figure768,
    Figure758,
    /// The number of players and the event mode.
    PlayerCount,
    EventMode,
    /// The stage (1 .. 6) and the level (1 .. 5).
    Stage,
    Level,
    /// The current player.
    Current,
    /// The answer to the last yes / no question.
    Answer,
    /// The race's stage and level.
    RaceStage,
    RaceLevel,
    /// The cheat switch and the cheat bits.
    CheatSwitch,
    Cheats,
    /// The race's result (-1 none).
    RaceResult,
    /// The race's two figures.
    RaceFigureFFC,
    RaceFigure000,
    /// A best was made.
    Qualifies,
    /// Player 0's `entry` field.
    Player0Entry,
}

/// The steps, by the place they are called at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Step {
    ClearHudBits = 0,
    HideAllInGameOptions = 1,
    /// `menu::secret_car_list` on the secret car list.
    SecretCarList = 2,
    SelectCarLoop = 3,
    TakeShownCar = 4,
    NextStageLoop = 5,
    TakeStageBest = 6,
    /// `service::repair_car`.
    RepairCar = 7,
    /// `race::single_rally_race` (kind 2, from 0).
    Race = 8,
    ScoreStage = 9,
    ResultsLoop = 10,
    /// `menu::keep_arcade_stage_best` with the stage's record in the level's table.
    KeepStageBest = 11,
    CheckStageTime = 12,
    ClearTimeMarks = 13,
    EnterStageTime = 14,
    BestTimesLoop = 15,
    CheckLevelScore = 16,
    ClearScoreMarks = 17,
    EnterLevelScore = 18,
    BestScoresLoop = 19,
    LevelEnd = 20,
    /// `arcade_next_level_review_loop` (the next level), `arcade_retry_review_loop` (a
    /// retry), `arcade_review_options_loop` (the next stage).
    NextLevelReview = 21,
    RetryReview = 22,
    StageReview = 23,
    /// `menu::show_in_game_options` with the values the event was entered with.
    ShowInGameOptions = 24,
}

pub trait Arcade {
    type Error: From<Invalid>;
    fn get(&mut self, v: Var) -> u32;
    fn set(&mut self, v: Var, value: u32);
    fn flag(&mut self, s: Switch) -> bool;
    fn set_flag(&mut self, s: Switch, on: bool);
    fn step(&mut self, s: Step) -> Result<(), Self::Error>;

    fn yes(&mut self) -> bool {
        self.get(Var::Answer) == 1
    }
}

/// The cheat that keeps the race's switch off: bit 0x12.
pub const CHEAT: u32 = 1 << 0x12;
/// The last level, and the stages of a level.
pub(crate) const LAST_LEVEL: u32 = 5;
pub const STAGES: i32 = 6;
/// The race's result without a time.
pub(crate) const NO_RESULT: u32 = u32::MAX;

enum At {
    Level,
    Stage,
    LevelDone,
    Retry,
    End,
    Out,
}

/// A review: true when it ends the event (the ask-to-leave flag 1, or the left-with-carry flag not
/// 1).
fn review_ends<A: Arcade>(e: &mut A, s: Step) -> Result<bool, A::Error> {
    e.step(s)?;
    Ok(e.flag(Switch::AskLeave) || !e.flag(Switch::LeftWithCarry))
}

/// A step, then whether a yes was given.
fn screen<A: Arcade>(e: &mut A, s: Step) -> Result<bool, A::Error> {
    e.step(s)?;
    Ok(e.yes())
}

/// One stage: where the event goes next.
fn stage<A: Arcade>(e: &mut A) -> Result<At, A::Error> {
    e.set(Var::Failed, 0);
    if screen(e, Step::NextStageLoop)? {
        return Ok(At::End);
    }
    e.step(Step::TakeStageBest)?;
    let stage = e.get(Var::Stage);
    e.set(Var::RaceStage, stage);
    let level = e.get(Var::Level);
    e.set(Var::RaceLevel, level);
    e.set(Var::CheatSwitch, 1);
    if e.get(Var::Cheats) & CHEAT != 0 {
        e.set(Var::CheatSwitch, 0);
    }
    e.step(Step::RepairCar)?;
    e.step(Step::Race)?;
    if e.get(Var::RaceResult) == NO_RESULT {
        e.set(Var::Figure768, 0);
        e.set(Var::Failed, 1);
    } else {
        let figure = e.get(Var::RaceFigureFFC);
        e.set(Var::Figure758, figure);
        let figure = e.get(Var::RaceFigure000);
        e.set(Var::Figure768, figure);
    }
    e.set(Var::Current, 0);
    e.step(Step::ScoreStage)?;
    if e.get(Var::Failed) == 1 {
        return Ok(At::LevelDone);
    }
    if screen(e, Step::ResultsLoop)? {
        return Ok(At::End);
    }
    e.step(Step::KeepStageBest)?;
    if e.get(Var::Failed) != 1 {
        e.step(Step::CheckStageTime)?;
        if e.get(Var::Qualifies) == 1 {
            e.step(Step::ClearTimeMarks)?;
            e.step(Step::EnterStageTime)?;
            if screen(e, Step::BestTimesLoop)? {
                return Ok(At::End);
            }
        }
    }
    let next = e.get(Var::Stage).wrapping_add(1);
    e.set(Var::Stage, next);
    if e.get(Var::Stage) as i32 <= STAGES {
        if e.get(Var::Failed) == 1 {
            return Ok(At::Retry);
        }
        e.step(Step::StageReview)?;
        if e.flag(Switch::AskLeave) || e.flag(Switch::EventLeft) {
            return Ok(At::End);
        }
        return Ok(At::Stage);
    }
    e.step(Step::CheckLevelScore)?;
    if e.get(Var::Qualifies) == 1 {
        e.step(Step::ClearScoreMarks)?;
        e.step(Step::EnterLevelScore)?;
        if screen(e, Step::BestScoresLoop)? {
            return Ok(At::End);
        }
    }
    Ok(At::LevelDone)
}

/// `menu::arcade_event` (player 0 there, the caller's check).
pub fn arcade_event<A: Arcade>(e: &mut A) -> Result<(), A::Error> {
    e.step(Step::ClearHudBits)?;
    e.set(Var::Player0Entry, 1);
    e.set(Var::Failed, 0);
    e.step(Step::HideAllInGameOptions)?;
    for v in [Var::TotalA8, Var::Total798, Var::TotalB8, Var::Figure768] {
        e.set(v, 0);
    }
    e.set(Var::PlayerCount, 1);
    e.set(Var::EventMode, 0x1D);
    e.set_flag(Switch::EventLeft, false);
    e.set(Var::Stage, 1);
    let mut at = At::Level;
    loop {
        at = match at {
            At::Level => {
                e.set(Var::Current, 0);
                e.step(Step::SecretCarList)?;
                e.step(Step::SelectCarLoop)?;
                if e.flag(Switch::LeftByKey) {
                    At::Out
                } else {
                    e.step(Step::SecretCarList)?;
                    e.step(Step::TakeShownCar)?;
                    At::Stage
                }
            }
            At::Stage => stage(e)?,
            At::LevelDone => {
                e.step(Step::LevelEnd)?;
                if e.get(Var::Failed) == 1 {
                    At::Retry
                } else if e.get(Var::Level) == LAST_LEVEL || review_ends(e, Step::NextLevelReview)?
                {
                    At::End
                } else {
                    let next = e.get(Var::Level).wrapping_add(1);
                    e.set(Var::Level, next);
                    e.set(Var::Stage, 1);
                    for v in [Var::TotalB8, Var::Total798, Var::TotalA8] {
                        e.set(v, 0);
                    }
                    At::Level
                }
            }
            At::Retry => {
                if review_ends(e, Step::RetryReview)? {
                    At::End
                } else {
                    for v in [Var::TotalA8, Var::Total798, Var::TotalB8] {
                        e.set(v, 0);
                    }
                    e.set(Var::Stage, 1);
                    e.set(Var::Player0Entry, 1);
                    e.set(Var::Failed, 0);
                    At::Stage
                }
            }
            At::End => {
                e.set_flag(Switch::EventOver, true);
                At::Out
            }
            At::Out => break,
        };
    }
    e.set_flag(Switch::AskLeave, false);
    e.set_flag(Switch::LeftWithCarry, false);
    e.step(Step::ShowInGameOptions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// The variables, the steps noted; each step's writes by script (per call in turn).
    #[derive(Default)]
    struct Log {
        vars: HashMap<String, u32>,
        switches: HashMap<String, bool>,
        steps: Vec<String>,
        script: HashMap<String, Vec<Vec<(Var, u32)>>>,
        switch_script: HashMap<String, Vec<Vec<(Switch, bool)>>>,
        calls: HashMap<String, usize>,
    }

    impl Arcade for Log {
        type Error = Invalid;
        fn get(&mut self, v: Var) -> u32 {
            *self.vars.get(&format!("{v:?}")).unwrap_or(&0)
        }
        fn set(&mut self, v: Var, value: u32) {
            self.vars.insert(format!("{v:?}"), value);
        }
        fn flag(&mut self, s: Switch) -> bool {
            *self.switches.get(&format!("{s:?}")).unwrap_or(&false)
        }
        fn set_flag(&mut self, s: Switch, on: bool) {
            self.switches.insert(format!("{s:?}"), on);
        }
        fn step(&mut self, s: Step) -> Result<(), Invalid> {
            let name = format!("{s:?}");
            self.steps.push(name.clone());
            let k = *self
                .calls
                .entry(name.clone())
                .and_modify(|k| *k += 1)
                .or_insert(1)
                - 1;
            if self.steps.len() > 500 {
                return Err(Invalid("runs on"));
            }
            for (v, x) in self
                .script
                .get(&name)
                .and_then(|w| w.get(k))
                .cloned()
                .unwrap_or_default()
            {
                self.set(v, x);
            }
            for (s, on) in self
                .switch_script
                .get(&name)
                .and_then(|w| w.get(k))
                .cloned()
                .unwrap_or_default()
            {
                self.set_flag(s, on);
            }
            Ok(())
        }
    }

    fn count(log: &Log, s: &str) -> usize {
        log.steps.iter().filter(|x| *x == s).count()
    }

    #[test]
    fn a_level_through() {
        let mut log = Log::default();
        log.set(Var::Level, LAST_LEVEL);
        // The stage reviews go on (the left-with-carry flag 1 is not looked at there).
        arcade_event(&mut log).unwrap();
        assert_eq!(count(&log, "Race"), 6);
        assert_eq!(count(&log, "StageReview"), 5);
        assert_eq!(count(&log, "LevelEnd"), 1);
        assert!(log.flag(Switch::EventOver));
        assert_eq!(log.steps.last().unwrap(), "ShowInGameOptions");
    }

    #[test]
    fn retry_and_leave() {
        let mut log = Log::default();
        log.script
            .insert("Race".into(), vec![vec![(Var::RaceResult, NO_RESULT)]]);
        log.switch_script.insert(
            "RetryReview".into(),
            vec![vec![(Switch::LeftWithCarry, true)]],
        );
        log.script
            .insert("NextStageLoop".into(), vec![vec![], vec![(Var::Answer, 1)]]);
        arcade_event(&mut log).unwrap();
        assert_eq!(count(&log, "LevelEnd"), 1);
        assert_eq!(count(&log, "RetryReview"), 1);
        assert_eq!(count(&log, "ResultsLoop"), 0);
        assert_eq!(log.get(Var::Stage), 1);
        assert!(log.flag(Switch::EventOver));

        let mut log = Log::default();
        log.switch_script.insert(
            "SelectCarLoop".into(),
            vec![vec![(Switch::LeftByKey, true)]],
        );
        arcade_event(&mut log).unwrap();
        assert!(!log.flag(Switch::EventOver));
        assert_eq!(count(&log, "SecretCarList"), 1);
    }
}
