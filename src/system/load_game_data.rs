//! `system::load_game_data` (0x411239): loads what the game keeps between runs, and gives
//! every name that is empty a default.
//!
//! In:  nothing
//! Out: every register preserved
//! In this order:
//! - `saves::load_options`, `saves::load_best_times`, `settings::read_settings` (again:
//!   the settings file outranks the kept options), `install::read_sku`, `dword_518BB0`
//!   + 1, `settings::read_driver_names`, `physics::apply_patches`;
//! - by the setting `allsecretcars` (`g_allSecretCars`): 0 `menu::remove_secret_cars`; 1
//!   the cheats' bits 0x1FBC set and `menu::apply_cheats`; anything else neither;
//! - the first player's name (`byte_755784`), if empty: the user's name
//!   (`system::user_name`, 0x41085A: `GetUserNameA`, 0x1F bytes) cut down by
//!   `strings::copy_printable` in place, and if that leaves nothing, string 0x18C; the
//!   other three players' (0x20 bytes apart), if empty: strings 0x18D .. 0x18F;
//! - the machine's name (`byte_52AB90`), if empty: the computer's name
//!   (`system::computer_name`, 0x40 bytes) the same way, else string 0x14D;
//! - the name at `dword_5BEC08`, if empty: a random one of the 180 strings from 0x5A5 on
//!   (`math::random_below`), drawn again while what is there starts "andr", and
//!   `g_tyreChangeRecord` = 0x5208 (210.00 s, the tyre change's best time);
//! - all 100 saved games are read (`saves::read_savegame_slot`) and every slot's name
//!   that is empty becomes string 0x4E0 and the slot's number in two digits; the same for
//!   the replays (string 0x4E2) and the car setups (string 0x4E1).
//!
//! One caller, `start`.
//! Refuses a default name that does not fit its place, and what its parts refuse. A refusal
//! comes with the work half done, and the original then starts from the top.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::install::read_sku::SKU;
use crate::menu::cheats_store::CHEAT_VARS;
use crate::menu::name_lists::NAME_LEN;
use crate::menu::names_store::NAME_BUFFERS;
use crate::net::store::NET;
use crate::race::tyre_store::TYRE_CHANGE;
use crate::session::Bytes;
use crate::session::Dword;
use crate::strings::append_string_by_id::push_string;
use crate::strings::copy_printable::copy_printable;

// ---- game data ----

// ---- Rust version ----

// Converted: `rc2k_core::system::game_data` (the choices, the default names, the drawn
// driver's name).
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
    /// The user's name to this place.
    UserName(u32),
    /// The computer's name to this place.
    ComputerName(u32),
    /// A random number under this.
    RandomBelow(u32),
    ReadSavegameSlot(u32),
    ReadReplaySlot(u32),
    ReadSetupSlot(u32),
}

/// Does a call; a result only matters for `RandomBelow`.
pub type Outside<'a> = &'a mut dyn FnMut(Call) -> Result<u32, Refused>;

/// The string and what follows it, with a NUL, at `to`.
fn default_name(to: *mut u8, room: usize, id: u32, after: &[u8]) -> Result<(), BadInput> {
    let mut string = Vec::new();
    push_string(&mut string, id)?;
    let name = rc2k_core::system::game_data::default_name(&string, after, room)
        .ok_or(BadInput("a default name longer than its place"))?;
    crate::pointers::GamePtr::new(crate::pointers::game_address(to))
        .write_bytes(&name, "a default name")?;
    Ok(())
}

/// A list of 100 names: every empty one becomes the string and the slot's number.
fn number_empty_names<const N: usize>(names: &Bytes<N>, id: u32) -> Result<(), BadInput> {
    for slot in 0..SLOTS {
        let name = slot as usize * NAME_LEN;
        if names.get(name) == 0 {
            let mut string = Vec::new();
            push_string(&mut string, id)?;
            let bytes = rc2k_core::system::game_data::slot_name(&string, slot, NAME_LEN)
                .ok_or(BadInput("a default name longer than its place"))?;
            crate::pointers::GamePtr::new(
                (crate::pointers::game_address(names.as_ptr())).wrapping_add(name as u32),
            )
            .write_bytes(&bytes, "a default slot name")?;
        }
    }
    Ok(())
}

