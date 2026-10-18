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
//! Shadow mode runs the original alone: its parts are compared in their own hooks.
//! Self-test: a redirected copy whose fourteen kinds of call go to the test, on names of
//! the test's in the game's places.
//! Refuses a default name that does not fit its place, and what its parts refuse. A
//! refusal comes with the work half done, and the original then starts from the top.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::menu::name_lists::{NAME_LEN, REPLAY_NAMES_RVA, SAVEGAME_NAMES_RVA};
use crate::menu::remove_secret_cars::CHEATS_RVA;
use crate::redirect::RedirectedCopy;
use crate::service::harness::{Region, at, capture, restore};
use crate::strings::append_string_by_id::push_string;
use crate::strings::copy_printable::copy_printable;
use crate::strings::format_u32::format_u32;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;
use windows_sys::Win32::System::WindowsProgramming::GetUserNameA;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("system::load_game_data", rva(0x411239), thunk, &ORIGINAL)
    .private_copy(LEN)
    .relocate_calls(&CALLS)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const RUNS_RVA: usize = rva(0x518BB0); // dword_518BB0
const ALL_SECRET_CARS_RVA: usize = rva(0x758710); // g_allSecretCars
const PLAYER_NAMES_RVA: usize = rva(0x755784); // byte_755784, four of 0x20 bytes
const PLAYER_NAME_IDS: [u32; 4] = [0x18C, 0x18D, 0x18E, 0x18F];
const MACHINE_NAME_RVA: usize = rva(0x52AB90); // byte_52AB90
const MACHINE_NAME_ROOM: usize = 0x40;
const MACHINE_NAME_ID: u32 = 0x14D;
const DRIVER_NAME_RVA: usize = rva(0x5BEC08); // dword_5BEC08
const TYRE_CHANGE_RECORD_RVA: usize = rva(0x5BEC28); // g_tyreChangeRecord (hundredths)
const DRIVER_NAME_IDS: (u32, u32) = (0x5A5, 0xB4);
const SETUP_NAMES_RVA: usize = rva(0x527880); // byte_527880
const SLOTS: u32 = 100;
const ALL_CARS_CHEATS: u32 = 0x1FBC;
const USER_NAME_ROOM: u32 = 0x1F;
/// More draws than a name can need.
const MOST_DRAWS: usize = 10_000;

// ---- Rust version ----

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
        let mut name = Vec::new();
        push_string(&mut name, id);
        name.extend_from_slice(after);
        name.push(0);
        if name.len() > room {
            return Err(BadInput("a default name longer than its place"));
        }
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
                default_name(name, NAME_LEN, id, &format_u32(slot, 0x12))?;
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
        if game_read::<u32>(ALL_SECRET_CARS_RVA) == 0 {
            outside(Call::RemoveSecretCars)?;
        }
        if game_read::<u32>(ALL_SECRET_CARS_RVA) == 1 {
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
                if game_read::<u32>(DRIVER_NAME_RVA) != u32::from_le_bytes(*b"andr") {
                    break name.len();
                }
                draws += 1;
                if draws == MOST_DRAWS {
                    return Err(BadInput("no driver's name that can be taken").into());
                }
            };
            game_write(DRIVER_NAME_RVA + end, 0u8);
            game_write(TYRE_CHANGE_RECORD_RVA, 0x5208u32);
        }

        for slot in 0..SLOTS {
            outside(Call::ReadSavegameSlot(slot))?;
        }
        number_empty_names(SAVEGAME_NAMES_RVA, 0x4E0)?;
        for slot in 0..SLOTS {
            outside(Call::ReadReplaySlot(slot))?;
        }
        number_empty_names(REPLAY_NAMES_RVA, 0x4E2)?;
        for slot in 0..SLOTS {
            outside(Call::ReadSetupSlot(slot))?;
        }
        number_empty_names(SETUP_NAMES_RVA, 0x4E1)?;
        Ok(())
    }
}

