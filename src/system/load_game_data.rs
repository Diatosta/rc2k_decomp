//! Loads what the game keeps between runs, and gives every name that is empty a default. In
//! order: the options, the best times and the settings (again: the settings file outranks the
//! kept options), the SKU, the run count + 1, the driver names and the physics patches; by
//! `allsecretcars`, 0 removes the secret cars and 1 sets the cheats' bits 0x1FBC; an empty first
//! player's name becomes the user's name (its printable part, else string 0x18C), the other three
//! players' strings 0x18D .. 0x18F; an empty machine name the computer's name, else string 0x14D;
//! an empty tyre-change record name a random one of the 180 strings from 0x5A5 (drawn again while
//! it starts "andr"), its time 0x5208 (210.00 s); every empty saved game's, replay's and car
//! setup's name string 0x4E0, 0x4E2 or 0x4E1 and the slot's number in two digits.
//! Refuses a default name that does not fit its place, and what its parts refuse.

use crate::errors::{BadInput, Refused};
use crate::menu::name_lists::NAME_LEN;
use crate::owners::Owners;
use crate::strings::append_string_by_id::push_string;
use crate::strings::copy_printable::printable_from;

use rc2k_core::system::game_data::{
    ALL_CARS_CHEATS, DRIVER_NAME_IDS, MACHINE_NAME_ID, PLAYER_NAME_IDS, REPLAY_NAME_ID,
    SAVEGAME_NAME_ID, SETUP_NAME_ID, SLOTS, TYRE_CHANGE_RECORD, USER_NAME_ROOM, all_secret_cars,
    drawn_again, removes_secret_cars,
};

const MACHINE_NAME_ROOM: usize = 0x40;
/// More draws than a name can need.
/// The computer's name with its NUL (room for `MACHINE_NAME_ROOM` bytes), if it can be had.
fn computer_name() -> Option<Vec<u8>> {
    let mut name = crate::platform::os::computer_name(MACHINE_NAME_ROOM as u32)?;
    name.push(0);
    Some(name)
}

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
    o: Owners<'static>,
    to: &impl crate::memory::session::ByteBuffer,
    at: usize,
    room: usize,
    id: u32,
    after: &[u8],
) -> Result<(), BadInput> {
    let mut string = Vec::new();
    push_string(
        crate::strings::append_string_by_id::StringOwners::of(o),
        &mut string,
        id,
    )?;
    let name = rc2k_core::system::game_data::default_name(&string, after, room)
        .ok_or(BadInput("a default name longer than its place"))?;
    to.write_bytes_at(at, &name, "a default name")?;
    Ok(())
}

/// A list of 100 names: every empty one becomes the string and the slot's number.
fn number_empty_names(
    o: Owners<'static>,
    names: &impl crate::memory::session::ByteBuffer,
    id: u32,
) -> Result<(), BadInput> {
    for slot in 0..SLOTS {
        let name = slot as usize * NAME_LEN;
        if names.get_byte(name) == 0 {
            let mut string = Vec::new();
            push_string(
                crate::strings::append_string_by_id::StringOwners::of(o),
                &mut string,
                id,
            )?;
            let bytes = rc2k_core::system::game_data::slot_name(&string, slot, NAME_LEN)
                .ok_or(BadInput("a default name longer than its place"))?;
            // The name bytes at their slot in the names buffer.
            names.write_bytes_at(name, &bytes, "a default slot name")?;
        }
    }
    Ok(())
}

