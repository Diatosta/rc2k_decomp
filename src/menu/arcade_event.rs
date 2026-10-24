use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::menu::event_flow::Flow;
use crate::menu::player::player_at;
use crate::menu::screen_loop::{Carried, run_screen_carried};
use rc2k_core::menu::arcade_event::{self as arcade, Hand, Step, Var};

fn g(a: usize) -> u32 {
    unsafe { game_read(rva(a)) }
}
fn p(a: usize, v: u32) {
    unsafe { game_write(rva(a), v) }
}
fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The game for `rc2k_core::menu::arcade_event`: the variables at their addresses, the
/// steps' Rust versions with the registers the original gives them (one set carried
/// through).
struct Game {
    f: Flow,
    player0: u32,
}

fn address(v: Var) -> usize {
    match v {
        Var::Failed => 0x7587F4,
        Var::TotalA8 => 0x7587A8,
        Var::Total798 => 0x758798,
        Var::TotalB8 => 0x7587B8,
        Var::Figure768 => 0x758768,
        Var::Figure758 => 0x758758,
        Var::PlayerCount => 0x755984,
        Var::EventMode => 0x755988,
        Var::Leave => 0x75555D,
        Var::Ended565 => 0x755565,
        Var::Ended569 => 0x755569,
        Var::Stage => 0x7587F0,
        Var::Level => 0x7587EC,
        Var::Current => 0x75598C,
        Var::CarLeft => 0x758806,
        Var::Answer => 0x75524C,
        Var::RaceStage => 0x75AFF4,
        Var::RaceLevel => 0x75AFF8,
        Var::CheatSwitch => 0x75ACF0,
        Var::Cheats => 0x75870C,
        Var::RaceResult => 0x75B028,
        Var::RaceFigureFFC => 0x75AFFC,
        Var::RaceFigure000 => 0x75B000,
        Var::Qualifies => 0x755918,
        Var::Done => 0x755980,
        Var::Player0Entry => unreachable!("player 0's field is not at an address"),
    }
}

impl arcade::Arcade for Game {
    type Error = Refused;

    fn get(&mut self, v: Var) -> u32 {
        g(address(v))
    }

    fn set(&mut self, v: Var, value: u32) {
        match v {
            Var::Player0Entry => {
                self.f.record = self.player0;
                put(self.player0 + 0xD0, value);
            }
            _ => p(address(v), value),
        }
    }

    fn step(&mut self, s: Step) -> Result<(), Refused> {
        use crate::menu as m;
        use crate::menu::arcade_steps as a;
        let c = &mut self.f.c;
        unsafe {
            match s {
                Step::ClearPlayers284 => m::clear_players_284::clear_players_284()?,
                Step::ResetCars750724 => m::reset_cars_750724::reset_cars_750724()?,
                Step::SecretCarList => {
                    c.cars = game_addr(rva(0x75A042)) as u32;
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
                Step::NextLevelReview => {
                    run_screen_carried(&m::review_options_4882e2_loop::SCREEN, c)?
                }
                Step::RetryReview => run_screen_carried(&m::review_options_48861d_loop::SCREEN, c)?,
                Step::StageReview => run_screen_carried(&m::arcade_review_options_loop::SCREEN, c)?,
                // With the registers the event was entered with: it reads none.
                Step::ShowInGameOptions => m::in_game_options_items::show_in_game_options(),
            }
        }
        Ok(())
    }

    fn hand(&mut self, h: Hand) -> Result<(), Refused> {
        let f = &mut self.f;
        match h {
            Hand::Kept(v) => f.c.arg = v,
            Hand::StageRecord => {
                f.c.palette = g(0x7587EC);
                f.record = g(0x73555A + 4 * f.c.palette as usize);
                f.c.arg = g(0x7587F0).wrapping_mul(0xA4);
                f.record = f.record.wrapping_add(f.c.arg);
            }
        }
        Ok(())
    }
}

/// The Rust version. `c` the carried values the event is entered with.
pub unsafe fn arcade_event(c: &Carried) -> Result<(), Refused> {
    let player0 = player_at(0);
    if player0 == 0 {
        return Err(BadInput("no player 0").into());
    }
    let mut game = Game {
        f: Flow::new(c),
        player0,
    };
    // Converted: `rc2k_core::menu::arcade_event::arcade_event`.
    arcade::arcade_event(&mut game)
}
