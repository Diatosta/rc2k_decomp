//! Makes the string table from the language file ("Compiling lexicon..." in the game's log). The
//! compiler proper is in `strings/lexicon.rs`. Around it: zeroes the 0xC44 bytes of work variables,
//! takes `kilobytes` * 0x400 bytes on the memory list, reads the first file matching
//! `files\%locale\*.txt`, compiles, writes the table and makes it the game's table, logs its counts
//! when the log level is 3 or more, and gives the memory back. There must be 0x757 messages, or it
//! is fatal error 0x80E18052; an error in the file is logged with its line and ends in fatal error
//! 0x80E18051. The work variables that are pointers into the given-back memory are left 0. The
//! fatal errors are raised here (`compiled`).

use crate::errors::{FatalError, Refused};
use crate::files::read_first_match::read_first_match;
use crate::files::scratch::Scratch;
use crate::logfile::log_hex8::log_hex8;
use crate::logfile::log_int::log_int;
use crate::logfile::write_line::level;
use crate::memory::MemoryList;
use crate::memory::free_list::free_list;
use crate::memory::session::PlainDword;
use crate::strings::owner::Lexicon;
use rc2k_core::strings::lexicon::{Compiled, compile};

/// The work variables' size; they are written here and read by nothing.
pub const WORK_LEN: usize = 0xC44;
/// The work variables that are counts, as offsets into the block, in `Compiled`'s order; before
/// them the flags word at 0, after them the unused count at 0x30.
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

/// The lexicon compiler's memory (the glyph builder takes the same work memory).
#[derive(rc2k_derive::Cells)]
pub struct LexiconMemory {
    /// The memory's kilobytes (the `imageheapsize` setting; the glyph tables' work memory is sized
    /// by it too).
    pub kilobytes: PlainDword,
    /// `LexiconMemory.memory.head`: the list the memory is taken from (16 bytes).
    pub memory: crate::memory::ListCells,
}

impl LexiconMemory {
    /// The lexicon's memory.
    pub const fn new() -> Self {
        LexiconMemory {
            kilobytes: PlainDword::new(0x400),
            memory: crate::memory::ListCells::new(MemoryList {
                head: 0,
                total: 0,
                cursor: 0,
                remaining: 0,
            }),
        }
    }
}

/// The record all empty messages share: +4 how often it was used (never reset), +0xC its index
/// entry. Written here only.
pub const EMPTY_MESSAGE_LEN: usize = 0x10;
const PATTERN: &[u8] = b"files\\%locale\\*.txt";
const FILE_ROOM: u32 = 0x40000;
const MESSAGE_COUNT: u32 = 0x757;

/// The fatal errors; the `\\n` is the reporter's.
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

/// `lexicon::report` for an error: "LC:" "ERROR:" and the reason on one line; the line's number
/// and text are not logged (the core does not keep the line).
fn report(why: &str) {
    let why = why.strip_prefix("language file: ").unwrap_or(why);
    let mut text = b"LC:ERROR:".to_vec();
    text.extend_from_slice(why.as_bytes());
    text.push(0);
    line(&text, true);
}

/// Reads the language file and compiles it, changing nothing of the game's but what the file's
/// finding and reading leave (the search record, the last file's path). A file that does not load
/// or does not compile is a fatal error: one that fails the compiler is reported in the log first.
/// Memory and table overflows are fatal error 0x80E18051 here.
pub fn compiled(lexicon: &Lexicon, scratch: &Scratch) -> Result<Compiled, FatalError> {
    let memory = lexicon.memory.kilobytes.get().wrapping_mul(0x400);
    if memory < FILE_ROOM {
        report("the lexicon's memory is smaller than the file may be");
        return Err(NOT_CREATED);
    }
    let mut text = vec![0u8; FILE_ROOM as usize];
    let read = read_first_match(scratch, PATTERN, &mut text, 0)
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

fn line(text: &[u8], newline: bool) {
    debug_assert!(text.ends_with(&[0]));
    crate::logfile::log_write::log_nul(text, newline, 1);
}

pub fn compile_lexicon(
    strings: &crate::strings::owner::Strings,
    scratch: &Scratch,
) -> Result<(), Refused> {
    {
        line(b"Compiling lexicon...\0", true);
        let compiled = compiled(&strings.lexicon, scratch)?;
        strings.lexicon.work.write(&[0; WORK_LEN]);
        // The lexicon's memory list, for the game's memory figures: counted and given back (nothing
        // is put in it: the lexicon is built in Rust's memory).
        let list = &strings.lexicon.memory.memory;
        crate::memory::owned::count_on(
            list,
            strings.lexicon.memory.kilobytes.get().wrapping_mul(0x400),
        )?;

        for (offset, value) in counts(&compiled) {
            strings.lexicon.work.set_u32(offset, value);
        }
        strings.lexicon.empty_message.set_u32(
            4,
            strings
                .lexicon
                .empty_message
                .get_u32(4)
                .wrapping_add(compiled.empty_messages),
        );
        strings
            .lexicon
            .empty_message
            .set_u32(12, compiled.index_size);
        crate::strings::string_table::write_lexicon(strings, &compiled.table)?;
        crate::strings::string_table::publish(strings)?;

        if level() as i32 >= 3 {
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
