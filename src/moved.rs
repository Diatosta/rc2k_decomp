//! Phase 5, the image gone (`notes/conversion.md`): the game's variables that have left
//! RAL.EXE's image for memory Rust owns, and the one place that knows where each went.
//!
//! Code that names a moved variable asks its owner (`logfile::write_line::state`,
//! `strings::units::units`, ...), never this file. Only the game's address tables still
//! name variables by IDA address: the settings table (`var\ral.cfg`'s key -> the address of
//! its variable, `settings::read_settings`) and copy lists such as `race::replay`'s
//! `HEADER_MOVES`. They go through `place` (an IDA address) or `resolve` (an address as the
//! image holds it, relocated), which answer a moved variable's new home and anything else's
//! place in the image. `framework::game_addr` does not consult this: a stale
//! `game_addr(rva(X))` of a moved X reads the image's old, unchanging copy, so every use
//! of a moved address is converted when it moves (grep all its spellings: `0x4E00A0`,
//! `0x4E_00A0`, `{b} + 0xE00A0`, base + offset).

use crate::framework::{IMAGE_BASE, game_addr, rva};
use std::cell::UnsafeCell;

/// A block of the game's memory that now lives in Rust, at an address that does not move:
/// reached through raw pointers as the image's bytes were (the game's code hands such
/// addresses around in registers and tables). The same discipline as the image: whoever
/// writes keeps to one thread or the lock the original used (the log's buffers: the log's
/// lock).
#[repr(transparent)]
pub struct Place<T>(UnsafeCell<T>);

// SAFETY: as the image's memory: raw access only, under the original's discipline (above).
unsafe impl<T> Sync for Place<T> {}

impl<T> Place<T> {
    pub const fn new(value: T) -> Self {
        Self(UnsafeCell::new(value))
    }

    pub const fn ptr(&self) -> *mut T {
        self.0.get()
    }

    pub fn addr(&self) -> usize {
        self.ptr() as usize
    }
}

/// A moved range: IDA's address, its length, its new home.
struct Moved {
    ida: usize,
    len: usize,
    home: fn() -> usize,
}

