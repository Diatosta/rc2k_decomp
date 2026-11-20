//! The game for `rc2k_core::menu::flow`: the event flows' variables at their
//! addresses, their callees, the record the last search found and the car
//! list handed on to the car choice.

use crate::errors::{BadInput, FatalError, Fault, Refused};
use crate::memory::session::DwordLike;
use crate::menu::event_flow::*;
use crate::menu::event_store::EventWord;
use crate::menu::lists_store::CarList;
use crate::menu::owner::Menus;
use crate::menu::screen_id::ScreenId;
use crate::menu::screen_loop::{Entry, run_screen};
use crate::menu::standings::Which;
use crate::race::owner::Race;
use crate::records::stages_store::RecordRef;
use rc2k_core::menu::flow::{self, Picture, Var};
use rc2k_core::menu::leaving::Switch;

pub struct Game<'a> {
    /// The menus' owner.
    menus: &'a Menus,
    /// The game's owners, for the screens the flows enter.
    owners: crate::owners::Owners<'static>,
    /// The record `Call::FindRecord` found, read through `Var::Record`.
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

    /// The car list the next car choice shows, if one was handed.
    pub(super) fn handed_cars(&self) -> Option<CarList> {
        self.cars
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

/// The variables kept as plain fields.
#[derive(Clone, Copy)]
enum PlainField {
    Cheats,
    Answer,
    CarListKind,
    StartSection,
    StartSectionA8,
    Event(EventWord),
    ClassMode,
    ReviewKind,
    ChampMode,
    ChampResetTimes,
    ChampServices,
}

impl PlainField {
    fn get(self, menus: &Menus) -> u32 {
        let flow = menus.screens.flow.read();
        match self {
            PlainField::Cheats => menus.play.cheats.read().cheats,
            PlainField::Answer => menus.tables.pages.read().yes_no_answer,
            PlainField::CarListKind => flow.car_list_kind,
            PlainField::StartSection => flow.start_section,
            PlainField::StartSectionA8 => flow.start_section_a8,
            PlainField::Event(word) => word.get(&menus.screens.event),
            PlainField::ClassMode => menus.screens.state.read().menus.class_mode,
            PlainField::ReviewKind => menus.screens.state.read().menus.review_kind,
            PlainField::ChampMode => menus.play.champ_view.read().event_mode,
            PlainField::ChampResetTimes => menus.play.champ_view.read().reset_times,
            PlainField::ChampServices => menus.play.champ_view.read().services,
        }
    }

    fn set(self, menus: &Menus, value: u32) {
        match self {
            PlainField::Cheats => menus.play.cheats.write().cheats = value,
            PlainField::Answer => menus.tables.pages.write().yes_no_answer = value,
            PlainField::CarListKind => menus.screens.flow.write().car_list_kind = value,
            PlainField::StartSection => menus.screens.flow.write().start_section = value,
            PlainField::StartSectionA8 => menus.screens.flow.write().start_section_a8 = value,
            PlainField::Event(word) => word.set(&menus.screens.event, value),
            PlainField::ClassMode => menus.screens.state.write().menus.class_mode = value,
            PlainField::ReviewKind => menus.screens.state.write().menus.review_kind = value,
            PlainField::ChampMode => menus.play.champ_view.write().event_mode = value,
            PlainField::ChampResetTimes => menus.play.champ_view.write().reset_times = value,
            PlainField::ChampServices => menus.play.champ_view.write().services = value,
        }
    }
}

/// Where a variable is.
enum Place<'a> {
    /// A variable of the front end's or the physics block's.
    Cell(&'a dyn DwordLike),
    /// A field of a store of plain values.
    Plain(PlainField),
    /// Player n's field.
    Player(u32, PlayerField),
    /// The found record's field.
    Record(u32),
    /// A variable that names nothing.
    Refused(&'static str),
}

fn place<'a>(menus: &'a Menus, race: &'a Race, v: Var) -> Place<'a> {
    use Place::Cell;
    match v {
        Var::Answer => Place::Plain(PlainField::Answer),
        Var::PlayerCount => Place::Plain(PlainField::Event(EventWord::PlayerCount)),
        Var::EventMode => Place::Plain(PlainField::Event(EventWord::EventKind)),
        Var::EventType => Place::Plain(PlainField::Event(EventWord::EventType)),
        Var::Mode => Place::Plain(PlainField::ChampMode),
        Var::Current => Place::Plain(PlainField::Event(EventWord::CurrentPlayer)),
        Var::Round => Cell(&menus.play.results.round),
        Var::Qualifies => Place::Plain(PlainField::Event(EventWord::TrialQualifies)),
        Var::ServiceDue => Place::Plain(PlainField::Event(EventWord::ServiceDue)),
        Var::RaceResult => Cell(&race.shared.state.results.results[0].result),
        Var::RaceResult2 => Cell(&race.shared.state.results.results[1].result),
        Var::RaceResultOf(k) => Cell(&race.shared.state.result(k as usize).result),
        Var::CarListKind => Place::Plain(PlainField::CarListKind),
        Var::Cheats => Place::Plain(PlainField::Cheats),
        // The eight cleared figures are the four totals before the stage, then the four totals.
        Var::Cleared(k @ 0..4) => Cell(&menus.play.results.players[k as usize].before),
        Var::Cleared(k @ 4..8) => Cell(&menus.play.results.players[k as usize - 4].total),
        Var::Cleared(_) => Place::Refused("a cleared figure past the eight"),
        Var::PlayerTime(n) => Place::Player(n, PlayerField::Time),
        Var::PlayerEntry(n) => Place::Player(n, PlayerField::Entry),
        Var::Continuing => Place::Plain(PlainField::Event(EventWord::ContinuingSave)),
        Var::StagesDone => Place::Plain(PlainField::Event(EventWord::StagesDone)),
        Var::ResetTimes => Place::Plain(PlainField::ChampResetTimes),
        Var::Services => Place::Plain(PlainField::ChampServices),
        Var::ModeAB40 => Place::Plain(PlainField::ClassMode),
        Var::StartSection { a8: false } => Place::Plain(PlainField::StartSection),
        Var::StartSection { a8: true } => Place::Plain(PlainField::StartSectionA8),
        Var::ServiceTime => Place::Plain(PlainField::Event(EventWord::ServiceMinutes)),
        Var::CpuSpreadFirst => Place::Plain(PlainField::Event(EventWord::SpreadFirst)),
        Var::ServiceNext => Place::Plain(PlainField::Event(EventWord::ServiceNext)),
        Var::TheEnd => Place::Plain(PlainField::Event(EventWord::ChampionshipEnd)),
        Var::GoesOn => Place::Plain(PlainField::Event(EventWord::EventGoesOn)),
        Var::Section => Place::Plain(PlainField::Event(EventWord::Section)),
        Var::Index => Place::Plain(PlainField::Event(EventWord::StageInSection)),
        Var::ReviewKind => Place::Plain(PlainField::ReviewKind),
        Var::Record(offset) => Place::Record(offset),
        Var::SpeechChannel => Cell(&menus.options.menu_voices_volume),
    }
}

impl Game<'_> {
    /// A screen, entered with the car list handed (the only value a screen reads from its
    /// caller).
    fn screen(&self, screen: ScreenId) -> Result<(), Refused> {
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
            Place::Plain(field) => field.get(self.menus),
            Place::Player(n, field) => *field.of(&mut player(self.menus, n).get()),
            Place::Refused(why) => {
                self.fault.ok::<()>(Err(BadInput(why)));
                0
            }
            Place::Record(offset) => self.fault.ok(self
                .record
                .ok_or(BadInput("a record field read before a record was found"))
                .and_then(|r| r.word_at(&self.owners.system.records().stage_tables, offset))),
        }
    }

    fn set(&mut self, v: Var, value: u32) {
        match place(self.menus, self.owners.race, v) {
            Place::Cell(cell) => cell.set(value),
            Place::Plain(field) => field.set(self.menus, value),
            Place::Player(n, field) => player(self.menus, n).update(|q| *field.of(q) = value),
            Place::Refused(why) => {
                self.fault.ok::<()>(Err(BadInput(why)));
            }
            // The flows only read the record.
            Place::Record(_) => {
                self.fault
                    .ok::<()>(Err(BadInput("a write to a stage record")));
            }
        }
    }

    fn flag(&mut self, s: Switch) -> bool {
        switch_is_set(self.owners.menus, self.owners.system, s)
    }

    fn set_flag(&mut self, s: Switch, on: bool) {
        set_switch(self.owners.menus, self.owners.system, s, on);
    }

    fn run(&mut self, c: flow::Call) -> Result<(), Refused> {
        self.fault.check()?;
        let menus = self.menus;
        let entry = &Entry::new(self.owners);
        let tables = &self.owners.saves.best_tables;
        use crate::menu as m;
        use flow::Call as C;
        match c {
            C::BeginEvent => m::begin_event::begin_event(&menus.cars, &menus.play, &menus.screens)?,
            C::ShuffleDrivers => {
                m::shuffle_drivers::shuffle_drivers(menus, &self.owners.race.shared.random)?;
            }
            C::ClearResultSlots => m::clear_result_slots::clear_result_slots(menus)?,
            C::ResetDrivers => m::reset_drivers::reset_drivers(menus)?,
            C::ResetDriverCondition => m::reset_driver_condition::reset_driver_condition(menus)?,
            C::ClearHudBits => m::clear_hud_bits::clear_hud_bits(menus)?,
            C::TakePlayer1Units => {
                m::take_player1_units::take_player1_units(menus, self.owners.strings)?;
            }
            C::HideAllInGameOptions => {
                m::hide_all_in_game_options::hide_all_in_game_options(menus)?;
            }
            C::ClearEventState => m::clear_event_state::clear_event_state(menus)?,
            C::ReadyPlayers => m::ready_players::ready_players(menus)?,
            C::ResetSingleEvent => {
                m::reset_single_event::reset_single_event(
                    &menus.cars,
                    &menus.play,
                    &menus.screens,
                )?;
            }
            C::ChampionshipWelcome => m::championship_welcome::championship_welcome(self.owners)?,
            C::A8Welcome => m::a8_welcome::a8_welcome(self.owners)?,
            C::FindRecord { section, index } => {
                use crate::records::find_record::{find_record_in, has_section};
                let stages = &self.owners.system.records().stage_tables;
                let table = stages.championship();
                // A section past the table's end is refused.
                if !has_section(stages, table, section, 600)? {
                    return Err(BadInput("a record section past the table").into());
                }
                self.record = Some(find_record_in(stages, table, index as i32, section)?);
            }
            C::WelcomeLoop => self.screen(ScreenId::Welcome)?,
            C::RallyStagesLoop => self.screen(ScreenId::RallyStages)?,
            C::NextStageLoop => self.screen(ScreenId::NextStage)?,
            C::ServicePass => m::service_pass::service_pass(menus, entry)?,
            C::StandingsBFromAll => m::standings_b_from_all::standings_b_from_all(
                crate::strings::append_string_by_id::StringOwners::of(entry.owners),
                menus,
                tables,
            )?,
            C::SortBByFigure => {
                m::sort_standings_b_by_figure::sort_standings_b_by_figure(menus, tables)?;
            }
            C::AwardStagePoints => m::award_stage_points::award_stage_points(menus, tables)?,
            C::SortBByPoints => {
                m::sort_standings_b_by_points::sort_standings_b_by_points(menus, tables)?;
            }
            C::ClassChampionshipPass => {
                m::class_championship_pass::class_championship_pass(menus, entry, tables)?;
            }
            C::AwardOverallPoints => {
                m::award_overall_points::award_overall_points(menus, tables)?;
            }
            C::ClearStandingsB => m::standings_steps::clear_standings(Which::B, tables)?,
            C::SortBByScore => {
                m::sort_standings_b_by_score::sort_standings_b_by_score(menus, tables)?;
            }
            C::NumberAllB => {
                m::standings_steps::number_all(&menus.screens, &menus.play, Which::B, tables)?;
            }
            C::ShowChampionshipStandings => {
                m::show_championship_standings::show_championship_standings(
                    &menus.play.champ_view,
                    entry,
                    tables,
                )?;
            }
            C::SectionFinish => m::section_finish::section_finish(self.owners)?,
            C::ChampionshipStandingsReview => {
                self.screen(ScreenId::ChampionshipStandingsReview)?;
            }
            C::WalkSectionRecords => m::walk_section_records::walk_section_records(
                menus,
                self.owners.system,
                self.owners.race,
                tables,
            )?,
            C::StandingsAFromAll => m::standings_a_from_all::standings_a_from_all(
                crate::strings::append_string_by_id::StringOwners::of(entry.owners),
                &menus.cars,
                &menus.screens,
                &menus.play,
                tables,
            )?,
            C::ClassRallyResultsPass => {
                m::class_rally_results_pass::class_rally_results_pass(
                    &menus.screens,
                    &menus.play,
                    &menus.tables.pages,
                    entry,
                    tables,
                )?;
            }
            C::ClearStandingsA => m::standings_steps::clear_standings(Which::A, tables)?,
            C::NumberAllA => {
                m::standings_steps::number_all(&menus.screens, &menus.play, Which::A, tables)?;
            }
            C::ShowOverallRallyResults => {
                m::show_overall_rally_results::show_overall_rally_results(
                    &menus.play.champ_view,
                    entry,
                    tables,
                )?;
            }
            C::RunEventRaces => m::run_event_races::run_event_races(self.owners)?,
            C::AddPlayerTimes => m::add_player_times::add_player_times(menus)?,
            C::ClassStageResultsPass => {
                m::class_stage_results_pass::class_stage_results_pass(
                    &menus.screens,
                    &menus.play,
                    &menus.tables.pages,
                    entry,
                    tables,
                )?;
            }
            C::ShowOverallStageResults => {
                m::show_overall_stage_results::show_overall_stage_results(
                    &menus.play.champ_view,
                    entry,
                    tables,
                )?;
            }
            C::KeepRecordBest => m::keep_record_best::keep_record_best(menus, self.owners.system)?,
            C::RecordEntriesPass => m::record_entries_pass::record_entries_pass(
                menus,
                self.owners.system,
                self.owners.race,
                tables,
            )?,
            C::BestTimesLoop => self.screen(ScreenId::BestTimes)?,
            C::InspectionPass => m::inspection_pass::inspection_pass(menus, entry)?,
            C::ChampionshipServiceReview => {
                self.screen(ScreenId::ChampionshipServiceReview)?;
            }
            C::ChampionshipRacesReview => {
                self.screen(ScreenId::ChampionshipRacesReview)?;
            }
            C::ChampionshipFinish => m::championship_finish::championship_finish(self.owners)?,
            C::ClearScoreMarks => m::clear_score_marks::clear_score_marks(tables)?,
            C::BestClassScores => m::best_class_scores::best_class_scores(
                menus,
                &self.owners.saves.best_tables,
                entry,
            )?,
            C::BestOverallScores => {
                m::best_overall_scores::best_overall_scores(menus, entry, tables)?;
            }
            C::ChampionshipEndReview => self.screen(ScreenId::ChampionshipEndReview)?,
            C::ShowInGameOptions => m::in_game_options_items::show_in_game_options(menus)?,
            C::HideInGameOptions { players } => {
                m::in_game_options_items::hide_in_game_options(menus, players)?;
            }
            C::ResetPlayerTimes => {
                m::reset_player_times::reset_player_times(&menus.screens, &menus.play)?;
            }
            C::ReloadFrontEnd => m::reload_front_end::reload_front_end(self.owners)?,
            C::RepairCar => crate::service::repair_parts::repair_car_in(menus)?,
            C::CheatCarList => {
                if let Some(list) = m::cheat_car_list::cheat_car_list(menus) {
                    self.cars = Some(list);
                }
            }
            C::SelectCarLoop => self.screen(ScreenId::SelectCar)?,
            C::TakeShownCar => m::take_shown_car::take_shown_car(menus, self.cars)?,
            C::SelectStageLoop => self.screen(ScreenId::SelectStage)?,
            C::TimeTrialSelectStage => self.screen(ScreenId::TimeTrialSelectStage)?,
            C::TimeTrialService => m::time_trial_service::time_trial_service(menus, entry)?,
            C::Race { next, kind } => {
                crate::race::single_rally_race::single_rally_race(
                    self.owners,
                    u32::from(next),
                    kind,
                )?;
            }
            C::StageResultLoop => self.screen(ScreenId::StageResult)?,
            C::TimeTrialResultsLoop => self.screen(ScreenId::TimeTrialResults)?,
            C::SingleRallyTwoResults => self.screen(ScreenId::SingleRallyTwoResults)?,
            C::SingleRallyThreeFourResults => {
                self.screen(ScreenId::SingleRallyThreeFourResults)?;
            }
            C::SingleRallyOverallResults => {
                self.screen(ScreenId::SingleRallyOverallResults)?;
            }
            C::KeepStageBest => m::keep_stage_best::keep_stage_best(menus, self.owners.system)?,
            C::TimeTrialQualifies => {
                m::time_trial_qualifies::time_trial_qualifies(menus, self.owners.system, tables)?;
            }
            C::ClearTimeTrialMarks => {
                m::clear_time_trial_marks::clear_time_trial_marks(
                    menus,
                    self.owners.system,
                    tables,
                )?;
            }
            C::EnterTimeTrialBest => {
                m::enter_time_trial_best::enter_time_trial_best(
                    self.owners.race,
                    menus,
                    self.owners.system,
                    tables,
                )?;
            }
            C::TimeTrialBestTimesLoop => self.screen(ScreenId::TimeTrialBestTimes)?,
            C::SingleRallyReview => self.screen(ScreenId::SingleRallyReview)?,
            C::StartStream { channel } => {
                // ";3files\%speech\rally\rally0.wav"
                let name = crate::audio::names_store::NameList::RallySpeech.first_key();
                crate::sound::start_stream::start_stream(self.owners, name, channel);
            }
            C::WaitForCommands => {
                crate::sound::wait_for_commands::wait_in_game(self.owners);
            }
            C::StopStream => {
                crate::sound::stop_stream::stop_stream(self.owners);
            }
            C::ConvertPicture => {
                // Whether it worked is not looked at.
                crate::gfx::convert_picture::convert_picture_now_in(self.owners.gfx)?;
            }
            C::RunFrame => {
                // `screens::run_frame` with `menu::picture_frame` (whether the frame was
                // made is not looked at).
                if let Some(made) = crate::shell::frame::run_frame_now(self.owners, || {
                    m::picture_frame::picture_frame(self.owners)
                }) {
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
        let texts = &self.menus.texts;
        let picture: &dyn crate::memory::session::TextPlace = match p {
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
        Ok(key(crate::input::take_key::take_key(self.owners.input)))
    }

    fn wait_or_key(&mut self, ms: u32) -> Result<Option<u8>, Refused> {
        self.fault.check()?;
        Ok(key(crate::system::wait_or_key::wait_or_key(
            self.owners,
            ms,
        )))
    }

    fn stream_playing(&mut self) -> Result<bool, Refused> {
        self.fault.check()?;
        // The welcome polls this until a key comes: the window goes on meanwhile.
        crate::clock::sleep::sleep(10);
        crate::shell::frame::pump_messages_now(self.owners);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cleared_figures_are_the_totals_before_the_stage_then_the_totals() {
        let _lock = crate::test_support::test_lock();
        let menus = crate::test_support::own::menus();
        let race = crate::test_support::own::race();
        let players = &menus.play.results.players;
        let kept: Vec<(u32, u32)> = players
            .iter()
            .map(|p| (p.before.get(), p.total.get()))
            .collect();
        for k in 0..8u32 {
            match place(menus, race, Var::Cleared(k)) {
                Place::Cell(cell) => cell.set(0x100 + k),
                _ => panic!("cleared figure {k} is not a cell"),
            }
        }
        let read: Vec<(u32, u32)> = players
            .iter()
            .map(|p| (p.before.get(), p.total.get()))
            .collect();
        for (p, (before, total)) in players.iter().zip(kept) {
            p.before.set(before);
            p.total.set(total);
        }
        assert_eq!(
            read,
            vec![
                (0x100, 0x104),
                (0x101, 0x105),
                (0x102, 0x106),
                (0x103, 0x107)
            ]
        );
        assert!(matches!(
            place(menus, race, Var::Cleared(8)),
            Place::Refused(_)
        ));
    }
}
