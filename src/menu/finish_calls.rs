//! The championship finish routine's two starts (`menu::section_finish`,
//! `menu::championship_finish`): a player picked, the race's sounds and screens set up
//! round `drive::finish_race_0`, the front end set up again; this file holds the calls
//! they make, by place (each through the game at its address: hooks where ported, the
//! platform otherwise).

use crate::errors::{BadInput, Refused};
use crate::framework::{game_read, game_write, rva};
use rc2k_core::menu::finish::{self, Racer, Step, Var};

const SET_UP_RVA: [usize; 5] = [
    rva(0x75B100),
    rva(0x75B104),
    rva(0x75ACE8),
    rva(0x75ACE4),
    rva(0x75B0FC),
];
pub const ON_RVA: usize = rva(0x75B108); // dword_75B108

const KEPT_RVA: usize = rva(0x75ACE4); // dword_75ACE4
const LEADER_RVA: usize = rva(0x75ACCC); // dword_75ACCC
const EVENT_MODE_RVA: usize = rva(0x7556D0); // g_eventMode

/// The game for `rc2k_core::menu::finish`: the variables at their addresses, the finders'
/// Rust versions, the steps their Rust versions. `racer` is the record (the original's
/// edi) the racer fields are read and written through.
#[derive(Default)]
pub struct Game {
    pub racer: u32,
}

fn at(v: Var) -> usize {
    use crate::menu::standings::SECTION_RVA;
    match v {
        Var::On => ON_RVA,
        Var::Kept => KEPT_RVA,
        Var::Leader => LEADER_RVA,
        Var::EventMode => EVENT_MODE_RVA,
        Var::Section => SECTION_RVA,
        Var::SetUpCar => SET_UP_RVA[0],
        Var::SetUpDriver => SET_UP_RVA[1],
        Var::SetUpDriver2 => SET_UP_RVA[2],
        Var::SetUpSection => SET_UP_RVA[4],
        Var::Racer(_) => unreachable!("the racer's fields are not at an address"),
    }
}

impl finish::Finish for Game {
    type Error = Refused;

    fn get(&mut self, v: Var) -> u32 {
        match v {
            Var::Racer(offset) => crate::menu::standings::get(self.racer + offset),
            _ => unsafe { game_read(at(v)) },
        }
    }

    fn set(&mut self, v: Var, value: u32) {
        match v {
            Var::Racer(offset) => crate::menu::standings::put(self.racer + offset, value),
            _ => unsafe { game_write(at(v), value) },
        }
    }

    fn find_leader(&mut self, group_b: bool) -> Result<Option<u32>, Refused> {
        use crate::menu::standings::TABLE_B_RVA;
        let table = unsafe { game_read(TABLE_B_RVA) };
        let found = if group_b {
            unsafe { crate::menu::first_player_b::first_player_b(table, 0)? }
        } else {
            unsafe { crate::menu::first_player_unplaced::first_player_unplaced(table)? }
        };
        self.racer = found.entry;
        Ok((found.figure == 1).then_some(found.computers))
    }

    fn find_done(&mut self) -> Result<Option<u32>, Refused> {
        let found = unsafe { crate::menu::find_section_player::find_section_player()? };
        self.racer = found.record;
        Ok((found.index != u32::MAX).then_some(found.index))
    }

    fn racer(&mut self, r: Racer) -> Result<(), Refused> {
        use crate::menu::standings::player;
        match r {
            Racer::Leader(n) => {
                if player(n) == 0 {
                    return Err(BadInput("the leader's ecx past the players").into());
                }
                self.racer = player(n);
            }
            Racer::Done(n) => self.racer = player(n),
        }
        Ok(())
    }

    /// The original leaves the value in eax for the key taken at the end, which replaces
    /// its low byte and is not read: nothing to keep.
    fn keep(&mut self, _value: u32) {}

    fn step(&mut self, s: Step) -> Result<(), Refused> {
        match s {
            Step::RaceSoundObjects => crate::race::sound_tables::set_up_race_sound_objects(
                &crate::race::sound_tables::ObjectVars::game(),
            )?,
            Step::RaceSounds => crate::sound::set_up_race_sounds::set_up_race_sounds()?,
            Step::LoadingPicture => {
                crate::menu::show_loading_picture::show_loading_picture()?;
            }
            Step::Render3dTidy => unsafe { crate::race::stage_free::render3d_tidy()? },
            Step::ObjectsTidy => unsafe { crate::race::stage_free::objects_tidy()? },
            // `nullsub_6`.
            Step::Nothing => {}
            Step::ReleaseTextures => unsafe { crate::race::race_end::release_textures() },
            Step::FinishRace => unsafe { crate::drive::finish_race_0::finish_race_0()? },
            Step::StopMusic => crate::sound::stop_music::stop_music(),
            Step::MenuScreenMode => {
                crate::platform::set_menu_mode_now();
            }
            Step::SetUpEngine => unsafe { crate::scene::set_up_engine::set_up_engine()? },
            Step::FrontEndScene => unsafe {
                crate::scene::set_up_front_end_scene::set_up_front_end_scene()?
            },
            Step::Tune3 => crate::sound::play_tune_3::play_tune_3(),
            Step::TakeKey => {
                crate::input::take_key::take_key();
            }
        }
        Ok(())
    }
}
