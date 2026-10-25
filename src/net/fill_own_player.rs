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
use crate::net::lobby::{copy_printable, g};
use crate::net::store::NET;
use crate::race::state_store::RACE_STATE;
use crate::session::Dword;
use rc2k_core::net::record;

const CAR: usize = 0x755A54; // g_player1.car_number
const COLOURS: usize = 0x755A7C; // g_player1.field_30
const RACE_NUMBER: usize = 0x755A74; // g_player1.race_number
const NATIONALITY: usize = 0x755A50; // g_player1.nationality

/// The settings copied: (to, from).
const SETTINGS: [(&Dword, usize); 13] = [
    (&RACE_STATE.settings.cars[0].codriver_sex, 0x755BA0),
    (&RACE_STATE.settings.cars[0].race_number, 0x755A74),
    (&RACE_STATE.settings.cars[0].view, 0x755A58),
    (&RACE_STATE.settings.cars[0].map_area, 0x755BA4),
    (&RACE_STATE.settings.cars[0].rear_mirror, 0x755BA8),
    (&RACE_STATE.settings.cars[0].rev_counter, 0x755BAC),
    (&RACE_STATE.settings.cars[0].message_display, 0x755BB4),
    (&RACE_STATE.settings.cars[0].car_info, 0x755BB0),
    (&RACE_STATE.settings.cars[0].driver_info, 0x755BBC),
    (&RACE_STATE.settings.cars[0].progress_bar, 0x755BB8),
    (&RACE_STATE.settings.cars[0].pace_notes, 0x755BC0),
    (&RACE_STATE.settings.cars[0].metric_units, 0x755BC4),
    (&RACE_STATE.settings.cars[0].steering_sensitivity, 0x755B68),
];
/// Copied after the HUD's bits: (to, from).
const MORE: [(&Dword, usize); 5] = [
    (&RACE_STATE.settings.setting_0, 0x755650),
    (&RACE_STATE.settings.setting_1, 0x75565C),
    (&RACE_STATE.settings.setting_2, 0x755660),
    (&RACE_STATE.settings.setting_3, 0x755664),
    (&RACE_STATE.settings.setting_4, 0x755668),
];

/// The Rust version.
pub fn fill_own_player(name: u32) -> Result<(), Refused> {
    copy_printable(name, NET.player_name.addr() as u32, 0x1F)?;
    // Converted: `rc2k_core::net::record::own_record`.
    let own = record::own_record(
        g(CAR) as u8,
        g(COLOURS) as u8,
        g(NATIONALITY) as u8,
        RACE_STATE.settings.stage.get() as u8,
        RACE_STATE.settings.direction.get() as u8,
        g(RACE_NUMBER) as u8,
    );
    for (n, b) in own.into_iter().enumerate() {
        NET.own.head.set(n, b);
    }
    NET.own.head.set(
        record::RANDOM,
        unsafe { crate::math::next_random::next_random() } as u8,
    );
    NET.own.race_time.set(0);
    NET.own.total.set(0);
    for (to, from) in SETTINGS {
        to.set(g(from));
    }
    RACE_STATE.results.hud_bits_current.set(u32::MAX);
    for (to, from) in MORE {
        to.set(g(from));
    }
    RACE_STATE.results.finished.set(0);
    Ok(())
}
