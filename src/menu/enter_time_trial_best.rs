//! `menu::enter_time_trial_best` (0x48C8B6): the current player's time put in the tenth
//! of the chosen stage's best times and moved up to its place. `dword_7559EC` the stage's
//! +0x3C; the tenth entry's +0x14 the player's time (+0x88), +0x1C its +0x90, +8 the
//! nationality (+4), +0xC and +0x20 the name pointer (+0x18), +0x10 the car (+8), +4 1,
//! +0x40 1 (the new entry's mark), +0x50 `g_cheats`, +0x54 / +0x58 the sealed time
//! (`menu::sealed_player_time` with eax the stage's +0x3C), the 0x40 bytes at the player's
//! +0x20C at +0x5C, the name's first 31 bytes at +0x20; then `menu::rank_time_trial_best`.
//!
//! Callers: the time trial's events, when the player qualifies. The two callees are their Rust
//! versions, `sealed_player_time` with the registers the original has there (eax the stage's +0x3C,
//! ebx `dword_7558D0`, esi the player, edi the entry). Refuses (before anything) a current player
//! past the four or missing, a stage the tables do not give, or a name pointer of 0.

use crate::errors::Refused;
use crate::menu::event_flow::*;
use crate::menu::owner::Menus;
use crate::menu::standings::name_address_of;
use crate::session::Dword;

const STAGE_SLOT: &Dword = &crate::menu::owner::MENUS.screens.draw_work.figure; // dword_7559EC
// hhmm
pub fn enter_time_trial_best(menus: &Menus) -> Result<(), Refused> {
    let play = &menus.play;
    let screens = &menus.screens;
    use rc2k_core::menu::bests;
    let record = stage_record()?;
    stage_times()?; // a stage the tables do not give: refused
    let p = current_player()?;
    let name = name_address_of(&p.get())?;
    let slot = record.record()?.event_slot();
    set(STAGE_SLOT, slot);
    let cheats = get(CHEATS);
    let sealed = crate::menu::sealed_player_time::sealed_player_time(screens, slot)?;
    // Converted: `rc2k_core::saves::entries::BestTime::enter`, then `rank_time_trial_best`.
    use crate::menu::standings::name_of;
    let best = bests::NewBest {
        key: p.get().time,
        player: 1,
        nationality: p.get().nationality,
        name_at: name,
        car: p.get().car_number,
        group: p.get().class,
        name: name_of(menus, &p.get())?,
        entered: None,
        seal: Some(bests::Seal { cheats, sealed }),
        line: Some(p.get().best_line),
    };
    with_stage_times(|list| list[9].enter(&best))?;
    crate::menu::rank_time_trial_best::rank_time_trial_best(play)
}
