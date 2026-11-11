//! `system::load_game_data` (0x411239): loads what the game keeps between runs, and gives
//! every name that is empty a default.
//!
//! In this order:
//! - `saves::load_options`, `saves::load_best_times`, `settings::read_settings` (again:
//!   the settings file outranks the kept options), `install::read_sku`, `runs`
//!   + 1, `settings::read_driver_names`, `physics::apply_patches`;
//! - by the setting `allsecretcars` (`g_allSecretCars`): 0 `menu::remove_secret_cars`; 1
//!   the cheats' bits 0x1FBC set and `menu::apply_cheats`; anything else neither;
//! - the first player's name (`name`), if empty: the user's name
//!   (`system::user_name`: `GetUserNameA`, 0x1F bytes) cut down by
//!   `strings::copy_printable` in place, and if that leaves nothing, string 0x18C; the
//!   other three players' (0x20 bytes apart), if empty: strings 0x18D .. 0x18F;
//! - the machine's name (`machine_name`), if empty: the computer's name
//!   (`system::computer_name`, 0x40 bytes) the same way, else string 0x14D;
//! - the tyre change record's name, if empty: a random one of the 180 strings from 0x5A5 on
//!   (`math::random_below`), drawn again while what is there starts "andr", and
//!   `g_tyreChangeRecord` = 0x5208 (210.00 s, the tyre change's best time);
//! - all 100 saved games are read (`saves::read_savegame_slot`) and every slot's name
//!   that is empty becomes string 0x4E0 and the slot's number in two digits; the same for
//!   the replays (string 0x4E2) and the car setups (string 0x4E1).
//!
//! One caller, `start`. Refuses a default name that does not fit its place, and what its parts
//! refuse. A refusal comes with the work half done, and the original then starts from the top.

use crate::errors::{BadInput, Refused};
use crate::menu::name_lists::NAME_LEN;
use crate::net::owner::net;
use crate::race::owner::race;
use crate::service::owner::service;
use crate::session::Dword;
use crate::strings::append_string_by_id::push_string;
use crate::strings::copy_printable::printable_from;
use crate::system::owner::Start;

// ---- game data ----

// Converted: `rc2k_core::system::game_data` (the choices, the default names, the drawn
// driver's name).
use crate::menu::owner::menus;
use crate::saves::owner::saves;
use rc2k_core::system::game_data::{
    ALL_CARS_CHEATS, DRIVER_NAME_IDS, MACHINE_NAME_ID, PLAYER_NAME_IDS, REPLAY_NAME_ID,
    SAVEGAME_NAME_ID, SETUP_NAME_ID, SLOTS, TYRE_CHANGE_RECORD, USER_NAME_ROOM, all_secret_cars,
    drawn_again, removes_secret_cars,
};

const MACHINE_NAME_ROOM: usize = 0x40;
/// More draws than a name can need.
const MOST_DRAWS: usize = 10_000;

/// What the function calls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Call {
    LoadOptions,
    LoadBestTimes,
    ReadSettings,
    ReadSku,
    ReadDriverNames,
    ApplyPatches,
    RemoveSecretCars,
    ApplyCheats,
    /// The user's name to the first player's name.
    UserName,
    /// The computer's name to the machine name.
    ComputerName,
    /// A random number under this.
    RandomBelow(u32),
    ReadSavegameSlot(u32),
    ReadReplaySlot(u32),
    ReadSetupSlot(u32),
}

/// Does a call; a result only matters for `RandomBelow`.
pub type Outside<'a> = &'a mut dyn FnMut(Call) -> Result<u32, Refused>;

/// The string and what follows it, with a NUL, at `to`.
fn default_name(
    to: &impl crate::session::ByteBuffer,
    at: usize,
    room: usize,
    id: u32,
    after: &[u8],
) -> Result<(), BadInput> {
    let mut string = Vec::new();
    push_string(&mut string, id)?;
    let name = rc2k_core::system::game_data::default_name(&string, after, room)
        .ok_or(BadInput("a default name longer than its place"))?;
    to.write_bytes_at(at, &name, "a default name")?;
    Ok(())
}

