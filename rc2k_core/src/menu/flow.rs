//! What the event flows run on (2026-10-21): the game's variables, its callees and the
//! values the originals hand from one callee to the next, through `Game`. The flows
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
    /// `menu::next_record_kind`: the kind in `Reply::eax`.
    NextRecordKind,
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
    /// `menu::show_in_game_options`; `from_entry`: with the registers the flow was entered
    /// with (its last step, whose registers it leaves).
    ShowInGameOptions {
        from_entry: bool,
    },
    /// `menu::hide_in_game_options` for `players` (eax).
    HideInGameOptions {
        players: u32,
    },
    ResetPlayerTimes,
    ReloadFrontEnd,
    /// `service::repair_car`; `car` the split screen's (ecx).
    RepairCar {
        car: Option<u32>,
    },
    /// `menu::cheat_car_list` on the list handed (`Hand::CarList`).
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
    /// `menu::time_trial_qualifies`; `players` (ecx) for player 0.
    TimeTrialQualifies {
        players: Option<u32>,
    },
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
    /// `gfx::load_picture_with_overlay`: `Reply::carry` when it could not be read.
    LoadPicture(Picture),
    ConvertPicture,
    /// `input::take_key`: `Reply::zero` without a key, else the key in `Reply::eax`'s low
    /// byte.
    TakeKey,
    /// `screens::run_frame` with `menu::picture_frame`.
    RunFrame,
    /// `system::wait_or_key` (`ms`): as `TakeKey`.
    WaitOrKey {
        ms: u32,
    },
    /// `sound::stream_playing`: `Reply::zero` while it plays.
    StreamPlaying,
    ConfirmLeaving,
}

/// What a callee answered.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Reply {
    pub eax: u32,
    pub carry: bool,
    pub zero: bool,
}

impl Reply {
    /// A key's answer (`TakeKey`, `WaitOrKey`).
    pub fn key(self) -> Option<u8> {
        (!self.zero).then_some(self.eax as u8)
    }
}

/// What the originals leave for the callees after them, outside a call's own arguments.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hand {
    /// A car list (esi).
    CarList(CarList),
    /// Cheat bit 1 (the carry, the original's `bt`).
    CheatFlag(bool),
    /// A value the original keeps (eax: the races' results, a record's field).
    Kept(u32),
    /// The chosen stage's record (edi, ebx the sub, eax its number x 0xA4); refused for a
    /// stage the tables do not hold.
    Stage,
}

/// The game a flow runs on.
pub trait Game {
    type Error: From<Invalid>;
    fn get(&mut self, v: Var) -> u32;
    fn set(&mut self, v: Var, value: u32);
    fn call(&mut self, c: Call) -> Result<Reply, Self::Error>;
    fn hand(&mut self, h: Hand) -> Result<(), Self::Error>;
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

    /// A call whose answer is not looked at.
    fn run(&mut self, c: Call) -> Result<(), Self::Error> {
        self.call(c).map(|_| ())
    }
}

#[cfg(test)]
pub(crate) mod log {
    //! A `Game` for the flows' tests: the steps noted, the answers by script.

    use super::*;
    use std::collections::HashMap;

    #[derive(Default)]
    pub struct Log {
        pub steps: Vec<String>,
        pub vars: HashMap<String, u32>,
        /// Per callee (its `Debug` text up to a space or brace): the writes of its calls in
        /// turn, and the replies.
        pub writes: HashMap<String, Vec<Vec<(Var, u32)>>>,
        pub replies: HashMap<String, Vec<Reply>>,
        pub calls: HashMap<String, usize>,
    }

    fn name(c: &Call) -> String {
        let s = format!("{c:?}");
        s.split([' ', '(', '{']).next().unwrap().to_string()
    }

    impl Log {
        pub fn on(mut self, callee: &str, writes: Vec<Vec<(Var, u32)>>) -> Self {
            self.writes.insert(callee.to_string(), writes);
            self
        }

        pub fn reply(mut self, callee: &str, replies: Vec<Reply>) -> Self {
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

        fn call(&mut self, c: Call) -> Result<Reply, Invalid> {
            self.steps.push(format!("{c:?}"));
            let n = name(&c);
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
            Ok(self
                .replies
                .get(&n)
                .and_then(|r| r.get(k))
                .copied()
                .unwrap_or(Reply {
                    zero: true,
                    ..Reply::default()
                }))
        }

        fn hand(&mut self, h: Hand) -> Result<(), Invalid> {
            self.steps.push(format!("{h:?}"));
            Ok(())
        }

        fn fatal(&mut self, _: u32, message: &'static str) -> Invalid {
            Invalid(message)
        }
    }
}
