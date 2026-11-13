//! What the event flows run on: the game's variables, its callees, what the
//! callees answer and the car list the flows hand on to the car choice, through `Game`. The flows
//! themselves are `menu::rounds` (the single rally's and the time trial's events),
//! `menu::championship` (the two championships' driver) and `menu::welcome` (their
//! welcomes); the starters keep their own (`menu::starters`).

use super::leaving::Switch;
use crate::Invalid;

/// The game's variables the flows read and write.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Var {
    /// The answer to the last yes / no question.
    Answer,
    /// The number of players.
    PlayerCount,
    /// The event mode, 9 .. 0x18 (`menu::run_event_races`).
    EventMode,
    /// The kind of event: 3 the time trial, 4 and 5 the championships.
    EventType,
    /// The championship view's event mode: 2, 3, 0x13.
    Mode,
    /// The current player.
    Current,
    /// The rounds of a multi-player event.
    Round,
    /// The time qualifies.
    Qualifies,
    /// A service is due.
    ServiceDue,
    /// The race's two results (-1 none, -2 left).
    RaceResult,
    RaceResult2,
    /// 1 picks the single rally's first car list.
    CarListKind,
    /// The cheat bits.
    Cheats,
    /// The cleared flag of level k (k 0 .. 7).
    Cleared(u32),
    /// Player n's time (+0x88).
    PlayerTime(u32),
    /// Player n's `entry` field (+0xD0).
    PlayerEntry(u32),
    /// 1 when a saved game goes on.
    Continuing,
    /// The stages done.
    StagesDone,
    /// The players' times are reset.
    ResetTimes,
    /// The services.
    Services,
    /// A mode word the review sets.
    ModeAB40,
    /// The section to start at (the championship's, or the A8's).
    StartSection {
        a8: bool,
    },
    /// The service's four figures.
    ServiceTime,
    ServiceRecordC,
    CpuSpreadFirst,
    ServiceNext,
    /// The championship's end.
    TheEnd,
    /// The event goes on.
    GoesOn,
    /// `section`, `index`.
    Section,
    Index,
    /// The welcome's two figures.
    WelcomeFigure,
    Welcomed,
    /// The review's kind.
    ReviewKind,
    /// A dword of the record `Call::FindRecord` found (its offset).
    Record(u32),
    /// The speech's channel.
    SpeechChannel,
}

pub use super::car_lists::CarList;

/// The welcomes' pictures (with the title bar).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Picture {
    /// `mob_welc.png`.
    Championship,
    /// `a8_welc.png`.
    A8,
}

/// The callees, and what they give back.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Call {
    BeginEvent,
    ShuffleDrivers,
    ClearResultSlots,
    ResetDrivers,
    ResetDriverCondition,
    ClearHudBits,
    TakePlayer1Units,
    HideAllInGameOptions,
    ClearEventState,
    ReadyPlayers,
    ResetSingleEvent,
    ChampionshipWelcome,
    A8Welcome,
    /// `records::find_record` on the record table (its fields through `Var::Record`).
    FindRecord {
        section: u32,
        index: u32,
    },
    WelcomeLoop,
    RallyStagesLoop,
    NextStageLoop,
    ServicePass,
    StandingsBFromAll,
    SortBByFigure,
    AwardStagePoints,
    SortBByPoints,
    ClassChampionshipPass,
    AwardOverallPoints,
    ClearStandingsB,
    SortBByScore,
    NumberAllB,
    ShowChampionshipStandings,
    SectionFinish,
    ChampionshipStandingsReview,
    WalkSectionRecords,
    StandingsAFromAll,
    ClassRallyResultsPass,
    ClearStandingsA,
    NumberAllA,
    ShowOverallRallyResults,
    RunEventRaces,
    AddPlayerTimes,
    ClassStageResultsPass,
    ShowOverallStageResults,
    KeepRecordBest,
    RecordEntriesPass,
    BestTimesLoop,
    InspectionPass,
    ChampionshipServiceReview,
    ChampionshipRacesReview,
    ChampionshipFinish,
    ClearScoreMarks,
    BestClassScores,
    BestOverallScores,
    ChampionshipEndReview,
    /// `menu::show_in_game_options`.
    ShowInGameOptions,
    /// `menu::hide_in_game_options` for `players`.
    HideInGameOptions {
        players: u32,
    },
    ResetPlayerTimes,
    ReloadFrontEnd,
    /// `service::repair_car`: the current player's car.
    RepairCar,
    /// `menu::cheat_car_list` on the list handed (`Game::hand_cars`).
    CheatCarList,
    SelectCarLoop,
    TakeShownCar,
    SelectStageLoop,
    TimeTrialSelectStage,
    TimeTrialService,
    /// `race::single_rally_race`: `next` when another player's race follows, `kind`.
    Race {
        next: bool,
        kind: u32,
    },
    StageResultLoop,
    TimeTrialResultsLoop,
    SingleRallyTwoResults,
    SingleRallyThreeFourResults,
    SingleRallyOverallResults,
    KeepStageBest,
    /// `menu::time_trial_qualifies` for the current player.
    TimeTrialQualifies,
    ClearTimeTrialMarks,
    EnterTimeTrialBest,
    TimeTrialBestTimesLoop,
    SingleRallyReview,
    /// `sound::start_stream`: the speech `rally0.wav` on `channel`.
    StartStream {
        channel: u32,
    },
    WaitForCommands,
    StopStream,
    ConvertPicture,
    /// `screens::run_frame` with `menu::picture_frame`.
    RunFrame,
    ConfirmLeaving,
}