/// A list of 100 names: every empty one becomes the string and the slot's number.
fn number_empty_names(names: &impl crate::session::ByteBuffer, id: u32) -> Result<(), BadInput> {
    for slot in 0..SLOTS {
        let name = slot as usize * NAME_LEN;
        if names.get_byte(name) == 0 {
            let mut string = Vec::new();
            push_string(&mut string, id)?;
            let bytes = rc2k_core::system::game_data::slot_name(&string, slot, NAME_LEN)
                .ok_or(BadInput("a default name longer than its place"))?;
            // The name bytes at their slot in the names buffer.
            names.write_bytes_at(name, &bytes, "a default slot name")?;
        }
    }
    Ok(())
}

pub fn load_game_data(start: &Start, outside: Outside) -> Result<(), Refused> {
    let race = race();
    let net = net();
    let play = &menus().play;
    outside(Call::LoadOptions)?;
    outside(Call::LoadBestTimes)?;
    outside(Call::ReadSettings)?;
    outside(Call::ReadSku)?;
    start
        .run_count
        .runs
        .set(start.run_count.runs.get().wrapping_add(1));
    outside(Call::ReadDriverNames)?;
    outside(Call::ApplyPatches)?;
    if removes_secret_cars(play.cheats.all_secret_cars.get()) {
        outside(Call::RemoveSecretCars)?;
    }
    if all_secret_cars(play.cheats.all_secret_cars.get()) {
        play.cheats
            .cheats
            .set(play.cheats.cheats.get() | ALL_CARS_CHEATS);
        outside(Call::ApplyCheats)?;
    }

    for (player, id) in PLAYER_NAME_IDS.into_iter().enumerate() {
        // The player's name bytes at their slot in the names buffer.
        if play.names.player_names.get(player * NAME_LEN) != 0 {
            continue;
        }
        if player == 0 {
            let field = &play.names.player_names;
            outside(Call::UserName)?;
            let bytes = printable_from(&field.read(), USER_NAME_ROOM)?;
            field.write_at(0, &bytes, "a printable name")?;
            if play.names.player_names.get(player * NAME_LEN) != 0 {
                continue;
            }
        }
        default_name(
            &play.names.player_names,
            player * NAME_LEN,
            NAME_LEN,
            id,
            &[],
        )?;
    }

    if net.vars.machine_name.get(0) == 0 {
        let field = &net.vars.machine_name;
        outside(Call::ComputerName)?;
        let bytes = printable_from(&field.read(), MACHINE_NAME_ROOM as u32)?;
        field.write_at(0, &bytes, "a printable name")?;
        if net.vars.machine_name.get(0) == 0 {
            default_name(
                &net.vars.machine_name,
                0,
                MACHINE_NAME_ROOM,
                MACHINE_NAME_ID,
                &[],
            )?;
        }
    }

    if race.shared.tyre_change.record_name.get(0) == 0 {
        let mut draws = 0;
        let end = loop {
            let drawn = outside(Call::RandomBelow(DRIVER_NAME_IDS.1))?;
            let mut name = Vec::new();
            push_string(&mut name, DRIVER_NAME_IDS.0.wrapping_add(drawn))?;
            if name.len() >= NAME_LEN {
                return Err(BadInput("a driver's name longer than its place").into());
            }
            // No NUL yet: a short name leaves what was there after it.
            for (k, &b) in name.iter().enumerate() {
                race.shared.tyre_change.record_name.set(k, b);
            }
            if !drawn_again(std::array::from_fn(|k| {
                race.shared.tyre_change.record_name.get(k)
            })) {
                break name.len();
            }
            draws += 1;
            if draws == MOST_DRAWS {
                return Err(BadInput("no driver's name that can be taken").into());
            }
        };
        race.shared.tyre_change.record_name.set(end, 0);
        race.shared.tyre_change.record.set(TYRE_CHANGE_RECORD);
    }

    for slot in 0..SLOTS {
        outside(Call::ReadSavegameSlot(slot))?;
    }
    number_empty_names(&saves().save_games.names, SAVEGAME_NAME_ID)?;
    for slot in 0..SLOTS {
        outside(Call::ReadReplaySlot(slot))?;
    }
    number_empty_names(&race.shared.replays.names, REPLAY_NAME_ID)?;
    for slot in 0..SLOTS {
        outside(Call::ReadSetupSlot(slot))?;
    }
    number_empty_names(&service().set_ups.names, SETUP_NAME_ID)?;
    Ok(())
}

