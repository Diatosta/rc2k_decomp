//! The films: at the game's start the two logos (the first on 16-bit screens only), then the intro
//! over and over with the game's demonstration between, until a key; from the idle main menu the
//! scene let go first and set up again after. The setting `nofmv` leaves the films out.

/// The films.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Film {
    /// `;3files\fmv\allogo.avi`, `mflogo.mpg`, `intro.mpg`.
    AlLogo,
    MfLogo,
    Intro,
}

/// The steps of the film sequence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// Stops the music (`sound::stop_music`).
    StopMusic,
    /// `gfx::clear_front_surface`.
    ClearFrontSurface,
    /// `input::take_key`: `key_pressed` when a key was taken.
    TakeKey,
    /// `files::swap_missing_extension` on the film.
    SwapMissingExtension(Film),
    /// The player on the film; `key_pressed` when a key ended it.
    Player(Film),
    /// `scene::set_up_lit_engine`.
    LitEngine,
    /// `race::render3d_tidy`, `race::objects_tidy` and `race::release_textures`.
    Render3dTidy,
    ObjectsTidy,
    ReleaseTextures,
    /// `files::file_exists`: `film_missing` when the film is not there.
    FileExists(Film),
    /// `race::credits_race`, the demonstration.
    Demonstration,
    /// `sound::let_sounds_play`.
    LetSoundsPlay,
    /// `scene::set_up_engine`, `scene::set_up_front_end_scene`, `sound::play_tune_3`.
    SetUpEngine,
    FrontEndScene,
    Tune3,
}

/// What a step answered, for the steps that answer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StepAnswer {
    /// The film asked about is not there.
    pub film_missing: bool,
    /// A key was taken, or ended the film.
    pub key_pressed: bool,
}

pub trait Films {
    /// The setting `nofmv` is on.
    fn no_fmv(&mut self) -> bool;
    fn step(&mut self, s: Step) -> StepAnswer;
}

/// `menu::play_films`: `idle` from the idle main menu, else at the start (0).
pub fn play_films<F: Films>(e: &mut F, idle: bool) {
    e.step(Step::StopMusic);
    e.step(Step::ClearFrontSurface);
    e.step(Step::TakeKey);
    if !idle {
        if e.no_fmv() {
            return;
        }
        e.step(Step::SwapMissingExtension(Film::AlLogo));
        e.step(Step::Player(Film::AlLogo));
        e.step(Step::Player(Film::MfLogo));
        e.step(Step::ClearFrontSurface);
    }
    e.step(Step::LitEngine);
    if idle {
        for s in [Step::Render3dTidy, Step::ObjectsTidy, Step::ReleaseTextures] {
            e.step(s);
        }
    }
    loop {
        if !e.no_fmv()
            && !e.step(Step::FileExists(Film::Intro)).film_missing
            && e.step(Step::Player(Film::Intro)).key_pressed
        {
            break;
        }
        e.step(Step::Demonstration);
        e.step(Step::StopMusic);
        if e.step(Step::TakeKey).key_pressed {
            break;
        }
    }
    e.step(Step::LetSoundsPlay);
    e.step(Step::ClearFrontSurface);
    if idle {
        for s in [
            Step::SetUpEngine,
            Step::FrontEndScene,
            Step::SetUpEngine,
            Step::Tune3,
        ] {
            e.step(s);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Log {
        steps: Vec<Step>,
        no_fmv: bool,
        /// The keys: each `TakeKey` and `Player(Intro)` answers the next (true: a key).
        keys: Vec<bool>,
        intro: bool,
    }

    impl Films for Log {
        fn no_fmv(&mut self) -> bool {
            self.no_fmv
        }
        fn step(&mut self, s: Step) -> StepAnswer {
            self.steps.push(s);
            match s {
                Step::FileExists(_) => StepAnswer {
                    film_missing: !self.intro,
                    key_pressed: false,
                },
                Step::TakeKey | Step::Player(Film::Intro) => StepAnswer {
                    film_missing: false,
                    key_pressed: self.keys.remove(0),
                },
                _ => StepAnswer::default(),
            }
        }
    }

    #[test]
    fn films() {
        let mut log = Log {
            steps: vec![],
            no_fmv: true,
            keys: vec![false],
            intro: true,
        };
        play_films(&mut log, false);
        assert_eq!(log.steps.len(), 3);

        let mut log = Log {
            steps: vec![],
            no_fmv: false,
            keys: vec![false, false, true],
            intro: true,
        };
        play_films(&mut log, false);
        assert!(log.steps.contains(&Step::Player(Film::AlLogo)));
        assert_eq!(
            log.steps
                .iter()
                .filter(|s| **s == Step::Demonstration)
                .count(),
            1
        );

        let mut log = Log {
            steps: vec![],
            no_fmv: false,
            keys: vec![false, false, false, true],
            intro: false,
        };
        play_films(&mut log, true);
        assert!(!log.steps.contains(&Step::Player(Film::Intro)));
        assert_eq!(
            log.steps
                .iter()
                .filter(|s| **s == Step::Demonstration)
                .count(),
            3
        );
        assert_eq!(log.steps.last(), Some(&Step::Tune3));
    }
}
