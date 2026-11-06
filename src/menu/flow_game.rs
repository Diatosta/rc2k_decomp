//! The game for `rc2k_core::menu::flow` (2026-10-21): the event flows' variables at their
//! addresses, their callees' Rust versions, the record the last search found and the car
//! list handed on to the car choice.

use crate::errors::{BadInput, FatalError, Refused};
use crate::menu::event_flow::*;
use crate::menu::lists_store::CarList;
use crate::menu::owner::menus;
use crate::menu::run_championship as champ;
use crate::menu::screen_loop::{Entry, Screen, run_screen};
use crate::menu::standings::Which;
use crate::records::stages_store::RecordRef;
use crate::session::Dword;
use rc2k_core::menu::flow::{self, Picture, Var};

pub const SPEECH_CHANNEL: &Dword = &crate::menu::owner::MENUS.screens.option_work.speech_level; // dword_755698

#[derive(Default)]
pub struct Game {
    /// The record `Call::FindRecord` found (the original's edi), read through `Var::Record`.
    record: Option<RecordRef>,
    /// The car list the next car choice shows (`Game::hand_cars`, `Call::CheatCarList`).
    cars: Option<CarList>,
    /// A refused read or write of the core's variables (checked at each call and at the end).
    fault: Fault,
}

impl Game {
    pub fn new() -> Self {
        Self::default()
    }

    /// Runs a flow on this game; a refused read or write of its variables ends it.
    pub fn run<T>(
        mut self,
        flow: impl FnOnce(&mut Game) -> Result<T, Refused>,
    ) -> Result<T, Refused> {
        let done = flow(&mut self);
        self.fault.check()?;
        done
    }
}

/// The player's fields the flows read and write.
#[derive(Clone, Copy)]
enum PlayerField {
    Time,
    Entry,
}

impl PlayerField {
    fn of(self, p: &mut rc2k_core::menu::player::Player) -> &mut u32 {
        match self {
            PlayerField::Time => &mut p.time,
            PlayerField::Entry => &mut p.entry,
        }
    }
}

/// Where a variable is.
enum Place {
    /// A variable of the front end's or the physics block's.
    Cell(&'static Dword),
    /// Player n's field (edi the player).
    Player(u32, PlayerField),
    /// The found record's field.
    Record(u32),
    /// `g_quitting` (the session's).
    Quitting,
}

fn place(v: Var) -> Place {
    let menus = menus();
    use Place::Cell;
    match v {
        Var::Answer => Cell(ANSWER),
        Var::PlayerCount => Cell(PLAYER_COUNT),
        Var::EventMode => Cell(EVENT_MODE),
        Var::EventType => Cell(EVENT_TYPE),
        Var::Mode => Cell(MODE),
        Var::Current => Cell(CURRENT),
        Var::CarLeft => Cell(CAR_LEFT),
        Var::Leave => Cell(LEAVE),
        Var::Leave561 => Cell(LEAVE_561),
        Var::Ended565 => Cell(ENDED_565),
        Var::Ended569 => Cell(ENDED_569),
        Var::LeftA3C => Cell(LEFT_A3C),
        Var::Done => Cell(DONE),
        Var::Round => Cell(ROUND),
        Var::Qualifies => Cell(crate::menu::time_trial_qualifies::QUALIFIES),
        Var::Entered => Cell(crate::menu::time_trial_qualifies::ANY_QUALIFIED),
        Var::ServiceDue => Cell(champ::SERVICE_DUE),
        Var::RaceResult => Cell(&crate::race::owner::race().shared.state.results.results[0].result),
        Var::RaceResult2 => {
            Cell(&crate::race::owner::race().shared.state.results.results[1].result)
        }
        Var::CarListKind => Cell(CAR_LIST_KIND),
        Var::Cheats => Cell(CHEATS),
        Var::Cleared(k) => Cell(&menus.play.results.before[k as usize]),
        Var::PlayerTime(n) => Place::Player(n, PlayerField::Time),
        Var::PlayerEntry(n) => Place::Player(n, PlayerField::Entry),
        Var::Continuing => Cell(CONTINUING),
        Var::StagesDone => Cell(champ::STAGES_DONE),
        Var::ResetTimes => Cell(champ::RESET_TIMES),
        Var::Services => Cell(champ::SERVICES),
        Var::ModeAB40 => Cell(&menus.screens.state.menus.class_mode),
        Var::StartSection { a8: false } => Cell(champ::START_SECTION),
        Var::StartSection { a8: true } => Cell(champ::START_SECTION_A8),
        Var::ServiceTime => Cell(champ::SERVICE_TIME),
        Var::Service940 => Cell(champ::SERVICE_940),
        Var::Service944 => Cell(champ::SERVICE_944),
        Var::ServiceNext => Cell(champ::SERVICE_NEXT),
        Var::TheEnd => Cell(champ::THE_END),
        Var::GoesOn => Cell(champ::GOES_ON),
        Var::Section => Cell(SECTION),
        Var::Index => Cell(INDEX),
        Var::RallyDone => Cell(&menus.screens.state.menus.rally_done),
        Var::WelcomeFigure => Cell(champ::WELCOME_FIGURE),
        Var::Welcomed => Cell(champ::WELCOMED),
        Var::ReviewKind => Cell(&menus.screens.state.menus.review_kind),
        Var::Record(offset) => Place::Record(offset),
        Var::SpeechChannel => Cell(SPEECH_CHANNEL),
        Var::Quitting => Place::Quitting,
    }
}

impl Game {
    /// A screen, entered with the car list handed (the only value a screen reads from its
    /// caller).
    fn screen(&self, screen: &Screen) -> Result<(), Refused> {
        { run_screen(screen, &Entry { cars: self.cars }) }?;
        Ok(())
    }
}

/// A key's code, none for 0 (`input::take_key`'s and `system::wait_or_key`'s).
fn key(code: u8) -> Option<u8> {
    (code != 0).then_some(code)
}

impl flow::Game for Game {
    type Error = Refused;

