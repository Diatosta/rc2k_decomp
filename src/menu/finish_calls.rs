//! The championship finish routine's two starts (`menu::section_finish`,
//! `menu::championship_finish`): a player picked, the race's sounds and screens set up
//! round `drive::finish_race_0`, the front end set up again; this file holds the calls
//! they make, by place (each through the game at its address: hooks where ported, the
//! platform otherwise).

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read, game_write, rva};
use rc2k_core::menu::finish::{self, Racer, Step, Var};

/// The functions by place (the test's stand-ins are these places).
pub const CALLED: [usize; 15] = [
    0, 0x415C93, // 1 race::set_up_race_sound_objects
    0x40F02E, // 2 sound::set_up_race_sounds
    0x47CEA8, // 3 menu::show_loading_picture
    0x4433A7, // 4 race::render3d_tidy
    0x44346B, // 5 race::objects_tidy
    0x44347E, // 6 nullsub_6
    0x443396, // 7 race::release_textures
    0x4278C7, // 8 drive::finish_race_0
    0x40F826, // 9 sound::stop_music
    0x404823, // 10 gfx::set_menu_screen_mode
    0x442AF6, // 11 scene::set_up_engine
    0x474E04, // 12 scene::set_up_front_end_scene
    0x40F780, // 13 sound::play_tune_3
    0x407F95, // 14 input::take_key
];

/// A call by place (`CALLED`).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

const SET_UP_RVA: [usize; 5] = [
    rva(0x75B100),
    rva(0x75B104),
    rva(0x75ACE8),
    rva(0x75ACE4),
    rva(0x75B0FC),
];
pub const ON_RVA: usize = rva(0x75B108); // dword_75B108

/// In the game: each through the game at its address.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    // `nullsub_6`: nothing.
    if place == 6 {
        return Ok(());
    }
    crate::calls::call(rva(CALLED[place as usize]), r);
    Ok(())
}

const KEPT_RVA: usize = rva(0x75ACE4); // dword_75ACE4
const LEADER_RVA: usize = rva(0x75ACCC); // dword_75ACCC
const EVENT_MODE_RVA: usize = rva(0x7556D0); // g_eventMode

/// The game for `rc2k_core::menu::finish`: the variables at their addresses, the finders'
/// Rust versions on the registers carried through (`g`), the steps through `part`.
pub struct Game<'a> {
    pub g: Regs,
    pub part: Part<'a>,
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

impl finish::Finish for Game<'_> {
    type Error = Refused;

    fn get(&mut self, v: Var) -> u32 {
        match v {
            Var::Racer(offset) => crate::menu::standings::get(self.g.edi + offset),
            _ => unsafe { game_read(at(v)) },
        }
    }

    fn set(&mut self, v: Var, value: u32) {
        match v {
            Var::Racer(offset) => crate::menu::standings::put(self.g.edi + offset, value),
            _ => unsafe { game_write(at(v), value) },
        }
    }

    fn find_leader(&mut self, group_b: bool) -> Result<Option<u32>, Refused> {
        use crate::menu::standings::TABLE_B_RVA;
        self.g.edi = unsafe { game_read(TABLE_B_RVA) };
        self.g = if group_b {
            self.g.ebx = 0;
            unsafe { crate::menu::first_player_b::first_player_b(&self.g)? }
        } else {
            self.g.ebx = 0x63;
            unsafe { crate::menu::first_player_unplaced::first_player_unplaced(&self.g)? }
        };
        Ok((self.g.edx == 1).then_some(self.g.ecx))
    }

    fn find_done(&mut self) -> Result<Option<u32>, Refused> {
        self.g = unsafe { crate::menu::find_section_player::find_section_player(&self.g)? };
        Ok((self.g.ecx != u32::MAX).then_some(self.g.ecx))
    }

    fn racer(&mut self, r: Racer) -> Result<(), Refused> {
        use crate::menu::standings::player;
        match r {
            Racer::Leader(n) => {
                if player(n) == 0 {
                    return Err(BadInput("the leader's ecx past the players").into());
                }
                self.g.edi = player(n);
            }
            Racer::Done(n) => self.g.edi = player(n),
        }
        Ok(())
    }

    fn keep(&mut self, value: u32) {
        self.g.eax = value;
    }

    fn step(&mut self, s: Step) -> Result<(), Refused> {
        (self.part)(s as u32, &mut self.g)
    }
}
