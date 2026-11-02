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
use crate::logfile::log_hex8::log_hex8;
use crate::logfile::log_int::log_int;
use crate::logfile::write_line::level;
use crate::memory::MemoryList;
use crate::memory::free_list::free_list;
use crate::session::Bytes;
use crate::session::{Dword, Shared};
use crate::strings::string_table::{lexicon, use_lexicon};
use rc2k_core::strings::lexicon::{Compiled, compile};

// ---- game data ----

/// The work variables: out of the image since 2026-10-24 (phase 5; they were
/// `dword_94EAB4` ..). Written here, read by nothing now (the original's compiler read
/// them).
pub const WORK_LEN: usize = 0xC44;
pub static WORK: Bytes<WORK_LEN> = Bytes::new([0; WORK_LEN]);
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

/// IDA's address of the lexicon's memory variables.
pub const MEMORY_START: usize = 0x519554;
/// Their length.
pub const MEMORY_LEN: usize = 0x519598 - MEMORY_START;

/// The lexicon compiler's memory (the glyph builder takes the same work memory).
#[repr(C)]
pub struct LexiconMemory {
    /// `dword_519554`: the memory's kilobytes (`imageheapsize`, a `var\ral.cfg` setting;
    /// the glyph tables' work memory is sized by it too).
    pub kilobytes: Dword,
    /// `g_lexiconMemory` (`stru_519558`): the list the memory is taken from (16 bytes).
    pub memory: Shared<MemoryList>,
    /// `0x519568` .. `0x519598`: nothing reads or writes them (a gap).
    pub _gap_14: [Dword; 12],
}

/// The lexicon's memory; the first values are the image's.
pub static LEXICON: LexiconMemory = LexiconMemory {
    kilobytes: Dword::new(0x400),
    memory: Shared::new(MemoryList {
        head: 0,
        total: 0,
        cursor: 0,
        remaining: 0,
    }),
    _gap_14: [const { Dword::new(0) }; 12],
};

/// Where the block is (`moved::MOVED`'s home for IDA 0x519554).
pub fn memory_home() -> usize {
    crate::pointers::game_address(&raw const LEXICON) as usize
}
/// The record all empty messages share (`unk_4F3034` in the image until 2026-10-24): +4 how
/// often it was used (never reset), +0xC its index entry. Written here only.
pub const EMPTY_MESSAGE_LEN: usize = 0x10;
pub static EMPTY_MESSAGE: Bytes<EMPTY_MESSAGE_LEN> = Bytes::new([0; EMPTY_MESSAGE_LEN]);
/// `aFilesLocaleTxt` (0x4E0770).
const PATTERN: &[u8] = b"files\\%locale\\*.txt\0";
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
fn report(why: &str) {
    let why = why.strip_prefix("language file: ").unwrap_or(why);
    let mut text = b"LC:ERROR:".to_vec();
    text.extend_from_slice(why.as_bytes());
    text.push(0);
    line(&text, true);
}

/// Reads the language file and compiles it, changing nothing of the game's but what the
/// file's finding and reading leave (the search record, the last file's path). A file that
/// does not load or does not compile is the original's fatal error: one that fails the
/// compiler is reported in the log first. Memory and table overflows, which the original
/// reports as 0x80E1801x / 2x, are 0x80E18051 here.
pub fn compiled() -> Result<Compiled, FatalError> {
    let memory = LEXICON.kilobytes.get().wrapping_mul(0x400);
    if memory < FILE_ROOM {
        report("the lexicon's memory is smaller than the file may be");
        return Err(NOT_CREATED);
    }
    let lent = crate::pointers::LentBytes::from_vec(vec![0u8; FILE_ROOM as usize]);
    let read = read_first_match(
        crate::pointers::GamePtr::to(PATTERN),
        crate::pointers::GamePtr::new(lent.addr()),
        FILE_ROOM,
        0,
    )
    .ok()
    .flatten()
    .ok_or(NOT_LOADED)?;
    let mut text = lent.to_vec();
    drop(lent);
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

pub fn compile_lexicon() -> Result<(), Refused> {
    {
        line(b"Compiling lexicon...\0", true);
        let compiled = compiled()?;
        WORK.write(&[0; WORK_LEN]);
        // The original's memory, for the game's memory figures: counted and given back
        // (nothing is put in it: the lexicon is built in Rust's memory).
        let list = crate::pointers::GamePtr::to(&LEXICON.memory);
        crate::memory::owned::count_on(list, LEXICON.kilobytes.get().wrapping_mul(0x400))?;

        for (offset, value) in counts(&compiled) {
            WORK.set_u32(offset, value);
        }
        EMPTY_MESSAGE.set_u32(
            4,
            EMPTY_MESSAGE
                .get_u32(4)
                .wrapping_add(compiled.empty_messages),
        );
        EMPTY_MESSAGE.set_u32(12, compiled.index_size);
        crate::pointers::GamePtr::new(crate::pointers::game_address(lexicon()))
            .write_bytes(&compiled.table, "the lexicon's memory")?;
        use_lexicon();
        crate::strings::string_table::publish()?;

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

#[cfg(test)]
mod tests {
    use crate::moved::image_bytes;

    #[test]
    fn lexicon_memory_first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(
            &super::LEXICON,
            super::MEMORY_START,
            super::MEMORY_START + super::MEMORY_LEN,
        );
        assert_eq!(
            crate::moved::place(0x519558),
            super::LEXICON.memory.addr() as usize
        );
    }

    #[test]
    fn pattern_and_first_values_are_the_games() {
        assert_eq!(image_bytes(0x4E0770, super::PATTERN.len()), super::PATTERN);
        assert!(
            image_bytes(0x94EAB4, super::WORK_LEN)
                .iter()
                .all(|&b| b == 0)
        );
        assert!(
            image_bytes(0x4F3034, super::EMPTY_MESSAGE_LEN)
                .iter()
                .all(|&b| b == 0)
        );
        assert!(
            image_bytes(0x4E07F8, super::super::string_table::LEXICON_ROOM)
                .iter()
                .all(|&b| b == 0)
        );
        assert_eq!(image_bytes(0x4E07D4, 4), [0; 4]);
    }
}
