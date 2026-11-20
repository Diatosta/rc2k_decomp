use crate::errors::Refused;
use crate::memory::session::DwordLike;
use crate::memory::session::PlainDword;
use crate::menu::event_flow::switch_cell;
use crate::menu::lists_store::CarList;
use crate::menu::owner::Menus;
use crate::menu::screen_id::ScreenId;
use crate::menu::screen_loop::{Entry, run_screen};
use crate::race::owner::Race;
use rc2k_core::menu::arcade_event::{self as arcade, Step, Var};
use rc2k_core::menu::leaving::Switch;

/// The game for `rc2k_core::menu::arcade_event`: the variables at their addresses, the
/// steps and the car list the car choice was shown.
struct Game<'a> {
    menus: &'a Menus,
    owners: crate::owners::Owners<'static>,
    cars: Option<CarList>,
    player0: crate::menu::standings::PlayerCell<'a>,
    /// A launch's answer to the car choice, until it has given it.
    answer: Option<crate::menu::launch::CarAnswer<'a>>,
}

/// The variables that are in the physics block.
fn cell(race: &Race, v: Var) -> Option<&dyn DwordLike> {
    Some(match v {
        Var::RaceStage => &race.shared.state.results.race_stage,
        Var::RaceLevel => &race.shared.state.results.race_level,
        Var::CheatSwitch => &race.shared.state.settings.direction,
        Var::RaceResult => &race.shared.state.results.results[0].result,
        Var::RaceFigureFFC => &race.shared.state.results.figure_ffc,
        Var::RaceFigure000 => &race.shared.state.results.figure_000,
        _ => return None,
    })
}

fn address(menus: &Menus, v: Var) -> &'_ PlainDword {
    match v {
        Var::Failed => &menus.play.arcade.retry,
        Var::TotalA8 => &menus.play.results.players[0].before,
        Var::Total798 => &menus.play.results.players[0].sum,
        Var::TotalB8 => &menus.play.results.players[0].total,
        Var::Figure768 => &menus.play.results.players[0].position,
        Var::Figure758 => &menus.play.results.players[0].time_bonus,
        Var::PlayerCount => &menus.screens.event.player_count,
        Var::EventMode => &menus.screens.event.event_kind,
        Var::Stage => &menus.play.arcade.stage,
        Var::Level => &menus.play.arcade.level,
        Var::Current => &menus.screens.event.current_player,
        Var::Answer => &menus.tables.pages.yes_no_answer,
        Var::Cheats => &menus.play.cheats.cheats,
        Var::Qualifies => &menus.screens.event.trial.qualifies,
        Var::Player0Entry => unreachable!("player 0's field is not at an address"),
        Var::RaceStage
        | Var::RaceLevel
        | Var::CheatSwitch
        | Var::RaceResult
        | Var::RaceFigureFFC
        | Var::RaceFigure000 => unreachable!("a variable of the physics block, `cell`"),
    }
}