/// Every range moved so far, by IDA address. Kept in step with the owners' statics; ranges
/// do not overlap (`tests::moved_ranges_are_apart`).
const MOVED: &[Moved] = &[
    // logfile (`logfile::write_line::state`, the buffers)
    Moved {
        ida: 0x4E0094, // g_logEnabled
        len: 4,
        home: || crate::logfile::write_line::state::ENABLED.as_ptr() as usize,
    },
    Moved {
        ida: 0x4E0098, // g_logLevel (a `var\ral.cfg` setting, `logfile`)
        len: 4,
        home: || crate::logfile::write_line::state::LEVEL.as_ptr() as usize,
    },
    Moved {
        ida: 0x4E009C, // g_logLock
        len: 4,
        home: || crate::logfile::write_line::state::LOCK.as_ptr() as usize,
    },
    Moved {
        ida: 0x4E00A0, // g_logIndent
        len: 4,
        home: || crate::logfile::write_line::state::INDENT.as_ptr() as usize,
    },
    Moved {
        ida: 0x4E00A4, // g_logAtLineStart
        len: 4,
        home: || crate::logfile::write_line::state::LINE_START.as_ptr() as usize,
    },
    Moved {
        ida: 0x93E030, // dst: the numbers' text
        len: crate::logfile::numeric::SCRATCH_LEN,
        home: || crate::logfile::numeric::SCRATCH.addr(),
    },
    Moved {
        ida: 0x93E080, // the log's line buffer
        len: crate::logfile::write_line::LINE_BUF_LEN,
        home: || crate::logfile::write_line::LINE_BUF.addr(),
    },
    Moved {
        ida: 0x4E073A, // g_freeSpaceKb
        len: 4,
        home: || crate::logfile::start_log::FREE_SPACE_KB.as_ptr() as usize,
    },
    // math
    Moved {
        ida: 0x4E0140, // g_randomState
        len: 4,
        home: || crate::math::next_random::STATE.as_ptr() as usize,
    },
    // system's start time, read by `logfile::date_text` and `math::seed_random`
    Moved {
        ida: 0x51839F, // byte_51839F, dword_5183A0, dword_5183A4
        len: crate::system::keep_start_time::KEPT_LEN,
        home: || crate::system::keep_start_time::KEPT.addr(),
    },
    // strings
    Moved {
        ida: 0x4E0144, // dword_4E0144: names keep their case
        len: 4,
        home: || crate::strings::copy_printable::KEEP_CASE.as_ptr() as usize,
    },
    Moved {
        ida: 0x4E0148, // g_timeShowHours
        len: 4,
        home: || crate::strings::append_time::SHOW_HOURS.as_ptr() as usize,
    },
    Moved {
        ida: 0x4E0544, // g_sealCodeText
        len: crate::strings::format_seal_code::SEAL_TEXT_LEN,
        home: || crate::strings::format_seal_code::SEAL_TEXT.addr(),
    },
    Moved {
        ida: 0x4E0758, // dword_4E0758: the lexicon recompiled on a key (a `var\ral.cfg` setting)
        len: 4,
        home: || crate::strings::maybe_compile_lexicon::RECOMPILE_LEVEL.as_ptr() as usize,
    },
    Moved {
        ida: 0x4E0798, // g_localeFile (g_noticeKind at +8)
        len: crate::strings::load_language::LOCALE_LEN,
        home: || crate::strings::load_language::LOCALE.addr(),
    },
    Moved {
        ida: 0x4E07D4, // g_stringTable
        len: 4,
        home: || crate::strings::string_table::STRING_TABLE.as_ptr() as usize,
    },
    Moved {
        ida: 0x4E07F8, // g_lexicon
        len: rc2k_core::strings::lexicon::TABLE_ROOM,
        home: || crate::strings::string_table::LEXICON.addr(),
    },
    Moved {
        ida: 0x4F0BC8, // g_lowerCase
        len: 0x100,
        home: || crate::strings::same_text::LOWER.addr(),
    },
    Moved {
        ida: 0x4F0CC8, // g_plainLetters
        len: 0x100,
        home: || crate::strings::load_language::PLAIN_LETTERS.addr(),
    },
    Moved {
        ida: 0x4F3034, // the record the empty messages share
        len: crate::strings::compile_lexicon::EMPTY_MESSAGE_LEN,
        home: || crate::strings::compile_lexicon::EMPTY_MESSAGE.addr(),
    },
    Moved {
        ida: 0x51DE2C, // g_isImperialUnits
        len: 4,
        home: || crate::strings::units::UNITS.as_ptr() as usize,
    },
    Moved {
        ida: 0x94EAB4, // dword_94EAB4: the lexicon compiler's work variables
        len: crate::strings::compile_lexicon::WORK_LEN,
        home: || crate::strings::compile_lexicon::WORK.addr(),
    },
    // memory (the allocator's figures and own heap, `memory::state`)
    Moved {
        ida: 0x4E0578, // dword_4E0578: the most bytes held
        len: 4,
        home: || crate::memory::state::MOST_HELD.as_ptr() as usize,
    },
    Moved {
        ida: 0x4E057C, // dword_4E057C: the bytes held now
        len: 4,
        home: || crate::memory::state::HELD.as_ptr() as usize,
    },
    Moved {
        ida: 0x4E0580, // dword_4E0580: the turnover
        len: 4,
        home: || crate::memory::state::TURNOVER.as_ptr() as usize,
    },
    Moved {
        ida: 0x4E0584, // g_ownHeap (a `var\ral.cfg` setting, `altmemenable`)
        len: 4,
        home: || crate::memory::state::OWN_HEAP.as_ptr() as usize,
    },
    Moved {
        ida: 0x4E0588, // g_ownHeapKb (a `var\ral.cfg` setting, `altmemsize`)
        len: 4,
        home: || crate::memory::state::OWN_HEAP_KB.as_ptr() as usize,
    },
    Moved {
        ida: 0x4E058C, // g_ownHeapBase
        len: 4,
        home: || crate::memory::state::HEAP_BASE.as_ptr() as usize,
    },
    Moved {
        ida: 0x4E0590, // g_ownHeapCursor
        len: 4,
        home: || crate::memory::state::HEAP_CURSOR.as_ptr() as usize,
    },
    // settings (`read_driver_names`' work areas), threading (the thread slots)
    Moved {
        ida: 0x529DA5, // aName000: the driver names' key, made in place
        len: 8,
        home: || crate::settings::read_driver_names::KEY.addr(),
    },
    Moved {
        ida: 0x95ECF8, // dword_95ECF8 (a pointer a driver) and unk_95EFC8 (the names' pool), up to 0x960000
        len: 0x1308,
        home: || crate::settings::read_driver_names::NAMES.addr(),
    },
    Moved {
        ida: 0x5184F4, // the thread slots: input, audio, network, shutdown
        len: crate::threading::thread_slot::SLOT_COUNT * crate::threading::thread_slot::SLOT_LEN,
        home: || crate::threading::thread_slot::SLOTS.addr(),
    },
    // text (the blended font and palette, the kerning rows)
    Moved {
        ida: 0x4F3B74, // g_blendedPalette
        len: 0x400,
        home: || crate::text::blend_font::BLENDED_PALETTE.addr(),
    },
    Moved {
        ida: 0x4F3F74, // g_fontBlendMode (a `var\ral.cfg` setting, `textflashmode`)
        len: 4,
        home: || crate::text::blend_font::BLEND_MODE.as_ptr() as usize,
    },
    Moved {
        ida: 0x51E384, // g_blendedFont
        len: 0x20,
        home: || crate::text::blend_font::BLENDED_FONT.addr(),
    },
    Moved {
        ida: 0x93E830, // g_kerningRowState
        len: rc2k_core::text::KERNING_ROWS * 2,
        home: || crate::text::glyph_kerning::STATE.addr(),
    },
    // files (the search record, the paths' buffers)
    Moved {
        ida: 0x4E05A4, // hFindFile
        len: 4,
        home: || crate::files::state::FIND_HANDLE.as_ptr() as usize,
    },
    Moved {
        ida: 0x4E05A8, // FindFileData (WIN32_FIND_DATAA)
        len: 0x13E,
        home: || crate::files::state::FIND_DATA.addr(),
    },
    Moved {
        ida: 0x4E06E6, // the attributes a search passes over
        len: 4,
        home: || crate::files::state::SKIPPED_ATTRIBUTES.as_ptr() as usize,
    },
    Moved {
        ida: 0x4F52B0, // g_lastFilePath
        len: 4,
        home: || crate::files::state::LAST_FILE_PATH.as_ptr() as usize,
    },
    Moved {
        ida: 0x5BE9BD, // g_packedPath
        len: crate::files::state::PACKED_PATH_LEN,
        home: || crate::files::state::PACKED_PATH.addr(),
    },
    Moved {
        ida: 0x60B2CC, // g_readBytes
        len: 4,
        home: || crate::files::state::BYTES_READ.as_ptr() as usize,
    },
    Moved {
        ida: 0x94E8B0, // byte_94E8B0 (numbered paths) and the resolver's buffer at 0x94E9B0
        len: crate::files::state::PATHS_LEN,
        home: || crate::files::state::PATHS.addr(),
    },
    // fatal (the reporter's variables, `fatal::state`)
    Moved {
        ida: 0x4E0598, // dword_4E0598: the Win32 error named in a report
        len: 4,
        home: || crate::fatal::state::W32.as_ptr() as usize,
    },
    Moved {
        ida: 0x4F52A4, // g_fatalCode
        len: 4,
        home: || crate::fatal::state::CODE.as_ptr() as usize,
    },
    Moved {
        ida: 0x4F52A8, // g_fatalMessage
        len: 4,
        home: || crate::fatal::state::MESSAGE.as_ptr() as usize,
    },
    Moved {
        ida: 0x4F66FD, // Caption ("Rally: Guru Meditation")
        len: crate::fatal::state::CAPTION_LEN,
        home: || crate::fatal::state::CAPTION.addr(),
    },
    Moved {
        ida: 0x4F673A, // aEaxXxxxxxxxEbx: the registers' template
        len: crate::fatal::state::TEMPLATE_LEN,
        home: || crate::fatal::state::TEMPLATE.addr(),
    },
    Moved {
        ida: 0x4F67BC, // g_fatalReported
        len: 1,
        home: || crate::fatal::state::REPORTED.as_ptr() as usize,
    },
    Moved {
        ida: 0x4F67BD, // cursor: the string added to every report
        len: crate::fatal::state::EXTRA_LEN,
        home: || crate::fatal::state::EXTRA.addr(),
    },
    Moved {
        ida: 0x9526F8, // Text: the report
        len: crate::fatal::state::TEXT_LEN,
        home: || crate::fatal::state::TEXT.addr(),
    },
    // files (the install record's variables, `files::state`)
    Moved {
        ida: 0x5189A4, // g_mediaMode
        len: 4,
        home: || crate::files::state::MEDIA_MODE.as_ptr() as usize,
    },
    Moved {
        ida: 0x5189A8, // g_baseDirectory, then g_localeDirectory at +0x100
        len: crate::files::state::DIRECTORIES_LEN,
        home: || crate::files::state::DIRECTORIES.addr(),
    },
];

