use crate::errors::Refused;
use crate::menu::lists_store::CarList;
use crate::menu::screen_loop::{Entry, Screen, run_screen};
use crate::race::state_store::RACE_STATE;
use crate::session::Dword;
use rc2k_core::menu::arcade_event::{self as arcade, Step, Var};

/// The game for `rc2k_core::menu::arcade_event`: the variables at their addresses, the
/// steps' Rust versions and the car list the car choice was shown.
struct Game {
    cars: Option<CarList>,
    player0: crate::menu::standings::PlayerCell,
}

/// The variables that are in the physics block.
fn cell(v: Var) -> Option<&'static Dword> {
    Some(match v {
        Var::RaceStage => &RACE_STATE.results.race_stage,
        Var::RaceLevel => &RACE_STATE.results.race_level,
        Var::CheatSwitch => &RACE_STATE.settings.direction,
        Var::RaceResult => &RACE_STATE.results.results[0].result,
        Var::RaceFigureFFC => &RACE_STATE.results.figure_ffc,
        Var::RaceFigure000 => &RACE_STATE.results.figure_000,
        _ => return None,
    })
}

fn address(v: Var) -> &'static Dword {
    match v {
        Var::Failed => &crate::menu::owner::menus().play.arcade.retry,
        Var::TotalA8 => &crate::menu::owner::menus().play.results.before[0],
        Var::Total798 => &crate::menu::owner::menus().play.results.sums[0],
        Var::TotalB8 => &crate::menu::owner::menus().play.results.totals[0],
        Var::Figure768 => &crate::menu::owner::menus().play.results.positions[0],
        Var::Figure758 => &crate::menu::owner::menus().play.results.time_bonus[0],
        Var::PlayerCount => &crate::menu::owner::menus().screens.event.player_count,
        Var::EventMode => &crate::menu::owner::menus().screens.event.event_kind,
        Var::Leave => &crate::menu::owner::menus().screens.loop_flags.went_back,
        Var::Ended565 => &crate::menu::owner::menus().screens.loop_flags.ended_b,
        Var::Ended569 => &crate::menu::owner::menus().screens.loop_flags.ended_a,
        Var::Stage => &crate::menu::owner::menus().play.arcade.stage,
        Var::Level => &crate::menu::owner::menus().play.arcade.level,
        Var::Current => &crate::menu::owner::menus().screens.event.current_player,
        Var::CarLeft => &crate::menu::owner::menus().tables.captions.screen_left,
        Var::Answer => &crate::menu::owner::menus().tables.pages.yes_no_answer,
        Var::Cheats => &crate::menu::owner::menus().play.cheats.cheats,
        Var::Qualifies => &crate::menu::owner::menus().screens.event.time_qualifies,
        Var::Done => &crate::menu::owner::menus().screens.event.event_done,
        Var::Player0Entry => unreachable!("player 0's field is not at an address"),
        Var::RaceStage
        | Var::RaceLevel
        | Var::CheatSwitch
        | Var::RaceResult
        | Var::RaceFigureFFC
        | Var::RaceFigure000 => unreachable!("a variable of the physics block, `cell`"),
    }
}

impl arcade::Arcade for Game {
    type Error = Refused;

    fn get(&mut self, v: Var) -> u32 {
        match cell(v) {
            Some(cell) => cell.get(),
            None => address(v).get(),
        }
    }

    fn set(&mut self, v: Var, value: u32) {
        match v {
            Var::Player0Entry => self.player0.update(|p| p.entry = value),
            _ => match cell(v) {
                Some(cell) => cell.set(value),
                None => address(v).set(value),
            },
        }
    }

    fn step(&mut self, s: Step) -> Result<(), Refused> {
        use crate::menu as m;
        use crate::menu::arcade_steps as a;
        match s {
            Step::ClearPlayers284 => m::clear_players_284::clear_players_284()?,
            Step::ResetCars750724 => m::reset_cars_750724::reset_cars_750724()?,
            Step::SecretCarList => {
                self.cars = Some(m::cheat_car_list::cheat_car_list().unwrap_or(CarList::Second));
            }
            Step::SelectCarLoop => self.screen(&m::select_car_loop::SCREEN)?,
            Step::TakeShownCar => m::take_shown_car::take_shown_car(self.cars)?,
            Step::NextStageLoop => self.screen(&m::arcade_next_stage_loop::SCREEN)?,
            Step::TakeStageBest => a::take_arcade_stage_best()?,
            Step::RepairCar => crate::service::repair_parts::repair_car()?,
            // From 0 (the original clears eax for the repair just before).
            Step::Race => crate::race::single_rally_race::single_rally_race(0, 2)?,
            Step::ScoreStage => a::score_arcade_stage()?,
            Step::ResultsLoop => self.screen(&m::arcade_results_loop::SCREEN)?,
            Step::KeepStageBest => {
                // The stage's record in the level's table.
                let level = crate::records::stages_store::STAGE_TABLES
                    .level_table(crate::menu::owner::menus().play.arcade.level.get())?;
                a::keep_arcade_stage_best(
                    level.row(crate::menu::owner::menus().play.arcade.stage.get())?,
                )?;
            }
            Step::CheckStageTime => a::check_arcade_stage_time()?,
            Step::ClearTimeMarks => a::clear_arcade_time_marks()?,
            Step::EnterStageTime => a::enter_arcade_stage_time()?,
            Step::BestTimesLoop => self.screen(&m::arcade_best_times_loop::SCREEN)?,
            Step::CheckLevelScore => a::check_arcade_level_score()?,
            Step::ClearScoreMarks => a::clear_arcade_score_marks()?,
            Step::EnterLevelScore => a::enter_arcade_level_score()?,
            Step::BestScoresLoop => self.screen(&m::best_scores_48a69e_loop::SCREEN)?,
            Step::LevelEnd => m::arcade_level_end::arcade_level_end()?,
            Step::NextLevelReview => self.screen(&m::review_options_4882e2_loop::SCREEN)?,
            Step::RetryReview => self.screen(&m::review_options_48861d_loop::SCREEN)?,
            Step::StageReview => self.screen(&m::arcade_review_options_loop::SCREEN)?,
            Step::ShowInGameOptions => m::in_game_options_items::show_in_game_options()?,
        }
        Ok(())
    }
}

impl Game {
    /// A screen, entered with the car list (the only value a screen reads from its caller).
    fn screen(&self, screen: &Screen) -> Result<(), Refused> {
        { run_screen(screen, &Entry { cars: self.cars }) }?;
        Ok(())
    }
}

/// The Rust version.
pub fn arcade_event() -> Result<(), Refused> {
    let player0 = crate::menu::standings::player(0);
    let mut game = Game {
        cars: None,
        player0,
    };
    // Converted: `rc2k_core::menu::arcade_event::arcade_event`.
    arcade::arcade_event(&mut game)
}
