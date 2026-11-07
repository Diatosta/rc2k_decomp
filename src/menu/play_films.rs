//! `menu::play_films` (0x4161CD): the films: the two logos at the start, then the intro,
//! over and over with the game's own demonstration between, until a key.
//!
//! In:  eax = 0 at the game's start, 1 from the main menu when it has stood idle
//! Out: every register preserved
//! - `sub_40F826`, `sub_4059F4`, and a pending key is taken and thrown away.
//! - At the start only: nothing at all (straight to the end) with the setting `nofmv`;
//!   else `allogo.avi` if the screen has 16 bits (its extension swapped by
//!   `files::swap_missing_extension` if the file is missing), then `mflogo.mpg`, each
//!   through the player `sub_41AE39` with ebx = 0; then `sub_4059F4`.
//! - `sub_415515`; from the idle menu also `sub_4433A7`, `sub_44346B`, `nullsub_6`,
//!   `sub_443396`.
//! - Then round and round: without `nofmv`, and if `intro.mpg` is there, the intro
//!   through the player, which ends the round if it comes back with the zero flag
//!   clear; else `sub_463666` (ebp kept across it), `sub_40F826`, and a key ends the
//!   round.
//! - `sub_409B76`, `sub_4059F4`; from the idle menu also `sub_442AF6`, `sub_474E04`,
//!   `sub_442AF6`, `sub_40F780`.
//!
//! What the unnamed ones do has not been read. Callers: `menu::run_front_end` (with 0),
//! `menu::main_menu_loop`'s idle step (with 1), and 0x470593. Rust callers run its `entry`
//! (`routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::log;
use crate::menu::owner::Panels;
use crate::menu::owner::menus;
use crate::system::owner::system;
use rc2k_core::menu::films::{self, Film, Flags, Step};

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
    pub intro: crate::session::Bytes<22>,
    /// `a3filesFmvMflog` (`0x52980B`).
    pub mflogo: crate::session::Bytes<23>,
    /// `a3filesFmvAllog` (`0x529822`).
    pub allogo: crate::session::Bytes<23>,
    /// `0x529839` .. `0x52983C`: nothing reads or writes them (a gap).
    pub _gap_44: crate::session::Bytes<3>,
}

/// The films' names (the leading `;3` is the game's own marker for the file functions).
impl Films {
    pub const fn new() -> Self {
        Films {
            intro: crate::session::Bytes::new(*b";3files\\fmv\\intro.mpg\0"),
            mflogo: crate::session::Bytes::new(*b";3files\\fmv\\mflogo.mpg\0"),
            allogo: crate::session::Bytes::new(*b";3files\\fmv\\allogo.avi\0"),
            _gap_44: crate::session::Bytes::new([0; 3]),
        }
    }
}

// ---- Rust version ----

/// The game for `rc2k_core::menu::films`: the settings, the steps their Rust versions.
/// A step that refuses ends the films: its error is kept (`failed`), every step after it
/// does nothing and answers "a key was pressed", and `play_films` returns the error.
struct Game {
    failed: Option<Refused>,
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

impl Game {
    fn run(&mut self, s: Step) -> Result<Flags, Refused> {
        let scene = crate::scene::owner::scene();
        let panels = &menus().panels;
        let mut flags = Flags::default();
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
                crate::scene::set_up_lit_engine::set_up_lit_engine(
                    crate::race::owner::race(),
                    scene,
                )?;
            }
            Step::Render3dTidy => {
                crate::race::stage_free::render3d_tidy(crate::race::owner::race())?;
            }
            Step::ObjectsTidy => crate::race::stage_free::objects_tidy(crate::race::owner::race())?,
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
            // Fixed (2026-10-24, `wiki/Fixed-bugs.md`): the players' absences cleared
            // first, as the original's other caller of the demonstration (`sub_443104`)
            // does; the original's films kept the quit answer of the last race left by
            // the quit question, and its demonstration ended at its first frame.
            Step::Demonstration => {
                crate::race::race_shell::clear_player_absences();
                crate::race::credits_race::credits_race(crate::race::owner::race())?;
            }
            Step::Step409B76 => crate::sound::let_sounds_play::let_sounds_play(),
            Step::SetUpEngine => crate::scene::set_up_engine::set_up_engine()?,
            Step::FrontEndScene => crate::scene::set_up_front_end_scene::set_up_front_end_scene()?,
            Step::Tune3 => crate::sound::play_tune_3::play_tune_3(),
        }

        Ok(flags)
    }
}

impl films::Films for Game {
    fn no_fmv(&mut self) -> bool {
        crate::film::NO_FMV.get() != 0
    }

    fn screen_bits(&mut self) -> u32 {
        system().session.screen.bits.get()
    }

    fn step(&mut self, s: Step) -> Flags {
        if self.failed.is_some() {
            // "A key was pressed", which ends the loop.
            return Flags {
                carry: true,
                zero: false,
            };
        }
        match self.run(s) {
            Ok(flags) => flags,
            Err(e) => {
                self.failed = Some(e);
                Flags {
                    carry: true,
                    zero: false,
                }
            }
        }
    }
}

/// The Rust version (`idle`: from the main menu when it has stood idle, else the game's
/// start). The original keeps every register of its caller.
pub fn play_films(idle: bool) -> Result<(), Refused> {
    log(&format!(
        "menu::play_films: the Rust version runs (mode {})",
        u32::from(idle)
    ));
    let mut game = Game { failed: None };
    // Converted: `rc2k_core::menu::films::play_films`.
    films::play_films(&mut game, idle);
    log("menu::play_films: the Rust version has come back");
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
        let panels = &menus().panels;
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
