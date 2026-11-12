//! The game for `rc2k_core::menu::flow`: the event flows' variables at their
//! addresses, their callees, the record the last search found and the car
//! list handed on to the car choice.

use crate::errors::{BadInput, FatalError, Refused};
use crate::menu::event_flow::*;
use crate::menu::lists_store::CarList;
use crate::menu::owner::Menus;
use crate::menu::screen_loop::{Entry, Screen, run_screen};
use crate::menu::standings::Which;
use crate::race::owner::Race;
use crate::records::stages_store::RecordRef;
use crate::session::DwordLike;
use rc2k_core::menu::flow::{self, Picture, Var};
use rc2k_core::menu::leaving::Switch;

pub struct Game<'a> {
    /// The menus' owner.
    menus: &'a Menus,
    /// The game's owners, for the screens the flows enter.
    owners: crate::owners::Owners<'static>,
    /// The record `Call::FindRecord` found (the original's edi), read through `Var::Record`.
    record: Option<RecordRef>,
    /// The car list the next car choice shows (`Game::hand_cars`, `Call::CheatCarList`).
    cars: Option<CarList>,
    /// A refused read or write of the core's variables (checked at each call and at the end).
    fault: Fault,
}

impl<'a> Game<'a> {
    pub fn new(o: crate::owners::Owners<'static>) -> Self {
        Self {
            menus: o.menus,
            owners: o,
            record: None,
            cars: None,
            fault: Fault::default(),
        }
    }

