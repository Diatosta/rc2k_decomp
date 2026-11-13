use crate::errors::{BadInput, Refused};
use crate::menu::event_flow::Fault;
use crate::menu::owner::Menus;
use crate::owners::Owners;
use crate::race::owner::Race;
use crate::session::DwordLike;
use rc2k_core::menu::finish::{self, Racer, Step, Var};

/// The game for `rc2k_core::menu::finish`: the variables, the finders and the steps. `racer` is
/// the record the racer fields are read through: a player's.
pub struct Game<'a> {
    menus: &'a Menus,
    owners: Owners<'static>,
    pub racer: RacerAt<'a>,
    /// A refused read or write of the core's variables (checked at each step and at the end).
    fault: Fault,
}

impl<'a> Game<'a> {
    /// Runs a flow on a new game; a refused read or write of its variables ends it.
    pub fn run(
        o: Owners<'static>,
        flow: impl FnOnce(&mut Game<'a>) -> Result<(), Refused>,
    ) -> Result<(), Refused> {
        let mut game = Game {
            menus: o.menus,
            owners: o,
            racer: RacerAt::None,
            fault: Fault::default(),
        };
        let done = flow(&mut game);
        game.fault.check()?;
        done
    }
}

/// The player the racer's fields are read from. The flows read the racer only after choosing a
/// player (`Racer::Leader`) or finding one done (`find_done`): no entry is read as one.
#[derive(Clone, Copy)]
pub enum RacerAt<'a> {
    None,
    Player(crate::menu::standings::PlayerCell<'a>),
}

const NO_RACER: crate::errors::BadInput = BadInput("a racer's field with no player chosen");
const RACER_SET: crate::errors::BadInput = BadInput("a racer's field written");

impl Game<'_> {
    fn racer_get(&self, v: Var) -> u32 {
        let RacerAt::Player(p) = self.racer else {
            self.fault.ok::<u32>(Err(NO_RACER));
            return 0;
        };
        let p = p.get();
        match v {
            Var::RacerNumber => p.race_number,
            _ => p.car_number,
        }
    }
}

/// The variables that are in static cells.
fn cell<'a>(menus: &'a Menus, race: &'a Race, v: Var) -> Option<&'a dyn DwordLike> {
    Some(match v {
        Var::On => &race.shared.state.results.finished,
        Var::Kept => &race.shared.state.settings.kept,
        Var::Leader => &race.shared.state.settings.a8_game,
        Var::SetUpCar => &race.shared.state.results.finish_player,
        Var::SetUpDriver => &race.shared.state.results.finish_kind,
        Var::SetUpDriver2 => &race.shared.state.settings.car_kind_current,
        Var::SetUpSection => &race.shared.state.results.stage_index,
        Var::EventMode => &menus.play.champ_view.event_mode,
        Var::Section => &menus.screens.event.section,
        _ => return None,
    })
}

impl finish::Finish for Game<'_> {
    type Error = Refused;

    fn get(&mut self, v: Var) -> u32 {
        let menus = self.menus;
        match v {
            Var::RacerNumber | Var::RacerCar => self.racer_get(v),
            _ => match cell(menus, self.owners.race, v) {
                Some(cell) => cell.get(),
                None => unreachable!("the racer's fields are not at an address"),
            },
        }
    }

    fn set(&mut self, v: Var, value: u32) {
        let menus = self.menus;
        match v {
            Var::RacerNumber | Var::RacerCar => self.fault.ok(Err(RACER_SET)),
            _ => match cell(menus, self.owners.race, v) {
                Some(cell) => cell.set(value),
                None => unreachable!("the racer's fields are not at an address"),
            },
        }
    }

    fn find_leader(&mut self, group_b: bool) -> Result<Option<u32>, Refused> {
        let found = if group_b {
            crate::menu::first_player_b::first_player_b(0)?
        } else {
            crate::menu::first_player_unplaced::first_player_unplaced()?
        };
        self.racer = RacerAt::None;
        Ok((found.figure == 1).then_some(found.computers))
    }

    fn find_done(&mut self) -> Result<Option<u32>, Refused> {
        let found = crate::menu::find_section_player::find_section_player(self.menus)?;
        self.racer = RacerAt::Player(found.record);
        Ok((found.index != u32::MAX).then_some(found.index))
    }

    fn racer(&mut self, r: Racer) -> Result<(), Refused> {
        use crate::menu::standings::player;
        match r {
            Racer::Leader(n) => {
                if n >= 4 {
                    return Err(BadInput("the leader's ecx past the players").into());
                }
                self.racer = RacerAt::Player(player(self.menus, n));
            }
            Racer::Done(n) => self.racer = RacerAt::Player(player(self.menus, n)),
        }
        Ok(())
    }

    /// The value left for the key taken at the end is not read: nothing to keep.
    fn step(&mut self, s: Step) -> Result<(), Refused> {
        let (sound, race) = (self.owners.sound, self.owners.race);
        self.fault.check()?;
        match s {
            Step::RaceSoundObjects => crate::race::sound_tables::set_up_race_sound_objects(
                self.owners,
                &crate::race::sound_tables::ObjectVars::of(sound),
            )?,
            Step::RaceSounds => crate::sound::set_up_race_sounds::set_up_race_sounds(self.owners)?,
            Step::LoadingPicture => {
                crate::menu::show_loading_picture::show_loading_picture_in(self.owners)?;
            }
            Step::Render3dTidy => {
                crate::race::stage_free::render3d_tidy(race)?;
            }
            Step::ObjectsTidy => crate::race::stage_free::objects_tidy(race)?,
            Step::Nothing => {}
            Step::ReleaseTextures => {
                crate::race::race_end::release_textures(self.owners.scene, self.owners.gfx);
            }
            Step::FinishRace => {
                crate::drive::finish_race_0::finish_race_0(race, self.owners)?;
            }
            Step::StopMusic => {
                crate::sound::stop_music::stop_music(self.owners);
            }
            Step::MenuScreenMode => {
                crate::platform::set_menu_mode_now(self.owners.gfx, self.owners.system);
            }
            Step::SetUpEngine => {
                crate::scene::set_up_engine::set_up_engine_in(race, self.owners.scene)?;
            }
            Step::FrontEndScene => {
                crate::scene::set_up_front_end_scene::set_up_front_end_scene_in(self.owners)?;
            }
            Step::Tune3 => crate::sound::play_tune_3::play_tune_3(self.owners),
            Step::TakeKey => {
                crate::input::take_key::take_key(self.owners.input);
            }
        }
        Ok(())
    }
}
