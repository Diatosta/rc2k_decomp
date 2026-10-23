//! What the four functions that make up default best times share: the entry they fill
//! (`saves::default_time_entry`, 0x47CB74), the checkpoint times it has guessed
//! (`physics::guess_checkpoint_times`, 0x4650F6), and their hooks' and self-tests'
//! machinery. Both are plain functions here.
//!
//! An entry is 0x9C bytes. `default_time_entry` gives it a random driver (a number under
//! 0xAA): the driver's +8 to the entry's +0x1C, +0x14 to +8, +0x18 to +0x10, the id of
//! the name's string to +0xC, the name (without a NUL) to +0x20, and 0xB to +0x50. While
//! `dword_755A0C` is 1 it first guesses the stage's checkpoint times from the entry's
//! time (in eax) and the stage (in ebp), writes some twenty-six pieces to the game's log
//! about it ("guesscheckpointtimes:start" ..), and copies the sixteen times to +0x5C on.
//! `guess_checkpoint_times`: the sixteen dwords at `dword_71C7C0` = 0; for a stage up to
//! 0x24, with `dword_71C730[stage]` checkpoints, checkpoint n's time is the time *
//! (n + 1) / the count.
//! Converted: `rc2k_core::saves::defaults` (everything worked out before anything is
//! written; `GameSource` and `apply` here).

use crate::errors::BadInput;
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::math::random_below::random_below;
use crate::saves::drivers::Driver;
use rc2k_core::Invalid;
use rc2k_core::saves::defaults::{Filled, Record};

pub const WITH_CHECKPOINTS_RVA: usize = rva(0x755A0C); // dword_755A0C
const CHECKPOINTS_RVA: usize = rva(0x71C7C0); // dword_71C7C0[16]
const CHECKPOINT_COUNTS_RVA: usize = rva(0x71C730); // dword_71C730[stage]
pub const WORK_A_RVA: usize = rva(0x7559AC); // seconds
pub const WORK_B_RVA: usize = rva(0x7559B0); // dword_7559B0
/// Ten entries.
pub const STAGE_LEN: u32 = 0x618;
pub const RECORD_LEN: u32 = 0xA4;

/// A piece written to the game's log: a text of the game's (its address) or a number,
/// with or without a new line after it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Piece {
    Text(u32, bool),
    Number(u32, bool),
}

pub type Log<'a> = &'a mut dyn FnMut(Piece);

/// The log pieces as the game's log functions write them.
pub fn to_the_log(piece: Piece) {
    unsafe {
        match piece {
            Piece::Text(text, newline) => {
                crate::menu::text_entry::log_l1(text as *const u8, newline)
            }
            Piece::Number(number, newline) => {
                crate::logfile::log_int::log_int(number as i32, newline, 1)
            }
        }
    }
}

pub fn to_nowhere(_: Piece) {}

/// The game's side of the core's functions (`rc2k_core::saves::defaults`): its random
/// numbers, its drivers, its checkpoint counts and its log (or none).
pub struct GameSource<'a> {
    pub log: Log<'a>,
}

impl rc2k_core::saves::defaults::Source for GameSource<'_> {
    fn below(&mut self, limit: u32) -> Result<u32, Invalid> {
        unsafe { random_below(limit) }.map_err(|e| Invalid(e.0))
    }

    fn driver(&mut self, number: u32) -> Result<rc2k_core::saves::defaults::Driver, Invalid> {
        unsafe {
            let driver = Driver::number(number).map_err(|e| Invalid(e.0))?;
            let (name_id, name) = driver.name().map_err(|e| Invalid(e.0))?;
            Ok(rc2k_core::saves::defaults::Driver {
                fields: [driver.dword(8), driver.dword(0x14), driver.dword(0x18)],
                name_id,
                name,
            })
        }
    }

    fn checkpoint_count(&mut self, stage: u32) -> u32 {
        unsafe { game_read(CHECKPOINT_COUNTS_RVA + stage as usize * 4) }
    }

    fn log(&mut self, piece: rc2k_core::saves::defaults::Piece) {
        use rc2k_core::saves::defaults::{Piece as P, Text};
        let text = |t| {
            let at = match t {
                Text::Dashes => 0x4DE6C5,
                Text::Start => 0x4DE6FB,
                Text::Passing => 0x4DE716,
                Text::Stage => 0x4DE729,
                Text::CheckData => 0x4DE735,
                Text::Done => 0x4DE743,
                Text::DashesAfter => 0x4DE75D,
            };
            game_addr(rva(at)) as u32
        };
        (self.log)(match piece {
            P::Text(t, newline) => Piece::Text(text(t), newline),
            P::Number(n, newline) => Piece::Number(n, newline),
        })
    }
}

/// The record at `at` (0xA4 bytes, unaligned).
pub unsafe fn read_record(at: u32) -> Record {
    let d = |o: u32| unsafe { ((at + o) as *const u32).read_unaligned() };
    Record {
        tag: d(0),
        track: d(4),
        length: d(0x18),
        time: d(0x24),
        picked: d(0x3C),
    }
}

/// Writes what a function of the time tables worked out: its blocks into `tables` (the
/// function's own, then the rallies'; address and length), and the work variables.
pub unsafe fn apply(filled: &Filled, tables: &[(u32, usize)]) {
    unsafe {
        for block in &filled.blocks {
            let (at, len) = tables[block.table];
            block.write(std::slice::from_raw_parts_mut(at as *mut u8, len));
        }
        game_write(WORK_A_RVA, filled.seconds);
        if let Some(sum) = filled.sum {
            game_write(WORK_B_RVA, sum);
        }
        if let Some(c) = filled.checkpoints {
            game_write(CHECKPOINTS_RVA, c);
        }
    }
}

/// Whether the game's switch for the checkpoint times is on.
pub fn with_checkpoints() -> bool {
    unsafe { game_read::<u32>(WITH_CHECKPOINTS_RVA) == 1 }
}

/// How many records a table has before its -1 (the first is not looked at for the end).
pub unsafe fn records(table: u32) -> Result<usize, BadInput> {
    (1..=0x1000usize)
        .find(|&n| unsafe {
            ((table + n as u32 * RECORD_LEN) as *const u32).read_unaligned() == u32::MAX
        })
        .ok_or(BadInput("a record table without its end"))
}