pub fn load_game_data(outside: Outside) -> Result<(), Refused> {
    outside(Call::LoadOptions)?;
    outside(Call::LoadBestTimes)?;
    outside(Call::ReadSettings)?;
    outside(Call::ReadSku)?;
    RUN_COUNT.runs.set(RUN_COUNT.runs.get().wrapping_add(1));
    outside(Call::ReadDriverNames)?;
    outside(Call::ApplyPatches)?;
    if removes_secret_cars(CHEAT_VARS.all_secret_cars.get()) {
        outside(Call::RemoveSecretCars)?;
    }
    if all_secret_cars(CHEAT_VARS.all_secret_cars.get()) {
        CHEAT_VARS
            .cheats
            .set(CHEAT_VARS.cheats.get() | ALL_CARS_CHEATS);
        outside(Call::ApplyCheats)?;
    }

    for (player, id) in PLAYER_NAME_IDS.into_iter().enumerate() {
        let name = NAME_BUFFERS.player_names.addr() + (player * NAME_LEN) as u32;
        if NAME_BUFFERS.player_names.get(player * NAME_LEN) != 0 {
            continue;
        }
        if player == 0 {
            let place = crate::pointers::GamePtr::new(name);
            outside(Call::UserName(place.addr()))?;
            copy_printable(place, place, USER_NAME_ROOM)?;
            if NAME_BUFFERS.player_names.get(player * NAME_LEN) != 0 {
                continue;
            }
        }
        default_name(crate::pointers::host_at(name as usize), NAME_LEN, id, &[])?;
    }

    if NET.machine_name.get(0) == 0 {
        let place = crate::pointers::GamePtr::to(&NET.machine_name);
        outside(Call::ComputerName(place.addr()))?;
        copy_printable(place, place, MACHINE_NAME_ROOM as u32)?;
        if NET.machine_name.get(0) == 0 {
            default_name(
                NET.machine_name.as_ptr(),
                MACHINE_NAME_ROOM,
                MACHINE_NAME_ID,
                &[],
            )?;
        }
    }

    if TYRE_CHANGE.record_name.get(0) == 0 {
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
                TYRE_CHANGE.record_name.set(k, b);
            }
            if !drawn_again(std::array::from_fn(|k| TYRE_CHANGE.record_name.get(k))) {
                break name.len();
            }
            draws += 1;
            if draws == MOST_DRAWS {
                return Err(BadInput("no driver's name that can be taken").into());
            }
        };
        TYRE_CHANGE.record_name.set(end, 0);
        TYRE_CHANGE.record.set(TYRE_CHANGE_RECORD);
    }

    for slot in 0..SLOTS {
        outside(Call::ReadSavegameSlot(slot))?;
    }
    number_empty_names(
        &crate::saves::write_savegame_slot::SAVE_GAMES.names,
        SAVEGAME_NAME_ID,
    )?;
    for slot in 0..SLOTS {
        outside(Call::ReadReplaySlot(slot))?;
    }
    number_empty_names(&crate::race::replay::REPLAYS.names, REPLAY_NAME_ID)?;
    for slot in 0..SLOTS {
        outside(Call::ReadSetupSlot(slot))?;
    }
    number_empty_names(
        &crate::service::read_setup_slot::SET_UPS.names,
        SETUP_NAME_ID,
    )?;
    Ok(())
}

/// The calls done by the Rust versions.
fn in_game(call: Call) -> Result<u32, Refused> {
    match call {
        Call::LoadOptions => {
            crate::saves::load_options::load_game_options()?;
        }
        Call::LoadBestTimes => {
            crate::saves::load_best_times::load_game_best_times()?;
        }
        Call::ReadSettings => crate::settings::read_settings::read_settings()?,
        Call::ReadSku => {
            crate::install::read_sku::read_sku(crate::pointers::GamePtr::to(&SKU.path))?
        }
        Call::ReadDriverNames => crate::settings::read_driver_names::read_driver_names(
            crate::settings::read_driver_names::path(),
        )?,
        Call::ApplyPatches => crate::physics::apply_patches::apply_patches(),
        Call::RemoveSecretCars => crate::menu::remove_secret_cars::remove_secret_cars()?,
        Call::ApplyCheats => crate::menu::apply_cheats::apply_cheats(&mut |line| {
            crate::logfile::log_write::log_nul(line, true, 1)
        })?,
        Call::UserName(to) => {
            if let Some(mut name) = crate::os::user_name(USER_NAME_ROOM) {
                name.push(0);
                crate::pointers::GamePtr::new(to).write_bytes(&name, "the user's name")?;
            }
        }
        Call::ComputerName(to) => {
            crate::system::check_developer_machine::computer_name(to);
        }
        Call::RandomBelow(limit) => return Ok(crate::math::random_below::random_below(limit)?),
        Call::ReadSavegameSlot(slot) => {
            crate::saves::read_savegame_slot::read_savegame_slot(slot)?;
        }
        Call::ReadReplaySlot(slot) => {
            crate::saves::read_replay_slot::read_replay_slot(slot)?;
        }
        Call::ReadSetupSlot(slot) => {
            crate::service::read_setup_slot::read_setup_slot(slot)?;
        }
    }
    Ok(0)
}

/// The Rust version on the game's own calls.
pub fn load_in_game() -> Result<(), Refused> {
    load_game_data(&mut in_game)
}

/// How many times the game was run (IDA 0x518BB0 .. 0x518BB4, the image's layout).
#[repr(C)]
pub struct RunCount {
    /// 0x518BB0: `g_runCount`: the game's runs counted (`system::load_game_data`; the save file keeps it).
    pub runs: Dword,
}

pub static RUN_COUNT: RunCount = RunCount {
    runs: Dword::new(0x0),
};

impl RunCount {
    pub const START: usize = 0x518BB0;
    pub const LEN: usize = 0x4;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&RUN_COUNT as *const RunCount) as usize
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
                crate::pointers::host_at(RunCount::home()) as *const u8,
                RunCount::LEN,
            )
        };
        assert_eq!(mine, &image[..]);
    }
}