    fn get(&mut self, v: Var) -> u32 {
        match place(v) {
            Place::Cell(cell) => cell.get(),
            Place::Player(n, field) => *field.of(&mut player(n).get()),
            Place::Record(offset) => self.fault.ok(self
                .record
                .ok_or(BadInput("a record field read before a record was found"))
                .and_then(|r| r.word_at(offset))),
            Place::Quitting => crate::system::owner::system().session.flags.quitting.get(),
        }
    }

    fn set(&mut self, v: Var, value: u32) {
        match place(v) {
            Place::Cell(cell) => cell.set(value),
            Place::Player(n, field) => player(n).update(|q| *field.of(q) = value),
            // The flows only read the record.
            Place::Record(_) => {
                self.fault
                    .ok::<()>(Err(BadInput("a write to a stage record")));
            }
            Place::Quitting => crate::system::owner::system()
                .session
                .flags
                .quitting
                .set(value),
        }
    }

    fn run(&mut self, c: flow::Call) -> Result<(), Refused> {
        self.fault.check()?;
        use crate::menu as m;
        use flow::Call as C;
        match c {
            C::BeginEvent => m::begin_event::begin_event()?,
            C::ShuffleDrivers => m::shuffle_drivers::shuffle_drivers()?,
            C::ClearResultSlots => m::clear_result_slots::clear_result_slots()?,
            C::ResetDrivers => m::reset_drivers::reset_drivers()?,
            C::ResetDriverCondition => m::reset_driver_condition::reset_driver_condition()?,
            C::ClearPlayers284 => m::clear_players_284::clear_players_284()?,
            C::TakePlayer1Units => m::take_player1_units::take_player1_units()?,
            C::ResetCars750724 => m::reset_cars_750724::reset_cars_750724()?,
            C::ClearEventState => m::clear_event_state::clear_event_state()?,
            C::ReadyPlayers => m::ready_players::ready_players()?,
            C::ResetSingleEvent => m::reset_single_event::reset_single_event()?,
            C::ChampionshipWelcome => m::championship_welcome::championship_welcome()?,
            C::A8Welcome => m::a8_welcome::a8_welcome()?,
            C::FindRecord { section, index } => {
                use crate::records::find_record::{find_record_in, has_section};
                let table = crate::system::owner::system()
                    .records
                    .stage_tables
                    .championship();
                // The original has no end-of-table check: a section past the table's
                // is refused.
                if !has_section(table, section, 600)? {
                    return Err(BadInput("a record section past the table").into());
                }
                self.record = Some(find_record_in(table, index as i32, section)?);
            }
            C::WelcomeLoop => self.screen(&m::welcome_loop::SCREEN)?,
            C::RallyStagesLoop => self.screen(&m::rally_stages_loop::SCREEN)?,
            C::NextStageLoop => self.screen(&m::next_stage_loop::SCREEN)?,
            C::ServicePass => m::service_pass::service_pass()?,
            C::StandingsBFromAll => m::standings_b_from_all::standings_b_from_all()?,
            C::SortBByFigure => m::sort_standings_b_by_figure::sort_standings_b_by_figure()?,
            C::AwardStagePoints => m::award_stage_points::award_stage_points()?,
            C::SortBByPoints => m::sort_standings_b_by_points::sort_standings_b_by_points()?,
            C::ClassChampionshipPass => {
                m::class_championship_pass::class_championship_pass()?;
            }
            C::AwardOverallPoints => {
                m::award_overall_points::award_overall_points()?;
            }
            C::ClearStandingsB => m::standings_steps::clear_standings(Which::B)?,
            C::SortBByScore => m::sort_standings_b_by_score::sort_standings_b_by_score()?,
            C::NumberAllB => {
                m::standings_steps::number_all(Which::B)?;
            }
            C::ShowChampionshipStandings => {
                m::show_championship_standings::show_championship_standings()?;
            }
            C::SectionFinish => m::section_finish::section_finish()?,
            C::Review480D46 => self.screen(&m::review_options_480d46_loop::SCREEN)?,
            C::WalkSectionRecords => m::walk_section_records::walk_section_records()?,
            C::StandingsAFromAll => m::standings_a_from_all::standings_a_from_all()?,
            C::ClassRallyResultsPass => {
                m::class_rally_results_pass::class_rally_results_pass()?;
            }
            C::ClearStandingsA => m::standings_steps::clear_standings(Which::A)?,
            C::NumberAllA => {
                m::standings_steps::number_all(Which::A)?;
            }
            C::ShowOverallRallyResults => {
                m::show_overall_rally_results::show_overall_rally_results()?;
            }
            C::RunEventRaces => m::run_event_races::run_event_races()?,
            C::AddPlayerTimes => m::add_player_times::add_player_times()?,
            C::ClassStageResultsPass => {
                m::class_stage_results_pass::class_stage_results_pass()?;
            }
            C::ShowOverallStageResults => {
                m::show_overall_stage_results::show_overall_stage_results()?;
            }
            C::KeepRecordBest => m::keep_record_best::keep_record_best()?,
            C::RecordEntriesPass => m::record_entries_pass::record_entries_pass()?,
            C::BestTimesLoop => self.screen(&m::best_times_loop::SCREEN)?,
            C::InspectionPass => m::inspection_pass::inspection_pass()?,
            C::Review47D5A5 => self.screen(&m::review_options_47d5a5_loop::SCREEN)?,
            C::Review48108E => self.screen(&m::review_options_48108e_loop::SCREEN)?,
            C::ChampionshipFinish => m::championship_finish::championship_finish()?,
            C::ClearScoreMarks => m::clear_score_marks::clear_score_marks()?,
            C::BestClassScores => m::best_class_scores::best_class_scores()?,
            C::BestOverallScores => m::best_overall_scores::best_overall_scores()?,
            C::Review481429 => self.screen(&m::review_options_481429_loop::SCREEN)?,
            C::ShowInGameOptions => m::in_game_options_items::show_in_game_options()?,
            C::HideInGameOptions { players } => {
                m::in_game_options_items::hide_in_game_options(players)?;
            }
            C::ResetPlayerTimes => m::reset_player_times::reset_player_times()?,
            C::ReloadFrontEnd => m::reload_front_end::reload_front_end()?,
            C::RepairCar => crate::service::repair_parts::repair_car()?,
            C::CheatCarList => {
                if let Some(list) = m::cheat_car_list::cheat_car_list() {
                    self.cars = Some(list);
                }
            }
            C::SelectCarLoop => self.screen(&m::select_car_loop::SCREEN)?,
            C::TakeShownCar => m::take_shown_car::take_shown_car(self.cars)?,
            C::SelectStageLoop => self.screen(&m::select_stage_loop::SCREEN)?,
            C::TimeTrialSelectStage => self.screen(&m::time_trial_select_stage_loop::SCREEN)?,
            C::TimeTrialService => m::time_trial_service::time_trial_service()?,
            C::Race { next, kind } => {
                crate::race::single_rally_race::single_rally_race(u32::from(next), kind)?;
            }
            C::StageResultLoop => self.screen(&m::stage_result_loop::SCREEN)?,
            C::TimeTrialResultsLoop => self.screen(&m::time_trial_results_loop::SCREEN)?,
            C::Results48584B => self.screen(&m::results_48584b_loop::SCREEN)?,
            C::Results482E9B => self.screen(&m::results_482e9b_loop::SCREEN)?,
            C::Screen483163 => self.screen(&m::screen_483163_loop::SCREEN)?,
            C::KeepStageBest => m::keep_stage_best::keep_stage_best()?,
            C::TimeTrialQualifies => m::time_trial_qualifies::time_trial_qualifies()?,
            C::ClearTimeTrialMarks => m::clear_time_trial_marks::clear_time_trial_marks()?,
            C::EnterTimeTrialBest => m::enter_time_trial_best::enter_time_trial_best()?,
            C::TimeTrialBestTimesLoop => self.screen(&m::time_trial_best_times_loop::SCREEN)?,
            C::Review485F5E => self.screen(&m::review_options_485f5e_loop::SCREEN)?,
            C::StartStream { channel } => {
                // ";3files\%speech\rally\rally0.wav"
                use crate::audio::names_store::SampleNames;
                let name = core::mem::offset_of!(SampleNames, rally_speech) as u32;
                crate::sound::start_stream::start_stream(name, channel);
            }
            C::WaitForCommands => crate::sound::wait_for_commands::wait_in_game(),
            C::StopStream => crate::sound::stop_stream::stop_stream(),
            C::ConvertPicture => {
                // Whether it worked is not looked at.
                crate::gfx::convert_picture::convert_picture_now()?;
            }
            C::RunFrame => {
                // `screens::run_frame` with `menu::picture_frame` (whether the frame was
                // made is not looked at).
                if let Some(made) = crate::platform::run_frame_now(m::picture_frame::picture_frame)
                {
                    made?;
                }
            }
            C::ConfirmLeaving => {
                // The answer is read through `Var::Answer`.
                crate::menu::screen_loop::confirm_leaving()?;
            }
        }
        Ok(())
    }

