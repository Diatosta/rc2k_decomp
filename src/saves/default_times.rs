//! The made-up entries of the best times: `default_time_entry` and `guess_checkpoint_times`.
//!
//! An entry is 0x9C bytes. `default_time_entry` gives it a random driver (a number under 0xAA):
//! the driver's class to the entry's `group`, `nation`, `car`, the id of the
//! name's string to `name_id`, the name (without a NUL) to `name`, and 0xB to `unread_50`. While the
//! checkpoint switch is 1 it first guesses the stage's checkpoint times from the entry's time
//! and the stage, writes some twenty-six pieces to the game's log about it
//! ("guesscheckpointtimes:start" ..), and copies the sixteen times to `checkpoints`.
//! `guess_checkpoint_times`: the sixteen dwords of `guessed_checkpoints` = 0; for a stage up to
//! 0x24, with `checkpoint_counts[stage]` checkpoints, checkpoint n's time is the time *
//! (n + 1) / the count. Everything is worked out before anything is written (`GameSource` and
//! `apply` here).

use crate::errors::BadInput;
use crate::menu::owner::{Menus, Screens};
use crate::race::owner::Race;
use crate::records::stages_store::{RecordRef, RecordTable};
use crate::saves::drivers::Driver;
use rc2k_core::Invalid;
use rc2k_core::saves::defaults::{Filled, Record};

/// A piece written to the game's log: a text or a number, with or without a new line after it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Piece {
    Text(&'static std::ffi::CStr, bool),
    Number(u32, bool),
}

pub type Log<'a> = &'a mut dyn FnMut(Piece);

/// The log pieces as the game's log functions write them.
pub fn to_the_log(piece: Piece) {
    match piece {
        Piece::Text(text, newline) => {
            crate::logfile::log_write::log_nul(text.to_bytes(), newline, 1);
        }
        Piece::Number(number, newline) => {
            crate::logfile::log_int::log_int(number as i32, newline, 1);
        }
    }
}

pub(super) fn to_nowhere(_: Piece) {}

/// The game's side of the core's functions (`rc2k_core::saves::defaults`): its random
/// numbers, its drivers, its checkpoint counts and its log (or none).
pub(super) struct GameSource<'a> {
    pub log: Log<'a>,
    pub so: crate::strings::append_string_by_id::StringOwners<'a>,
    pub menus: &'a Menus,
    pub race: &'a Race,
}

impl rc2k_core::saves::defaults::Source for GameSource<'_> {
    fn below(&mut self, limit: u32) -> Result<u32, Invalid> {
        self.race
            .shared
            .random
            .random_below(limit)
            .map_err(|e| Invalid(e.0))
    }

    fn driver(&mut self, number: u32) -> Result<rc2k_core::saves::defaults::Driver, Invalid> {
        let driver = Driver::of(self.menus, number).map_err(|e| Invalid(e.0))?;
        let (name_id, name) = driver.name(self.so).map_err(|e| Invalid(e.0))?;
        Ok(rc2k_core::saves::defaults::Driver {
            fields: [driver.class(), driver.nation(), driver.car()],
            name_id: rc2k_core::saves::name_id::NameId::String(name_id),
            name,
        })
    }

    fn checkpoint_count(&mut self, stage: u32) -> u32 {
        self.race.shared.work.get().checkpoint_counts[stage as usize].get()
    }

    fn log(&mut self, piece: rc2k_core::saves::defaults::Piece) {
        use rc2k_core::saves::defaults::{Piece as P, Text};
        let text = |t| -> &'static std::ffi::CStr {
            match t {
                Text::Dashes => c"-----------------------------------------------------",
                Text::Start => c"guesscheckpointtimes:start",
                Text::Passing => c"passing:(time:ebx)",
                Text::Stage => c"(stage:eax)",
                Text::CheckData => c"testcheckdata",
                Text::Done => c"guesscheckpointtimes:done",
                Text::DashesAfter => c"-----------------------------------------------------",
            }
        };
        (self.log)(match piece {
            P::Text(t, newline) => Piece::Text(text(t), newline),
            P::Number(n, newline) => Piece::Number(n, newline),
        });
    }
}

/// What the made-up times read of `record`: the first and last classes' times, the event
/// slot.
pub fn read_record(
    stages: &crate::records::stages_store::StageTables,
    record: RecordRef,
) -> Result<Record, BadInput> {
    let r = record.record(stages)?;
    let times = r.class_times();
    Ok(Record {
        tag: r.tag(),
        track: r.track(),
        length: times[0],
        time: times[3],
        picked: r.event_slot(),
    })
}

/// Where a function of the time tables writes: a typed stage table, or the game's memory
/// (a rally's table, `RALLY_TIMES`).
#[derive(Clone, Copy)]
pub enum Target<'a> {
    Stages(&'a crate::memory::owned::Owned<rc2k_core::saves::entries::StageTimes>),
    Rally(&'a [crate::saves::best_times_store::TimeEntry; 20]),
}

/// Writes what a function of the time tables worked out: its blocks into `tables` (the
/// function's own, then the rallies'), and the work variables. Refuses a block past its
/// table.
pub fn apply(
    screens: &Screens,
    race: &Race,
    filled: &Filled,
    tables: &[Target<'_>],
) -> Result<(), BadInput> {
    {
        for block in &filled.blocks {
            match tables[block.table] {
                Target::Stages(t) => t.with(|t| {
                    let all = t.stages.as_flattened_mut();
                    for (n, e) in block.entries.iter().enumerate() {
                        all.get_mut(block.first + n)
                            .ok_or(BadInput("a made-up time past its table"))?
                            .fill_made_up(e, block.time_at == 0x18);
                    }
                    Ok::<_, BadInput>(())
                })??,
                Target::Rally(table) => {
                    for (n, e) in block.entries.iter().enumerate() {
                        table
                            .get(block.first + n)
                            .ok_or(BadInput("a made-up time past its table"))?
                            .fill_made_up(e, block.time_at)?;
                    }
                }
            }
        }
        if let Some(sum) = filled.sum {
            screens.draw_work.write().button_x = sum;
        }
        if let Some(c) = filled.checkpoints {
            for (cell, v) in race.shared.work.get().guessed_checkpoints.iter().zip(c) {
                cell.set(v);
            }
        }
    }
    Ok(())
}

/// How many records a table has before its -1 (the first is not looked at for the end).
pub fn records(
    stages: &crate::records::stages_store::StageTables,
    table: RecordTable,
) -> Result<usize, BadInput> {
    for n in 1..=0x1000u32 {
        if table.row(stages, n)?.tag(stages) == u32::MAX {
            return Ok(n as usize);
        }
    }
    Err(BadInput("a record table without its end"))
}