pub fn load_game_data(o: Owners<'static>, outside: Outside) -> Result<(), Refused> {
    let start = &o.system.start();
    let race = o.race;
    let net = o.net;
    let play = &o.menus.play;
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
    if removes_secret_cars(play.cheats.read().all_secret_cars) {
        outside(Call::RemoveSecretCars)?;
    }
    if all_secret_cars(play.cheats.read().all_secret_cars) {
        play.cheats.write().cheats |= ALL_CARS_CHEATS;
        outside(Call::ApplyCheats)?;
    }

    for (player, id) in PLAYER_NAME_IDS.into_iter().enumerate() {
        // The player's name bytes at their slot in the names buffer.
        if play.slots.names.player_names.get(player * NAME_LEN) != 0 {
            continue;
        }
        if player == 0 {
            let field = &play.slots.names.player_names;
            outside(Call::UserName)?;
            let bytes = printable_from(&field.read(), USER_NAME_ROOM)?;
            field.write_at(0, &bytes, "a printable name")?;
            if play.slots.names.player_names.get(player * NAME_LEN) != 0 {
                continue;
            }
        }
        default_name(
            o,
            &play.slots.names.player_names,
            player * NAME_LEN,
            NAME_LEN,
            id,
            &[],
        )?;
    }

    if net.setup().machine_name.get(0) == 0 {
        let field = &net.setup().machine_name;
        outside(Call::ComputerName)?;
        let bytes = printable_from(&field.read(), MACHINE_NAME_ROOM as u32)?;
        field.write_at(0, &bytes, "a printable name")?;
        if net.setup().machine_name.get(0) == 0 {
            default_name(
                o,
                &net.setup().machine_name,
                0,
                MACHINE_NAME_ROOM,
                MACHINE_NAME_ID,
                &[],
            )?;
        }
    }

    if race.shared.tyre_change().record_name[0] == 0 {
        let mut draws = 0;
        let end = loop {
            let drawn = outside(Call::RandomBelow(DRIVER_NAME_IDS.1))?;
            let mut name = Vec::new();
            push_string(
                crate::strings::append_string_by_id::StringOwners::of(o),
                &mut name,
                DRIVER_NAME_IDS.0.wrapping_add(drawn),
            )?;
            if name.len() >= NAME_LEN {
                return Err(BadInput("a driver's name longer than its place").into());
            }
            // No NUL yet: a short name leaves what was there after it.
            race.shared.tyre_change_mut().record_name[..name.len()].copy_from_slice(&name);
            let first: [u8; 4] = race.shared.tyre_change().record_name[..4]
                .try_into()
                .expect("four bytes");
            if !drawn_again(first) {
                break name.len();
            }
            draws += 1;
            if draws == MOST_DRAWS {
                return Err(BadInput("no driver's name that can be taken").into());
            }
        };
        let mut tyre = race.shared.tyre_change_mut();
        tyre.record_name[end] = 0;
        tyre.record = TYRE_CHANGE_RECORD;
    }

    for slot in 0..SLOTS {
        outside(Call::ReadSavegameSlot(slot))?;
    }
    number_empty_names(o, &o.saves.save_games.names, SAVEGAME_NAME_ID)?;
    for slot in 0..SLOTS {
        outside(Call::ReadReplaySlot(slot))?;
    }
    number_empty_names(o, &race.shared.replays().names, REPLAY_NAME_ID)?;
    for slot in 0..SLOTS {
        outside(Call::ReadSetupSlot(slot))?;
    }
    number_empty_names(o, &o.menus.service.set_ups.names, SETUP_NAME_ID)?;
    Ok(())
}

/// The calls the loading makes, done by the game's Rust functions.
fn in_game(o: Owners<'static>, call: Call) -> Result<u32, Refused> {
    let race = o.race;
    let menus = o.menus;
    match call {
        Call::LoadOptions => {
            crate::saves::options_files::load(o)?;
        }
        Call::LoadBestTimes => {
            crate::saves::load_best_times::load_game_best_times(o.saves, o.system)?;
        }
        Call::ReadSettings => crate::settings::read_settings::read_settings(o)?,
        Call::ReadSku => {
            crate::install::read_sku::read_sku(o.system, crate::install::read_sku::SKU_PATH)?;
        }
        Call::ReadDriverNames => {
            if let Some(names) = crate::settings::read_driver_names::read_driver_names(
                o.system,
                crate::settings::read_driver_names::path(),
            )? {
                crate::settings::read_driver_names::apply_driver_names(menus, o.strings, &names)?;
            }
        }
        Call::ApplyPatches => crate::physics::apply_patches::apply_patches(
            race.shared.physics_tables.get(),
            race.course.numbers.get(),
            race.shared.work.get(),
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
            if let Some(mut name) = crate::platform::os::user_name(USER_NAME_ROOM) {
                name.push(0);
                menus
                    .play
                    .slots
                    .names
                    .player_names
                    .write_at(0, &name, "the user's name")?;
            }
        }
        Call::ComputerName => {
            if let Some(name) = computer_name() {
                o.net
                    .setup()
                    .machine_name
                    .write_at(0, &name, "the computer's name")?;
            }
        }
        Call::RandomBelow(limit) => return Ok(o.race.shared.random.random_below(limit)?),
        Call::ReadSavegameSlot(slot) => {
            crate::saves::read_savegame_slot::read_savegame_slot_in(o.saves, slot)?;
        }
        Call::ReadReplaySlot(slot) => {
            crate::saves::read_replay_slot::read_replay_slot(slot, race)?;
        }
        Call::ReadSetupSlot(slot) => {
            crate::service::read_setup_slot::read_setup_slot(&o.menus.service, slot)?;
        }
    }
    Ok(0)
}

/// Loads with the game's own calls.
pub fn load_in_game(o: Owners<'static>) -> Result<(), Refused> {
    load_game_data(o, &mut |call| in_game(o, call))
}

/// How many times the game was run.
pub struct RunCount {
    /// The game's runs counted (the save file keeps it).
    pub runs: std::cell::Cell<u32>,
}

impl RunCount {
    pub const fn new() -> Self {
        RunCount {
            runs: std::cell::Cell::new(0x0),
        }
    }
}
