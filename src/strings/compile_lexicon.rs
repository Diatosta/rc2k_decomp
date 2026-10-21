//! `strings::compile_lexicon` (0x40356E): makes the string table from the language file.
//!
//! The original ("Compiling lexicon..." in the game's log) is a compiler of 25 functions on
//! memory of its own; what they do is in `strings/lexicon.rs`. Around that: it zeroes its
//! 0xC44 bytes of work variables (`dword_94EAB4` on), takes `dword_519554` * 0x400 bytes on
//! the list `stru_519558`, reads the first file matching `files\%locale\*.txt` (at most
//! 0x40000 bytes), compiles, writes the table at 0x4E07F8 and points `g_stringTable` at it,
//! logs its counts when the log level is 3 or more, and gives the memory back. There must
//! be 0x757 messages, or it is fatal error 0x80E18052; an error in the file is logged with
//! its line and ends in fatal error 0x80E18051 (the original's error paths jump back to its
//! own start with `dword_94EAB0` set, and it ends there).
//! The Rust version leaves the work variables that are pointers into the given-back memory
//! 0 (the original leaves them dangling), and raises the fatal errors itself (`compiled`).
//! Callers: `strings::load_language` (from `start` and the installer) and
//! `strings::maybe_compile_lexicon`.
//! Hook removed 2026-10-21 (conversion phase 4); the original's registers: none in, every
//! register preserved.

use crate::errors::{FatalError, Refused};
use crate::files::read_first_match::read_first_match;
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::logfile::log_hex8::log_hex8;
use crate::logfile::log_int::log_int;
use crate::logfile::log_write::log_write;
use crate::logfile::write_line::LOG_LEVEL_RVA;
use crate::memory::MemoryList;
use crate::memory::allocate::allocate;
use crate::memory::free_list::free_list;
use rc2k_core::strings::lexicon::{Compiled, compile};

// ---- game data ----

const WORK_RVA: usize = rva(0x94EAB4); // dword_94EAB4: the work variables
const WORK_LEN: usize = 0xC44;
/// The work variables that are counts, as offsets into the block, in `Compiled`'s order;
/// before them the flags word (0x11000000) at 0, after them the unused count at 0x30.
const PADDED: usize = 0x04;
const LINES: usize = 0x08;
const MESSAGES: usize = 0x14;
const SOURCE_SIZE: usize = 0x18;
const WORDS_ENCOUNTERED: usize = 0x1C;
const WORDS_STORED: usize = 0x20;
const HIGHEST_WORD: usize = 0x24;
const PHRASES_ENCOUNTERED: usize = 0x28;
const PHRASES_STORED: usize = 0x2C;
const SUBPHRASES_STORED: usize = 0x30;
const HIGHEST_PHRASE: usize = 0x34;
const INDEX_SIZE: usize = 0xC3C;
const TOTAL_SIZE: usize = 0xC40;
const WORK_FLAGS: u32 = 0x1100_0000;

const MEMORY_LIST_RVA: usize = rva(0x519558); // stru_519558
const MEMORY_KILOBYTES_RVA: usize = rva(0x519554); // dword_519554
const TABLE_RVA: usize = rva(0x4E07F8); // unk_4E07F8
const STRING_TABLE_RVA: usize = rva(0x4E07D4); // g_stringTable
/// unk_4F3034: the record all empty messages share; +4 how often it was used (never reset),
/// +0xC its index entry.
const EMPTY_MESSAGE_RVA: usize = rva(0x4F3034);
const PATTERN_RVA: usize = rva(0x4E0770); // "files\%locale\*.txt"
const FILE_ROOM: u32 = 0x40000;
const MESSAGE_COUNT: u32 = 0x757;

// ---- Rust version ----

/// The original's fatal errors (`loc_4D815E`, `loc_4D812C`, `loc_4D8145`); the `\\n` is the
/// reporter's, as in RAL.EXE.
const NOT_LOADED: FatalError = FatalError {
    code: 0x80E1_8010,
    message: "Error loading lexicon file",
};
const NOT_CREATED: FatalError = FatalError {
    code: 0x80E1_8051,
    message: "Error(s) in lexicon.txt - lexicon not created\\nSee logfile.txt for more information",
};
const WRONG_COUNT: FatalError = FatalError {
    code: 0x80E1_8052,
    message: "Too many or too few tokens defined in lexicon.txt",
};

/// `lexicon::report` (0x4040CC) for an error: "LC:" "ERROR:" and the reason on one line.
/// The original adds "on line" with the line's number and its text; the core does not
/// keep the line (`notes/reference.md` section 15).
unsafe fn report(why: &str) {
    let why = why.strip_prefix("language file: ").unwrap_or(why);
    let mut text = b"LC:ERROR:".to_vec();
    text.extend_from_slice(why.as_bytes());
    text.push(0);
    unsafe { line(&text, true) };
}