/// The game a flow runs on.
pub trait Game {
    type Error: From<Invalid>;
    fn get(&mut self, v: Var) -> u32;
    fn set(&mut self, v: Var, value: u32);
    fn flag(&mut self, s: Switch) -> bool;
    fn set_flag(&mut self, s: Switch, on: bool);
    /// A callee whose answer the flows do not read.
    fn run(&mut self, c: Call) -> Result<(), Self::Error>;
    /// `menu::next_record_kind`: the kind of the record after the found one.
    fn next_record_kind(&mut self) -> Result<u32, Self::Error>;
    /// `gfx::load_picture_with_overlay`: false when the picture could not be read.
    fn load_picture(&mut self, p: Picture) -> Result<bool, Self::Error>;
    /// `input::take_key`: the key waiting, if any.
    fn take_key(&mut self) -> Result<Option<u8>, Self::Error>;
    /// `system::wait_or_key`: a key within `ms` milliseconds, if any.
    fn wait_or_key(&mut self, ms: u32) -> Result<Option<u8>, Self::Error>;
    /// `sound::stream_playing`: whether the speech still plays.
    fn stream_playing(&mut self) -> Result<bool, Self::Error>;
    /// The car list the next car choice shows.
    fn hand_cars(&mut self, list: CarList);
    /// The fatal error `code` with `message`.
    fn fatal(&mut self, code: u32, message: &'static str) -> Self::Error;

    /// `v` one more (wrapping).
    fn inc(&mut self, v: Var) {
        let x = self.get(v).wrapping_add(1);
        self.set(v, x);
    }

    /// The last yes / no question answered yes (1).
    fn yes(&mut self) -> bool {
        self.get(Var::Answer) == 1
    }
}

#[cfg(test)]
pub(crate) mod log {
    //! A `Game` for the flows' tests: the steps noted, the answers by script.

    use super::*;
    use std::collections::HashMap;

    /// A scripted answer of a callee that has one.
    #[derive(Clone, Copy, Debug)]
    pub enum Said {
        Key(Option<u8>),
        Playing(bool),
        Unreadable,
        Kind(u32),
    }

    #[derive(Default)]
    pub struct Log {
        pub steps: Vec<String>,
        pub vars: HashMap<String, u32>,
        pub switches: HashMap<String, bool>,
        /// Per callee (its `Debug` text up to a space, brace or parenthesis): the writes of
        /// its calls in turn, and the answers.
        pub writes: HashMap<String, Vec<Vec<(Var, u32)>>>,
        /// The same for the leaving switches.
        pub switch_writes: HashMap<String, Vec<Vec<(Switch, bool)>>>,
        pub replies: HashMap<String, Vec<Said>>,
        pub calls: HashMap<String, usize>,
    }

    fn name(step: &str) -> String {
        step.split([' ', '(', '{']).next().unwrap().to_string()
    }

