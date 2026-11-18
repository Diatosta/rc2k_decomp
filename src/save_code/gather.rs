//! The player's state as it is now, read out of the game: the options in the stores, the best
//! times' tables, and the car set-up and saved game files.

use crate::errors::Refused;
use crate::files::numbered_path::numbered_path_bytes;
use crate::files::read_file::read_file_into;
use crate::owners::Owners;
use rc2k_core::saves::code::best::{self, DriverNames};
use rc2k_core::saves::code::{
    BestTimes, GAME_STATE_LEN, Game, NAME_LEN, PlayerState, SETUP_VALUES, SLOTS, Setup, settings,
};
use rc2k_core::saves::kept;

/// The tables of `var\hst.dat` the code carries, in the file's order (`BestTables::typed`).
pub const FILE_TABLES: [usize; 6] = [1, 2, 3, 4, 7, 8];
/// A saved game's file.
const GAME_FILE_LEN: usize = 0x8000;
const GAME_HEAD_LEN: usize = 0x30;

/// The names of the game's 180 drivers, which its made-up best times carry.
pub fn driver_names(o: Owners<'static>) -> Result<DriverNames, Refused> {
    let so = crate::strings::append_string_by_id::StringOwners::of(o);
    let mut names = Vec::new();
    for n in 0..crate::saves::drivers::DRIVERS {
        names.push(
            crate::saves::drivers::Driver::of(o.menus, n)?
                .name(so, o.menus)?
                .1,
        );
    }
    Ok(DriverNames(names))
}

/// The six tables' bytes, if the best times are loaded.
pub fn current_tables(o: Owners<'static>) -> Result<Option<[Vec<u8>; 6]>, Refused> {
    let times = &o.saves.best_times;
    if [times.allocated.get(), times.loaded.get()] != [1, 1] {
        return Ok(None);
    }
    let mut tables: [Vec<u8>; 6] = Default::default();
    for (table, n) in tables.iter_mut().zip(FILE_TABLES) {
        *table = o.saves.best_tables.typed(n).bytes()?;
    }
    Ok(Some(tables))
}

fn now() -> u32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as u32)
}

/// The car set-ups the files hold, by slot.
pub fn setups(o: Owners<'static>) -> Result<Vec<Setup>, Refused> {
    let template = o.menus.service.template.read();
    let mut out = Vec::new();
    for slot in 0..SLOTS as u32 {
        let path = numbered_path_bytes(&template[..template.len() - 1], slot)?;
        let mut file = [0u8; kept::SETUP_LEN];
        let read = read_file_into(&path[..], &mut file, 0)?;
        if !read.ok {
            continue;
        }
        let Some(name) = kept::setup_name(&file) else {
            continue;
        };
        let mut values = [0u32; SETUP_VALUES];
        for (n, v) in values.iter_mut().enumerate() {
            *v = u32::from_le_bytes(
                file[0x30 + 4 * n..0x34 + 4 * n]
                    .try_into()
                    .expect("4 bytes"),
            );
        }
        out.push(Setup {
            slot: slot as u8,
            name,
            values,
        });
    }
    Ok(out)
}

/// The saved games the files hold, by slot.
pub fn games(o: Owners<'static>) -> Result<Vec<Game>, Refused> {
    let template = o.saves.files.savegame_template.read();
    let mut out = Vec::new();
    let mut file = vec![0u8; GAME_FILE_LEN];
    for slot in 0..SLOTS as u32 {
        let path = numbered_path_bytes(&template[..template.len() - 1], slot)?;
        file.fill(0);
        let read = read_file_into(&path[..], &mut file, 0)?;
        if !read.ok {
            continue;
        }
        let Some(name) = kept::slot_name(&file, kept::SAVEGAME, kept::SAVEGAME_VERSION) else {
            continue;
        };
        out.push(Game {
            slot: slot as u8,
            name,
            state: file[GAME_HEAD_LEN..GAME_HEAD_LEN + GAME_STATE_LEN].to_vec(),
        });
    }
    Ok(out)
}

/// The whole state now.
pub fn gather(o: Owners<'static>) -> Result<PlayerState, Refused> {
    let options = crate::saves::options_state::capture(o)?;
    let names = driver_names(o)?;
    let best = match current_tables(o)? {
        Some(tables) => {
            let rallies: Vec<&[u8]> = options.records.rallies.iter().map(|b| &b.0[..]).collect();
            let events: Vec<&[u8]> = options.records.events.iter().map(|b| &b.0[..]).collect();
            let players: Vec<Vec<u8>> = options.players.names.iter().map(|n| n.0.clone()).collect();
            let mut tyre_name = [0u8; NAME_LEN];
            tyre_name.copy_from_slice(&options.records.tyre_change_holder.0[..NAME_LEN]);
            Some(BestTimes {
                real: best::extract(&tables, &names),
                rallies: best::extract_rallies(&rallies, &names),
                records: best::extract_records(&options.records.championship.0, &events, &players),
                tyre_change: Some((tyre_name, options.records.tyre_change_time)),
            })
        }
        None => None,
    };
    Ok(PlayerState {
        made: now(),
        program: env!("CARGO_PKG_VERSION").to_owned(),
        settings: Some(
            settings::text_of(&options).map_err(|_| {
                crate::errors::BadInput("the settings could not be put into a code")
            })?,
        ),
        best,
        setups: Some(setups(o)?),
        games: Some(games(o)?),
    })
}
