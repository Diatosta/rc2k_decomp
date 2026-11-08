//! What the event flows run on (2026-10-21): the game's variables, its callees, what the
//! callees answer and the car list the flows hand on to the car choice, through `Game`. The flows
//! themselves are `menu::rounds` (the single rally's and the time trial's events),
//! `menu::championship` (the two championships' driver) and `menu::welcome` (their
//! welcomes); the starters keep their own (`menu::starters`).

use crate::Invalid;

/// The game's variables the flows read and write.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Var {
    /// `g_yesNoAnswer`.
    Answer,
    /// `dword_755984`.
    PlayerCount,
    /// `dword_755988` (9 .. 0x18, `menu::run_event_races`).
    EventMode,
    /// `dword_755954`: 3 the time trial, 4 and 5 the championships.
    EventType,
    /// `g_eventMode`: 2, 3, 0x13.
    Mode,
    /// `dword_75598C`: the current player.
    Current,
    /// `dword_758806`: the car choice was left.
    CarLeft,
    /// `dword_75555D`, `dword_755561`.
    Leave,
    Leave561,
    /// `dword_755565`, `dword_755569`.
    Ended565,
    Ended569,
    /// `dword_755A3C`.
    LeftA3C,
    /// `dword_755980`: 1 when an event is over.
    Done,
    /// `dword_758714`: the rounds of a multi-player event.
    Round,
    /// `dword_755918`: the time qualifies.
    Qualifies,
    /// `dword_755920`: one was entered in the best times.
    Entered,
    /// `dword_755938`: a service due.
    ServiceDue,
    /// `dword_75B028`, `dword_75B098`: the race's results (-1 none, -2 left).
    RaceResult,
    RaceResult2,
    /// `dword_7554D8`: 1 picks the single rally's first car list.
    CarListKind,
    /// `g_cheats`.
    Cheats,
    /// `dword_7587A8` + 4 x k (k 0 .. 7).
    Cleared(u32),
    /// Player n's time (+0x88; the original's edi the player).
    PlayerTime(u32),
    /// Player n's +0xD0 (the original's edi the player).
    PlayerEntry(u32),
    /// `dword_755970`: 1 when a saved game goes on.
    Continuing,
    /// `dword_755950`: the stages done.
    StagesDone,
    /// `dword_755250`: the players' times reset.
    ResetTimes,
    /// `dword_755254`: the services.
    Services,
    /// `dword_75AB40`.
    ModeAB40,
    /// `dword_7554C0` (the championship's), `dword_7554C4` (the A8's): the section to start at.
    StartSection {
        a8: bool,
    },
    /// `dword_75593C`, `dword_755940`, `dword_755944`, `dword_75594C`: the service's.
    ServiceTime,
    Service940,
    Service944,
    ServiceNext,
    /// `dword_755928`: the championship's end.
    TheEnd,
    /// `dword_75592C`: the event goes on.
    GoesOn,
    /// `section`, `index`.
    Section,
    Index,
    /// `dword_75AB20`: the rally's results reached.
    RallyDone,
    /// `dword_755924`, `dword_755934`: the welcome's.
    WelcomeFigure,
    Welcomed,
    /// `dword_75AAD8`: the review's kind.
    ReviewKind,
    /// A dword of the record `Call::FindRecord` found (its offset).
    Record(u32),
    /// `dword_755698`: the speech's channel.
    SpeechChannel,
    /// `g_quitting`.
    Quitting,
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

/// The callees, with what the originals give them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Call {
    BeginEvent,
    ShuffleDrivers,
    ClearResultSlots,
    ResetDrivers,
    ResetDriverCondition,
    ClearPlayers284,
    TakePlayer1Units,
    ResetCars750724,
    ClearEventState,
    ReadyPlayers,
    ResetSingleEvent,
    ChampionshipWelcome,
    A8Welcome,
    /// `records::find_record` on `g_recordTable` (its fields through `Var::Record`).
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
    Review480D46,
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
    Review47D5A5,
    Review48108E,
    ChampionshipFinish,
    ClearScoreMarks,
    BestClassScores,
    BestOverallScores,
    Review481429,
    /// `menu::show_in_game_options`.
    ShowInGameOptions,
    /// `menu::hide_in_game_options` for `players` (eax).
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
    /// `race::single_rally_race`: `next` (eax) when another player's race follows, `kind`
    /// (ebx).
    Race {
        next: bool,
        kind: u32,
    },
    StageResultLoop,
    TimeTrialResultsLoop,
    Results48584B,
    Results482E9B,
    Screen483163,
    KeepStageBest,
    /// `menu::time_trial_qualifies` for the current player.
    TimeTrialQualifies,
    ClearTimeTrialMarks,
    EnterTimeTrialBest,
    TimeTrialBestTimesLoop,
    Review485F5E,
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
    /// The car list the next car choice shows (the original's esi).
    fn hand_cars(&mut self, list: CarList);
    /// The fatal error `code` with `message`.
    fn fatal(&mut self, code: u32, message: &'static str) -> Self::Error;

    /// `v` one more (wrapping).
    fn inc(&mut self, v: Var) {
        let x = self.get(v).wrapping_add(1);
        self.set(v, x);
    }

    /// `g_yesNoAnswer` 1.
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
        /// Per callee (its `Debug` text up to a space, brace or parenthesis): the writes of
        /// its calls in turn, and the answers.
        pub writes: HashMap<String, Vec<Vec<(Var, u32)>>>,
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
