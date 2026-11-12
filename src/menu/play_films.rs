//! `menu::play_films` (0x4161CD): the films: the two logos at the start, then the intro,
//! over and over with the game's own demonstration between, until a key.
//!
//! - `sound::stop_music`, and a pending key is taken and thrown away.
//! - At the start only: nothing at all (straight to the end) with the setting `nofmv`;
//!   else `allogo.avi` if the screen has 16 bits (its extension swapped by
//!   `files::swap_missing_extension` if the file is missing), then `mflogo.mpg`, each
//!   through the player 0x41AE39 with ebx = 0; then `gfx::clear_front_surface`.
//! - `scene::set_up_lit_engine`; from the idle menu also `race::render3d_tidy`, `nullsub_6`.
//! - Then round and round: without `nofmv`, and if `intro.mpg` is there, the intro
//!   through the player, which ends the round if it comes back with the zero flag
//!   clear; else `race::credits_race` (ebp kept across it), and a key ends the
//!   round.
//! - `sound::let_sounds_play`, `gfx::clear_front_surface`; from the idle menu also
//!   `scene::set_up_engine`.
//!
//! What the unnamed ones do has not been read. Callers: `menu::run_front_end` (with 0),
//! `menu::main_menu_loop`'s idle step (with 1), and 0x470593. Rust callers run its `entry`
//! (`routes.rs`).

use crate::errors::Refused;
use crate::framework::log;
use crate::menu::owner::Panels;
use crate::owners::Owners;
use rc2k_core::menu::films::{self, Film, Step, StepAnswer};

// ---- game data ----

/// IDA's address of the films' names.
pub const FILMS_START: usize = 0x5297F5;
/// Their length.
pub const FILMS_LEN: usize = 0x52983C - FILMS_START;

/// The films' file names.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Films {
    /// `a3filesFmvIntro` (`0x5297F5`).
    pub intro: crate::session::PlainBytes<22>,
    /// `a3filesFmvMflog` (`0x52980B`).
    pub mflogo: crate::session::PlainBytes<23>,
    /// `a3filesFmvAllog` (`0x529822`).
    pub allogo: crate::session::PlainBytes<23>,
    /// `0x529839` .. `0x52983C`: nothing reads or writes them (a gap).
    pub _gap_44: crate::session::PlainBytes<3>,
}

/// The films' names (the leading `;3` is the game's own marker for the file functions).
impl Films {
    pub const fn new() -> Self {
        Films {
            intro: crate::session::PlainBytes::new(*b";3files\\fmv\\intro.mpg\0"),
            mflogo: crate::session::PlainBytes::new(*b";3files\\fmv\\mflogo.mpg\0"),
            allogo: crate::session::PlainBytes::new(*b";3files\\fmv\\allogo.avi\0"),
            _gap_44: crate::session::PlainBytes::new([0; 3]),
        }
    }
}

/// The game for `rc2k_core::menu::films`: the settings and the steps.
/// A step that refuses ends the films: its error is kept (`failed`), every step after it
/// does nothing and answers "a key was pressed", and `play_films` returns the error.
struct Game<'a> {
    failed: Option<Refused>,
    owners: Owners<'a>,
}

fn film(panels: &Panels, f: Film) -> &dyn crate::session::TextPlace {
    match f {
        Film::AlLogo => &panels.films.allogo,
        Film::MfLogo => &panels.films.mflogo,
        Film::Intro => &panels.films.intro,
    }
}

/// The film player (`gfx::play_film`, the platform's) for the file at `path`: the zero
/// flag it leaves (the film's end).
fn play(path: &dyn crate::session::TextPlace) -> bool {
    crate::film::play_film_now(path)
}

impl Game<'_> {
    fn run(&mut self, s: Step) -> Result<StepAnswer, Refused> {
        let o = self.owners;
        let (scene, race) = (o.scene, o.race);
        let panels = &o.menus.panels;
        let mut flags = StepAnswer::default();
        match s {
            Step::StopMusic => crate::sound::stop_music::stop_music(),
            Step::Step4059F4 => crate::gfx::clear_front_surface::clear_front_surface()?,
            Step::TakeKey => {
                flags.zero = crate::input::take_key::take_key() == 0;
            }
            Step::SwapMissingExtension(f) => {
                crate::files::swap_missing_extension::swap_missing_extension(film(panels, f))?;
            }
            Step::Player(f) => {
                flags.zero = play(film(panels, f.unwrap_or(Film::AlLogo)));
            }
            Step::LitEngine => {
                crate::scene::set_up_lit_engine::set_up_lit_engine(race, scene)?;
            }
            Step::Render3dTidy => {
                crate::race::stage_free::render3d_tidy(race)?;
            }
            Step::ObjectsTidy => crate::race::stage_free::objects_tidy(race)?,
            // `nullsub_6`.
            Step::Nothing => {}
            Step::ReleaseTextures => crate::race::race_end::release_textures(),
            Step::FileExists(f) => {
                let there = crate::files::file_exists::file_exists(
                    &film(panels, f).text("a film's path")?,
                )?;
                flags.carry = !there;
                flags.zero = there;
            }
            // Fixed (`wiki/Fixed-bugs.md`): the players' absences cleared
            // first, as the original's other caller of the demonstration (0x443104)
            // does; the original's films kept the quit answer of the last race left by
            // the quit question, and its demonstration ended at its first frame.
            Step::Demonstration => {
                crate::race::race_shell::clear_player_absences();
                crate::race::credits_race::credits_race(race)?;
            }
            Step::Step409B76 => crate::sound::let_sounds_play::let_sounds_play(),
            Step::SetUpEngine => crate::scene::set_up_engine::set_up_engine_in(race, scene)?,
            Step::FrontEndScene => crate::scene::set_up_front_end_scene::set_up_front_end_scene()?,
            Step::Tune3 => crate::sound::play_tune_3::play_tune_3(),
        }

        Ok(flags)
    }
}

impl films::Films for Game<'_> {
    fn no_fmv(&mut self) -> bool {
        crate::film::NO_FMV.get() != 0
    }

    fn screen_bits(&mut self) -> u32 {
        self.owners.system.session.screen.bits.get()
    }

    fn step(&mut self, s: Step) -> StepAnswer {
        if self.failed.is_some() {
            // "A key was pressed", which ends the loop.
            return StepAnswer {
                carry: true,
                zero: false,
            };
        }
        match self.run(s) {
            Ok(flags) => flags,
            Err(e) => {
                self.failed = Some(e);
                StepAnswer {
                    carry: true,
                    zero: false,
                }
            }
        }
    }
}

/// `idle`: from the main menu when it has stood idle, else the game's start.
pub fn play_films(o: Owners<'_>, idle: bool) -> Result<(), Refused> {
    log(&format!(
        "menu::play_films: runs (mode {})",
        u32::from(idle)
    ));
    let mut game = Game {
        failed: None,
        owners: o,
    };
    // The logic is in `rc2k_core::menu::films::play_films`.
    films::play_films(&mut game, idle);
    log("menu::play_films: done");
    match game.failed {
        Some(e) => Err(e),
        None => Ok(()),
    }
}

#[cfg(test)]
mod films_tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        let panels = &crate::menu::owner::menus().panels;
        crate::gfx::first_values_are_the_games(&panels.films, FILMS_START, FILMS_START + FILMS_LEN);
        assert_eq!(
            crate::moved::place(0x52980B),
            panels.films.mflogo.addr() as usize
        );
        assert_eq!(
            crate::moved::place(0x529822),
            panels.films.allogo.addr() as usize
        );
    }
}