impl arcade::Arcade for Game<'_> {
    type Error = Refused;

    fn get(&mut self, v: Var) -> u32 {
        let menus = self.menus;
        match cell(self.owners.race, v) {
            Some(cell) => cell.get(),
            None => address(menus, v).get(),
        }
    }

    fn set(&mut self, v: Var, value: u32) {
        let menus = self.menus;
        match v {
            Var::Player0Entry => self.player0.update(|p| p.entry = value),
            _ => match cell(self.owners.race, v) {
                Some(cell) => cell.set(value),
                None => address(menus, v).set(value),
            },
        }
    }

    fn flag(&mut self, s: Switch) -> bool {
        switch_cell(self.owners.menus, self.owners.system, s).is_set()
    }

    fn set_flag(&mut self, s: Switch, on: bool) {
        switch_cell(self.owners.menus, self.owners.system, s).set(on);
    }

    fn step(&mut self, s: Step) -> Result<(), Refused> {
        let sound = self.owners.sound;
        let menus = self.menus;
        let play = &menus.play;
        let tables = &self.owners.saves.best_tables;
        use crate::menu as m;
        use crate::menu::arcade_steps as a;
        match s {
            Step::ClearHudBits => m::clear_hud_bits::clear_hud_bits(menus)?,
            Step::HideAllInGameOptions => {
                m::hide_all_in_game_options::hide_all_in_game_options(menus)?;
            }
            Step::SecretCarList => {
                self.cars =
                    Some(m::cheat_car_list::cheat_car_list(menus).unwrap_or(CarList::Second));
            }
            Step::SelectCarLoop => {
                let handed = self.cars;
                let answered = match (self.answer.as_mut(), handed) {
                    (Some(answer), Some(list)) => answer(list)?,
                    _ => None,
                };
                match answered {
                    Some(list) => {
                        self.answer = None;
                        self.cars = Some(list);
                        arcade::Arcade::set_flag(self, Switch::LeftByKey, false);
                    }
                    None => self.screen(ScreenId::SelectCar)?,
                }
            }
            Step::TakeShownCar => m::take_shown_car::take_shown_car(menus, self.cars)?,
            Step::NextStageLoop => self.screen(ScreenId::ArcadeNextStage)?,
            Step::TakeStageBest => {
                a::take_arcade_stage_best(play, self.owners.system, self.owners.race, tables)?;
            }
            Step::RepairCar => crate::service::repair_parts::repair_car_in(menus)?,
            // The repair's result is not used: the race starts from 0.
            Step::Race => crate::race::single_rally_race::single_rally_race(self.owners, 0, 2)?,
            Step::ScoreStage => a::score_arcade_stage(menus)?,
            Step::ResultsLoop => self.screen(ScreenId::ArcadeResults)?,
            Step::KeepStageBest => {
                // The stage's record in the level's table.
                let stages = &self.owners.system.records().stage_tables;
                let level = stages.level_table(play.arcade.level.get())?;
                a::keep_arcade_stage_best(
                    menus,
                    stages,
                    level.row(stages, play.arcade.stage.get())?,
                )?;
            }
            Step::CheckStageTime => a::check_arcade_stage_time(menus, self.owners.system, tables)?,
            Step::ClearTimeMarks => a::clear_arcade_time_marks(play, self.owners.system, tables)?,
            Step::EnterStageTime => {
                a::enter_arcade_stage_time(menus, self.owners.system, self.owners.race, tables)?;
            }
            Step::BestTimesLoop => self.screen(ScreenId::ArcadeBestTimes)?,
            Step::CheckLevelScore => a::check_arcade_level_score(menus, tables)?,
            Step::ClearScoreMarks => a::clear_arcade_score_marks(play, tables)?,
            Step::EnterLevelScore => a::enter_arcade_level_score(menus, tables)?,
            Step::BestScoresLoop => self.screen(ScreenId::ArcadeBestScores)?,
            Step::LevelEnd => {
                m::arcade_level_end::arcade_level_end(menus, sound, &Entry::new(self.owners))?;
            }
            Step::NextLevelReview => self.screen(ScreenId::ArcadeNextLevelReview)?,
            Step::RetryReview => self.screen(ScreenId::ArcadeRetryReview)?,
            Step::StageReview => self.screen(ScreenId::ArcadeReviewOptions)?,
            Step::ShowInGameOptions => m::in_game_options_items::show_in_game_options(menus)?,
        }
        Ok(())
    }
}

impl Game<'_> {
    /// A screen, entered with the car list (the only value a screen reads from its caller).
    fn screen(&self, screen: ScreenId) -> Result<(), Refused> {
        run_screen(screen, &Entry::with_cars(self.owners, self.cars))?;
        Ok(())
    }
}

pub fn arcade_event(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    arcade_event_with(o, None)
}

/// The arcade event with a launch's answer to its first car choice.
pub(super) fn arcade_event_with<'a>(
    o: crate::owners::Owners<'static>,
    answer: Option<crate::menu::launch::CarAnswer<'a>>,
) -> Result<(), Refused> {
    let menus = o.menus;
    let player0 = crate::menu::standings::player(menus, 0);
    let mut game = Game {
        menus,
        owners: o,
        cars: None,
        player0,
        answer,
    };
    arcade::arcade_event(&mut game)
}
