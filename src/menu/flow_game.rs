//! The game for `rc2k_core::menu::flow` (2026-10-21): the event flows' variables at their
//! addresses, their callees through an `event_flow::Call` with the registers the originals
//! give them, and what the originals hand on in registers (one set carried through the
//! whole flow, as their `pusha` keeps it).

use crate::errors::{FatalError, Refused};
use crate::framework::Regs;
use crate::menu::championship_welcome::{addr, zero};
use crate::menu::event_flow::*;
use crate::menu::run_championship as champ;
use rc2k_core::menu::flow::{self, CarList, Hand, Picture, Reply, Var};

pub const SPEECH: usize = 0x50960E; // ";3files\%speech\rally\rally0.wav"
pub const SPEECH_CHANNEL: usize = 0x755698; // dword_755698
pub const CHAMPIONSHIP_PICTURE: usize = 0x731C09; // "files\g\graphics\screens\mob_welc.png"
pub const A8_PICTURE: usize = 0x731C2F; // "files\g\graphics\screens\a8_welc.png"
pub const TITLE_BAR: usize = 0x731C54; // "files\g\graphics\screens\titlebar.png"
const CLEARED: usize = 0x7587A8; // dword_7587A8 .. dword_7587C4
const PLAYER_TIME: u32 = 0x88;
const PLAYER_ENTRY: u32 = 0xD0;

pub struct Game<'a> {
    entry: Regs,
    g: Regs,
    call: Call<'a>,
    /// The registers `ShowInGameOptions { from_entry: true }` left.
    left: Option<Regs>,
}

impl<'a> Game<'a> {
    pub fn new(entry: &Regs, call: Call<'a>) -> Self {
        Self {
            entry: *entry,
            g: *entry,
            call,
            left: None,
        }
    }

    /// The registers the flow leaves: the last call's from the entry's, else the entry's.
    pub fn left(&self) -> Regs {
        self.left.unwrap_or(self.entry)
    }
}

/// Where a variable is.
enum Place {
    At(usize),
    /// Player n's field (edi the player).
    Player(u32, u32),
    /// The found record's field.
    Record(u32),
}

fn place(v: Var) -> Place {
    use Place::At;
    match v {
        Var::Answer => At(ANSWER),
        Var::PlayerCount => At(PLAYER_COUNT),
        Var::EventMode => At(EVENT_MODE),
        Var::EventType => At(EVENT_TYPE),
        Var::Mode => At(MODE),
        Var::Current => At(CURRENT),
        Var::CarLeft => At(CAR_LEFT),
        Var::Leave => At(LEAVE),
        Var::Leave561 => At(LEAVE_561),
        Var::Ended565 => At(ENDED_565),
        Var::Ended569 => At(ENDED_569),
        Var::LeftA3C => At(LEFT_A3C),
        Var::Done => At(DONE),
        Var::Round => At(ROUND),
        Var::Qualifies => At(crate::menu::time_trial_qualifies::QUALIFIES),
        Var::Entered => At(crate::menu::time_trial_qualifies::ANY_QUALIFIED),
        Var::ServiceDue => At(champ::SERVICE_DUE),
        Var::RaceResult => At(RACE_RESULT),
        Var::RaceResult2 => At(RACE_RESULT_2),
        Var::CarListKind => At(CAR_LIST_KIND),
        Var::Cheats => At(CHEATS),
        Var::Cleared(k) => At(CLEARED + 4 * k as usize),
        Var::PlayerTime(n) => Place::Player(n, PLAYER_TIME),
        Var::PlayerEntry(n) => Place::Player(n, PLAYER_ENTRY),
        Var::Continuing => At(CONTINUING),
        Var::StagesDone => At(champ::STAGES_DONE),
        Var::ResetTimes => At(champ::RESET_TIMES),
        Var::Services => At(champ::SERVICES),
        Var::ModeAB40 => At(champ::MODE_AB40),
        Var::StartSection { a8: false } => At(champ::START_SECTION),
        Var::StartSection { a8: true } => At(champ::START_SECTION_A8),
        Var::ServiceTime => At(champ::SERVICE_TIME),
        Var::Service940 => At(champ::SERVICE_940),
        Var::Service944 => At(champ::SERVICE_944),
        Var::ServiceNext => At(champ::SERVICE_NEXT),
        Var::TheEnd => At(champ::THE_END),
        Var::GoesOn => At(champ::GOES_ON),
        Var::Section => At(SECTION),
        Var::Index => At(INDEX),
        Var::RallyDone => At(champ::RALLY_DONE),
        Var::WelcomeFigure => At(champ::WELCOME_FIGURE),
        Var::Welcomed => At(champ::WELCOMED),
        Var::ReviewKind => At(champ::REVIEW_KIND),
        Var::Record(offset) => Place::Record(offset),
        Var::SpeechChannel => At(SPEECH_CHANNEL),
        Var::Quitting => At(QUITTING),
    }
}