/// Reads the language file and compiles it, changing nothing of the game's but what the
/// file's finding and reading leave (the search record, the last file's path). A file that
/// does not load or does not compile is the original's fatal error: one that fails the
/// compiler is reported in the log first. Memory and table overflows, which the original
/// reports as 0x80E1801x / 2x, are 0x80E18051 here.
pub unsafe fn compiled() -> Result<Compiled, FatalError> {
    unsafe {
        let memory = game_read::<u32>(MEMORY_KILOBYTES_RVA).wrapping_mul(0x400);
        if memory < FILE_ROOM {
            report("the lexicon's memory is smaller than the file may be");
            return Err(NOT_CREATED);
        }
        let mut text = vec![0u8; FILE_ROOM as usize];
        let read = read_first_match(
            game_addr(PATTERN_RVA) as *const u8,
            text.as_mut_ptr(),
            FILE_ROOM,
            0,
        )
        .ok()
        .flatten()
        .ok_or(NOT_LOADED)?;
        text.truncate(read as usize);
        let compiled = compile(&text, memory).map_err(|e| {
            report(e.0);
            NOT_CREATED
        })?;
        if compiled.messages != MESSAGE_COUNT {
            return Err(WRONG_COUNT);
        }
        Ok(compiled)
    }
}

/// The counts as the work variables hold them: (offset, value).
fn counts(compiled: &Compiled) -> [(usize, u32); 14] {
    [
        (0, WORK_FLAGS),
        (PADDED, compiled.padded),
        (LINES, compiled.lines),
        (MESSAGES, compiled.messages),
        (SOURCE_SIZE, compiled.source_size),
        (WORDS_ENCOUNTERED, compiled.words_encountered),
        (WORDS_STORED, compiled.words_stored),
        (HIGHEST_WORD, compiled.highest_word),
        (PHRASES_ENCOUNTERED, compiled.phrases_encountered),
        (PHRASES_STORED, compiled.phrases_stored),
        (SUBPHRASES_STORED, 0),
        (HIGHEST_PHRASE, compiled.highest_phrase),
        (INDEX_SIZE, compiled.index_size),
        (TOTAL_SIZE, compiled.table.len() as u32),
    ]
}

unsafe fn line(text: &[u8], newline: bool) {
    debug_assert!(text.ends_with(&[0]));
    unsafe { log_write(text.as_ptr(), newline, 1) };
}

pub unsafe fn compile_lexicon() -> Result<(), Refused> {
    unsafe {
        line(b"Compiling lexicon...\0", true);
        let compiled = compiled()?;
        std::ptr::write_bytes(game_addr(WORK_RVA) as *mut u8, 0, WORK_LEN);
        // The original's memory, for the game's memory figures: taken and given back.
        let list = game_addr(MEMORY_LIST_RVA) as *mut MemoryList;
        let memory = allocate(
            list,
            game_read::<u32>(MEMORY_KILOBYTES_RVA).wrapping_mul(0x400),
        )?;
        (*list).cursor = memory as u32;
        (*list).remaining = (*list).total;

        for (offset, value) in counts(&compiled) {
            game_write(WORK_RVA + offset, value);
        }
        let empty = EMPTY_MESSAGE_RVA;
        game_write(
            empty + 4,
            game_read::<u32>(empty + 4).wrapping_add(compiled.empty_messages),
        );
        game_write(empty + 0xC, compiled.index_size);
        std::ptr::copy(
            compiled.table.as_ptr(),
            game_addr(TABLE_RVA) as *mut u8,
            compiled.table.len(),
        );
        game_write(STRING_TABLE_RVA, game_addr(TABLE_RVA) as u32);

        if game_read::<i32>(LOG_LEVEL_RVA) >= 3 {
            let number = |text: &[u8], value: u32| {
                line(text, false);
                log_int(value as i32, true, 1);
            };
            let token = |text: &[u8], value: u32| {
                line(text, false);
                log_hex8(value, true, 1);
            };
            number(b"LC: Messages defined\t=\0", compiled.messages);
            number(b"LC: Source text size\t=\0", compiled.source_size);
            number(b"LC: Words encountered\t=\0", compiled.words_encountered);
            number(b"LC: Words stored\t=\0", compiled.words_stored);
            token(b"LC: Highest Token\t=\0", compiled.highest_word);
            number(
                b"LC: Phrases encountered\t=\0",
                compiled.phrases_encountered,
            );
            number(b"LC: Phrases stored\t=\0", compiled.phrases_stored);
            number(b"LC: Subphrases stored\t=\0", 0);
            token(b"LC: Highest Token\t=\0", compiled.highest_phrase);
            number(b"LC: Lexicon total size\t=\0", compiled.table.len() as u32);
            number(b"LC: Lexicon index size\t=\0", compiled.index_size);
        }
        line(b"Compilation complete\0", true);
        free_list(list)?;
        line(b"\0", true);
        Ok(())
    }
}
