//! The championship's finish routines (converted 2026-10-21 from
//! `menu::championship_finish` 0x4A14CB and `menu::section_finish` 0x4A9439): who drives
//! the finish (the championship's leader, else the first player done with the section),
//! the race's set-up from that player, the steps of the finish and the front end's set-up
//! again, a key taken at the end.

use crate::Invalid;

/// The variables.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Var {
    /// `dword_75B108`: the finish is the leader's.
    On,
    /// `dword_75ACE4`.
    Kept,
    /// `dword_75ACCC`: a leader was found.
    Leader,
    /// `g_eventMode`.
    EventMode,
    /// `section`.
    Section,
    /// The race's set-up: `dword_75B100`, `dword_75B104`, `dword_75ACE8`, `dword_75B0FC`.
    SetUpCar,
    SetUpDriver,
    SetUpDriver2,
    SetUpSection,
    /// The chosen player's race number (+0x28) and car (+8).
    RacerNumber,
    RacerCar,
}

/// The steps, by the place the hook crate calls them at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Step {
    /// `race::set_up_race_sound_objects`.
    RaceSoundObjects = 1,
    /// `sound::set_up_race_sounds`.
    RaceSounds = 2,
    /// `menu::show_loading_picture`.
    LoadingPicture = 3,
    /// `race::render3d_tidy`, `race::objects_tidy`, `nullsub_6`, `race::release_textures`.
    Render3dTidy = 4,
    ObjectsTidy = 5,
    Nothing = 6,
    ReleaseTextures = 7,
    /// `drive::finish_race_0`.
    FinishRace = 8,
    /// `sound::stop_music`.
    StopMusic = 9,
    /// `gfx::set_menu_screen_mode`.
    MenuScreenMode = 10,
    /// `scene::set_up_engine`, `scene::set_up_front_end_scene`.
    SetUpEngine = 11,
    FrontEndScene = 12,
    /// `sound::play_tune_3`.
    Tune3 = 13,
    /// `input::take_key`.
    TakeKey = 14,
}

use Step::*;

/// The finish raced: up to the race's end.
pub const RACE: [Step; 8] = [
    RaceSoundObjects,
    RaceSounds,
    LoadingPicture,
    Render3dTidy,
    ObjectsTidy,
    Nothing,
    ReleaseTextures,
    FinishRace,
];
/// The front end's set-up again (with the race's sounds once more).
pub const FRONT_END: [Step; 9] = [
    StopMusic,
    MenuScreenMode,
    RaceSounds,
    LoadingPicture,
    SetUpEngine,
    FrontEndScene,
    SetUpEngine,
    StopMusic,
    Tune3,
];
/// `menu::section_finish`'s: without the race's sounds.
pub const SECTION_FRONT_END: [Step; 8] = [
    StopMusic,
    MenuScreenMode,
    LoadingPicture,
    SetUpEngine,
    FrontEndScene,
    SetUpEngine,
    StopMusic,
    Tune3,
];

/// The player who drives the finish.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Racer {
    /// The championship's leader (player n; refused when missing).
    Leader(u32),
    /// The first player done with the section.
    Done(u32),
}

/// The game a finish runs on.
pub trait Finish {
    type Error: From<Invalid>;
    fn get(&mut self, v: Var) -> u32;
    fn set(&mut self, v: Var, value: u32);
    /// The leader in standings table B (`group_b`: `menu::first_player_b` in group 0,
    /// else `menu::first_player_unplaced`): its player when one was found.
    fn find_leader(&mut self, group_b: bool) -> Result<Option<u32>, Self::Error>;
    /// The first player done with the section (`menu::find_section_player`), if any.
    fn find_done(&mut self) -> Result<Option<u32>, Self::Error>;
    /// The player the set-up reads (`Var::Racer`).
    fn racer(&mut self, r: Racer) -> Result<(), Self::Error>;
    fn step(&mut self, s: Step) -> Result<(), Self::Error>;
}

/// The race's set-up from the racer: its race number, its car twice, `dword_75ACE4` 0, the
/// section.
fn set_up<F: Finish>(e: &mut F) {
    let car = e.get(Var::RacerNumber);
    e.set(Var::SetUpCar, car);
    let driver = e.get(Var::RacerCar);
    e.set(Var::SetUpDriver, driver);
    let driver = e.get(Var::RacerCar);
    e.set(Var::SetUpDriver2, driver);
    e.set(Var::Kept, 0);
    let section = e.get(Var::Section);
    e.set(Var::SetUpSection, section);
}

