//! The game for `rc2k_core::menu::flow` (2026-10-21): the event flows' variables at their
//! addresses, their callees through an `event_flow::Call` with the registers the originals
//! give them, and what the originals hand on in registers (one set carried through the
//! whole flow, as their `pusha` keeps it).

use crate::errors::{BadInput, FatalError, Refused};
use crate::framework::{game_addr, rva};
use crate::menu::event_flow::*;
use crate::menu::lists_store::MENU_LISTS;
use crate::menu::option_work_store::OPTION_WORK;
use crate::menu::results_store::RESULT_FIGURES;
use crate::menu::run_championship as champ;
use crate::menu::screen_loop::Carried;
use crate::menu::screen_loop::run_screen_carried;
use crate::menu::state_store::MENU_STATE;
use crate::race::state_store::RACE_STATE;
use crate::session::Dword;
use rc2k_core::menu::flow::{self, CarList, Hand, Picture, Reply, Var};

pub const PICTURE_FRAME: usize = 0x46F3FA; // menu::picture_frame, handed to the frame
pub const SPEECH_CHANNEL: &Dword = &OPTION_WORK.speech_level; // dword_755698
const PLAYER_TIME: u32 = 0x88;
const PLAYER_ENTRY: u32 = 0xD0;

pub struct Game {
    pub f: Flow,
    /// A refused read or write of the core's variables (checked at each call and at the end).
    fault: Fault,
}

impl Game {
    pub fn new(c: &Carried) -> Self {
        Self {
            f: Flow::new(c),
            fault: Fault::default(),
        }
    }

