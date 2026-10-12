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

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, game_addr, game_read, game_write, rva};
use crate::math::next_random::STATE_RVA;
use crate::math::random_below::random_below;
use crate::saves::drivers::{Driver, put, put_bytes};
use crate::screens::harness::{CallOriginal, Left};
use crate::service::harness::{Region, at, dispatch_replayed_state};

pub const WITH_CHECKPOINTS_RVA: usize = rva(0x755A0C); // dword_755A0C
const CHECKPOINTS_RVA: usize = rva(0x71C7C0); // dword_71C7C0[16]
const CHECKPOINT_COUNTS_RVA: usize = rva(0x71C730); // dword_71C730[stage]
const MOST_CHECKPOINTS: u32 = 16;
pub const WORK_A_RVA: usize = rva(0x7559AC); // seconds
pub const WORK_B_RVA: usize = rva(0x7559B0); // dword_7559B0
pub const ENTRY_LEN: u32 = 0x9C;
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

/// `physics::guess_checkpoint_times` (0x4650F6).
pub unsafe fn guess_checkpoint_times(stage: u32, time: u32) -> Result<(), BadInput> {
    unsafe {
        let count = if stage > 0x24 {
            0
        } else {
            let count: u32 = game_read(CHECKPOINT_COUNTS_RVA + stage as usize * 4);
            if count == 0 || count > MOST_CHECKPOINTS {
                return Err(BadInput(
                    "a stage with no checkpoints, or more than sixteen",
                ));
            }
            count
        };
        for n in 0..MOST_CHECKPOINTS {
            let guessed = if n < count {
                ((n as u64 + 1) * time as u64 / count as u64) as u32
            } else {
                0
            };
            game_write(CHECKPOINTS_RVA + n as usize * 4, guessed);
        }
        Ok(())
    }
}

/// `saves::default_time_entry` (0x47CB74): `time` is eax and `stage` is ebp there.
pub unsafe fn default_time_entry(
    entry: u32,
    time: u32,
    stage: u32,
    log: Log,
) -> Result<(), Refused> {
    unsafe {
        let text = |at: usize, newline: bool| Piece::Text(game_addr(rva(at)) as u32, newline);
        let with_checkpoints = game_read::<u32>(WITH_CHECKPOINTS_RVA) == 1;
        if with_checkpoints {
            log(text(0x4DE6C5, true)); // a line of dashes
            log(text(0x4DE6FB, true)); // "guesscheckpointtimes:start"
            log(text(0x4DE716, false)); // "passing:(time:ebx)"
            log(Piece::Number(time, false));
            log(text(0x4DE729, false)); // "(stage:eax)"
            log(Piece::Number(stage, true));
            guess_checkpoint_times(stage, time)?;
            log(text(0x4DE735, false)); // "testcheckdata"
            for n in 0..MOST_CHECKPOINTS as usize {
                log(Piece::Number(
                    game_read(CHECKPOINTS_RVA + n * 4),
                    n % 8 == 7,
                ));
            }
            log(text(0x4DE743, true)); // "guesscheckpointtimes:done"
            log(text(0x4DE75D, true)); // a line of dashes
        }
        let driver = Driver::number(random_below(0xAA)?)?;
        let (id, name) = driver.name()?;
        put(entry, 0x1C, driver.dword(8));
        put(entry, 8, driver.dword(0x14));
        put(entry, 0x50, 0xB);
        if with_checkpoints {
            for n in 0..MOST_CHECKPOINTS {
                put(
                    entry,
                    0x5C + n * 4,
                    game_read(CHECKPOINTS_RVA + n as usize * 4),
                );
            }
        }
        put(entry, 0x10, driver.dword(0x18));
        put(entry, 0xC, id);
        put_bytes(entry, 0x20, &name);
        Ok(())
    }
}

/// Ten entries from `entry` on, each with its time at +0x14, the times `step` apart.
pub unsafe fn ten_entries(
    entry: u32,
    time: u32,
    step: u32,
    stage: u32,
    log: Log,
) -> Result<(), Refused> {
    for n in 0..10 {
        let (entry, time) = (entry + n * ENTRY_LEN, time.wrapping_add(n * step));
        unsafe {
            put(entry, 0x14, time);
            default_time_entry(entry, time, stage, log)?;
        }
    }
    Ok(())
}

/// The record of the championship's table that is its `stage`th with tag 0.
pub unsafe fn stage_record(table: u32, stage: u32, records: usize) -> Result<u32, BadInput> {
    (0..records as u32)
        .map(|n| table + n * RECORD_LEN)
        .filter(|&record| unsafe { (record as *const u32).read_unaligned() } == 0)
        .nth(stage as usize)
        .ok_or(BadInput("fewer stages than counted"))
}

/// How many records a table has before its -1 (the first is not looked at for the end).
pub unsafe fn records(table: u32) -> Result<usize, BadInput> {
    (1..=0x1000usize)
        .find(|&n| unsafe {
            ((table + n as u32 * RECORD_LEN) as *const u32).read_unaligned() == u32::MAX
        })
        .ok_or(BadInput("a record table without its end"))
}

/// What all four may write besides their tables.
pub fn shared_regions() -> Vec<Region> {
    vec![
        (
            game_addr(CHECKPOINTS_RVA) as u32,
            MOST_CHECKPOINTS as usize * 4,
        ),
        at(WORK_A_RVA),
        at(WORK_B_RVA),
        at(STATE_RVA),
    ]
}

/// The `dispatch` of the four: the original first (it may log), then the Rust version
/// without the log on what was put back.
pub fn dispatch_times(
    hook: &RawHook,
    r: &mut Regs,
    call_original: CallOriginal,
    tables: Result<Vec<Region>, BadInput>,
    run: &dyn Fn(&Regs, Log) -> Result<(), Refused>,
) {
    let tables = match tables {
        Ok(tables) => tables,
        Err(e) => {
            hook.count_call();
            unsafe { crate::fatal::raise::refused(hook, e, r, call_original) };
            return;
        }
    };
    let regions = [tables, shared_regions()].concat();
    dispatch_replayed_state(
        hook,
        r,
        call_original,
        &regions,
        &|r| run(r, &mut to_the_log).map(|()| Left::nothing()),
        &|r| run(r, &mut to_nowhere).map(|()| Left::nothing()),
    );
}

/// The game's variable that holds a table, as a region of `len` bytes; refuses none.
pub fn table_region(variable: usize, len: u32) -> Result<Region, BadInput> {
    match unsafe { game_read::<u32>(variable) } {
        0 => Err(BadInput("a table of best times is missing")),
        table => Ok((table, len as usize)),
    }
}
