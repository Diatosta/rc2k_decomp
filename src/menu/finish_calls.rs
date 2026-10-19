//! The championship finish routine's two starts (`menu::section_finish`,
//! `menu::championship_finish`): a player picked, the race's sounds and screens set up
//! round `drive::finish_race_0`, the front end set up again; this file holds the calls
//! they make, by place (each through the game at its address: hooks where ported, the
//! platform otherwise).

use crate::errors::Refused;
use crate::framework::{Regs, call_game, game_addr, game_write, rva};

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

/// The stand-ins' table for `race::mud::copy`.
pub const PLACES: [(usize, u32); 14] = [
    (0x415C93, 1),
    (0x40F02E, 2),
    (0x47CEA8, 3),
    (0x4433A7, 4),
    (0x44346B, 5),
    (0x44347E, 6),
    (0x443396, 7),
    (0x4278C7, 8),
    (0x40F826, 9),
    (0x404823, 10),
    (0x442AF6, 11),
    (0x474E04, 12),
    (0x40F780, 13),
    (0x407F95, 14),
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

/// The race's set-up from the player edi: `dword_75B100` its +0x28, `dword_75B104` and
/// `dword_75ACE8` its +8, `dword_75ACE4` 0, `dword_75B0FC` the section.
pub unsafe fn set_up(g: &Regs) {
    use crate::menu::standings::{SECTION_RVA, get};
    unsafe {
        game_write(SET_UP_RVA[0], get(g.edi + 0x28));
        game_write(SET_UP_RVA[1], get(g.edi + 8));
        game_write(SET_UP_RVA[2], get(g.edi + 8));
        game_write(SET_UP_RVA[3], 0u32);
        game_write(
            SET_UP_RVA[4],
            crate::framework::game_read::<u32>(SECTION_RVA),
        );
    }
}

/// The calls of `places`, in order.
pub fn calls(g: &mut Regs, places: &[u32], part: &mut Part) -> Result<(), Refused> {
    for &p in places {
        part(p, g)?;
    }
    Ok(())
}

/// In the game: each through the game at its address.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    // `nullsub_6`: nothing.
    if place == 6 {
        return Ok(());
    }
    unsafe { call_game(r, game_addr(rva(CALLED[place as usize]))) };
    Ok(())
}

/// The regions the starts write (the tests').
pub fn regions() -> Vec<crate::service::harness::Region> {
    use crate::service::harness::at;
    let mut v: Vec<_> = SET_UP_RVA.map(at).to_vec();
    v.extend([ON_RVA, rva(0x75ACCC)].map(at));
    v
}