    /// A flow that goes on from the registers this one has (a nested flow: its changes
    /// are its own).
    pub fn nested(f: &Flow) -> Self {
        Self {
            f: *f,
            fault: Fault::default(),
        }
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

pub fn addr(a: usize) -> u32 {
    game_addr(rva(a))
}

/// Where a variable is.
enum Place {
    /// A variable of the front end's or the physics block's.
    Cell(&'static Dword),
    /// Player n's field (edi the player).
    Player(u32, u32),
    /// The found record's field.
    Record(u32),
    /// `g_quitting` (the session's).
    Quitting,
}

fn place(v: Var) -> Place {
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
        Var::RaceResult => Cell(&RACE_STATE.results.results[0].result),
        Var::RaceResult2 => Cell(&RACE_STATE.results.results[1].result),
        Var::CarListKind => Cell(CAR_LIST_KIND),
        Var::Cheats => Cell(CHEATS),
        Var::Cleared(k) => Cell(&RESULT_FIGURES.before[k as usize]),
        Var::PlayerTime(n) => Place::Player(n, PLAYER_TIME),
        Var::PlayerEntry(n) => Place::Player(n, PLAYER_ENTRY),
        Var::Continuing => Cell(CONTINUING),
        Var::StagesDone => Cell(champ::STAGES_DONE),
        Var::ResetTimes => Cell(champ::RESET_TIMES),
        Var::Services => Cell(champ::SERVICES),
        Var::ModeAB40 => Cell(&MENU_STATE.menus.class_mode),
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
        Var::RallyDone => Cell(&MENU_STATE.menus.rally_done),
        Var::WelcomeFigure => Cell(champ::WELCOME_FIGURE),
        Var::Welcomed => Cell(champ::WELCOMED),
        Var::ReviewKind => Cell(&MENU_STATE.menus.review_kind),
        Var::Record(offset) => Place::Record(offset),
        Var::SpeechChannel => Cell(SPEECH_CHANNEL),
        Var::Quitting => Place::Quitting,
    }
}

impl Game {
    /// A search's or walk's record (the original's edi) and the carried values it leaves.
    fn screen(&mut self, screen: &crate::menu::screen_loop::Screen) -> Result<(), Refused> {
        run_screen_carried(screen, &mut self.f.c)
    }

    /// The flags `input::take_key` and its like leave.
    fn key_flags(&mut self, key: u8) {
        self.f.zero = key == 0;
    }

    /// `callee`'s answer.
    fn reply(&self) -> Reply {
        Reply {
            eax: self.f.c.arg,
            carry: self.f.c.carry,
            zero: self.f.zero,
        }
    }
}

impl flow::Game for Game {
    type Error = Refused;

    fn get(&mut self, v: Var) -> u32 {
        match place(v) {
            Place::Cell(cell) => cell.get(),
            Place::Player(n, offset) => self.fault.get(player(n) + offset),
            Place::Record(offset) => self.fault.get(self.f.record + offset),
            Place::Quitting => crate::session::FLAGS.quitting.get(),
        }
    }

    fn set(&mut self, v: Var, value: u32) {
        match place(v) {
            Place::Cell(cell) => cell.set(value),
            Place::Player(n, offset) => {
                self.f.record = player(n);
                self.fault.put(self.f.record + offset, value);
            }
            Place::Record(offset) => self.fault.put(self.f.record + offset, value),
            Place::Quitting => crate::session::FLAGS.quitting.set(value),
        }
    }

    fn call(&mut self, c: flow::Call) -> Result<Reply, Refused> {
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
            C::ChampionshipWelcome => m::championship_welcome::championship_welcome(&self.f)?,
            C::A8Welcome => m::a8_welcome::a8_welcome(&self.f)?,
            C::FindRecord { section, index } => {
                use crate::records::find_record::{find_record_in, has_section};
                self.f.record = crate::records::stages_store::STAGE_TABLES.championship();
                self.f.c.palette = section;
                self.f.c.arg = index;
                // The original has no end-of-table check: a section past the table's
                // is refused.
                if !has_section(self.f.record, section, 600)? {
                    return Err(BadInput("a record section past the table").into());
                }
                self.f.record = find_record_in(self.f.record, index as i32, section)?;
            }
            C::NextRecordKind => self.f.c.arg = m::next_record_kind::next_record_kind()?,
            C::WelcomeLoop => self.screen(&m::welcome_loop::SCREEN)?,
            C::RallyStagesLoop => self.screen(&m::rally_stages_loop::SCREEN)?,
            C::NextStageLoop => self.screen(&m::next_stage_loop::SCREEN)?,
            C::ServicePass => m::service_pass::service_pass(&self.f.c)?,
            C::StandingsBFromAll => m::standings_b_from_all::standings_b_from_all()?,
            C::SortBByFigure => m::sort_standings_b_by_figure::sort_standings_b_by_figure()?,
            C::AwardStagePoints => m::award_stage_points::award_stage_points()?,
            C::SortBByPoints => m::sort_standings_b_by_points::sort_standings_b_by_points()?,
            C::ClassChampionshipPass => {
                m::class_championship_pass::class_championship_pass(&self.f.c)?
            }
            C::AwardOverallPoints => {
                let awarded = m::award_overall_points::award_overall_points()?;
                awarded.carry_on(&mut self.f.c);
            }
            C::ClearStandingsB => m::clear_standings_b::clear_standings_b()?,
            C::SortBByScore => m::sort_standings_b_by_score::sort_standings_b_by_score()?,
            C::NumberAllB => {
                let numbered = m::number_all_b::number_all_b()?;
                numbered.carry_on(&mut self.f.c);
            }
            C::ShowChampionshipStandings => {
                m::show_championship_standings::show_championship_standings(&self.f.c)?
            }
            C::SectionFinish => m::section_finish::section_finish()?,
            C::Review480D46 => self.screen(&m::review_options_480d46_loop::SCREEN)?,
            C::WalkSectionRecords => m::walk_section_records::walk_section_records()?,
            C::StandingsAFromAll => m::standings_a_from_all::standings_a_from_all()?,
            C::ClassRallyResultsPass => {
                m::class_rally_results_pass::class_rally_results_pass(&self.f.c)?
            }
            C::ClearStandingsA => m::clear_standings_a::clear_standings_a()?,
            C::NumberAllA => {
                let numbered = m::number_all_a::number_all_a()?;
                numbered.carry_on(&mut self.f.c);
            }
            C::ShowOverallRallyResults => {
                m::show_overall_rally_results::show_overall_rally_results(&self.f.c)?
            }
            C::RunEventRaces => m::run_event_races::run_event_races(&mut self.f)?,
            C::AddPlayerTimes => m::add_player_times::add_player_times()?,
            C::ClassStageResultsPass => {
                m::class_stage_results_pass::class_stage_results_pass(&self.f.c)?
            }
            C::ShowOverallStageResults => {
                m::show_overall_stage_results::show_overall_stage_results(&self.f.c)?
            }
            C::KeepRecordBest => m::keep_record_best::keep_record_best()?,
            C::RecordEntriesPass => m::record_entries_pass::record_entries_pass()?,
            C::BestTimesLoop => self.screen(&m::best_times_loop::SCREEN)?,
            C::InspectionPass => m::inspection_pass::inspection_pass(&self.f.c)?,
            C::Review47D5A5 => self.screen(&m::review_options_47d5a5_loop::SCREEN)?,
            C::Review48108E => self.screen(&m::review_options_48108e_loop::SCREEN)?,
            C::ChampionshipFinish => m::championship_finish::championship_finish()?,
            C::ClearScoreMarks => m::clear_score_marks::clear_score_marks()?,
            C::BestClassScores => m::best_class_scores::best_class_scores(&self.f.c)?,
            C::BestOverallScores => m::best_overall_scores::best_overall_scores(&self.f.c)?,
            C::Review481429 => self.screen(&m::review_options_481429_loop::SCREEN)?,
            // `from_entry`: the entry's registers are the ones it leaves, and it
            // reads none.
            C::ShowInGameOptions { .. } => m::in_game_options_items::show_in_game_options()?,
            C::HideInGameOptions { players } => {
                self.f.c.arg = players;
                m::in_game_options_items::hide_in_game_options(players)?
            }
            C::ResetPlayerTimes => m::reset_player_times::reset_player_times()?,
            C::ReloadFrontEnd => m::reload_front_end::reload_front_end()?,
            C::RepairCar { car } => {
                if let Some(car) = car {
                    self.f.c.measure_x = car;
                }
                crate::service::repair_parts::repair_car()?
            }
            C::CheatCarList => {
                m::cheat_car_list::carry_on(m::cheat_car_list::cheat_car_list(), &mut self.f.c)
            }
            C::SelectCarLoop => self.screen(&m::select_car_loop::SCREEN)?,
            C::TakeShownCar => m::take_shown_car::take_shown_car(self.f.c.cars)?,
            C::SelectStageLoop => self.screen(&m::select_stage_loop::SCREEN)?,
            C::TimeTrialSelectStage => self.screen(&m::time_trial_select_stage_loop::SCREEN)?,
            C::TimeTrialService => m::time_trial_service::time_trial_service(&self.f)?,
            C::Race { next, kind } => {
                self.f.c.arg = u32::from(next);
                self.f.c.palette = kind;
                crate::race::single_rally_race::single_rally_race(self.f.c.arg, kind)?
            }
            C::StageResultLoop => self.screen(&m::stage_result_loop::SCREEN)?,
            C::TimeTrialResultsLoop => self.screen(&m::time_trial_results_loop::SCREEN)?,
            C::Results48584B => self.screen(&m::results_48584b_loop::SCREEN)?,
            C::Results482E9B => self.screen(&m::results_482e9b_loop::SCREEN)?,
            C::Screen483163 => self.screen(&m::screen_483163_loop::SCREEN)?,
            C::KeepStageBest => m::keep_stage_best::keep_stage_best()?,
            C::TimeTrialQualifies { players } => {
                if let Some(players) = players {
                    self.f.c.measure_x = players;
                }
                m::time_trial_qualifies::time_trial_qualifies()?
            }
            C::ClearTimeTrialMarks => m::clear_time_trial_marks::clear_time_trial_marks()?,
            C::EnterTimeTrialBest => m::enter_time_trial_best::enter_time_trial_best()?,
            C::TimeTrialBestTimesLoop => self.screen(&m::time_trial_best_times_loop::SCREEN)?,
            C::Review485F5E => self.screen(&m::review_options_485f5e_loop::SCREEN)?,
            C::StartStream { channel } => {
                self.f.c.arg = crate::audio::names_store::SAMPLE_NAMES.rally_speech.addr(); // ";3files\%speech\rally\rally0.wav"
                self.f.c.measure_x = channel;
                crate::sound::start_stream::start_stream(self.f.c.arg, channel);
            }
            C::WaitForCommands => crate::sound::wait_for_commands::wait_in_game(),
            C::StopStream => crate::sound::stop_stream::stop_stream(),
            C::LoadPicture(p) => {
                let texts = &crate::menu::texts_store::MENU_TEXTS;
                let picture: &dyn crate::session::TextPlace = match p {
                    Picture::Championship => &texts.mob_welc_png,
                    Picture::A8 => &texts.a8_welc_png,
                };
                // ebx, the title bar's path, is what the loop's next steps read as a palette.
                self.f.c.palette = texts.titlebar_png.addr();
                self.f.c.carry = crate::gfx::load_picture_with_overlay::load_picture_with_overlay(
                    picture,
                    &texts.titlebar_png,
                )?;
            }
            C::ConvertPicture => {
                self.f.c.carry = crate::gfx::convert_picture::convert_picture_now()?
            }
            C::TakeKey => {
                let key = crate::input::take_key::take_key();
                self.f.c.arg = (self.f.c.arg & !0xFF) | u32::from(key);
                self.key_flags(key);
            }
            C::RunFrame => {
                // `screens::run_frame` with `menu::picture_frame`: the carry says
                // the frame was not made; the zero flag is then eax == 0 (it is not).
                self.f.c.arg = addr(PICTURE_FRAME);
                match crate::platform::run_frame_now(m::picture_frame::picture_frame) {
                    None => {
                        self.f.c.carry = false;
                        self.f.zero = true;
                    }
                    Some(made) => {
                        let made = made?;
                        self.f.c.carry = !made;
                        if made {
                            self.f.zero = false;
                        }
                    }
                }
            }
            C::WaitOrKey { ms } => {
                self.f.c.arg = ms;
                let key = crate::system::wait_or_key::wait_or_key(ms);
                self.f.c.arg = u32::from(key);
                self.key_flags(key);
                self.f.c.carry = false;
            }
            C::StreamPlaying => {
                // The welcome polls this until a key comes: the window goes on meanwhile.
                crate::system::sleep::sleep(10);
                crate::platform::pump_messages_now();
                let (zero, carry) = crate::sound::stream_playing::stream_playing();
                self.f.zero = zero;
                self.f.c.carry = carry;
            }
            C::ConfirmLeaving => {
                let answer = crate::menu::screen_loop::confirm_leaving(&self.f.c)?;
                self.f.c.carry = answer < 1;
                self.f.zero = answer == 1;
            }
        }

        Ok(self.reply())
    }

    fn hand(&mut self, h: Hand) -> Result<(), Refused> {
        match h {
            Hand::CarList(list) => {
                self.f.c.cars = match list {
                    CarList::First => MENU_LISTS.cars_first[0].addr(),
                    CarList::Second => MENU_LISTS.cars_second[0].addr(),
                    CarList::Cheat => MENU_LISTS.cheat_cars[0].addr(),
                }
            }
            Hand::CheatFlag(on) => self.f.c.carry = on,
            Hand::Kept(v) => self.f.c.arg = v,
            Hand::Stage => {
                self.f.record = stage_record()?;
                self.f.c.palette = get(STAGE_SUB);
                self.f.c.arg = get(STAGE_NUMBER).wrapping_mul(STAGE_RECORD);
            }
        }
        Ok(())
    }

    fn fatal(&mut self, code: u32, message: &'static str) -> Refused {
        FatalError { code, message }.into()
    }
}