/// The calls the loading makes, done by the game's Rust functions.
fn in_game(call: Call) -> Result<u32, Refused> {
    let race = crate::race::owner::race();
    let menus = menus();
    match call {
        Call::LoadOptions => {
            crate::saves::load_options::load_game_options(menus)?;
        }
        Call::LoadBestTimes => {
            crate::saves::load_best_times::load_game_best_times(crate::saves::owner::saves())?;
        }
        Call::ReadSettings => crate::settings::read_settings::read_settings()?,
        Call::ReadSku => {
            crate::install::read_sku::read_sku(
                crate::install::owner::install(),
                &crate::install::owner::install().sku.path.text(),
            )?;
        }
        Call::ReadDriverNames => crate::settings::read_driver_names::read_driver_names(
            menus,
            crate::settings::read_driver_names::path(),
        )?,
        Call::ApplyPatches => crate::physics::apply_patches::apply_patches(
            &race.shared.physics_tables,
            &race.course.numbers,
            &race.shared.work,
        ),
        Call::RemoveSecretCars => crate::menu::remove_secret_cars::remove_secret_cars(menus)?,
        Call::ApplyCheats => crate::menu::apply_cheats::apply_cheats(
            menus,
            &mut |line| {
                crate::logfile::log_write::log_nul(line, true, 1);
            },
            race,
        )?,
        Call::UserName => {
            if let Some(mut name) = crate::os::user_name(USER_NAME_ROOM) {
                name.push(0);
                menus
                    .play
                    .names
                    .player_names
                    .write_at(0, &name, "the user's name")?;
            }
        }
        Call::ComputerName => {
            if let Some(name) = crate::system::check_developer_machine::computer_name() {
                net()
                    .vars
                    .machine_name
                    .write_at(0, &name, "the computer's name")?;
            }
        }
        Call::RandomBelow(limit) => return Ok(crate::math::random_below::random_below(limit)?),
        Call::ReadSavegameSlot(slot) => {
            crate::saves::read_savegame_slot::read_savegame_slot(slot)?;
        }
        Call::ReadReplaySlot(slot) => {
            crate::saves::read_replay_slot::read_replay_slot(slot, race)?;
        }
        Call::ReadSetupSlot(slot) => {
            crate::service::read_setup_slot::read_setup_slot(service(), slot)?;
        }
    }
    Ok(0)
}

/// Loads with the game's own calls.
pub fn load_in_game(system: &crate::system::owner::System) -> Result<(), Refused> {
    load_game_data(&system.start, &mut in_game)
}

/// How many times the game was run (IDA 0x518BB0 .. 0x518BB4, the image's layout).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct RunCount {
    /// 0x518BB0: `g_runCount`: the game's runs counted (`system::load_game_data`; the save file keeps it).
    pub runs: Dword,
}

impl RunCount {
    pub const fn new() -> Self {
        RunCount {
            runs: Dword::new(0x0),
        }
    }
}

impl RunCount {
    pub const START: usize = 0x518BB0;
    pub const LEN: usize = 0x4;
    #[cfg(test)]
    /// Where the byte IDA calls `START` is (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const crate::system::owner::system().start.run_count)
            as usize
    }
}

#[cfg(test)]
mod run_count_tests {
    use super::*;

    #[test]
    fn runcount_starts_as_the_games() {
        let size = size_of::<RunCount>();
        assert!(RunCount::LEN <= size && size <= (RunCount::LEN).next_multiple_of(4));
        let image = crate::moved::image_bytes(RunCount::START, RunCount::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(RunCount::home()).cast_const(),
                RunCount::LEN,
            )
        };
        assert_eq!(mine, &image[..]);
    }
}
