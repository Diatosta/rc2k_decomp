use crate::errors::{BadInput, Refused};
use crate::menu::arcade_store::ARCADE_GAME;
use crate::menu::captions_store::CAPTIONS;
use crate::menu::cheats_store::CHEAT_VARS;
use crate::menu::event_flow::{Fault, Flow};
use crate::menu::event_store::EVENT_STATE;
use crate::menu::lists_store::MENU_LISTS;
use crate::menu::loop_store::LOOP_FLAGS;
use crate::menu::pages_store::MENU_PAGES;
use crate::menu::player::player_at;
use crate::menu::results_store::RESULT_FIGURES;
use crate::menu::screen_loop::{Carried, run_screen_carried};
use crate::race::state_store::RACE_STATE;
use crate::session::Dword;
use rc2k_core::menu::arcade_event::{self as arcade, Hand, Step, Var};

/// The game for `rc2k_core::menu::arcade_event`: the variables at their addresses, the
/// steps' Rust versions with the registers the original gives them (one set carried
/// through).
struct Game {
    f: Flow,
    player0: u32,
    /// A refused read or write of the core's variables (checked at each step and at the end).
    fault: Fault,
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
        Var::Failed => &ARCADE_GAME.retry,
        Var::TotalA8 => &RESULT_FIGURES.before[0],
        Var::Total798 => &RESULT_FIGURES.sums[0],
        Var::TotalB8 => &RESULT_FIGURES.totals[0],
        Var::Figure768 => &RESULT_FIGURES.positions[0],
        Var::Figure758 => &RESULT_FIGURES.time_bonus[0],
        Var::PlayerCount => &EVENT_STATE.player_count,
        Var::EventMode => &EVENT_STATE.event_kind,
        Var::Leave => &LOOP_FLAGS.went_back,
        Var::Ended565 => &LOOP_FLAGS.ended_b,
        Var::Ended569 => &LOOP_FLAGS.ended_a,
        Var::Stage => &ARCADE_GAME.stage,
        Var::Level => &ARCADE_GAME.level,
        Var::Current => &EVENT_STATE.current_player,
        Var::CarLeft => &CAPTIONS.screen_left,
        Var::Answer => &MENU_PAGES.yes_no_answer,
        Var::Cheats => &CHEAT_VARS.cheats,
        Var::Qualifies => &EVENT_STATE.time_qualifies,
        Var::Done => &EVENT_STATE.event_done,
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
            Var::Player0Entry => {
                self.f.record = self.player0;
                self.fault.put(self.player0 + 0xD0, value);
            }
            _ => match cell(v) {
                Some(cell) => cell.set(value),
                None => address(v).set(value),
            },
        }
    }

    fn step(&mut self, s: Step) -> Result<(), Refused> {
        self.fault.check()?;
        use crate::menu as m;
        use crate::menu::arcade_steps as a;
        let c = &mut self.f.c;
        match s {
            Step::ClearPlayers284 => m::clear_players_284::clear_players_284()?,
            Step::ResetCars750724 => m::reset_cars_750724::reset_cars_750724()?,
            Step::SecretCarList => {
                c.cars = MENU_LISTS.cars_second[0].addr();
                m::cheat_car_list::carry_on(m::cheat_car_list::cheat_car_list(), c);
            }
            Step::SelectCarLoop => run_screen_carried(&m::select_car_loop::SCREEN, c)?,
            Step::TakeShownCar => m::take_shown_car::take_shown_car(c.cars)?,
            Step::NextStageLoop => run_screen_carried(&m::arcade_next_stage_loop::SCREEN, c)?,
            Step::TakeStageBest => a::take_arcade_stage_best()?,
            Step::RepairCar => {
                c.arg = 0;
                crate::service::repair_parts::repair_car()?
            }
            Step::Race => {
                c.palette = 2;
                crate::race::single_rally_race::single_rally_race(c.arg, 2)?
            }
            Step::ScoreStage => a::score_arcade_stage()?,
            Step::ResultsLoop => run_screen_carried(&m::arcade_results_loop::SCREEN, c)?,
            Step::KeepStageBest => a::keep_arcade_stage_best(self.f.record)?,
            Step::CheckStageTime => a::check_arcade_stage_time()?,
            Step::ClearTimeMarks => a::clear_arcade_time_marks()?,
            Step::EnterStageTime => a::enter_arcade_stage_time()?,
            Step::BestTimesLoop => run_screen_carried(&m::arcade_best_times_loop::SCREEN, c)?,
            Step::CheckLevelScore => a::check_arcade_level_score()?,
            Step::ClearScoreMarks => a::clear_arcade_score_marks()?,
            Step::EnterLevelScore => a::enter_arcade_level_score()?,
            Step::BestScoresLoop => run_screen_carried(&m::best_scores_48a69e_loop::SCREEN, c)?,
            Step::LevelEnd => m::arcade_level_end::arcade_level_end(c)?,
            Step::NextLevelReview => run_screen_carried(&m::review_options_4882e2_loop::SCREEN, c)?,
            Step::RetryReview => run_screen_carried(&m::review_options_48861d_loop::SCREEN, c)?,
            Step::StageReview => run_screen_carried(&m::arcade_review_options_loop::SCREEN, c)?,
            // With the registers the event was entered with: it reads none.
            Step::ShowInGameOptions => m::in_game_options_items::show_in_game_options()?,
        }

        Ok(())
    }

    fn hand(&mut self, h: Hand) -> Result<(), Refused> {
        let f = &mut self.f;
        match h {
            Hand::Kept(v) => f.c.arg = v,
            Hand::StageRecord => {
                f.c.palette = ARCADE_GAME.level.get();
                f.record = crate::records::stages_store::STAGE_TABLES.level_table(f.c.palette)?;
                f.c.arg = ARCADE_GAME.stage.get().wrapping_mul(0xA4);
                f.record = f.record.wrapping_add(f.c.arg);
            }
        }
        Ok(())
    }
}

/// The Rust version. `c` the carried values the event is entered with.
pub fn arcade_event(c: &Carried) -> Result<(), Refused> {
    let player0 = player_at(0);
    if player0 == 0 {
        return Err(BadInput("no player 0").into());
    }
    let mut game = Game {
        f: Flow::new(c),
        player0,
        fault: Fault::default(),
    };
    // Converted: `rc2k_core::menu::arcade_event::arcade_event`.
    let done = arcade::arcade_event(&mut game);
    game.fault.check()?;
    done
}