    fn next_record_kind(&mut self) -> Result<u32, Refused> {
        self.fault.check()?;
        crate::menu::next_record_kind::next_record_kind()
    }

    fn load_picture(&mut self, p: Picture) -> Result<bool, Refused> {
        self.fault.check()?;
        let texts = &menus().tables.texts;
        let picture: &dyn crate::session::TextPlace = match p {
            Picture::Championship => &texts.mob_welc_png,
            Picture::A8 => &texts.a8_welc_png,
        };
        let failed = crate::gfx::load_picture_with_overlay::load_picture_with_overlay(
            picture,
            &texts.titlebar_png,
        )?;
        Ok(!failed)
    }

    fn take_key(&mut self) -> Result<Option<u8>, Refused> {
        self.fault.check()?;
        Ok(key(crate::input::take_key::take_key()))
    }

    fn wait_or_key(&mut self, ms: u32) -> Result<Option<u8>, Refused> {
        self.fault.check()?;
        Ok(key(crate::system::wait_or_key::wait_or_key(ms)))
    }

    fn stream_playing(&mut self) -> Result<bool, Refused> {
        self.fault.check()?;
        // The welcome polls this until a key comes: the window goes on meanwhile.
        crate::system::sleep::sleep(10);
        crate::platform::pump_messages_now();
        let (playing, _) = crate::sound::stream_playing::stream_playing();
        Ok(playing)
    }

    fn hand_cars(&mut self, list: CarList) {
        self.cars = Some(list);
    }

    fn fatal(&mut self, code: u32, message: &'static str) -> Refused {
        FatalError { code, message }.into()
    }
}
