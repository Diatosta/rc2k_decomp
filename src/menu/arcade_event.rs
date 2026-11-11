use crate::errors::Refused;
use crate::menu::event_flow::switch_cell;
use crate::menu::lists_store::CarList;
use crate::menu::owner::Menus;
use crate::menu::screen_loop::{Entry, Screen, run_screen};
use crate::race::owner::race;
use crate::session::DwordLike;
use crate::session::PlainDword;
use crate::system::owner::system;
use rc2k_core::menu::arcade_event::{self as arcade, Step, Var};
use rc2k_core::menu::leaving::Switch;

/// The game for `rc2k_core::menu::arcade_event`: the variables at their addresses, the
/// steps and the car list the car choice was shown.
struct Game<'a> {
    menus: &'a Menus,
    cars: Option<CarList>,
    player0: crate::menu::standings::PlayerCell<'a>,
}

/// The variables that are in the physics block.
fn cell(v: Var) -> Option<&'static dyn DwordLike> {
    let race = race();
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
        Var::TotalA8 => &menus.play.results.before[0],
        Var::Total798 => &menus.play.results.sums[0],
        Var::TotalB8 => &menus.play.results.totals[0],
        Var::Figure768 => &menus.play.results.positions[0],
        Var::Figure758 => &menus.play.results.time_bonus[0],
        Var::PlayerCount => &menus.screens.event.player_count,
        Var::EventMode => &menus.screens.event.event_kind,
        Var::Stage => &menus.play.arcade.stage,
        Var::Level => &menus.play.arcade.level,
        Var::Current => &menus.screens.event.current_player,
        Var::Answer => &menus.tables.pages.yes_no_answer,
        Var::Cheats => &menus.play.cheats.cheats,
        Var::Qualifies => &menus.screens.event.time_qualifies,
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
        match cell(v) {
            Some(cell) => cell.get(),
            None => address(menus, v).get(),
        }
    }

    fn set(&mut self, v: Var, value: u32) {
        let menus = self.menus;
        match v {
            Var::Player0Entry => self.player0.update(|p| p.entry = value),
            _ => match cell(v) {
                Some(cell) => cell.set(value),
                None => address(menus, v).set(value),
            },
        }
    }

    fn flag(&mut self, s: Switch) -> bool {
        switch_cell(s).is_set()
    }

    fn set_flag(&mut self, s: Switch, on: bool) {
        switch_cell(s).set(on);
    }

    fn step(&mut self, s: Step) -> Result<(), Refused> {
        let sound = crate::sound::owner::sound();
        let menus = self.menus;
        let play = &menus.play;
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
            Step::SelectCarLoop => self.screen(&m::select_car_loop::SCREEN)?,
            Step::TakeShownCar => m::take_shown_car::take_shown_car(menus, self.cars)?,
            Step::NextStageLoop => self.screen(&m::arcade_next_stage_loop::SCREEN)?,
            Step::TakeStageBest => a::take_arcade_stage_best(play)?,
            Step::RepairCar => crate::service::repair_parts::repair_car()?,
            // From 0 (the original clears eax for the repair just before).
            Step::Race => crate::race::single_rally_race::single_rally_race(0, 2)?,
            Step::ScoreStage => a::score_arcade_stage(menus)?,
            Step::ResultsLoop => self.screen(&m::arcade_results_loop::SCREEN)?,
            Step::KeepStageBest => {
                // The stage's record in the level's table.
                let level = system()
                    .records
                    .stage_tables
                    .level_table(play.arcade.level.get())?;
                a::keep_arcade_stage_best(menus, level.row(play.arcade.stage.get())?)?;
            }
            Step::CheckStageTime => a::check_arcade_stage_time(menus)?,
            Step::ClearTimeMarks => a::clear_arcade_time_marks(play)?,
            Step::EnterStageTime => a::enter_arcade_stage_time(menus)?,
            Step::BestTimesLoop => self.screen(&m::arcade_best_times_loop::SCREEN)?,
            Step::CheckLevelScore => a::check_arcade_level_score(menus)?,
            Step::ClearScoreMarks => a::clear_arcade_score_marks(play)?,
            Step::EnterLevelScore => a::enter_arcade_level_score(menus)?,
            Step::BestScoresLoop => self.screen(&m::arcade_best_scores_loop::SCREEN)?,
            Step::LevelEnd => m::arcade_level_end::arcade_level_end(menus, sound)?,
            Step::NextLevelReview => self.screen(&m::arcade_next_level_review_loop::SCREEN)?,
            Step::RetryReview => self.screen(&m::arcade_retry_review_loop::SCREEN)?,
            Step::StageReview => self.screen(&m::arcade_review_options_loop::SCREEN)?,
            Step::ShowInGameOptions => m::in_game_options_items::show_in_game_options(menus)?,
        }
        Ok(())
    }
}

impl Game<'_> {
    /// A screen, entered with the car list (the only value a screen reads from its caller).
    fn screen(&self, screen: &Screen) -> Result<(), Refused> {
        { run_screen(screen, &Entry::with_cars(self.menus, self.cars)) }?;
        Ok(())
    }
}

pub fn arcade_event(menus: &Menus) -> Result<(), Refused> {
    let player0 = crate::menu::standings::player(menus, 0);
    let mut game = Game {
        menus,
        cars: None,
        player0,
    };
    // The logic is in `rc2k_core::menu::arcade_event::arcade_event`.
    arcade::arcade_event(&mut game)
}