// ---- hook ----

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
                game_addr(crate::settings::read_driver_names::PATH_RVA) as *const u8,
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

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    if let Err(e) = unsafe { load_game_data(&mut in_game) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

// ---- self-test ----

const LEN: usize = 0x20C;
const ENTRY_BYTES: [u8; 8] = [0x60, 0xE8, 0xF3, 0x98, 0x09, 0x00, 0xE8, 0x11];
const CALLS: [usize; 37] = [
    0x1, 0x6, 0xB, 0x10, 0x1B, 0x20, 0x2E, 0x46, 0x56, 0x62, 0x72, 0x77, 0x8D, 0x92, 0xA7, 0xAC,
    0xC1, 0xC6, 0xD6, 0xE2, 0xF2, 0xF7, 0x10B, 0x11B, 0x12C, 0x142, 0x166, 0x16C, 0x171, 0x187,
    0x1AB, 0x1B1, 0x1B6, 0x1CC, 0x1F0, 0x1F6, 0x1FB,
];
/// The callees that go to the test, in the order of the stand-ins' places.
const TO_THE_TEST: [usize; 14] = [
    rva(0x4AAB32), // saves::load_options
    rva(0x4AB155), // saves::load_best_times
    rva(0x4114A6), // settings::read_settings
    rva(0x411696), // install::read_sku
    rva(0x4162DB), // settings::read_driver_names
    rva(0x4648BD), // physics::apply_patches
    rva(0x471BC7), // menu::remove_secret_cars
    rva(0x480AC1), // menu::apply_cheats
    rva(0x41085A), // system::user_name
    rva(0x410848), // system::computer_name
    rva(0x4018A2), // math::random_below
    rva(0x4AB4FE), // saves::read_savegame_slot
    rva(0x41ECAD), // saves::read_replay_slot
    rva(0x4AB31E), // service::read_setup_slot
];

const REFUSED_DRAW: u32 = 20;
const REFUSED_NAME: &[u8] = b"andrzej k\0";

/// What a case gives the two names' askers and the random numbers, and what was called.
struct Script {
    user: &'static [u8],
    computer: &'static [u8],
    draws: Vec<u32>,
    drawn: usize,
    calls: Vec<(u32, u32)>,
}

static SCRIPT: Mutex<Script> = Mutex::new(Script {
    user: b"",
    computer: b"",
    draws: Vec::new(),
    drawn: 0,
    calls: Vec::new(),
});

/// A stand-in's place and registers: notes the call (with the slot, the limit) and
/// answers the three that give something.
fn handler(place: u32, r: &mut Regs) {
    let mut script = SCRIPT.lock().unwrap();
    let noted = match place {
        10 => r.eax,
        11..=13 => r.ebx,
        _ => 0,
    };
    script.calls.push((place, noted));
    match place {
        8 | 9 => {
            let name = if place == 8 {
                script.user
            } else {
                script.computer
            };
            unsafe { std::ptr::copy(name.as_ptr(), r.ebx as *mut u8, name.len()) };
        }
        10 => {
            let n = script.drawn.min(script.draws.len() - 1);
            r.eax = script.draws[n];
            script.drawn += 1;
        }
        _ => {}
    }
}

/// The Rust version's calls, through the same handler.
fn through_the_test(call: Call) -> Result<u32, Refused> {
    let (place, eax, ebx) = match call {
        Call::LoadOptions => (0, 0, 0),
        Call::LoadBestTimes => (1, 0, 0),
        Call::ReadSettings => (2, 0, 0),
        Call::ReadSku => (3, 0, 0),
        Call::ReadDriverNames => (4, 0, 0),
        Call::ApplyPatches => (5, 0, 0),
        Call::RemoveSecretCars => (6, 0, 0),
        Call::ApplyCheats => (7, 0, 0),
        Call::UserName(to) => (8, 0, to),
        Call::ComputerName(to) => (9, 0, to),
        Call::RandomBelow(limit) => (10, limit, 0),
        Call::ReadSavegameSlot(slot) => (11, 0, slot),
        Call::ReadReplaySlot(slot) => (12, 0, slot),
        Call::ReadSetupSlot(slot) => (13, 0, slot),
    };
    let mut r = Regs {
        eax,
        ebx,
        ..Regs::default()
    };
    act(place, &mut r);
    Ok(r.eax)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if crate::strings::string_table::entry_count() < 0x5A5 {
        return t.skip("string table not loaded");
    }
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x411239)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| {
                TO_THE_TEST
                    .iter()
                    .position(|&to| to == callee)
                    .map(|place| stand_in(place as u32))
            },
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let lists = [SAVEGAME_NAMES_RVA, REPLAY_NAMES_RVA, SETUP_NAMES_RVA];
    let mut regions: Vec<Region> = vec![
        at(RUNS_RVA),
        at(CHEATS_RVA),
        (game_addr(PLAYER_NAMES_RVA) as u32, 4 * NAME_LEN),
        (game_addr(MACHINE_NAME_RVA) as u32, MACHINE_NAME_ROOM),
        (game_addr(DRIVER_NAME_RVA) as u32, NAME_LEN + 4),
    ];
    regions.extend(lists.map(|list| (game_addr(list) as u32, SLOTS as usize * NAME_LEN)));
    // The pointer the string 0x5A5 + 20 is taken from, when it has one: one case puts a
    // name there that may not be taken.
    let refused_name = rva(0x95D664) + (DRIVER_NAME_IDS.0 as usize + REFUSED_DRAW as usize) * 4;
    let inputs = [at(ALL_SECRET_CARS_RVA), at(refused_name)];
    let saved = (capture(&regions), capture(&inputs));

    // (what, allsecretcars, the names there are: players, machine, driver, every third
    // slot; the user's and the computer's name; the random numbers)
    type Case = (
        &'static str,
        u32,
        bool,
        &'static [u8],
        &'static [u8],
        &'static [u32],
    );
    let cases: [Case; 8] = [
        (
            "everything empty",
            0,
            false,
            b"Some User\0",
            b"MACHINE-7\0",
            &[3],
        ),
        (
            "everything there",
            1,
            true,
            b"Some User\0",
            b"MACHINE-7\0",
            &[3],
        ),
        ("no names to be had", 2, false, b"\0", b"\0", &[0]),
        (
            "names of signs only",
            0,
            false,
            b"\x01\x02\x7F\0",
            b"\x1F\0",
            &[0xB3],
        ),
        (
            "a long user name",
            1,
            false,
            b"A user with quite a long name indeed\0",
            b"x\0",
            &[90],
        ),
        (
            "several draws",
            0xFFFF_FFFF,
            false,
            b"u\0",
            b"c\0",
            &[5, 6, 7, 8, 9, 10, 11],
        ),
        (
            "a name that is refused twice",
            1,
            false,
            b"u\0",
            b"c\0",
            &[REFUSED_DRAW, REFUSED_DRAW, 21],
        ),
        ("the last name", 0, false, b"U\0", b"C\0", &[0xB3, 1]),
    ];
    for (what, all_cars, there, user, computer, draws) in cases {
        let before = Regs::SENTINEL;
        let run = |rust: bool| unsafe {
            restore(&regions, &vec![0; saved.0.len()]);
            game_write(RUNS_RVA, 41u32);
            game_write(CHEATS_RVA, 0x4_0001u32);
            game_write(ALL_SECRET_CARS_RVA, all_cars);
            restore(&inputs[1..], &saved.1[4..]);
            if draws[0] == REFUSED_DRAW {
                game_write(refused_name, REFUSED_NAME.as_ptr() as u32);
            }
            // What follows an empty name's first byte is not looked at: left as rubbish.
            for n in 0..4 {
                let name = game_addr(PLAYER_NAMES_RVA + n * NAME_LEN) as *mut u8;
                name.add(1).write_bytes(0x55, NAME_LEN - 1);
            }
            let fill = |to: usize, text: &[u8]| {
                std::ptr::copy(text.as_ptr(), game_addr(to) as *mut u8, text.len())
            };
            if there {
                for n in 0..4 {
                    fill(PLAYER_NAMES_RVA + n * NAME_LEN, b"p\0");
                }
                fill(MACHINE_NAME_RVA, b"m\0");
                fill(DRIVER_NAME_RVA, b"d\0");
            }
            for list in lists {
                for slot in (0..SLOTS as usize).step_by(if there { 1 } else { 3 }) {
                    fill(list + slot * NAME_LEN, b"kept\0");
                }
            }
            *SCRIPT.lock().unwrap() = Script {
                user,
                computer,
                draws: draws.to_vec(),
                drawn: 0,
                calls: Vec::new(),
            };
            set_handler(handler);
            let ended = if rust {
                match load_game_data(&mut through_the_test) {
                    Ok(()) => 0,
                    Err(_) => 2,
                }
            } else {
                let mut ro = before;
                call_game(&mut ro, copy.addr());
                !ro.preserved_except(&before, 0) as u32
            };
            (
                ended,
                capture(&regions),
                SCRIPT.lock().unwrap().calls.clone(),
            )
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0 == 0, || {
            format!(
                "{what}: original ended {} with {} calls; rust ended {} with {} calls; first different call {:?}; first different byte left {:?}",
                theirs.0,
                theirs.2.len(),
                mine.0,
                mine.2.len(),
                theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b),
                theirs.1.iter().zip(&mine.1).position(|(a, b)| a != b),
            )
        });
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
