//! `strings::compile_lexicon` (0x40356E): makes the string table from the language file.
//!
//! In:  nothing
//! Out: every register preserved
//! The original ("Compiling lexicon..." in the game's log) is a compiler of 25 functions on
//! memory of its own; what they do is in `strings/lexicon.rs`. Around that: it zeroes its
//! 0xC44 bytes of work variables (`dword_94EAB4` on), takes `dword_519554` * 0x400 bytes on
//! the list `stru_519558`, reads the first file matching `files\%locale\*.txt` (at most
//! 0x40000 bytes), compiles, writes the table at 0x4E07F8 and points `g_stringTable` at it,
//! logs its counts when the log level is 3 or more, and gives the memory back. There must
//! be 0x757 messages, or it is fatal error 0x80E18052; an error in the file is logged with
//! its line and ends in fatal error 0x80E18051.
//! The original's error paths jump back to its own start with `dword_94EAB0` set (the stack
//! pointer it started with): such an entry is handed to the original as it is, which is
//! also why the thunk is ungated.
//! The Rust version leaves the work variables that are pointers into the given-back memory
//! 0 (the original leaves them dangling), and refuses, leaving it to the original to
//! report: a file that does not load or has an error, a message count other than 0x757,
//! and anything else `lexicon::compile` refuses.
//! Shadow mode compiles twice and compares the whole table and the counts. The self-test
//! does the same once, and then lets the Rust
//! version put its table in place and looks again. The game's own call at start comes
//! before the hooks are in place (they are installed from a thread of their own), so the
//! self-test is where both versions really run.
//! Callers: `sub_40331F` (the language's loading, from `start` and three more) and one more.