/// A callee's address, with the registers the original gives it.
fn callee(c: flow::Call, g: &mut Regs) -> usize {
    use flow::Call as C;
    match c {
        C::BeginEvent => BEGIN_EVENT,
        C::ShuffleDrivers => SHUFFLE_DRIVERS,
        C::ClearResultSlots => CLEAR_RESULT_SLOTS,
        C::ResetDrivers => RESET_DRIVERS,
        C::ResetDriverCondition => RESET_DRIVER_CONDITION,
        C::ClearPlayers284 => CLEAR_PLAYERS_284,
        C::TakePlayer1Units => TAKE_PLAYER1_UNITS,
        C::ResetCars750724 => RESET_CARS_750724,
        C::ClearEventState => CLEAR_EVENT_STATE,
        C::ReadyPlayers => READY_PLAYERS,
        C::ResetSingleEvent => RESET_SINGLE_EVENT,
        C::ChampionshipWelcome => CHAMPIONSHIP_WELCOME,
        C::A8Welcome => A8_WELCOME,
        C::FindRecord { section, index } => {
            g.edi = addr(RECORD_TABLE);
            g.ebx = section;
            g.eax = index;
            FIND_RECORD
        }
        C::NextRecordKind => NEXT_RECORD_KIND,
        C::WelcomeLoop => WELCOME_LOOP,
        C::RallyStagesLoop => RALLY_STAGES_LOOP,
        C::NextStageLoop => NEXT_STAGE_LOOP,
        C::ServicePass => SERVICE_PASS,
        C::StandingsBFromAll => STANDINGS_B_FROM_ALL,
        C::SortBByFigure => SORT_B_BY_FIGURE,
        C::AwardStagePoints => AWARD_STAGE_POINTS,
        C::SortBByPoints => SORT_B_BY_POINTS,
        C::ClassChampionshipPass => CLASS_CHAMPIONSHIP_PASS,
        C::AwardOverallPoints => AWARD_OVERALL_POINTS,
        C::ClearStandingsB => CLEAR_STANDINGS_B,
        C::SortBByScore => SORT_B_BY_SCORE,
        C::NumberAllB => NUMBER_ALL_B,
        C::ShowChampionshipStandings => SHOW_CHAMPIONSHIP_STANDINGS,
        C::SectionFinish => SECTION_FINISH,
        C::Review480D46 => REVIEW_480D46,
        C::WalkSectionRecords => WALK_SECTION_RECORDS,
        C::StandingsAFromAll => STANDINGS_A_FROM_ALL,
        C::ClassRallyResultsPass => CLASS_RALLY_RESULTS_PASS,
        C::ClearStandingsA => CLEAR_STANDINGS_A,
        C::NumberAllA => NUMBER_ALL_A,
        C::ShowOverallRallyResults => SHOW_OVERALL_RALLY_RESULTS,
        C::RunEventRaces => RUN_EVENT_RACES,
        C::AddPlayerTimes => ADD_PLAYER_TIMES,
        C::ClassStageResultsPass => CLASS_STAGE_RESULTS_PASS,
        C::ShowOverallStageResults => SHOW_OVERALL_STAGE_RESULTS,
        C::KeepRecordBest => KEEP_RECORD_BEST,
        C::RecordEntriesPass => RECORD_ENTRIES_PASS,
        C::BestTimesLoop => BEST_TIMES_LOOP,
        C::InspectionPass => INSPECTION_PASS,
        C::Review47D5A5 => REVIEW_47D5A5,
        C::Review48108E => REVIEW_48108E,
        C::ChampionshipFinish => CHAMPIONSHIP_FINISH,
        C::ClearScoreMarks => CLEAR_SCORE_MARKS,
        C::BestClassScores => BEST_CLASS_SCORES,
        C::BestOverallScores => BEST_OVERALL_SCORES,
        C::Review481429 => REVIEW_481429,
        C::ShowInGameOptions { .. } => SHOW_IN_GAME_OPTIONS,
        C::HideInGameOptions { players } => {
            g.eax = players;
            HIDE_IN_GAME_OPTIONS
        }
        C::ResetPlayerTimes => RESET_PLAYER_TIMES,
        C::ReloadFrontEnd => RELOAD_FRONT_END,
        C::RepairCar { car } => {
            if let Some(car) = car {
                g.ecx = car;
            }
            REPAIR_CAR
        }
        C::CheatCarList => CHEAT_CAR_LIST,
        C::SelectCarLoop => SELECT_CAR_LOOP,
        C::TakeShownCar => TAKE_SHOWN_CAR,
        C::SelectStageLoop => SELECT_STAGE_LOOP,
        C::TimeTrialSelectStage => TIME_TRIAL_SELECT_STAGE,
        C::TimeTrialService => TIME_TRIAL_SERVICE,
        C::Race { next, kind } => {
            g.eax = u32::from(next);
            g.ebx = kind;
            RACE
        }
        C::StageResultLoop => STAGE_RESULT_LOOP,
        C::TimeTrialResultsLoop => TIME_TRIAL_RESULTS_LOOP,
        C::Results48584B => RESULTS_48584B,
        C::Results482E9B => RESULTS_482E9B,
        C::Screen483163 => SCREEN_483163,
        C::KeepStageBest => KEEP_STAGE_BEST,
        C::TimeTrialQualifies { players } => {
            if let Some(players) = players {
                g.ecx = players;
            }
            TIME_TRIAL_QUALIFIES
        }
        C::ClearTimeTrialMarks => CLEAR_TIME_TRIAL_MARKS,
        C::EnterTimeTrialBest => ENTER_TIME_TRIAL_BEST,
        C::TimeTrialBestTimesLoop => TIME_TRIAL_BEST_TIMES_LOOP,
        C::Review485F5E => REVIEW_485F5E,
        C::StartStream { channel } => {
            g.eax = addr(SPEECH);
            g.ecx = channel;
            START_STREAM
        }
        C::WaitForCommands => WAIT_FOR_COMMANDS,
        C::StopStream => STOP_STREAM,
        C::LoadPicture(p) => {
            g.eax = addr(match p {
                Picture::Championship => CHAMPIONSHIP_PICTURE,
                Picture::A8 => A8_PICTURE,
            });
            g.ebx = addr(TITLE_BAR);
            LOAD_PICTURE_WITH_OVERLAY
        }
        C::ConvertPicture => CONVERT_PICTURE,
        C::TakeKey => TAKE_KEY,
        C::RunFrame => {
            g.eax = addr(PICTURE_FRAME);
            RUN_FRAME
        }
        C::WaitOrKey { ms } => {
            g.eax = ms;
            WAIT_OR_KEY
        }
        C::StreamPlaying => STREAM_PLAYING,
        C::ConfirmLeaving => CONFIRM_LEAVING,
    }
}

