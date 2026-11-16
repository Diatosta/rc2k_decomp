//! `menu::play_films`: the two logos at the start, then the intro, over and over with the
//! game's own demonstration between, until a key.
//! - `sound::stop_music`; a pending key is taken and thrown away.
//! - At the start only: nothing at all with the setting `nofmv`; else `allogo.avi`
//!   (the extension swapped by `files::swap_missing_extension` if missing), then
//!   `mflogo.mpg`, each through the film player; then `gfx::clear_front_surface`.
//! - `scene::set_up_lit_engine`; from the idle menu also `race::render3d_tidy`.
//! - Round and round: without `nofmv` and if `intro.mpg` is there, the intro, which ends the
//!   round if the player reports it was stopped; else `race::credits_race`, and a key ends it.
//! - `sound::let_sounds_play`, `gfx::clear_front_surface`; from the idle menu also
//!   `scene::set_up_engine`.

use crate::errors::Refused;
use crate::logfile::program_log::log;
use crate::menu::owner::Panels;
use crate::owners::Owners;
use rc2k_core::menu::films::{self, Film, Step, StepAnswer};

/// The films' file names.
#[derive(rc2k_derive::Cells)]
pub struct Films {
    /// The intro film's path.
    pub intro: crate::memory::session::PlainBytes<22>,
    /// The `mflogo` film's path.
    pub mflogo: crate::memory::session::PlainBytes<23>,
    /// The `allogo` film's path.
    pub allogo: crate::memory::session::PlainBytes<23>,
}

/// The films' names (the leading `;3` is the game's own marker for the file functions).
impl Films {
    pub const fn new() -> Self {
        Films {
            intro: crate::memory::session::PlainBytes::new(*b";3files\\fmv\\intro.mpg\0"),
            mflogo: crate::memory::session::PlainBytes::new(*b";3files\\fmv\\mflogo.mpg\0"),
            allogo: crate::memory::session::PlainBytes::new(*b";3files\\fmv\\allogo.avi\0"),
        }
    }
}

/// The game for `rc2k_core::menu::films`: the settings and the steps.
/// A step that refuses ends the films: its error is kept (`failed`), every step after it
/// does nothing and answers "a key was pressed", and `play_films` returns the error.
struct Game {
    failed: Option<Refused>,
    owners: Owners<'static>,
}

fn film(panels: &Panels, f: Film) -> &dyn crate::memory::session::TextPlace {
    match f {
        Film::AlLogo => &panels.films.allogo,
        Film::MfLogo => &panels.films.mflogo,
        Film::Intro => &panels.films.intro,
    }
}

/// The film player (the platform's) for the file at `path`: whether the film ran to its end
/// (false: a key ended it).
fn play(o: Owners<'static>, path: &dyn crate::memory::session::TextPlace) -> bool {
    crate::platform::film::play_film_now(o, path)
}

impl Game {
    fn run(&mut self, s: Step) -> Result<StepAnswer, Refused> {
        let o = self.owners;
        let (scene, race) = (o.scene, o.race);
        let panels = &o.menus.panels;
        let mut answer = StepAnswer::default();
        match s {
            Step::StopMusic => crate::sound::stop_music::stop_music(o),
            Step::ClearFrontSurface => {
                crate::gfx::clear_front_surface::clear_front_surface(o.gfx, o.system)?;
            }
            Step::TakeKey => {
                answer.key_pressed = crate::input::take_key::take_key(o.input) != 0;
            }
            Step::SwapMissingExtension(f) => {
                crate::files::swap_missing_extension::swap_missing_extension(film(panels, f))?;
            }
            Step::Player(f) => {
                answer.key_pressed = !play(o, film(panels, f));
            }
            Step::LitEngine => {
                crate::scene::set_up_lit_engine::set_up_lit_engine(race, scene)?;
            }
            Step::Render3dTidy => {
                crate::race::stage_free::render3d_tidy(race, &scene.scratch)?;
            }
            Step::ObjectsTidy => crate::race::stage_free::objects_tidy(race, &scene.scratch)?,
            Step::ReleaseTextures => crate::race::race_end::release_textures(scene, o.gfx),
            Step::FileExists(f) => {
                let there = crate::files::file_exists::file_exists(
                    &film(panels, f).text("a film's path")?,
                )?;
                answer.film_missing = !there;
            }
            // Fixed (`wiki/Fixed-bugs.md`): the players' absences are cleared first, as the
            // other caller of the demonstration does; the films kept the quit answer of the last
            // race left by the quit question, and the demonstration ended at its first frame.
            Step::Demonstration => {
                crate::race::race_shell::clear_player_absences(race);
                crate::race::credits_race::credits_race(race, o)?;
            }
            Step::LetSoundsPlay => crate::sound::let_sounds_play::let_sounds_play_in(o.audio),
            Step::SetUpEngine => crate::scene::set_up_engine::set_up_engine_in(race, scene)?,
            Step::FrontEndScene => {
                crate::scene::set_up_front_end_scene::set_up_front_end_scene_in(o)?;
            }
            Step::Tune3 => crate::sound::play_tune_3::play_tune_3(o),
        }

        Ok(answer)
    }
}

impl films::Films for Game {
    fn no_fmv(&mut self) -> bool {
        self.owners.system.machine().no_fmv.get() != 0
    }

    fn step(&mut self, s: Step) -> StepAnswer {
        if self.failed.is_some() {
            // "A key was pressed", which ends the loop.
            return StepAnswer {
                film_missing: true,
                key_pressed: true,
            };
        }
        match self.run(s) {
            Ok(answer) => answer,
            Err(e) => {
                self.failed = Some(e);
                StepAnswer {
                    film_missing: true,
                    key_pressed: true,
                }
            }
        }
    }
}

/// `idle`: from the main menu when it has stood idle, else the game's start.
pub fn play_films(o: Owners<'static>, idle: bool) -> Result<(), Refused> {
    log(&format!(
        "menu::play_films: runs (mode {})",
        u32::from(idle)
    ));
    let mut game = Game {
        failed: None,
        owners: o,
    };
    films::play_films(&mut game, idle);
    log("menu::play_films: done");
    match game.failed {
        Some(e) => Err(e),
        None => Ok(()),
    }
}