fn steps<F: Finish>(e: &mut F, list: &[Step]) -> Result<(), F::Error> {
    for &s in list {
        e.step(s)?;
    }
    Ok(())
}

/// The key at the end (dropped).
fn take_key<F: Finish>(e: &mut F) -> Result<(), F::Error> {
    e.step(TakeKey)
}

/// `menu::championship_finish`: the leader looked for (in event mode 0x13 in group 0);
/// found, `dword_75ACCC` and `dword_75B108` 1 and the finish is its; else the first player
/// done with the section's (none: only the key). The leader's flag is 0 again before the
/// front end's set-up.
pub fn championship_finish<F: Finish>(e: &mut F) -> Result<(), F::Error> {
    e.set(Var::On, 0);
    e.set(Var::Kept, 0);
    let group_b = e.get(Var::EventMode) == 0x13;
    match e.find_leader(group_b)? {
        Some(n) => {
            if n >= 4 {
                return Err(Invalid("the leader's ecx past the players").into());
            }
            e.racer(Racer::Leader(n))?;
            e.set(Var::Leader, 1);
            e.set(Var::On, 1);
        }
        None => {
            e.set(Var::On, 0);
            if e.find_done()?.is_none() {
                return take_key(e);
            }
        }
    }
    set_up(e);
    steps(e, &RACE)?;
    e.set(Var::On, 0);
    steps(e, &FRONT_END)?;
    take_key(e)
}

/// `menu::section_finish`: the finish of the first player done with the section, if any;
/// then the key.
pub fn section_finish<F: Finish>(e: &mut F) -> Result<(), F::Error> {
    e.set(Var::On, 0);
    e.set(Var::Kept, 0);
    if let Some(n) = e.find_done()? {
        e.racer(Racer::Done(n))?;
        set_up(e);
        steps(e, &RACE)?;
        steps(e, &SECTION_FRONT_END)?;
    }
    take_key(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Log {
        steps: Vec<String>,
        leader: Option<u32>,
        done: Option<u32>,
        mode: u32,
    }

    impl Finish for Log {
        type Error = Invalid;
        fn get(&mut self, v: Var) -> u32 {
            match v {
                Var::EventMode => self.mode,
                Var::RacerNumber => 0x28,
                Var::RacerCar => 8,
                _ => 7,
            }
        }
        fn set(&mut self, v: Var, value: u32) {
            self.steps.push(format!("{v:?}={value}"));
        }
        fn find_leader(&mut self, group_b: bool) -> Result<Option<u32>, Invalid> {
            self.steps.push(format!("leader {group_b}"));
            Ok(self.leader)
        }
        fn find_done(&mut self) -> Result<Option<u32>, Invalid> {
            self.steps.push("done".into());
            Ok(self.done)
        }
        fn racer(&mut self, r: Racer) -> Result<(), Invalid> {
            self.steps.push(format!("{r:?}"));
            Ok(())
        }
        fn step(&mut self, s: Step) -> Result<(), Invalid> {
            self.steps.push(format!("{s:?}"));
            Ok(())
        }
    }

    #[test]
    fn finishes() {
        let mut log = Log {
            leader: Some(2),
            mode: 0x13,
            ..Log::default()
        };
        championship_finish(&mut log).unwrap();
        assert_eq!(log.steps[2], "leader true");
        assert_eq!(log.steps[3], "Leader(2)");
        assert!(log.steps.contains(&"SetUpCar=40".to_string()));
        assert_eq!(log.steps.iter().filter(|s| *s == "On=0").count(), 2);
        assert_eq!(log.steps.last().unwrap(), "TakeKey");

        let mut log = Log {
            leader: Some(4),
            ..Log::default()
        };
        assert!(championship_finish(&mut log).is_err());

        let mut log = Log::default();
        championship_finish(&mut log).unwrap();
        assert_eq!(&log.steps[4..], ["done", "TakeKey"]);

        let mut log = Log {
            done: Some(1),
            ..Log::default()
        };
        section_finish(&mut log).unwrap();
        assert_eq!(log.steps.iter().filter(|s| s.starts_with("On")).count(), 1);
        assert_eq!(log.steps.iter().filter(|s| *s == "RaceSounds").count(), 1);
        assert_eq!(log.steps.len(), 3 + 5 + 1 + 8 + 8 + 1);
    }
}