fn reply(g: &Regs) -> Reply {
    Reply {
        eax: g.eax,
        carry: g.carry(),
        zero: zero(g),
    }
}

impl flow::Game for Game<'_> {
    type Error = Refused;

    fn get(&mut self, v: Var) -> u32 {
        match place(v) {
            Place::At(a) => get(a),
            Place::Player(n, offset) => dword(player(n) + offset),
            Place::Record(offset) => dword(self.g.edi + offset),
        }
    }

    fn set(&mut self, v: Var, value: u32) {
        match place(v) {
            Place::At(a) => set(a, value),
            Place::Player(n, offset) => {
                self.g.edi = player(n);
                put(self.g.edi + offset, value);
            }
            Place::Record(offset) => put(self.g.edi + offset, value),
        }
    }

    fn call(&mut self, c: flow::Call) -> Result<Reply, Refused> {
        if let flow::Call::ShowInGameOptions { from_entry: true } = c {
            let mut e = self.entry;
            (self.call)(SHOW_IN_GAME_OPTIONS, &mut e)?;
            self.left = Some(e);
            return Ok(reply(&e));
        }
        let address = callee(c, &mut self.g);
        (self.call)(address, &mut self.g)?;
        Ok(reply(&self.g))
    }

    fn hand(&mut self, h: Hand) -> Result<(), Refused> {
        match h {
            Hand::CarList(list) => {
                self.g.esi = addr(match list {
                    CarList::First => CARS_FIRST,
                    CarList::Second => CARS_SECOND,
                    CarList::Cheat => CARS_CHEAT,
                })
            }
            Hand::CheatFlag(on) => self.g.set_carry(on),
            Hand::Kept(v) => self.g.eax = v,
            Hand::Stage => {
                self.g.edi = stage_record()?;
                self.g.ebx = get(STAGE_SUB);
                self.g.eax = get(STAGE_NUMBER).wrapping_mul(STAGE_RECORD);
            }
        }
        Ok(())
    }

    fn fatal(&mut self, code: u32, message: &'static str) -> Refused {
        FatalError { code, message }.into()
    }
}