/// Where the variable at IDA address `ida` lives now: its new home if it moved, else its
/// place in the image.
pub fn place(ida: usize) -> usize {
    match MOVED.iter().find(|m| ida.wrapping_sub(m.ida) < m.len) {
        Some(m) => (m.home)() + (ida - m.ida),
        None => game_addr(rva(ida)),
    }
}

#[allow(dead_code)] // the settings table now holds IDA addresses and asks `place`
/// `place` for an address as the image holds it (a relocated pointer of the game's data):
/// a moved variable's new home, anything else unchanged.
pub fn resolve(address: usize) -> usize {
    let ida = address.wrapping_sub(game_addr(0)).wrapping_add(IMAGE_BASE);
    match MOVED.iter().find(|m| ida.wrapping_sub(m.ida) < m.len) {
        Some(m) => (m.home)() + (ida - m.ida),
        None => address,
    }
}

/// RAL.EXE's bytes at IDA address `ida` as `data/ral_image.bin` holds them (before its
/// addresses are relocated): what the moved constants and initial values are checked
/// against while the image exists.
#[cfg(test)]
pub fn image_bytes(ida: usize, len: usize) -> Vec<u8> {
    let d = crate::standalone::IMAGE_DATA;
    let word = |at: usize| u32::from_le_bytes(d[at..at + 4].try_into().unwrap()) as usize;
    let (start, end) = (rva(ida), rva(ida) + len);
    let mut out = vec![0u8; len];
    let mut copy = |to: usize, bytes: &[u8]| {
        let (from, until) = (to.max(start), (to + bytes.len()).min(end));
        if from < until {
            out[from - start..until - start].copy_from_slice(&bytes[from - to..until - to]);
        }
    };
    assert_eq!(&d[..8], b"RC2KIMG1");
    let mut at = 20;
    for _ in 0..word(16) {
        let (section, flags) = (word(at), word(at + 8));
        at += 12;
        if flags & 4 != 0 {
            let count = word(at);
            at += 4;
            for _ in 0..count {
                let (piece, n) = (word(at), word(at + 4));
                copy(piece, &d[at + 8..at + 8 + n]);
                at += 8 + n;
            }
        }
        if flags & 2 != 0 {
            let raw = word(at);
            at += 4;
            let mut done = 0;
            while done < raw {
                let (zeros, literal) = (word(at), word(at + 4));
                copy(section + done + zeros, &d[at + 8..at + 8 + literal]);
                at += 8 + literal;
                done += zeros + literal;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moved_ranges_are_apart() {
        for (n, a) in MOVED.iter().enumerate() {
            assert!(a.len > 0);
            for b in &MOVED[n + 1..] {
                assert!(
                    a.ida + a.len <= b.ida || b.ida + b.len <= a.ida,
                    "{:#x} and {:#x} overlap",
                    a.ida,
                    b.ida
                );
            }
        }
    }

    #[test]
    fn image_bytes_reads_the_image() {
        assert_eq!(image_bytes(0x4E0000, 6), b"$VER: ");
        // bss reads as zeros
        assert_eq!(image_bytes(0x93E430, 4), [0; 4]);
    }
}
