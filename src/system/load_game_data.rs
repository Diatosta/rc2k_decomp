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
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::menu::name_lists::{NAME_LEN, REPLAY_NAMES_RVA, SAVEGAME_NAMES_RVA};
use crate::menu::remove_secret_cars::CHEATS_RVA;
use crate::strings::append_string_by_id::push_string;
use crate::strings::copy_printable::copy_printable;
use windows_sys::Win32::System::WindowsProgramming::GetUserNameA;

// ---- game data ----

const RUNS_RVA: usize = rva(0x518BB0); // dword_518BB0
const ALL_SECRET_CARS_RVA: usize = rva(0x758710); // g_allSecretCars
const PLAYER_NAMES_RVA: usize = rva(0x755784); // byte_755784, four of 0x20 bytes
const MACHINE_NAME_RVA: usize = rva(0x52AB90); // byte_52AB90
const MACHINE_NAME_ROOM: usize = 0x40;
const DRIVER_NAME_RVA: usize = rva(0x5BEC08); // dword_5BEC08
const TYRE_CHANGE_RECORD_RVA: usize = rva(0x5BEC28); // g_tyreChangeRecord (hundredths)
const SETUP_NAMES_RVA: usize = rva(0x527880); // byte_527880
/// More draws than a name can need.
const MOST_DRAWS: usize = 10_000;

// ---- Rust version ----

// Converted: `rc2k_core::system::game_data` (the choices, the default names, the drawn
// driver's name).
use rc2k_core::system::game_data::{
    ALL_CARS_CHEATS, DRIVER_NAME_IDS, MACHINE_NAME_ID, PLAYER_NAME_IDS, REPLAY_NAME_ID,
    SAVEGAME_NAME_ID, SETUP_NAME_ID, SLOTS, TYRE_CHANGE_RECORD, USER_NAME_ROOM, all_secret_cars,
    drawn_again, removes_secret_cars,
};

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
unsafe fn default_name(to: usize, room: usize, id: u32, after: &[u8]) -> Result<(), BadInput> {
    unsafe {
        let mut string = Vec::new();
        push_string(&mut string, id);
        let name = rc2k_core::system::game_data::default_name(&string, after, room)
            .ok_or(BadInput("a default name longer than its place"))?;
        std::ptr::copy(name.as_ptr(), game_addr(to) as *mut u8, name.len());
        Ok(())
    }
}

/// A list of 100 names: every empty one becomes the string and the slot's number.
unsafe fn number_empty_names(names: usize, id: u32) -> Result<(), BadInput> {
    unsafe {
        for slot in 0..SLOTS {
            let name = names + slot as usize * NAME_LEN;
            if game_read::<u8>(name) == 0 {
                let mut string = Vec::new();
                push_string(&mut string, id);
                let bytes = rc2k_core::system::game_data::slot_name(&string, slot, NAME_LEN)
                    .ok_or(BadInput("a default name longer than its place"))?;
                std::ptr::copy(bytes.as_ptr(), game_addr(name) as *mut u8, bytes.len());
            }
        }
        Ok(())
    }
}