use crate::errors::{BadInput, Refused};
use crate::files::read_first_match::read_first_match;
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::logfile::log_hex8::log_hex8;
use crate::logfile::log_int::log_int;
use crate::logfile::log_write::log_write;
use crate::logfile::write_line::LOG_LEVEL_RVA;
use crate::memory::MemoryList;
use crate::memory::allocate::allocate;
use crate::memory::free_list::free_list;
use crate::strings::lexicon::{Compiled, compile};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("strings::compile_lexicon", rva(0x40356E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

// ---- game data ----

const STARTED_WITH_RVA: usize = rva(0x94EAB0); // dword_94EAB0: esp while a compilation runs
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

/// Reads the language file and compiles it, changing nothing of the game's but what the
/// file's finding and reading leave (the search record, the last file's path).
pub unsafe fn compiled() -> Result<Compiled, BadInput> {
    unsafe {
        let memory = game_read::<u32>(MEMORY_KILOBYTES_RVA).wrapping_mul(0x400);
        if memory < FILE_ROOM {
            return Err(BadInput(
                "the lexicon's memory is smaller than the file may be",
            ));
        }
        let mut text = vec![0u8; FILE_ROOM as usize];
        let read = read_first_match(
            game_addr(PATTERN_RVA) as *const u8,
            text.as_mut_ptr(),
            FILE_ROOM,
            0,
        )?
        .ok_or(BadInput("the language file does not load"))?;
        text.truncate(read as usize);
        let compiled = compile(&text, memory)?;
        if compiled.messages != MESSAGE_COUNT {
            return Err(BadInput(
                "too many or too few messages in the language file",
            ));
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
        if game_read::<u32>(STARTED_WITH_RVA) != 0 {
            return Err(BadInput("a compilation is running").into());
        }
        let compiled = compiled()?;

        line(b"Compiling lexicon...\0", true);
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

// ---- hook ----

/// What of the original's result is not as `compiled` says, or None.
unsafe fn difference(compiled: &Compiled, empty_before: u32) -> Option<String> {
    unsafe {
        let table =
            std::slice::from_raw_parts(game_addr(TABLE_RVA) as *const u8, compiled.table.len());
        if let Some(byte) = table.iter().zip(&compiled.table).position(|(a, b)| a != b) {
            return Some(format!(
                "byte {byte:#x} of the table: original {:#x} rust {:#x}",
                table[byte], compiled.table[byte]
            ));
        }
        if game_read::<u32>(STRING_TABLE_RVA) != game_addr(TABLE_RVA) as u32 {
            return Some("g_stringTable is not the table".into());
        }
        for (offset, value) in counts(compiled) {
            let theirs: u32 = game_read(WORK_RVA + offset);
            if theirs != value {
                return Some(format!(
                    "the work variable at {offset:#x}: original {theirs:#x} rust {value:#x}"
                ));
            }
        }
        let empty = EMPTY_MESSAGE_RVA;
        if game_read::<u32>(empty + 4).wrapping_sub(empty_before) != compiled.empty_messages
            || game_read::<u32>(empty + 0xC) != compiled.index_size
        {
            return Some("the empty message's record".into());
        }
        None
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    // One of the original's error paths coming back to its start: it goes on by itself.
    if unsafe { game_read::<u32>(STARTED_WITH_RVA) } != 0 {
        unsafe { call_original(r) };
        return;
    }
    if !shadow() {
        if let Err(e) = unsafe { compile_lexicon() } {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
        }
        return;
    }
    let mine = unsafe { compiled() };
    let empty_before: u32 = unsafe { game_read(EMPTY_MESSAGE_RVA + 4) };
    let mut o = *r;
    unsafe { call_original(&mut o) };
    // It came back, so the file compiled: the Rust version must not have refused it.
    let wrong = match &mine {
        Ok(compiled) => unsafe { difference(compiled, empty_before) },
        Err(e) => Some(format!("the rust version refuses: {e:?}")),
    };
    if wrong.is_some() || !o.preserved_except(r, 0) {
        HOOK.mismatch(|| format!("{wrong:?}"));
    }
}

// ---- self-test: the game's own language file, compiled by both ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if unsafe { game_read::<u32>(STARTED_WITH_RVA) } != 0 {
        return t.skip("a compilation is running");
    }
    let mine = unsafe { compiled() };
    let empty_before: u32 = unsafe { game_read(EMPTY_MESSAGE_RVA + 4) };
    // The lexicon in use is not what the compiler leaves: `strings::load_language` corrects
    // it afterwards. Until 2026-10-12 this test left a freshly compiled one, without the
    // corrections, in its place.
    let in_use = [
        (game_addr(TABLE_RVA) as u32, 0x4F07C8 - 0x4E07F8),
        (game_addr(STRING_TABLE_RVA) as u32, 4),
        (game_addr(WORK_RVA) as u32, WORK_LEN),
    ];
    let saved = crate::service::harness::capture(&in_use);
    let before = Regs::SENTINEL;
    let mut ro = before;
    // The original writes a fresh table over the one in use.
    unsafe { call_original(&mut ro) };
    let wrong = match &mine {
        Ok(compiled) => unsafe { difference(compiled, empty_before) },
        Err(e) => Some(format!("the rust version refuses: {e:?}")),
    };
    unsafe { game_write(EMPTY_MESSAGE_RVA + 4, empty_before) };
    let preserved = ro.preserved_except(&before, 0);
    t.check(wrong.is_none() && preserved, || {
        format!("the game's language file: preserved={preserved}; {wrong:?}")
    });

    // And the Rust version for real, which the game's start never reaches (the table is
    // compiled before the hooks are in place): what it leaves must be the same again.
    let done = unsafe { compile_lexicon() };
    let wrong = match (&mine, &done) {
        (Ok(compiled), Ok(())) => unsafe { difference(compiled, empty_before) },
        _ => Some(format!("the rust version refuses: {done:?}")),
    };
    unsafe { game_write(EMPTY_MESSAGE_RVA + 4, empty_before) };
    t.check(wrong.is_none(), || {
        format!(
            "the game's language file, compiled and put in place by the rust version: {wrong:?}"
        )
    });
    crate::service::harness::restore(&in_use, &saved);
    t.finish()
}
