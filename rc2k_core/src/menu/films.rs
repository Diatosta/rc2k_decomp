//! The films (converted 2026-10-21 from `menu::play_films` 0x4161CD): at the game's start
//! the two logos (the first on 16-bit screens only), then the intro over and over with
//! the game's demonstration between, until a key; from the idle main menu the scene let
//! go first and set up again after. The setting `nofmv` leaves the films out.

/// The films.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Film {
    /// `;3files\fmv\allogo.avi`, `mflogo.mpg`, `intro.mpg`.
    AlLogo,
    MfLogo,
    Intro,
}

/// The steps, at the game's addresses in the hook crate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// 0x40F826 (`sound::stop_music`).
    StopMusic,
    /// `gfx::clear_front_surface`.
    Step4059F4,
    /// `input::take_key`: the zero flag clear with a key.
    TakeKey,
    /// `files::swap_missing_extension` on the film (eax).
    SwapMissingExtension(Film),
    /// The player (0x41AE39, ebx 0) on the film, or on the path the step before left
    /// (None); the zero flag clear when a key ended it.
    Player(Option<Film>),
    /// `scene::set_up_lit_engine`.
    LitEngine,
    /// `race::render3d_tidy`, `race::objects_tidy`, `nullsub_6`, `race::release_textures`.
    Render3dTidy,
    ObjectsTidy,
    Nothing,
    ReleaseTextures,
    /// `files::file_exists`: the carry set when it is not there.
    FileExists(Film),
    /// `race::credits_race`, the demonstration (ebp kept across it).
    Demonstration,
    /// `sound::let_sounds_play`.
    Step409B76,
    /// `scene::set_up_engine`, `scene::set_up_front_end_scene`, `sound::play_tune_3`.
    SetUpEngine,
    FrontEndScene,
    Tune3,
}

/// What a step answered.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Flags {
    pub carry: bool,
    pub zero: bool,
}

pub trait Films {
    /// `g_noFmv` not 0.
    fn no_fmv(&mut self) -> bool;
    /// `g_screenBits`.
    fn screen_bits(&mut self) -> u32;
    fn step(&mut self, s: Step) -> Flags;
}

/// The screen depth that shows the first logo.
pub const LOGO_BITS: u32 = 0x10;

/// `menu::play_films`: `idle` (eax 1) from the idle main menu, else at the start (0).
pub fn play_films<F: Films>(e: &mut F, idle: bool) {
    e.step(Step::StopMusic);
    e.step(Step::Step4059F4);
    e.step(Step::TakeKey);
    if !idle {
        if e.no_fmv() {
            return;
        }
        if e.screen_bits() == LOGO_BITS {
            e.step(Step::SwapMissingExtension(Film::AlLogo));
            e.step(Step::Player(None));
        }
        e.step(Step::Player(Some(Film::MfLogo)));
        e.step(Step::Step4059F4);
    }
    e.step(Step::LitEngine);
    if idle {
        for s in [
            Step::Render3dTidy,
            Step::ObjectsTidy,
            Step::Nothing,
            Step::ReleaseTextures,
        ] {
            e.step(s);
        }
    }
    loop {
        if !e.no_fmv()
            && !e.step(Step::FileExists(Film::Intro)).carry
            && !e.step(Step::Player(Some(Film::Intro))).zero
        {
            break;
        }
        e.step(Step::Demonstration);
        e.step(Step::StopMusic);
        if !e.step(Step::TakeKey).zero {
            break;
        }
    }
    e.step(Step::Step409B76);
    e.step(Step::Step4059F4);
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
        bits: u32,
        /// The keys: each `TakeKey` and `Player(Intro)` answers the next (true: a key).
        keys: Vec<bool>,
        intro: bool,
    }

    impl Films for Log {
        fn no_fmv(&mut self) -> bool {
            self.no_fmv
        }
        fn screen_bits(&mut self) -> u32 {
            self.bits
        }
        fn step(&mut self, s: Step) -> Flags {
            self.steps.push(s);
            match s {
                Step::FileExists(_) => Flags {
                    carry: !self.intro,
                    zero: false,
                },
                Step::TakeKey | Step::Player(Some(Film::Intro)) => Flags {
                    carry: false,
                    zero: !self.keys.remove(0),
                },
                _ => Flags::default(),
            }
        }
    }

    #[test]
    fn films() {
        let mut log = Log {
            steps: vec![],
            no_fmv: true,
            bits: 0x10,
            keys: vec![false],
            intro: true,
        };
        play_films(&mut log, false);
        assert_eq!(log.steps.len(), 3);

        let mut log = Log {
            steps: vec![],
            no_fmv: false,
            bits: 0x10,
            keys: vec![false, false, true],
            intro: true,
        };
        play_films(&mut log, false);
        assert!(log.steps.contains(&Step::Player(None)));
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
            bits: 0x20,
            keys: vec![false, false, false, true],
            intro: false,
        };
        play_films(&mut log, true);
        assert!(!log.steps.contains(&Step::Player(Some(Film::Intro))));
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