pub unsafe fn load_game_data(outside: Outside) -> Result<(), Refused> {
    unsafe {
        outside(Call::LoadOptions)?;
        outside(Call::LoadBestTimes)?;
        outside(Call::ReadSettings)?;
        outside(Call::ReadSku)?;
        game_write(RUNS_RVA, game_read::<u32>(RUNS_RVA).wrapping_add(1));
        outside(Call::ReadDriverNames)?;
        outside(Call::ApplyPatches)?;
        if removes_secret_cars(game_read(ALL_SECRET_CARS_RVA)) {
            outside(Call::RemoveSecretCars)?;
        }
        if all_secret_cars(game_read(ALL_SECRET_CARS_RVA)) {
            game_write(CHEATS_RVA, game_read::<u32>(CHEATS_RVA) | ALL_CARS_CHEATS);
            outside(Call::ApplyCheats)?;
        }

        for (player, id) in PLAYER_NAME_IDS.into_iter().enumerate() {
            let name = PLAYER_NAMES_RVA + player * NAME_LEN;
            if game_read::<u8>(name) != 0 {
                continue;
            }
            if player == 0 {
                let place = game_addr(name) as *mut u8;
                outside(Call::UserName(place as u32))?;
                copy_printable(place, place, USER_NAME_ROOM)?;
                if game_read::<u8>(name) != 0 {
                    continue;
                }
            }
            default_name(name, NAME_LEN, id, &[])?;
        }

        if game_read::<u8>(MACHINE_NAME_RVA) == 0 {
            let place = game_addr(MACHINE_NAME_RVA) as *mut u8;
            outside(Call::ComputerName(place as u32))?;
            copy_printable(place, place, MACHINE_NAME_ROOM as u32)?;
            if game_read::<u8>(MACHINE_NAME_RVA) == 0 {
                default_name(MACHINE_NAME_RVA, MACHINE_NAME_ROOM, MACHINE_NAME_ID, &[])?;
            }
        }

        if game_read::<u8>(DRIVER_NAME_RVA) == 0 {
            let mut draws = 0;
            let end = loop {
                let drawn = outside(Call::RandomBelow(DRIVER_NAME_IDS.1))?;
                let mut name = Vec::new();
                push_string(&mut name, DRIVER_NAME_IDS.0.wrapping_add(drawn));
                if name.len() >= NAME_LEN {
                    return Err(BadInput("a driver's name longer than its place").into());
                }
                // No NUL yet: a short name leaves what was there after it.
                std::ptr::copy(
                    name.as_ptr(),
                    game_addr(DRIVER_NAME_RVA) as *mut u8,
                    name.len(),
                );
                if !drawn_again(game_read(DRIVER_NAME_RVA)) {
                    break name.len();
                }
                draws += 1;
                if draws == MOST_DRAWS {
                    return Err(BadInput("no driver's name that can be taken").into());
                }
            };
            game_write(DRIVER_NAME_RVA + end, 0u8);
            game_write(TYRE_CHANGE_RECORD_RVA, TYRE_CHANGE_RECORD);
        }

        for slot in 0..SLOTS {
            outside(Call::ReadSavegameSlot(slot))?;
        }
        number_empty_names(SAVEGAME_NAMES_RVA, SAVEGAME_NAME_ID)?;
        for slot in 0..SLOTS {
            outside(Call::ReadReplaySlot(slot))?;
        }
        number_empty_names(REPLAY_NAMES_RVA, REPLAY_NAME_ID)?;
        for slot in 0..SLOTS {
            outside(Call::ReadSetupSlot(slot))?;
        }
        number_empty_names(SETUP_NAMES_RVA, SETUP_NAME_ID)?;
        Ok(())
    }
}

/// The calls done by the Rust versions.
fn in_game(call: Call) -> Result<u32, Refused> {
    unsafe {
        match call {
            Call::LoadOptions => {
                crate::saves::load_options::load_game_options()?;
            }
            Call::LoadBestTimes => {
                crate::saves::load_best_times::load_game_best_times()?;
            }
            Call::ReadSettings => crate::settings::read_settings::read_settings()?,
            Call::ReadSku => crate::install::read_sku::read_sku(game_addr(
                crate::install::read_sku::PATH_RVA,
            ) as *const u8)?,
            Call::ReadDriverNames => crate::settings::read_driver_names::read_driver_names(
                crate::settings::read_driver_names::path(),
            )?,
            Call::ApplyPatches => crate::physics::apply_patches::apply_patches(),
            Call::RemoveSecretCars => crate::menu::remove_secret_cars::remove_secret_cars()?,
            Call::ApplyCheats => crate::menu::apply_cheats::apply_cheats(&mut |line| {
                crate::menu::text_entry::log_l1(line, true)
            })?,
            Call::UserName(to) => {
                let mut room = USER_NAME_ROOM;
                GetUserNameA(to as *mut u8, &mut room);
            }
            Call::ComputerName(to) => {
                crate::system::check_developer_machine::computer_name(to as *mut u8);
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
    }
    Ok(0)
}

/// The Rust version on the game's own calls.
pub fn load_in_game() -> Result<(), Refused> {
    unsafe { load_game_data(&mut in_game) }
}
