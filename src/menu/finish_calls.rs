use crate::errors::{BadInput, Refused};
use crate::menu::champ_store::CHAMP_VIEW;
use crate::menu::event_flow::Fault;
use crate::race::state_store::RACE_STATE;
use crate::session::Dword;
use rc2k_core::menu::finish::{self, Racer, Step, Var};

/// The game for `rc2k_core::menu::finish`: the variables at their addresses, the finders'
/// Rust versions, the steps their Rust versions. `racer` is the record (the original's
/// edi) the racer fields are read and written through: a player's, or a standings entry.
pub struct Game {
    pub racer: RacerAt,
    /// A refused read or write of the core's variables (checked at each step and at the end).
    fault: Fault,
}

impl Game {
    /// Runs a flow on a new game; a refused read or write of its variables ends it.
    pub fn run(flow: impl FnOnce(&mut Game) -> Result<(), Refused>) -> Result<(), Refused> {
        let mut game = Game {
            racer: RacerAt::None,
            fault: Fault::default(),
        };
        let done = flow(&mut game);
        game.fault.check()?;
        done
    }
}

/// Where the racer's fields are: a player's record or an entry of
/// the championship history (table B).
#[derive(Clone, Copy)]
pub enum RacerAt {
    None,
    Player(crate::menu::standings::PlayerCell),
    History(usize),
}

const NO_RACER: crate::errors::BadInput = BadInput("a racer's field with no racer chosen");
const PAST_ENTRY: crate::errors::BadInput = BadInput("a racer's field past its entry");

impl Game {
    fn racer_get(&self, offset: u32) -> u32 {
        match self.racer {
            RacerAt::None => {
                self.fault.ok::<u32>(Err(NO_RACER));
                0
            }
            RacerAt::Player(p) => p.get().word(offset as usize),
            RacerAt::History(n) => {
                self.fault.ok(
                    crate::menu::standings::with_b(|t| t.entries[n].dword_at(offset))
                        .and_then(|v| v.ok_or(PAST_ENTRY)),
                )
            }
        }
    }

    fn racer_set(&self, offset: u32, value: u32) {
        match self.racer {
            RacerAt::None => self.fault.ok(Err(NO_RACER)),
            RacerAt::Player(p) => p.update(|q| q.store(&[(offset as usize, value)])),
            RacerAt::History(n) => self.fault.ok(crate::menu::standings::with_b(|t| {
                t.entries[n].set_dword_at(offset, value)
            })
            .and_then(|v| v.ok_or(PAST_ENTRY))),
        }
    }
}

/// The variables that are in static cells.
fn cell(v: Var) -> Option<&'static Dword> {
    Some(match v {
        Var::On => &RACE_STATE.results.finished,
        Var::Kept => &RACE_STATE.settings.kept,
        Var::Leader => &RACE_STATE.settings.a8_game,
        Var::SetUpCar => &RACE_STATE.results.finish_player,
        Var::SetUpDriver => &RACE_STATE.results.finish_kind,
        Var::SetUpDriver2 => &RACE_STATE.settings.car_kind_current,
        Var::SetUpSection => &RACE_STATE.results.stage_index,
        Var::EventMode => &CHAMP_VIEW.event_mode,
        Var::Section => &crate::menu::event_store::EVENT_STATE.section,
        _ => return None,
    })
}

impl finish::Finish for Game {
    type Error = Refused;

    fn get(&mut self, v: Var) -> u32 {
        match v {
            Var::Racer(offset) => self.racer_get(offset),
            _ => match cell(v) {
                Some(cell) => cell.get(),
                None => unreachable!("the racer's fields are not at an address"),
            },
        }
    }

    fn set(&mut self, v: Var, value: u32) {
        match v {
            Var::Racer(offset) => self.racer_set(offset, value),
            _ => match cell(v) {
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
        self.racer = RacerAt::History(found.entry);
        Ok((found.figure == 1).then_some(found.computers))
    }

    fn find_done(&mut self) -> Result<Option<u32>, Refused> {
        let found = crate::menu::find_section_player::find_section_player()?;
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
                self.racer = RacerAt::Player(player(n));
            }
            Racer::Done(n) => self.racer = RacerAt::Player(player(n)),
        }
        Ok(())
    }

    /// The original leaves the value in eax for the key taken at the end, which replaces
    /// its low byte and is not read: nothing to keep.
    fn step(&mut self, s: Step) -> Result<(), Refused> {
        self.fault.check()?;
        match s {
            Step::RaceSoundObjects => crate::race::sound_tables::set_up_race_sound_objects(
                &crate::race::sound_tables::ObjectVars::game(),
            )?,
            Step::RaceSounds => crate::sound::set_up_race_sounds::set_up_race_sounds()?,
            Step::LoadingPicture => {
                crate::menu::show_loading_picture::show_loading_picture()?;
            }
            Step::Render3dTidy => crate::race::stage_free::render3d_tidy()?,
            Step::ObjectsTidy => crate::race::stage_free::objects_tidy()?,
            // `nullsub_6`.
            Step::Nothing => {}
            Step::ReleaseTextures => crate::race::race_end::release_textures(),
            Step::FinishRace => crate::drive::finish_race_0::finish_race_0()?,
            Step::StopMusic => crate::sound::stop_music::stop_music(),
            Step::MenuScreenMode => {
                crate::platform::set_menu_mode_now();
            }
            Step::SetUpEngine => crate::scene::set_up_engine::set_up_engine()?,
            Step::FrontEndScene => crate::scene::set_up_front_end_scene::set_up_front_end_scene()?,
            Step::Tune3 => crate::sound::play_tune_3::play_tune_3(),
            Step::TakeKey => {
                crate::input::take_key::take_key();
            }
        }
        Ok(())
    }
}
