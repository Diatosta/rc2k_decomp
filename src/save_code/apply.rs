//! A state read from a code, written into the game the way the game itself saves: the options
//! into the stores and `var/rc2k.toml`, the best times into the tables and `var\hst.dat`, the
//! set-ups and saved games into their files, which the game reads again.
//!
//! `plan` makes everything and writes nothing; the question is asked on what it holds, and
//! only a yes goes on to `commit`.

use crate::errors::{BadInput, Refused};
use crate::files::numbered_path::numbered_path_bytes;
use crate::files::write_file::write_file_from;
use crate::owners::Owners;
use crate::save_code::gather;
use rc2k_core::options::Options;
use rc2k_core::saves::code::best;
use rc2k_core::saves::code::{CodeError, PlayerState, Summary, settings};
use rc2k_core::saves::kept;

/// What can stop an import before anything is written.
#[derive(Debug)]
pub enum Problem {
    Code(CodeError),
    Game(&'static str),
}

impl From<CodeError> for Problem {
    fn from(e: CodeError) -> Self {
        Self::Code(e)
    }
}

impl From<Refused> for Problem {
    fn from(_: Refused) -> Self {
        Self::Game("the game's own state could not be read")
    }
}

impl std::fmt::Display for Problem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Code(e) => e.fmt(f),
            Self::Game(why) => f.write_str(why),
        }
    }
}

/// An import ready to be written.
pub struct Plan {
    options: Option<Options>,
    tables: Option<[Vec<u8>; 6]>,
    state: PlayerState,
    /// What the code holds and what is here now, for the question.
    pub code: Summary,
    pub now: Summary,
}

/// Everything the state would change, worked out and checked; nothing is written.
pub fn plan(o: Owners<'static>, state: PlayerState) -> Result<Plan, Problem> {
    let current = crate::saves::options_state::capture(o)?;
    let now = gather::gather(o)?.summary(&current)?;
    let mut options = match &state.settings {
        Some(text) => Some(settings::options_of(text, &current)?),
        None => None,
    };
    let mut tables = None;
    if let Some(times) = &state.best {
        let names = gather::driver_names(o)?;
        let Some(mut mine) = gather::current_tables(o)? else {
            return Err(Problem::Game("the best times are not loaded"));
        };
        best::merge(&mut mine, &times.real, &names)?;
        tables = Some(mine);
        let mut theirs = options.take().unwrap_or_else(|| current.clone());
        {
            let records = &mut theirs.records;
            let mut rallies: Vec<&mut [u8]> = records
                .rallies
                .iter_mut()
                .map(|b| b.0.as_mut_slice())
                .collect();
            best::merge_rallies(&mut rallies, &times.rallies, &names)?;
            let mut events: Vec<&mut [u8]> = records
                .events
                .iter_mut()
                .map(|b| b.0.as_mut_slice())
                .collect();
            best::merge_records(&mut records.championship.0, &mut events, &times.records);
            if let Some((name, time)) = times.tyre_change {
                records.tyre_change_holder.0 = name.to_vec();
                records.tyre_change_time = time;
            }
        }
        options = Some(theirs);
    }
    let code = state.summary(&current)?;
    Ok(Plan {
        options,
        tables,
        state,
        code,
        now,
    })
}

fn now_packed() -> u32 {
    crate::system::packed_local_time::packed_local_time()
}

/// Writes the plan: the options as the options pages save them, the best times' file, the
/// set-up and saved game files, then the lists of their names read again.
pub fn commit(o: Owners<'static>, plan: Plan) -> Result<(), Refused> {
    if let Some(options) = &plan.options {
        crate::saves::options_files::apply_and_save(o, options)?;
    }
    if let Some(tables) = &plan.tables {
        for (bytes, n) in tables.iter().zip(gather::FILE_TABLES) {
            o.saves.best_tables.typed(n).set_bytes(bytes)?;
        }
        if !crate::saves::save_best_times::in_game(o.saves)? {
            return Err(BadInput("the best times file could not be written").into());
        }
    }
    let time = now_packed();
    if let Some(setups) = &plan.state.setups {
        let template = o.menus.service.template.read();
        for s in setups {
            let path = numbered_path_bytes(&template[..template.len() - 1], u32::from(s.slot))?;
            let file = kept::setup_file(time, &s.name, s.values);
            if !write_file_from(&path[..], &file)?.ok {
                return Err(BadInput("a set-up file could not be written").into());
            }
            crate::service::read_setup_slot::read_setup_slot(&o.menus.service, u32::from(s.slot))?;
        }
    }
    if let Some(games) = &plan.state.games {
        let template = o.saves.files.savegame_template.read();
        for g in games {
            let path = numbered_path_bytes(&template[..template.len() - 1], u32::from(g.slot))?;
            let mut file = vec![0u8; 0x8000];
            file[..0x30].copy_from_slice(&kept::slot_head(
                kept::SAVEGAME,
                kept::SAVEGAME_VERSION,
                time,
                &g.name,
            ));
            file[0x30..0x30 + g.state.len()].copy_from_slice(&g.state);
            if !write_file_from(&path[..], &file)?.ok {
                return Err(BadInput("a saved game file could not be written").into());
            }
            crate::saves::read_savegame_slot::read_savegame_slot_in(o.saves, u32::from(g.slot))?;
        }
    }
    Ok(())
}
