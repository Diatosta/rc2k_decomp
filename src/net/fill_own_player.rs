//! `net::fill_own_player` (0x4181A3): this machine's player set up for the network game from
//! player 1 (`g_player1`, 0x755A3C on): its name (`strings::copy_printable`, 0x1F room) to
//! `byte_52AB70`; its record (`byte_52AD64`): state 0, the car, its colours (+0x30), the
//! nationality, the stage (`dword_75ACD4`; 0 is made 0x29 with direction 0) and its
//! direction (`dword_75ACF0`), the race number, a random byte (`math::next_random`), the
//! time and total 0; and the race's copy of the player's settings (`dword_75AD00` ..
//! `dword_75AE24`: the co-driver, the race number, the view, the HUD's switches, the
//! steering), the HUD's bits (`dword_75B00C`) -1, `dword_75B108` 0.
//!
//! In:  `name` (eax: the player's name)
//! Out: nothing (every register kept, `pusha` .. `popa`)
//! Callers: `net::start_link`, `net::open_lobby`.

use crate::errors::Refused;
use crate::menu::player::Player;
use crate::net::lobby::copy_printable;
use crate::net::store::NET;
use crate::race::state_store::RACE_STATE;
use crate::session::Dword;
use rc2k_core::net::record;

/// A setting copied: (to, from player 1's record).
type Setting = (&'static Dword, fn(&Player) -> u32);

/// The settings copied.
const SETTINGS: [Setting; 13] = [
    (&RACE_STATE.settings.cars[0].codriver_sex, |p| {
        p.codriver_sex
    }),
    (&RACE_STATE.settings.cars[0].race_number, |p| p.race_number),
    (&RACE_STATE.settings.cars[0].view, |p| p.view),
    (&RACE_STATE.settings.cars[0].map_area, |p| p.map_area),
    (&RACE_STATE.settings.cars[0].rear_mirror, |p| p.rear_mirror),
    (&RACE_STATE.settings.cars[0].rev_counter, |p| p.rev_counter),
    (&RACE_STATE.settings.cars[0].message_display, |p| {
        p.message_display
    }),
    (&RACE_STATE.settings.cars[0].car_info, |p| p.car_info),
    (&RACE_STATE.settings.cars[0].driver_info, |p| p.driver_info),
    (&RACE_STATE.settings.cars[0].progress_bar, |p| {
        p.progress_bar
    }),
    (&RACE_STATE.settings.cars[0].pace_notes, |p| p.pace_notes),
    (&RACE_STATE.settings.cars[0].metric_units, |p| {
        p.metric_units
    }),
    (&RACE_STATE.settings.cars[0].steering_sensitivity, |p| {
        p.steering_sensitivity
    }),
];
/// Copied after the HUD's bits: (to, from).
const MORE: [(&Dword, &Dword); 5] = [
    (
        &RACE_STATE.settings.setting_0,
        &crate::menu::owner::MENUS.screens.option_work.setting_0,
    ),
    (
        &RACE_STATE.settings.setting_1,
        &crate::menu::owner::MENUS.screens.option_work.setting_1,
    ),
    (
        &RACE_STATE.settings.setting_2,
        &crate::menu::owner::MENUS.screens.option_work.setting_2,
    ),
    (
        &RACE_STATE.settings.setting_3,
        &crate::menu::owner::MENUS.screens.option_work.setting_3,
    ),
    (
        &RACE_STATE.settings.setting_4,
        &crate::menu::owner::MENUS.screens.option_work.setting_4,
    ),
];

/// The Rust version.
pub fn fill_own_player(name: impl Into<crate::place::Place>) -> Result<(), Refused> {
    copy_printable(name, &NET.player_name, 0x1F)?;
    // Converted: `rc2k_core::net::record::own_record`.
    let player = crate::menu::owner::menus().play.players.players[0].get();
    let own = record::own_record(
        player.car_number as u8,
        player.gearing as u8,
        player.nationality as u8,
        RACE_STATE.settings.stage.get() as u8,
        RACE_STATE.settings.direction.get() as u8,
        player.race_number as u8,
    );
    for (n, b) in own.into_iter().enumerate() {
        NET.own.head.set(n, b);
    }
    NET.own
        .head
        .set(record::RANDOM, { crate::math::next_random::next_random() }
            as u8);
    NET.own.race_time.set(0);
    NET.own.total.set(0);
    for (to, from) in SETTINGS {
        to.set(from(&player));
    }
    RACE_STATE.results.hud_bits_current.set(u32::MAX);
    for (to, from) in MORE {
        to.set(from.get());
    }
    RACE_STATE.results.finished.set(0);
    Ok(())
}