    impl Log {
        pub fn on(mut self, callee: &str, writes: Vec<Vec<(Var, u32)>>) -> Self {
            self.writes.insert(callee.to_string(), writes);
            self
        }

        pub fn on_switches(mut self, callee: &str, writes: Vec<Vec<(Switch, bool)>>) -> Self {
            self.switch_writes.insert(callee.to_string(), writes);
            self
        }

        pub fn reply(mut self, callee: &str, replies: Vec<Said>) -> Self {
            self.replies.insert(callee.to_string(), replies);
            self
        }

        pub fn with(mut self, v: Var, value: u32) -> Self {
            self.vars.insert(format!("{v:?}"), value);
            self
        }

        pub fn count(&self, step: &str) -> usize {
            self.steps.iter().filter(|s| s.as_str() == step).count()
        }

        pub fn has(&self, step: &str) -> bool {
            self.count(step) > 0
        }

        /// `step` noted, its scripted writes made; its scripted answer, if any.
        fn note(&mut self, step: String) -> Option<Said> {
            let n = name(&step);
            self.steps.push(step);
            let k = *self
                .calls
                .entry(n.clone())
                .and_modify(|k| *k += 1)
                .or_insert(1)
                - 1;
            let writes = self
                .writes
                .get(&n)
                .and_then(|w| w.get(k))
                .cloned()
                .unwrap_or_default();
            for (v, x) in writes {
                self.vars.insert(format!("{v:?}"), x);
            }
            let switch_writes = self
                .switch_writes
                .get(&n)
                .and_then(|w| w.get(k))
                .cloned()
                .unwrap_or_default();
            for (s, on) in switch_writes {
                self.switches.insert(format!("{s:?}"), on);
            }
            self.replies.get(&n).and_then(|r| r.get(k)).copied()
        }

        fn key(&mut self, step: String) -> Option<u8> {
            match self.note(step) {
                Some(Said::Key(key)) => key,
                None => None,
                Some(other) => panic!("a key's answer scripted as {other:?}"),
            }
        }
    }

    impl Game for Log {
        type Error = Invalid;

        fn get(&mut self, v: Var) -> u32 {
            *self.vars.get(&format!("{v:?}")).unwrap_or(&0)
        }

        fn set(&mut self, v: Var, value: u32) {
            self.steps.push(format!("{v:?}={value}"));
            self.vars.insert(format!("{v:?}"), value);
        }

        fn flag(&mut self, s: Switch) -> bool {
            *self.switches.get(&format!("{s:?}")).unwrap_or(&false)
        }

        fn set_flag(&mut self, s: Switch, on: bool) {
            self.steps.push(format!("{s:?}={}", u32::from(on)));
            self.switches.insert(format!("{s:?}"), on);
        }

        fn run(&mut self, c: Call) -> Result<(), Invalid> {
            self.note(format!("{c:?}"));
            Ok(())
        }

        fn next_record_kind(&mut self) -> Result<u32, Invalid> {
            Ok(match self.note("NextRecordKind".into()) {
                Some(Said::Kind(kind)) => kind,
                None => 0,
                Some(other) => panic!("a record kind scripted as {other:?}"),
            })
        }

        fn load_picture(&mut self, p: Picture) -> Result<bool, Invalid> {
            Ok(!matches!(
                self.note(format!("LoadPicture({p:?})")),
                Some(Said::Unreadable)
            ))
        }

        fn take_key(&mut self) -> Result<Option<u8>, Invalid> {
            Ok(self.key("TakeKey".into()))
        }

        fn wait_or_key(&mut self, ms: u32) -> Result<Option<u8>, Invalid> {
            Ok(self.key(format!("WaitOrKey {{ ms: {ms} }}")))
        }

        fn stream_playing(&mut self) -> Result<bool, Invalid> {
            Ok(match self.note("StreamPlaying".into()) {
                Some(Said::Playing(playing)) => playing,
                None => true,
                Some(other) => panic!("the speech's playing scripted as {other:?}"),
            })
        }

        fn hand_cars(&mut self, list: CarList) {
            self.steps.push(format!("CarList({list:?})"));
        }

        fn fatal(&mut self, _: u32, message: &'static str) -> Invalid {
            Invalid(message)
        }
    }
}