    /// Runs a flow on this game; a refused read or write of its variables ends it.
    pub fn run<T>(
        mut self,
        flow: impl FnOnce(&mut Game<'a>) -> Result<T, Refused>,
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
enum Place<'a> {
    /// A variable of the front end's or the physics block's.
    Cell(&'a dyn DwordLike),
    /// Player n's field (edi the player).
    Player(u32, PlayerField),
    /// The found record's field.
    Record(u32),
}

fn place<'a>(menus: &'a Menus, race: &'a Race, v: Var) -> Place<'a> {
    use Place::Cell;
    match v {
        Var::Answer => Cell(&menus.tables.pages.yes_no_answer),
        Var::PlayerCount => Cell(&menus.screens.event.player_count),
        Var::EventMode => Cell(&menus.screens.event.event_kind),
        Var::EventType => Cell(&menus.screens.event.event_type),
        Var::Mode => Cell(&menus.play.champ_view.event_mode),
        Var::Current => Cell(&menus.screens.event.current_player),
        Var::Round => Cell(&menus.play.results.round),
        Var::Qualifies => Cell(&menus.screens.event.time_qualifies),
        Var::ServiceDue => Cell(&menus.screens.event.service_due),
        Var::RaceResult => Cell(&race.shared.state.results.results[0].result),
        Var::RaceResult2 => Cell(&race.shared.state.results.results[1].result),
        Var::CarListKind => Cell(&menus.screens.flow.car_list_kind),
        Var::Cheats => Cell(&menus.play.cheats.cheats),
        Var::Cleared(k) => Cell(&menus.play.results.before[k as usize]),
        Var::PlayerTime(n) => Place::Player(n, PlayerField::Time),
        Var::PlayerEntry(n) => Place::Player(n, PlayerField::Entry),
        Var::Continuing => Cell(&menus.screens.event.continuing_save),
        Var::StagesDone => Cell(&menus.screens.event.stages_done),
        Var::ResetTimes => Cell(&menus.tables.pages.reset_times),
        Var::Services => Cell(&menus.tables.pages.services),
        Var::ModeAB40 => Cell(&menus.screens.state.menus.class_mode),
        Var::StartSection { a8: false } => Cell(&menus.screens.flow.start_section),
        Var::StartSection { a8: true } => Cell(&menus.screens.flow.start_section_a8),
        Var::ServiceTime => Cell(&menus.screens.event.service_minutes),
        Var::ServiceRecordC => Cell(&menus.screens.event.service_record_c),
        Var::CpuSpreadFirst => Cell(&menus.screens.event.cpu_spread_first),
        Var::ServiceNext => Cell(&menus.screens.event.service_next),
        Var::TheEnd => Cell(&menus.screens.event.championship_end),
        Var::GoesOn => Cell(&menus.screens.event.event_goes_on),
        Var::Section => Cell(&menus.screens.event.section),
        Var::Index => Cell(&menus.screens.event.index),
        Var::WelcomeFigure => Cell(&menus.screens.event.welcome_figure),
        Var::Welcomed => Cell(&menus.screens.event.welcomed),
        Var::ReviewKind => Cell(&menus.screens.state.menus.review_kind),
        Var::Record(offset) => Place::Record(offset),
        Var::SpeechChannel => Cell(&menus.screens.option_work.speech_level),
    }
}

impl Game<'_> {
    /// A screen, entered with the car list handed (the only value a screen reads from its
    /// caller).
    fn screen(&self, screen: &Screen) -> Result<(), Refused> {
        run_screen(screen, &Entry::with_cars(self.owners, self.cars))?;
        Ok(())
    }
}

/// A key's code, none for 0 (`input::take_key`'s and `system::wait_or_key`'s).
fn key(code: u8) -> Option<u8> {
    (code != 0).then_some(code)
}

impl flow::Game for Game<'_> {
    type Error = Refused;

    fn get(&mut self, v: Var) -> u32 {
        match place(self.menus, self.owners.race, v) {
            Place::Cell(cell) => cell.get(),
            Place::Player(n, field) => *field.of(&mut player(self.menus, n).get()),
            Place::Record(offset) => self.fault.ok(self
                .record
                .ok_or(BadInput("a record field read before a record was found"))
                .and_then(|r| r.word_at(offset))),
        }
    }

    fn set(&mut self, v: Var, value: u32) {
        match place(self.menus, self.owners.race, v) {
            Place::Cell(cell) => cell.set(value),
            Place::Player(n, field) => player(self.menus, n).update(|q| *field.of(q) = value),
            // The flows only read the record.
            Place::Record(_) => {
                self.fault
                    .ok::<()>(Err(BadInput("a write to a stage record")));
            }
        }
    }

    fn flag(&mut self, s: Switch) -> bool {
        switch_cell(self.owners, s).is_set()
    }

    fn set_flag(&mut self, s: Switch, on: bool) {
        switch_cell(self.owners, s).set(on);
    }

    fn run(&mut self, c: flow::Call) -> Result<(), Refused> {
        self.fault.check()?;
        let menus = self.menus;
        let entry = &Entry::new(self.owners);
        use crate::menu as m;
        use flow::Call as C;
        match c {
            C::BeginEvent => m::begin_event::begin_event(menus)?,
            C::ShuffleDrivers => m::shuffle_drivers::shuffle_drivers(menus)?,
            C::ClearResultSlots => m::clear_result_slots::clear_result_slots(menus)?,
            C::ResetDrivers => m::reset_drivers::reset_drivers(menus)?,
            C::ResetDriverCondition => m::reset_driver_condition::reset_driver_condition(menus)?,
            C::ClearHudBits => m::clear_hud_bits::clear_hud_bits(menus)?,
            C::TakePlayer1Units => m::take_player1_units::take_player1_units(menus)?,
            C::HideAllInGameOptions => {
                m::hide_all_in_game_options::hide_all_in_game_options(menus)?;
            }
            C::ClearEventState => m::clear_event_state::clear_event_state(menus)?,
            C::ReadyPlayers => m::ready_players::ready_players(menus)?,
            C::ResetSingleEvent => m::reset_single_event::reset_single_event(menus)?,
            C::ChampionshipWelcome => m::championship_welcome::championship_welcome(self.owners)?,
            C::A8Welcome => m::a8_welcome::a8_welcome(self.owners)?,
            C::FindRecord { section, index } => {
                use crate::records::find_record::{find_record_in, has_section};
                let table = self.owners.system.records.stage_tables.championship();
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
            C::ServicePass => m::service_pass::service_pass(menus, entry)?,
            C::StandingsBFromAll => m::standings_b_from_all::standings_b_from_all(menus)?,
            C::SortBByFigure => m::sort_standings_b_by_figure::sort_standings_b_by_figure(menus)?,
            C::AwardStagePoints => m::award_stage_points::award_stage_points(menus)?,
            C::SortBByPoints => m::sort_standings_b_by_points::sort_standings_b_by_points(menus)?,
            C::ClassChampionshipPass => {
                m::class_championship_pass::class_championship_pass(menus, entry)?;
            }
            C::AwardOverallPoints => {
                m::award_overall_points::award_overall_points(menus)?;
            }
            C::ClearStandingsB => m::standings_steps::clear_standings(Which::B)?,
            C::SortBByScore => m::sort_standings_b_by_score::sort_standings_b_by_score(menus)?,
            C::NumberAllB => {
                m::standings_steps::number_all(menus, Which::B)?;
            }
            C::ShowChampionshipStandings => {
                m::show_championship_standings::show_championship_standings(menus, entry)?;
            }
            C::SectionFinish => m::section_finish::section_finish(self.owners)?,
            C::ChampionshipStandingsReview => {
                self.screen(&m::championship_standings_review_loop::SCREEN)?;
            }
            C::WalkSectionRecords => m::walk_section_records::walk_section_records(
                menus,
                self.owners.system,
                self.owners.race,
            )?,
            C::StandingsAFromAll => m::standings_a_from_all::standings_a_from_all(menus)?,
            C::ClassRallyResultsPass => {
                m::class_rally_results_pass::class_rally_results_pass(menus, entry)?;
            }
            C::ClearStandingsA => m::standings_steps::clear_standings(Which::A)?,
            C::NumberAllA => {
                m::standings_steps::number_all(menus, Which::A)?;
            }
            C::ShowOverallRallyResults => {
                m::show_overall_rally_results::show_overall_rally_results(menus, entry)?;
            }
            C::RunEventRaces => m::run_event_races::run_event_races(self.owners)?,
            C::AddPlayerTimes => m::add_player_times::add_player_times(menus)?,
            C::ClassStageResultsPass => {
                m::class_stage_results_pass::class_stage_results_pass(menus, entry)?;
            }
            C::ShowOverallStageResults => {
                m::show_overall_stage_results::show_overall_stage_results(menus, entry)?;
            }
            C::KeepRecordBest => m::keep_record_best::keep_record_best(menus, self.owners.system)?,
            C::RecordEntriesPass => m::record_entries_pass::record_entries_pass(
                menus,
                self.owners.system,
                self.owners.race,
            )?,
            C::BestTimesLoop => self.screen(&m::best_times_loop::SCREEN)?,
            C::InspectionPass => m::inspection_pass::inspection_pass(menus, entry)?,
            C::ChampionshipServiceReview => {
                self.screen(&m::championship_service_review_loop::SCREEN)?;
            }
            C::ChampionshipRacesReview => {
                self.screen(&m::championship_races_review_loop::SCREEN)?;
            }
            C::ChampionshipFinish => m::championship_finish::championship_finish(self.owners)?,
            C::ClearScoreMarks => m::clear_score_marks::clear_score_marks()?,
            C::BestClassScores => m::best_class_scores::best_class_scores(menus, entry)?,
            C::BestOverallScores => m::best_overall_scores::best_overall_scores(menus, entry)?,
            C::ChampionshipEndReview => self.screen(&m::championship_end_review_loop::SCREEN)?,
            C::ShowInGameOptions => m::in_game_options_items::show_in_game_options(menus)?,
            C::HideInGameOptions { players } => {
                m::in_game_options_items::hide_in_game_options(menus, players)?;
            }
            C::ResetPlayerTimes => m::reset_player_times::reset_player_times(menus)?,
            C::ReloadFrontEnd => m::reload_front_end::reload_front_end(self.owners)?,
            C::RepairCar => crate::service::repair_parts::repair_car_in(menus)?,
            C::CheatCarList => {
                if let Some(list) = m::cheat_car_list::cheat_car_list(menus) {
                    self.cars = Some(list);
                }
            }
            C::SelectCarLoop => self.screen(&m::select_car_loop::SCREEN)?,
            C::TakeShownCar => m::take_shown_car::take_shown_car(menus, self.cars)?,
            C::SelectStageLoop => self.screen(&m::select_stage_loop::SCREEN)?,
            C::TimeTrialSelectStage => self.screen(&m::time_trial_select_stage_loop::SCREEN)?,
            C::TimeTrialService => m::time_trial_service::time_trial_service(menus, entry)?,
            C::Race { next, kind } => {
                crate::race::single_rally_race::single_rally_race(
                    self.owners,
                    u32::from(next),
                    kind,
                )?;
            }
            C::StageResultLoop => self.screen(&m::stage_result_loop::SCREEN)?,
            C::TimeTrialResultsLoop => self.screen(&m::time_trial_results_loop::SCREEN)?,
            C::SingleRallyTwoResults => self.screen(&m::single_rally_two_results_loop::SCREEN)?,
            C::SingleRallyThreeFourResults => {
                self.screen(&m::single_rally_three_four_results_loop::SCREEN)?;
            }
            C::SingleRallyOverallResults => {
                self.screen(&m::single_rally_overall_results_loop::SCREEN)?;
            }
            C::KeepStageBest => m::keep_stage_best::keep_stage_best(menus, self.owners.system)?,
            C::TimeTrialQualifies => {
                m::time_trial_qualifies::time_trial_qualifies(menus, self.owners.system)?;
            }
            C::ClearTimeTrialMarks => {
                m::clear_time_trial_marks::clear_time_trial_marks(menus, self.owners.system)?;
            }
            C::EnterTimeTrialBest => {
                m::enter_time_trial_best::enter_time_trial_best(
                    self.owners.race,
                    menus,
                    self.owners.system,
                )?;
            }
            C::TimeTrialBestTimesLoop => self.screen(&m::time_trial_best_times_loop::SCREEN)?,
            C::SingleRallyReview => self.screen(&m::single_rally_review_loop::SCREEN)?,
            C::StartStream { channel } => {
                // ";3files\%speech\rally\rally0.wav"
                use crate::audio::names_store::SampleNames;
                let name = core::mem::offset_of!(SampleNames, rally_speech) as u32;
                crate::sound::start_stream::start_stream(
                    self.owners.audio,
                    self.owners.system,
                    name,
                    channel,
                );
            }
            C::WaitForCommands => {
                crate::sound::wait_for_commands::wait_in_game(
                    self.owners.audio,
                    self.owners.system,
                );
            }
            C::StopStream => {
                crate::sound::stop_stream::stop_stream(self.owners.audio, self.owners.system);
            }
            C::ConvertPicture => {
                // Whether it worked is not looked at.
                crate::gfx::convert_picture::convert_picture_now_in(
                    self.owners.gfx,
                    self.owners.system,
                )?;
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
                crate::menu::screen_loop::confirm_leaving(self.owners)?;
            }
        }
        Ok(())
    }

    fn next_record_kind(&mut self) -> Result<u32, Refused> {
        self.fault.check()?;
        crate::menu::next_record_kind::next_record_kind(self.menus, self.owners.system)
    }

    fn load_picture(&mut self, p: Picture) -> Result<bool, Refused> {
        self.fault.check()?;
        let texts = &self.menus.tables.texts;
        let picture: &dyn crate::session::TextPlace = match p {
            Picture::Championship => &texts.mob_welc_png,
            Picture::A8 => &texts.a8_welc_png,
        };
        let failed = crate::gfx::load_picture_with_overlay::load_picture_with_overlay_in(
            picture,
            &texts.titlebar_png,
            self.owners.gfx,
            self.owners.system,
            self.owners.strings,
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
        let (playing, _) = crate::sound::stream_playing::stream_playing(self.owners.audio);
        Ok(playing)
    }

    fn hand_cars(&mut self, list: CarList) {
        self.cars = Some(list);
    }

    fn fatal(&mut self, code: u32, message: &'static str) -> Refused {
        FatalError { code, message }.into()
    }
}
