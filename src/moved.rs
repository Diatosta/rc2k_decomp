//! Phase 5, the image gone (`notes/conversion.md`): the game's variables that have left
//! RAL.EXE's image for memory Rust owns, and the one place that knows where each went.
//!
//! Code that names a moved variable asks its owner (`logfile::write_line::state`,
//! `strings::units::units`, ...), never this file. Only the game's address tables still
//! name variables by IDA address: the settings table (`var\ral.cfg`'s key -> the address of
//! its variable, `settings::read_settings`) and copy lists such as `race::replay`'s
//! `HEADER_MOVES`. They go through `place` (an IDA address), which answers a moved
//! variable's new home and anything else's place in the image. The game's own pointers stay
//! as the image has them, IDA's addresses: each reader resolves its names or runs its code
//! by them, and a table that needs real addresses has them set at start (`standalone::start`).
//! `framework::game_addr` resolves a moved address through `home_of_rva`. The image itself
//! is gone (2026-10-24): its addresses stay reserved without access, so a read of one that
//! has no home ends the program naming it (`standalone`'s fault handler).

#![allow(unsafe_code)]

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

    pub fn addr(&self) -> u32 {
        crate::pointers::game_address(self.ptr())
    }
}

/// A moved range: IDA's address, its length, its new home.
pub(crate) struct Moved {
    pub(crate) ida: usize,
    pub(crate) len: usize,
    home: Home,
}

/// Where a moved range lives: a store of cells, read and written by offset
/// (`pointers::Store`), or, until its type is one, a function answering its address.
pub(crate) enum Home {
    Store(&'static dyn crate::pointers::Store),
    Address(fn() -> usize),
}

impl Home {
    /// The home's first byte as a game address.
    fn address(&self) -> usize {
        match self {
            Home::Store(store) => crate::pointers::game_address(store.first()) as usize,
            Home::Address(home) => home(),
        }
    }
}

/// Every range moved so far, by IDA address. Kept in step with the owners' statics; ranges
/// do not overlap (`tests::moved_ranges_are_apart`).
pub(crate) const MOVED: &[Moved] = &[
    // logfile (`logfile::write_line::state`, the buffers)
    Moved {
        ida: 0x4E0094, // g_logEnabled
        len: 4,
        home: Home::Address(|| {
            (crate::pointers::game_address(crate::logfile::write_line::state::ENABLED.as_ptr()))
                as usize
        }),
    },
    Moved {
        ida: 0x4E0098, // g_logLevel (a `var\ral.cfg` setting, `logfile`)
        len: 4,
        home: Home::Address(|| {
            (crate::pointers::game_address(crate::logfile::write_line::state::LEVEL.as_ptr()))
                as usize
        }),
    },
    Moved {
        ida: 0x4E009C, // g_logLock
        len: 4,
        home: Home::Address(|| {
            (crate::pointers::game_address(crate::logfile::write_line::state::LOCK.as_ptr()))
                as usize
        }),
    },
    Moved {
        ida: 0x4E00A0, // g_logIndent
        len: 4,
        home: Home::Address(|| {
            (crate::pointers::game_address(crate::logfile::write_line::state::INDENT.as_ptr()))
                as usize
        }),
    },
    Moved {
        ida: 0x4E00A4, // g_logAtLineStart
        len: 4,
        home: Home::Address(|| {
            (crate::pointers::game_address(crate::logfile::write_line::state::LINE_START.as_ptr()))
                as usize
        }),
    },
    Moved {
        ida: 0x93E030, // dst: the numbers' text
        len: crate::logfile::numeric::SCRATCH_LEN,
        home: Home::Address(|| crate::logfile::numeric::SCRATCH.addr() as usize),
    },
    Moved {
        ida: 0x4E073A, // g_freeSpaceKb
        len: 4,
        home: Home::Address(|| {
            (crate::pointers::game_address(crate::logfile::start_log::FREE_SPACE_KB.as_ptr()))
                as usize
        }),
    },
    // math
    Moved {
        ida: 0x4E0140, // g_randomState
        len: 4,
        home: Home::Address(|| {
            (crate::pointers::game_address(crate::math::next_random::STATE.as_ptr())) as usize
        }),
    },
    // system's start time, read by `logfile::date_text` and `math::seed_random`
    Moved {
        ida: 0x51839F, // byte_51839F, dword_5183A0, dword_5183A4
        len: crate::system::keep_start_time::KEPT_LEN,
        home: Home::Store(&crate::system::keep_start_time::KEPT),
    },
    // strings
    Moved {
        ida: 0x4E0144, // dword_4E0144: names keep their case
        len: 4,
        home: Home::Address(|| {
            (crate::pointers::game_address(crate::strings::copy_printable::KEEP_CASE.as_ptr()))
                as usize
        }),
    },
    Moved {
        ida: 0x4E0148, // g_timeShowHours
        len: 4,
        home: Home::Address(|| {
            (crate::pointers::game_address(crate::strings::append_time::SHOW_HOURS.as_ptr()))
                as usize
        }),
    },
    Moved {
        ida: 0x4E0544, // g_sealCodeText
        len: crate::strings::format_seal_code::SEAL_TEXT_LEN,
        home: Home::Address(|| crate::strings::format_seal_code::SEAL_TEXT.addr() as usize),
    },
    Moved {
        ida: 0x4E0758, // dword_4E0758: the lexicon recompiled on a key (a `var\ral.cfg` setting)
        len: 4,
        home: Home::Address(|| {
            (crate::pointers::game_address(
                crate::strings::maybe_compile_lexicon::RECOMPILE_LEVEL.as_ptr(),
            )) as usize
        }),
    },
    Moved {
        ida: 0x4E0798, // g_localeFile (g_noticeKind at +8)
        len: crate::strings::load_language::LOCALE_LEN,
        home: Home::Store(&crate::strings::load_language::LOCALE),
    },
    Moved {
        ida: 0x4E07D4, // g_stringTable
        len: 4,
        home: Home::Address(|| {
            (crate::pointers::game_address(crate::strings::string_table::STRING_TABLE.as_ptr()))
                as usize
        }),
    },
    Moved {
        ida: 0x4E07F8, // g_lexicon
        len: rc2k_core::strings::lexicon::TABLE_ROOM,
        home: Home::Address(|| crate::strings::string_table::LEXICON.addr() as usize),
    },
    Moved {
        ida: 0x4F0BC8, // g_lowerCase
        len: 0x100,
        home: Home::Store(&crate::strings::same_text::LOWER),
    },
    Moved {
        ida: 0x4F0CC8, // g_plainLetters
        len: 0x100,
        home: Home::Store(&crate::strings::load_language::PLAIN_LETTERS),
    },
    Moved {
        ida: 0x4F3034, // the record the empty messages share
        len: crate::strings::compile_lexicon::EMPTY_MESSAGE_LEN,
        home: Home::Store(&crate::strings::compile_lexicon::EMPTY_MESSAGE),
    },
    Moved {
        ida: 0x51DE2C, // g_isImperialUnits
        len: 4,
        home: Home::Address(|| {
            (crate::pointers::game_address(crate::strings::units::UNITS.as_ptr())) as usize
        }),
    },
    Moved {
        ida: 0x94EAB4, // dword_94EAB4: the lexicon compiler's work variables
        len: crate::strings::compile_lexicon::WORK_LEN,
        home: Home::Store(&crate::strings::compile_lexicon::WORK),
    },
    // memory (the allocator's figures and own heap, `memory::state`)
    Moved {
        ida: 0x4E0578, // dword_4E0578: the most bytes held
        len: 4,
        home: Home::Address(|| {
            (crate::pointers::game_address(crate::memory::state::MOST_HELD.as_ptr())) as usize
        }),
    },
    Moved {
        ida: 0x4E057C, // dword_4E057C: the bytes held now
        len: 4,
        home: Home::Address(|| {
            (crate::pointers::game_address(crate::memory::state::HELD.as_ptr())) as usize
        }),
    },
    Moved {
        ida: 0x4E0580, // dword_4E0580: the turnover
        len: 4,
        home: Home::Address(|| {
            (crate::pointers::game_address(crate::memory::state::TURNOVER.as_ptr())) as usize
        }),
    },
    Moved {
        ida: 0x4E0584, // g_ownHeap (a `var\ral.cfg` setting, `altmemenable`)
        len: 4,
        home: Home::Address(|| {
            (crate::pointers::game_address(crate::memory::state::OWN_HEAP.as_ptr())) as usize
        }),
    },
    Moved {
        ida: 0x4E0588, // g_ownHeapKb (a `var\ral.cfg` setting, `altmemsize`)
        len: 4,
        home: Home::Address(|| {
            (crate::pointers::game_address(crate::memory::state::OWN_HEAP_KB.as_ptr())) as usize
        }),
    },
    Moved {
        ida: 0x4E058C, // g_ownHeapBase
        len: 4,
        home: Home::Address(|| {
            (crate::pointers::game_address(crate::memory::state::HEAP_BASE.as_ptr())) as usize
        }),
    },
    Moved {
        ida: 0x4E0590, // g_ownHeapCursor
        len: 4,
        home: Home::Address(|| {
            (crate::pointers::game_address(crate::memory::state::HEAP_CURSOR.as_ptr())) as usize
        }),
    },
    // settings (`read_driver_names`' work areas), threading (the thread slots)
    Moved {
        ida: 0x529DA5, // aName000: the driver names' key, made in place
        len: 8,
        home: Home::Address(|| crate::settings::read_driver_names::KEY.addr() as usize),
    },
    Moved {
        ida: 0x95ECF8, // dword_95ECF8 (a pointer a driver) and unk_95EFC8 (the names' pool), up to 0x960000
        len: 0x1308,
        home: Home::Address(|| crate::settings::read_driver_names::NAMES.addr() as usize),
    },
    Moved {
        ida: 0x5184F4, // the thread slots: input, audio, network, shutdown
        len: crate::threading::thread_slot::SLOT_COUNT * crate::threading::thread_slot::SLOT_LEN,
        home: Home::Address(|| crate::threading::thread_slot::SLOTS.addr() as usize),
    },
    // text (the blended font and palette, the kerning rows)
    Moved {
        ida: 0x4F3B74, // g_blendedPalette
        len: 0x400,
        home: Home::Address(|| crate::text::blend_font::BLENDED_PALETTE.addr() as usize),
    },
    Moved {
        ida: 0x4F3F74, // g_fontBlendMode (a `var\ral.cfg` setting, `textflashmode`)
        len: 4,
        home: Home::Address(|| {
            (crate::pointers::game_address(crate::text::blend_font::BLEND_MODE.as_ptr())) as usize
        }),
    },
    Moved {
        ida: 0x51E384, // g_blendedFont
        len: 0x20,
        home: Home::Address(|| crate::text::blend_font::BLENDED_FONT.addr() as usize),
    },
    // files (the search record, the paths' buffers)
    Moved {
        ida: 0x4E05A4, // hFindFile
        len: 4,
        home: Home::Address(|| {
            (crate::pointers::game_address(crate::files::state::FIND_HANDLE.as_ptr())) as usize
        }),
    },
    Moved {
        ida: 0x4E05A8, // FindFileData (WIN32_FIND_DATAA)
        len: 0x13E,
        home: Home::Store(&crate::files::state::FIND_DATA),
    },
    Moved {
        ida: 0x4E06E6, // the attributes a search passes over
        len: 4,
        home: Home::Address(|| {
            (crate::pointers::game_address(crate::files::state::SKIPPED_ATTRIBUTES.as_ptr()))
                as usize
        }),
    },
    Moved {
        ida: 0x5BE9BD, // g_packedPath
        len: crate::files::state::PACKED_PATH_LEN,
        home: Home::Address(|| crate::files::state::PACKED_PATH.addr() as usize),
    },
    Moved {
        ida: 0x60B2CC, // g_readBytes
        len: 4,
        home: Home::Address(|| {
            (crate::pointers::game_address(crate::files::state::BYTES_READ.as_ptr())) as usize
        }),
    },
    Moved {
        ida: 0x94E8B0, // byte_94E8B0 (numbered paths) and the resolver's buffer at 0x94E9B0
        len: crate::files::state::PATHS_LEN,
        home: Home::Address(|| crate::files::state::PATHS.addr() as usize),
    },
    // fatal (the reporter's variables, `fatal::state`)
    Moved {
        ida: 0x4E0598, // dword_4E0598: the Win32 error named in a report
        len: 4,
        home: Home::Address(|| {
            (crate::pointers::game_address(crate::fatal::state::W32.as_ptr())) as usize
        }),
    },
    Moved {
        ida: 0x4F52A4, // g_fatalCode
        len: 4,
        home: Home::Address(|| {
            (crate::pointers::game_address(crate::fatal::state::CODE.as_ptr())) as usize
        }),
    },
    Moved {
        ida: 0x4F66FD, // Caption ("Rally: Guru Meditation")
        len: crate::fatal::state::CAPTION_LEN,
        home: Home::Store(&crate::fatal::state::CAPTION),
    },
    Moved {
        ida: 0x4F673A, // aEaxXxxxxxxxEbx: the registers' template
        len: crate::fatal::state::TEMPLATE_LEN,
        home: Home::Store(&crate::fatal::state::TEMPLATE),
    },
    Moved {
        ida: 0x4F67BC, // g_fatalReported
        len: 1,
        home: Home::Address(|| {
            (crate::pointers::game_address(crate::fatal::state::REPORTED.as_ptr())) as usize
        }),
    },
    Moved {
        ida: 0x4F67BD, // cursor: the string added to every report
        len: crate::fatal::state::EXTRA_LEN,
        home: Home::Store(&crate::fatal::state::EXTRA),
    },
    Moved {
        ida: 0x9526F8, // Text: the report
        len: crate::fatal::state::TEXT_LEN,
        home: Home::Store(&crate::fatal::state::TEXT),
    },
    // files (the install record's variables, `files::state`)
    Moved {
        ida: 0x5189A4, // g_mediaMode
        len: 4,
        home: Home::Address(|| {
            (crate::pointers::game_address(crate::files::state::MEDIA_MODE.as_ptr())) as usize
        }),
    },
    Moved {
        ida: 0x5189A8, // g_baseDirectory, then g_localeDirectory at +0x100
        len: crate::files::state::DIRECTORIES_LEN,
        home: Home::Address(|| crate::files::state::DIRECTORIES.addr() as usize),
    },
    // gfx / hud / d3d (phase 5; typed in phase 7)
    Moved {
        ida: crate::hud::store::START, // the HUD's variables, batch included
        len: crate::hud::store::LEN,
        home: Home::Address(|| crate::hud::store::hud_addr() as usize),
    },
    Moved {
        ida: crate::gfx::decoder::START, // The picture decoders' and glyph builder's variables (decoded picture, palette)
        len: crate::gfx::decoder::LEN,
        home: Home::Store(&crate::gfx::decoder::DECODER),
    },
    Moved {
        ida: crate::strings::compile_lexicon::MEMORY_START, // The lexicon's memory (kilobytes, the list)
        len: crate::strings::compile_lexicon::MEMORY_LEN,
        home: Home::Address(crate::strings::compile_lexicon::memory_home),
    },
    Moved {
        ida: crate::gfx::build_glyphs::START, // The sheet builder's variables (sheet table, built table, glyph counts)
        len: crate::gfx::build_glyphs::LEN,
        home: Home::Store(&crate::gfx::build_glyphs::SHEET),
    },
    Moved {
        ida: crate::gfx::pcx::START, // The PCX file being decoded (header, palette, stream, limits)
        len: crate::gfx::pcx::LEN,
        home: Home::Store(&crate::gfx::pcx::PCX),
    },
    Moved {
        ida: crate::gfx::read_picture_file::PNG_SETTING_START, // `nopngdecode`
        len: 4,
        home: Home::Store(&crate::gfx::read_picture_file::PNG_SETTING),
    },
    Moved {
        ida: crate::gfx::glyph_tables::START, // The glyph tables' variables (`bundleheapsize`, the list)
        len: crate::gfx::glyph_tables::LEN,
        home: Home::Address(crate::gfx::glyph_tables::home),
    },
    Moved {
        ida: crate::gfx::show_picture_file::SHOWING_START, // g_showingPicture
        len: 4,
        home: Home::Store(&crate::gfx::show_picture_file::SHOWING),
    },
    Moved {
        ida: crate::gfx::blit::START, // The blitters' variables (the clip rectangle, the skips of the visible part)
        len: crate::gfx::blit::LEN,
        home: Home::Store(&crate::gfx::blit::BLIT),
    },
    Moved {
        ida: crate::gfx::rescale_picture::START, // The rescaler's variables (the copy's memory, steps, weights, the pixel's parts)
        len: crate::gfx::rescale_picture::LEN,
        home: Home::Address(crate::gfx::rescale_picture::home),
    },
    Moved {
        ida: crate::gfx::strips::START, // The HUD's view strips' places (sizes, tops, lefts)
        len: crate::gfx::strips::LEN,
        home: Home::Store(&crate::gfx::strips::STRIPS),
    },
    Moved {
        ida: crate::gfx::fill_row_table::START, // The row table (the address of each row of a picture)
        len: crate::gfx::fill_row_table::LEN,
        home: Home::Address(crate::gfx::fill_row_table::home),
    },
    Moved {
        ida: crate::gfx::formats::START, // The pixel formats' work variables (format numbers, scratch bytes, component format)
        len: crate::gfx::formats::LEN,
        home: Home::Store(&crate::gfx::formats::FORMATS),
    },
    Moved {
        ida: crate::gfx::fill_thirds_table::START, // The thirds table (n / 3 for the sums of three components)
        len: 0x300,
        home: Home::Store(&crate::gfx::fill_thirds_table::THIRDS),
    },
    Moved {
        ida: crate::gfx::modes::START, // The screen mode tables (pixel kinds, sizes, the modes listed)
        len: crate::gfx::modes::LEN,
        home: Home::Store(&crate::gfx::modes::MODES),
    },
    Moved {
        ida: crate::gfx::converters::START, // The MMX converters' masks and the converters table
        len: crate::gfx::converters::LEN,
        home: Home::Address(crate::gfx::converters::home),
    },
    Moved {
        ida: crate::gfx::reports::START, // The 3D reports switch (dword_4F52AC)
        len: crate::gfx::reports::LEN,
        home: Home::Store(&crate::gfx::reports::REPORTS),
    },
    Moved {
        ida: crate::gfx::shots::START, // The screenshot switch, its lock and the file name's template
        len: crate::gfx::shots::LEN,
        home: Home::Store(&crate::gfx::shots::SHOTS),
    },
    Moved {
        ida: crate::gfx::picture_store::START, // The picture loader's variables (the name, flags, size, rows and buffer of the picture, the list)
        len: crate::gfx::picture_store::LEN,
        home: Home::Address(crate::gfx::picture_store::home),
    },
    Moved {
        ida: crate::gfx::texture_pixels::START, // The texture grading (reading switch, scale, saturation, the grades table)
        len: crate::gfx::texture_pixels::LEN,
        home: Home::Store(&crate::gfx::texture_pixels::GRADING),
    },
    Moved {
        ida: crate::gfx::bar_store::START, // The progress bar's variables (position, width, glyphs, percent)
        len: crate::gfx::bar_store::LEN,
        home: Home::Store(&crate::gfx::bar_store::BAR),
    },
    Moved {
        ida: crate::gfx::png_file::START, // The PNG decoder's file (pointer, length, memory list)
        len: crate::gfx::png_file::LEN,
        home: Home::Address(crate::gfx::png_file::home),
    },
    Moved {
        ida: crate::gfx::png_header::START, // The PNG header's fields (width, height, compression, colour type)
        len: crate::gfx::png_header::LEN,
        home: Home::Store(&crate::gfx::png_header::HEADER),
    },
    Moved {
        ida: crate::gfx::pcx_planes::START, // The screenshot writer's plane buffer (3 * width bytes, up to 1920 wide)
        len: crate::gfx::pcx_planes::LEN,
        home: Home::Store(&crate::gfx::pcx_planes::PLANES),
    },
    Moved {
        ida: crate::gfx::cards::START, // The 3D primary and secondary card switches
        len: crate::gfx::cards::LEN,
        home: Home::Store(&crate::gfx::cards::CARDS),
    },
    Moved {
        ida: crate::gfx::upload::START, // The texture upload's surface description (DDSURFACEDESC) and the mode-trial switch
        len: crate::gfx::upload::LEN,
        home: Home::Store(&crate::gfx::upload::UPLOAD),
    },
    Moved {
        ida: crate::install::store::InstallVars::START, // InstallVars
        len: crate::install::store::InstallVars::LEN,
        home: Home::Address(crate::install::store::InstallVars::home),
    },
    Moved {
        ida: crate::menu::build_notice::BuildNotice::START, // BuildNotice
        len: crate::menu::build_notice::BuildNotice::LEN,
        home: Home::Store(&crate::menu::build_notice::NOTICE),
    },
    Moved {
        ida: crate::install::store::DriveRows::START, // DriveRows
        len: crate::install::store::DriveRows::LEN,
        home: Home::Store(&crate::install::store::DRIVE_ROWS),
    },
    Moved {
        ida: crate::net::store::NetVars::START, // NetVars
        len: crate::net::store::NetVars::LEN,
        home: Home::Address(crate::net::store::NetVars::home),
    },
    Moved {
        ida: crate::install::store::LobbyKeys::START, // LobbyKeys
        len: crate::install::store::LobbyKeys::LEN,
        home: Home::Store(&crate::install::store::LOBBY_KEYS),
    },
    Moved {
        ida: crate::net::store::Button::START, // Button
        len: crate::net::store::Button::LEN,
        home: Home::Store(&crate::net::store::BUTTON),
    },
    Moved {
        ida: crate::gfx::typefaces::START, // The fonts (63 font objects)
        len: crate::gfx::typefaces::LEN,
        home: Home::Store(&crate::gfx::typefaces::FONTS),
    },
    Moved {
        ida: crate::strings::unit_labels::START, // The unit labels' string ids
        len: crate::strings::unit_labels::LEN,
        home: Home::Address(crate::strings::unit_labels::home),
    },
    Moved {
        ida: crate::saves::write_savegame_slot::START, // The saved games' file buffer and names
        len: crate::saves::write_savegame_slot::LEN,
        home: Home::Store(&crate::saves::write_savegame_slot::SAVE_GAMES),
    },
    Moved {
        ida: crate::service::read_setup_slot::START, // The set-ups' file head and names
        len: crate::service::read_setup_slot::LEN,
        home: Home::Store(&crate::service::read_setup_slot::SET_UPS),
    },
    Moved {
        ida: crate::race::replay::START, // The replays' header and names
        len: crate::race::replay::LEN,
        home: Home::Store(&crate::race::replay::REPLAYS),
    },
    Moved {
        ida: crate::install::installer::TEXTS_START, // The installer's texts
        len: crate::install::installer::TEXTS_LEN,
        home: Home::Store(&crate::install::installer::INSTALL_TEXTS),
    },
    Moved {
        ida: crate::saves::load_options::PATH_START, // var\ral.dat
        len: 12,
        home: Home::Store(&crate::saves::load_options::PATH),
    },
    Moved {
        ida: crate::saves::load_best_times::PATH_START, // var\hst.dat
        len: 12,
        home: Home::Store(&crate::saves::load_best_times::PATH),
    },
    Moved {
        ida: crate::race::replay_path::LAST_RACE_START, // The last race's replay file name
        len: crate::race::replay_path::LAST_RACE_LEN,
        home: Home::Store(&crate::race::replay_path::LAST_RACE),
    },
    Moved {
        ida: crate::saves::write_savegame_slot::TEMPLATE_START, // The saved game file name's template
        len: 27,
        home: Home::Store(&crate::saves::write_savegame_slot::TEMPLATE),
    },
    Moved {
        ida: crate::service::read_setup_slot::TEMPLATE_START, // The set-up file name's template
        len: 27,
        home: Home::Store(&crate::service::read_setup_slot::TEMPLATE),
    },
    Moved {
        ida: crate::race::find_snapshot_number::SNAPSHOT_START, // The snapshot picture's name
        len: 26,
        home: Home::Store(&crate::race::find_snapshot_number::SNAPSHOT),
    },
    Moved {
        ida: crate::race::replay_path::TEMPLATES_START, // The replay file names' templates
        len: crate::race::replay_path::TEMPLATES_LEN,
        home: Home::Store(&crate::race::replay_path::TEMPLATES),
    },
    Moved {
        ida: crate::menu::play_films::FILMS_START, // The films' file names
        len: crate::menu::play_films::FILMS_LEN,
        home: Home::Store(&crate::menu::play_films::FILMS),
    },
    Moved {
        ida: crate::screens::draw_message_box::BOX_START, // The message box's rectangle
        len: 16,
        home: Home::Address(crate::screens::draw_message_box::box_home),
    },
    Moved {
        ida: crate::race::frame_text::CREDITS_START, // The credits' state and script
        len: crate::race::frame_text::CREDITS_LEN,
        home: Home::Address(crate::race::frame_text::credits_home),
    },
    Moved {
        ida: crate::system::check_developer_machine::DEVELOPER_START, // g_developerMachine
        len: 4,
        home: Home::Store(&crate::system::check_developer_machine::DEVELOPER),
    },
    Moved {
        ida: crate::gfx::texture_upload::START, // The 3D card's tables and the textures' records
        len: crate::gfx::texture_upload::LEN,
        home: Home::Address(crate::gfx::texture_upload::home),
    },
    Moved {
        ida: crate::platform_3d::GUIDS_A_START, // DirectX interface ids (nothing reads them)
        len: crate::platform_3d::GUIDS_A_LEN,
        home: Home::Address(crate::platform_3d::guids_a_home),
    },
    Moved {
        ida: crate::film::NO_FMV_START, // g_noFmv
        len: 4,
        home: Home::Store(&crate::film::NO_FMV),
    },
    Moved {
        ida: crate::platform_3d::GUIDS_B_START, // DirectX interface ids (nothing reads them)
        len: crate::platform_3d::GUIDS_B_LEN,
        home: Home::Address(crate::platform_3d::guids_b_home),
    },
    Moved {
        ida: crate::gfx::build_game_fonts::START, // The game fonts' memory (`gamegfxheapsize`, the list)
        len: crate::gfx::build_game_fonts::LEN,
        home: Home::Address(crate::gfx::build_game_fonts::home),
    },
    Moved {
        ida: crate::scene::upload_car_icon::START, // The car icon upload's record
        len: 0x1C,
        home: Home::Address(crate::scene::upload_car_icon::home),
    },
    Moved {
        ida: crate::race::game_map::START, // The in-race map's variables
        len: crate::race::game_map::LEN,
        home: Home::Address(crate::race::game_map::home),
    },
    Moved {
        ida: crate::gfx::front::START, // The DirectDraw front and back surface handles
        len: crate::gfx::front::LEN,
        home: Home::Store(&crate::gfx::front::SURFACES),
    },
    Moved {
        ida: crate::gfx::locked::START, // The surface-locked switch
        len: crate::gfx::locked::LEN,
        home: Home::Store(&crate::gfx::locked::LOCKED),
    },
    Moved {
        ida: crate::gfx::windscreen_drop_batch::START, // The marks' windscreen drop batch drawn by d3d (the index and vertex counts, the fans, the records)
        len: crate::gfx::windscreen_drop_batch::LEN,
        home: Home::Store(&crate::gfx::windscreen_drop_batch::WINDSCREEN_DROP_BATCH),
    },
    // session: the screen, the window's flags, the machine, the clocks (`session`)
    Moved {
        ida: crate::session::SCREEN_IDA,
        len: crate::session::SCREEN_LEN,
        home: Home::Address(|| {
            crate::pointers::game_address(&raw const crate::session::SCREEN) as usize
        }),
    },
    Moved {
        ida: crate::session::FLAGS_IDA,
        len: crate::session::FLAGS_LEN,
        home: Home::Address(|| {
            crate::pointers::game_address(&raw const crate::session::FLAGS) as usize
        }),
    },
    Moved {
        ida: crate::session::MACHINE_IDA,
        len: crate::session::MACHINE_LEN,
        home: Home::Address(|| {
            crate::pointers::game_address(&raw const crate::session::MACHINE) as usize
        }),
    },
    Moved {
        ida: crate::session::CLOCKS_IDA,
        len: crate::session::CLOCKS_LEN,
        home: Home::Address(|| {
            crate::pointers::game_address(&raw const crate::session::CLOCKS) as usize
        }),
    },
    // audio: the audio thread's variables and the sample records (`audio::store`)
    Moved {
        ida: crate::audio::store::STATE_IDA,
        len: crate::audio::store::STATE_END - crate::audio::store::STATE_IDA,
        home: Home::Address(|| crate::audio::store::state_addr() as usize),
    },
    Moved {
        ida: crate::audio::store::SAMPLES_IDA,
        len: crate::audio::store::SAMPLES_END - crate::audio::store::SAMPLES_IDA,
        home: Home::Address(|| crate::audio::store::sample_records_addr() as usize),
    },
    Moved {
        ida: crate::audio::store::QUEUE_IDA,
        len: crate::audio::store::QUEUE_END - crate::audio::store::QUEUE_IDA,
        home: Home::Address(|| crate::audio::store::commands_addr() as usize),
    },
    Moved {
        ida: 0x6D1DD4, // g_eaxEnvironment
        len: 4,
        home: Home::Store(&crate::audio::store::ENVIRONMENT),
    },
    // sound: the race sounds' clock, and the spot picks .. CD audio block (`sound::store`)
    Moved {
        ida: 0x4F66DD, // the options file's four kept dwords and a gap (`saves::load_options::KEPT`)
        len: 0x14,
        home: Home::Address(|| {
            (crate::pointers::game_address(crate::saves::load_options::KEPT.as_ptr())) as usize
        }),
    },
    Moved {
        ida: 0x4F66F1, // the wheel surfaces (`sound::store::WHEEL_SURFACES`)
        len: 8,
        home: Home::Address(|| {
            (crate::pointers::game_address(crate::sound::store::WHEEL_SURFACES.as_ptr())) as usize
        }),
    },
    Moved {
        ida: 0x4F66F9, // the race record's seal key (`race::replay::SEAL_KEY`)
        len: 4,
        home: Home::Store(&crate::race::replay::SEAL_KEY),
    },
    Moved {
        ida: 0x512A10, // the race sounds' objects (`sound::store::RaceSounds`)
        len: 0x2580,
        home: Home::Address(|| crate::sound::store::race_sounds_addr() as usize),
    },
    Moved {
        ida: 0x5124E4, // the race sounds' clock (`sound::store::RaceClock`)
        len: 0x24,
        home: Home::Address(|| crate::sound::store::race_clock_addr() as usize),
    },
    Moved {
        ida: 0x516358, // the surface sounds' constants (`sound::store::SurfaceSounds`)
        len: 0x44,
        home: Home::Address(|| crate::sound::store::surface_sounds_addr() as usize),
    },
    Moved {
        ida: 0x51655C, // the stage sounds (`sound::store::StageSounds`)
        len: 0x364,
        home: Home::Address(|| crate::sound::store::stage_sounds_addr() as usize),
    },
    Moved {
        ida: crate::sound::store::SpotPicks::START, // The spot voices' pickers' work and the sound kinds per spot
        len: crate::sound::store::SpotPicks::LEN,
        home: Home::Store(&crate::sound::store::SPOT_PICKS),
    },
    Moved {
        ida: crate::sound::store::SettingsApplied::START, // Set once the kept sound settings are applied
        len: crate::sound::store::SettingsApplied::LEN,
        home: Home::Store(&crate::sound::store::SETTINGS_APPLIED),
    },
    Moved {
        ida: crate::input::store::ForceFeedback::START, // The force-feedback effects' clock, surface kinds and constants
        len: crate::input::store::ForceFeedback::LEN,
        home: Home::Store(&crate::input::store::FORCE_FEEDBACK),
    },
    Moved {
        ida: crate::sound::store::SoundOptions::START, // The kept sound options
        len: crate::sound::store::SoundOptions::LEN,
        home: Home::Store(&crate::sound::store::SOUND_OPTIONS),
    },
    Moved {
        ida: crate::sound::store::CdAudio::START, // The CD audio's state
        len: crate::sound::store::CdAudio::LEN,
        home: Home::Store(&crate::sound::store::CD_AUDIO),
    },
    // input: the devices, keyboard and mouse (`input::store`, typed)
    Moved {
        ida: crate::input::store::DEVICES_IDA,
        len: crate::input::store::DEVICES_LEN,
        home: Home::Address(|| crate::input::store::devices_addr() as usize),
    },
    Moved {
        ida: crate::input::store::KEYBOARD_IDA,
        len: crate::input::store::KEYBOARD_LEN,
        home: Home::Address(|| crate::input::store::keyboard_addr() as usize),
    },
    Moved {
        ida: crate::input::store::MOUSE_IDA,
        len: crate::input::store::MOUSE_LEN,
        home: Home::Address(|| crate::input::store::mouse_addr() as usize),
    },
    Moved {
        ida: 0x511B0C, // g_nextMenuSoundTime
        len: 4,
        home: Home::Store(&crate::sound::store::NEXT_MENU_SOUND_TIME),
    },
    Moved {
        ida: 0x6218F0, // g_raceSoundsOn
        len: 4,
        home: Home::Store(&crate::sound::store::RACE_SOUNDS_ON),
    },
    Moved {
        ida: crate::sound::store::CAR_STREAM_IDA, // the car stream's place, list and switch
        len: 12,
        home: Home::Address(|| crate::sound::store::CAR_STREAM.place.addr() as usize),
    },
    // menu: the front end's variables, typed (phase 7): the savegame preview (`saves::preview_store`),
    // the controller screen (`screens::controls_store`), the pages, car numbers, screen flow, loop
    // flags, key tables, option copies, championship view, names, event state and drawing work
    // variables (`menu::*_store`), the players, cars, cheats, results, arcade and captions
    // (`menu::players_store` ..), the menus (`menu::menus_store`) and the draw lists
    // (`menu::draw_lists_store`)
    Moved {
        ida: crate::saves::preview_store::START,
        len: crate::saves::preview_store::END - crate::saves::preview_store::START,
        home: Home::Address(|| crate::saves::preview_store::savegame_preview_addr() as usize),
    },
    Moved {
        ida: crate::screens::controls_store::START,
        len: crate::screens::controls_store::END - crate::screens::controls_store::START,
        home: Home::Address(|| crate::screens::controls_store::controls_screen_addr() as usize),
    },
    Moved {
        ida: crate::menu::players_store::START,
        len: crate::menu::players_store::END - crate::menu::players_store::START,
        home: Home::Address(|| crate::menu::players_store::players_addr() as usize),
    },
    Moved {
        ida: crate::menu::cars_store::START,
        len: crate::menu::cars_store::END - crate::menu::cars_store::START,
        home: Home::Address(|| crate::menu::cars_store::cars_addr() as usize),
    },
    Moved {
        ida: crate::menu::cheats_store::START,
        len: crate::menu::cheats_store::END - crate::menu::cheats_store::START,
        home: Home::Address(|| crate::menu::cheats_store::cheats_addr() as usize),
    },
    Moved {
        ida: crate::menu::results_store::START,
        len: crate::menu::results_store::END - crate::menu::results_store::START,
        home: Home::Address(|| crate::menu::results_store::results_addr() as usize),
    },
    Moved {
        ida: crate::menu::arcade_store::START,
        len: crate::menu::arcade_store::END - crate::menu::arcade_store::START,
        home: Home::Address(|| crate::menu::arcade_store::arcade_addr() as usize),
    },
    Moved {
        ida: crate::menu::captions_store::START,
        len: crate::menu::captions_store::END - crate::menu::captions_store::START,
        home: Home::Address(|| crate::menu::captions_store::captions_addr() as usize),
    },
    Moved {
        ida: crate::menu::pages_store::START,
        len: crate::menu::pages_store::END - crate::menu::pages_store::START,
        home: Home::Address(|| crate::menu::pages_store::menu_pages_addr() as usize),
    },
    Moved {
        ida: crate::menu::car_numbers_store::START,
        len: crate::menu::car_numbers_store::END - crate::menu::car_numbers_store::START,
        home: Home::Address(|| crate::menu::car_numbers_store::car_numbers_addr() as usize),
    },
    Moved {
        ida: crate::menu::flow_store::START,
        len: crate::menu::flow_store::END - crate::menu::flow_store::START,
        home: Home::Address(|| crate::menu::flow_store::screen_flow_addr() as usize),
    },
    Moved {
        ida: crate::menu::loop_store::START,
        len: crate::menu::loop_store::END - crate::menu::loop_store::START,
        home: Home::Address(|| crate::menu::loop_store::loop_flags_addr() as usize),
    },
    Moved {
        ida: crate::menu::keys_store::START,
        len: crate::menu::keys_store::END - crate::menu::keys_store::START,
        home: Home::Address(|| crate::menu::keys_store::key_tables_addr() as usize),
    },
    Moved {
        ida: crate::menu::option_work_store::START,
        len: crate::menu::option_work_store::END - crate::menu::option_work_store::START,
        home: Home::Address(|| crate::menu::option_work_store::option_work_addr() as usize),
    },
    Moved {
        ida: crate::menu::champ_store::START,
        len: crate::menu::champ_store::END - crate::menu::champ_store::START,
        home: Home::Address(|| crate::menu::champ_store::champ_view_addr() as usize),
    },
    Moved {
        ida: crate::menu::names_store::START,
        len: crate::menu::names_store::END - crate::menu::names_store::START,
        home: Home::Address(|| crate::menu::names_store::names_addr() as usize),
    },
    Moved {
        ida: crate::menu::event_store::START,
        len: crate::menu::event_store::END - crate::menu::event_store::START,
        home: Home::Address(|| crate::menu::event_store::event_addr() as usize),
    },
    Moved {
        ida: crate::menu::draw_work_store::START,
        len: crate::menu::draw_work_store::END - crate::menu::draw_work_store::START,
        home: Home::Address(|| crate::menu::draw_work_store::draw_work_addr() as usize),
    },
    // the race block's tail (phase 7): the menus' texts (`menu::texts_store`), the stages' tables (`records::stages_store`), the drivers (`menu::drivers_store`), the rallies' best times and the best times' memory (`saves::best_times_store`),
    // the screens' kept items (`menu::screen_items_store`), the items' sprite lists
    // (`menu::item_sprites_store`)
    Moved {
        ida: crate::menu::texts_store::START,
        len: crate::menu::texts_store::END - crate::menu::texts_store::START,
        home: Home::Address(|| crate::menu::texts_store::menu_texts_addr() as usize),
    },
    Moved {
        ida: crate::records::stages_store::START,
        len: crate::records::stages_store::END - crate::records::stages_store::START,
        home: Home::Address(|| crate::records::stages_store::stage_tables_addr() as usize),
    },
    Moved {
        ida: crate::menu::drivers_store::START,
        len: crate::menu::drivers_store::END - crate::menu::drivers_store::START,
        home: Home::Address(|| crate::menu::drivers_store::drivers_addr() as usize),
    },
    Moved {
        ida: crate::saves::best_times_store::RallyTimes::START,
        len: crate::saves::best_times_store::RallyTimes::END
            - crate::saves::best_times_store::RallyTimes::START,
        home: Home::Store(&crate::saves::best_times_store::RALLY_TIMES),
    },
    Moved {
        ida: crate::saves::best_times_store::START,
        len: crate::saves::best_times_store::END - crate::saves::best_times_store::START,
        home: Home::Address(|| crate::saves::best_times_store::best_times_addr() as usize),
    },
    Moved {
        ida: crate::menu::screen_items_store::START,
        len: crate::menu::screen_items_store::END - crate::menu::screen_items_store::START,
        home: Home::Address(|| crate::menu::screen_items_store::screen_items_addr() as usize),
    },
    Moved {
        ida: crate::menu::item_sprites_store::START,
        len: crate::menu::item_sprites_store::END - crate::menu::item_sprites_store::START,
        home: Home::Address(|| crate::menu::item_sprites_store::item_sprites_addr() as usize),
    },
    Moved {
        ida: crate::menu::menus_store::START,
        len: crate::menu::menus_store::END - crate::menu::menus_store::START,
        home: Home::Address(|| crate::menu::menus_store::menus_addr() as usize),
    },
    Moved {
        ida: crate::menu::draw_lists_store::START,
        len: crate::menu::draw_lists_store::END - crate::menu::draw_lists_store::START,
        home: Home::Address(|| crate::menu::draw_lists_store::draw_lists_addr() as usize),
    },
    // the former `rest` module's pieces: typed, each in its owner's module
    Moved {
        ida: crate::race::tyre_store::TyreChange::START,
        len: crate::race::tyre_store::TyreChange::LEN,
        home: Home::Address(crate::race::tyre_store::TyreChange::home),
    },
    Moved {
        ida: crate::race::seal_store::SealWork::START,
        len: crate::race::seal_store::SealWork::LEN,
        home: Home::Address(crate::race::seal_store::SealWork::home),
    },
    Moved {
        ida: crate::system::set_error_mode::ErrorMode::START,
        len: crate::system::set_error_mode::ErrorMode::LEN,
        home: Home::Store(&crate::system::set_error_mode::ERROR_MODE),
    },
    Moved {
        ida: crate::system::start_texts::StartTexts::START,
        len: crate::system::start_texts::StartTexts::LEN,
        home: Home::Address(crate::system::start_texts::StartTexts::home),
    },
    Moved {
        ida: crate::input::guids::InputGuids::START,
        len: crate::input::guids::InputGuids::LEN,
        home: Home::Address(crate::input::guids::InputGuids::home),
    },
    Moved {
        ida: crate::sound::sets_store::MenuSounds::START,
        len: crate::sound::sets_store::MenuSounds::LEN,
        home: Home::Address(crate::sound::sets_store::MenuSounds::home),
    },
    Moved {
        ida: crate::sound::sets_store::RaceSoundSets::START,
        len: crate::sound::sets_store::RaceSoundSets::LEN,
        home: Home::Address(crate::sound::sets_store::RaceSoundSets::home),
    },
    Moved {
        ida: crate::sound::sets_store::CueLists::START,
        len: crate::sound::sets_store::CueLists::LEN,
        home: Home::Address(crate::sound::sets_store::CueLists::home),
    },
    Moved {
        ida: crate::sound::sets_store::SoundKinds::START,
        len: crate::sound::sets_store::SoundKinds::LEN,
        home: Home::Address(crate::sound::sets_store::SoundKinds::home),
    },
    Moved {
        ida: crate::sound::sets_store::SurfaceSets::START,
        len: crate::sound::sets_store::SurfaceSets::LEN,
        home: Home::Address(crate::sound::sets_store::SurfaceSets::home),
    },
    Moved {
        ida: crate::sound::sets_store::SpotVoices::START,
        len: crate::sound::sets_store::SpotVoices::LEN,
        home: Home::Address(crate::sound::sets_store::SpotVoices::home),
    },
    Moved {
        ida: crate::install::vars::InstallerRun::START,
        len: crate::install::vars::InstallerRun::LEN,
        home: Home::Address(crate::install::vars::InstallerRun::home),
    },
    Moved {
        ida: crate::install::vars::InstallerNames::START,
        len: crate::install::vars::InstallerNames::LEN,
        home: Home::Address(crate::install::vars::InstallerNames::home),
    },
    Moved {
        ida: crate::install::vars::InstallerPages::START,
        len: crate::install::vars::InstallerPages::LEN,
        home: Home::Address(crate::install::vars::InstallerPages::home),
    },
    Moved {
        ida: crate::install::vars::InstallerPaths::START,
        len: crate::install::vars::InstallerPaths::LEN,
        home: Home::Address(crate::install::vars::InstallerPaths::home),
    },
    Moved {
        ida: crate::install::vars::InstallerPalette::START,
        len: crate::install::vars::InstallerPalette::LEN,
        home: Home::Address(crate::install::vars::InstallerPalette::home),
    },
    Moved {
        ida: crate::system::load_game_data::RunCount::START,
        len: crate::system::load_game_data::RunCount::LEN,
        home: Home::Store(&crate::system::load_game_data::RUN_COUNT),
    },
    Moved {
        ida: crate::install::read_sku::Sku::START,
        len: crate::install::read_sku::Sku::LEN,
        home: Home::Store(&crate::install::read_sku::SKU),
    },
    Moved {
        ida: crate::strings::keyboard_tables::KeyboardTables::START,
        len: crate::strings::keyboard_tables::KeyboardTables::LEN,
        home: Home::Address(crate::strings::keyboard_tables::KeyboardTables::home),
    },
    Moved {
        ida: crate::strings::text_buffer::TextBuffer::START,
        len: crate::strings::text_buffer::TextBuffer::LEN,
        home: Home::Address(crate::strings::text_buffer::TextBuffer::home),
    },
    Moved {
        ida: crate::files::scratch::Scratch::START,
        len: crate::files::scratch::Scratch::LEN,
        home: Home::Address(crate::files::scratch::Scratch::home),
    },
    Moved {
        ida: crate::gfx::screen_state::ScreenMode::START,
        len: crate::gfx::screen_state::ScreenMode::LEN,
        home: Home::Address(crate::gfx::screen_state::ScreenMode::home),
    },
    Moved {
        ida: crate::gfx::screen_state::DrawSurface::START,
        len: crate::gfx::screen_state::DrawSurface::LEN,
        home: Home::Address(crate::gfx::screen_state::DrawSurface::home),
    },
    Moved {
        ida: crate::d3d::cards_store::Cards3d::START,
        len: crate::d3d::cards_store::Cards3d::LEN,
        home: Home::Address(crate::d3d::cards_store::Cards3d::home),
    },
    Moved {
        ida: crate::gfx::fonts_store::Fonts::START,
        len: crate::gfx::fonts_store::Fonts::LEN,
        home: Home::Address(crate::gfx::fonts_store::Fonts::home),
    },
    Moved {
        ida: crate::audio::names_store::SampleNames::START,
        len: crate::audio::names_store::SampleNames::LEN,
        home: Home::Address(crate::audio::names_store::SampleNames::home),
    },
    Moved {
        ida: crate::gfx::choices_store::ScreenChoices::START,
        len: crate::gfx::choices_store::ScreenChoices::LEN,
        home: Home::Address(crate::gfx::choices_store::ScreenChoices::home),
    },
    Moved {
        ida: crate::gfx::jade_store::JadeFont::START,
        len: crate::gfx::jade_store::JadeFont::LEN,
        home: Home::Address(crate::gfx::jade_store::JadeFont::home),
    },
    // race: the image's zeroed tail (`race::buffers_store`)
    Moved {
        ida: crate::race::buffers_store::START,
        len: crate::race::buffers_store::END - crate::race::buffers_store::START,
        home: Home::Address(crate::race::buffers_store::home),
    },
    // race: the race core (`race::store`), the ranges around the holes and the left pieces
    Moved {
        ida: crate::race::store::OWN[0].0,
        len: crate::race::store::OWN[0].1 - crate::race::store::OWN[0].0,
        home: Home::Address(|| crate::race::store::home(crate::race::store::OWN[0].0)),
    },
    Moved {
        ida: crate::race::store::OWN[1].0,
        len: crate::race::store::OWN[1].1 - crate::race::store::OWN[1].0,
        home: Home::Address(|| crate::race::store::home(crate::race::store::OWN[1].0)),
    },
    Moved {
        ida: crate::race::store::OWN[2].0,
        len: crate::race::store::OWN[2].1 - crate::race::store::OWN[2].0,
        home: Home::Address(|| crate::race::store::home(crate::race::store::OWN[2].0)),
    },
    Moved {
        ida: crate::race::store::OWN[3].0,
        len: crate::race::store::OWN[3].1 - crate::race::store::OWN[3].0,
        home: Home::Address(|| crate::race::store::home(crate::race::store::OWN[3].0)),
    },
    Moved {
        ida: crate::race::store::OWN[4].0,
        len: crate::race::store::OWN[4].1 - crate::race::store::OWN[4].0,
        home: Home::Address(|| crate::race::store::home(crate::race::store::OWN[4].0)),
    },
    Moved {
        ida: crate::race::store::OWN[5].0,
        len: crate::race::store::OWN[5].1 - crate::race::store::OWN[5].0,
        home: Home::Address(|| crate::race::store::home(crate::race::store::OWN[5].0)),
    },
    Moved {
        ida: crate::race::store::OWN[6].0,
        len: crate::race::store::OWN[6].1 - crate::race::store::OWN[6].0,
        home: Home::Address(|| crate::race::store::home(crate::race::store::OWN[6].0)),
    },
    Moved {
        ida: crate::race::store::OWN[7].0,
        len: crate::race::store::OWN[7].1 - crate::race::store::OWN[7].0,
        home: Home::Address(|| crate::race::store::home(crate::race::store::OWN[7].0)),
    },
    Moved {
        ida: crate::race::store::OWN[8].0,
        len: crate::race::store::OWN[8].1 - crate::race::store::OWN[8].0,
        home: Home::Address(|| crate::race::store::home(crate::race::store::OWN[8].0)),
    },
    Moved {
        ida: crate::race::store::OWN[9].0,
        len: crate::race::store::OWN[9].1 - crate::race::store::OWN[9].0,
        home: Home::Address(|| crate::race::store::home(crate::race::store::OWN[9].0)),
    },
    Moved {
        ida: crate::race::store::OWN[10].0,
        len: crate::race::store::OWN[10].1 - crate::race::store::OWN[10].0,
        home: Home::Address(|| crate::race::store::home(crate::race::store::OWN[10].0)),
    },
    Moved {
        ida: crate::race::store::OWN[11].0,
        len: crate::race::store::OWN[11].1 - crate::race::store::OWN[11].0,
        home: Home::Address(|| crate::race::store::home(crate::race::store::OWN[11].0)),
    },
    Moved {
        ida: crate::race::store::OWN[12].0,
        len: crate::race::store::OWN[12].1 - crate::race::store::OWN[12].0,
        home: Home::Address(|| crate::race::store::home(crate::race::store::OWN[12].0)),
    },
    Moved {
        ida: crate::race::store::OWN[13].0,
        len: crate::race::store::OWN[13].1 - crate::race::store::OWN[13].0,
        home: Home::Address(|| crate::race::store::home(crate::race::store::OWN[13].0)),
    },
    Moved {
        ida: crate::race::store::OWN[14].0,
        len: crate::race::store::OWN[14].1 - crate::race::store::OWN[14].0,
        home: Home::Address(|| crate::race::store::home(crate::race::store::OWN[14].0)),
    },
    Moved {
        ida: crate::race::store::OWN[15].0,
        len: crate::race::store::OWN[15].1 - crate::race::store::OWN[15].0,
        home: Home::Address(|| crate::race::store::home(crate::race::store::OWN[15].0)),
    },
    Moved {
        ida: crate::race::store::OWN[16].0,
        len: crate::race::store::OWN[16].1 - crate::race::store::OWN[16].0,
        home: Home::Address(|| crate::race::store::home(crate::race::store::OWN[16].0)),
    },
    Moved {
        ida: crate::race::store::OWN[17].0,
        len: crate::race::store::OWN[17].1 - crate::race::store::OWN[17].0,
        home: Home::Address(|| crate::race::store::home(crate::race::store::OWN[17].0)),
    },
    Moved {
        ida: crate::race::store::OWN[18].0,
        len: crate::race::store::OWN[18].1 - crate::race::store::OWN[18].0,
        home: Home::Address(|| crate::race::store::home(crate::race::store::OWN[18].0)),
    },
    Moved {
        ida: crate::race::store::OWN[19].0,
        len: crate::race::store::OWN[19].1 - crate::race::store::OWN[19].0,
        home: Home::Address(|| crate::race::store::home(crate::race::store::OWN[19].0)),
    },
    Moved {
        ida: crate::race::store::OWN[20].0,
        len: crate::race::store::OWN[20].1 - crate::race::store::OWN[20].0,
        home: Home::Address(|| crate::race::store::home(crate::race::store::OWN[20].0)),
    },
    Moved {
        ida: crate::race::store::OWN[21].0,
        len: crate::race::store::OWN[21].1 - crate::race::store::OWN[21].0,
        home: Home::Address(|| crate::race::store::home(crate::race::store::OWN[21].0)),
    },
    Moved {
        ida: crate::race::store::OWN[22].0,
        len: crate::race::store::OWN[22].1 - crate::race::store::OWN[22].0,
        home: Home::Address(|| crate::race::store::home(crate::race::store::OWN[22].0)),
    },
    // race: the groups that left the race core together (`race::groups_store`)
    Moved {
        ida: crate::race::groups_store::VIEWER_AND_KINDS_RANGE.0,
        len: crate::race::groups_store::VIEWER_AND_KINDS_RANGE.1
            - crate::race::groups_store::VIEWER_AND_KINDS_RANGE.0,
        home: Home::Address(crate::race::groups_store::viewer_and_kinds_home),
    },
    Moved {
        ida: crate::race::groups_store::LIGHTS_AND_GEAR_RANGE.0,
        len: crate::race::groups_store::LIGHTS_AND_GEAR_RANGE.1
            - crate::race::groups_store::LIGHTS_AND_GEAR_RANGE.0,
        home: Home::Address(crate::race::groups_store::lights_and_gear_home),
    },
    Moved {
        ida: crate::race::groups_store::CARS_AND_REPLAY_RANGE.0,
        len: crate::race::groups_store::CARS_AND_REPLAY_RANGE.1
            - crate::race::groups_store::CARS_AND_REPLAY_RANGE.0,
        home: Home::Address(crate::race::groups_store::cars_and_replay_home),
    },
    Moved {
        ida: crate::race::groups_store::PANEL_AND_OVERLAY_RANGE.0,
        len: crate::race::groups_store::PANEL_AND_OVERLAY_RANGE.1
            - crate::race::groups_store::PANEL_AND_OVERLAY_RANGE.0,
        home: Home::Address(crate::race::groups_store::panel_and_overlay_home),
    },
    // race: the pieces that left the race core for statics of their own (`race::store::LEFT`)
    Moved {
        ida: crate::physics::split_tables_store::START,
        len: crate::physics::split_tables_store::END - crate::physics::split_tables_store::START,
        home: Home::Address(|| crate::physics::split_tables_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::race::league_store::START,
        len: crate::race::league_store::END - crate::race::league_store::START,
        home: Home::Address(|| crate::race::league_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::race::lights_store::START,
        len: crate::race::lights_store::END - crate::race::lights_store::START,
        home: Home::Address(|| crate::race::lights_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::race::level_paths_store::START,
        len: crate::race::level_paths_store::END - crate::race::level_paths_store::START,
        home: Home::Address(|| crate::race::level_paths_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::physics::pace_store::START,
        len: crate::physics::pace_store::END - crate::physics::pace_store::START,
        home: Home::Address(|| crate::physics::pace_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::physics::ground_probe_store::START,
        len: crate::physics::ground_probe_store::END - crate::physics::ground_probe_store::START,
        home: Home::Address(|| crate::physics::ground_probe_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::race::panel_clock_store::START,
        len: crate::race::panel_clock_store::END - crate::race::panel_clock_store::START,
        home: Home::Address(|| crate::race::panel_clock_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::scene::object_slots_store::START,
        len: crate::scene::object_slots_store::END - crate::scene::object_slots_store::START,
        home: Home::Address(|| crate::scene::object_slots_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::scene::light_tables_store::START,
        len: crate::scene::light_tables_store::END - crate::scene::light_tables_store::START,
        home: Home::Address(|| crate::scene::light_tables_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::race::error_list_store::START,
        len: crate::race::error_list_store::END - crate::race::error_list_store::START,
        home: Home::Address(|| crate::race::error_list_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::race::flares_store::START,
        len: crate::race::flares_store::END - crate::race::flares_store::START,
        home: Home::Address(|| crate::race::flares_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::race::shadow_skids_store::START,
        len: crate::race::shadow_skids_store::END - crate::race::shadow_skids_store::START,
        home: Home::Address(|| crate::race::shadow_skids_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::drive::finish_store::START,
        len: crate::drive::finish_store::END - crate::drive::finish_store::START,
        home: Home::Address(|| crate::drive::finish_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::race::replay_file_store::START,
        len: crate::race::replay_file_store::END - crate::race::replay_file_store::START,
        home: Home::Address(|| crate::race::replay_file_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::race::viewer_state_store::START,
        len: crate::race::viewer_state_store::END - crate::race::viewer_state_store::START,
        home: Home::Address(|| crate::race::viewer_state_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::race::numbers_store::START,
        len: crate::race::numbers_store::END - crate::race::numbers_store::START,
        home: Home::Address(|| crate::race::numbers_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::race::scratch_store::START,
        len: crate::race::scratch_store::END - crate::race::scratch_store::START,
        home: Home::Address(|| crate::race::scratch_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::race::course_lists_store::START,
        len: crate::race::course_lists_store::END - crate::race::course_lists_store::START,
        home: Home::Address(|| crate::race::course_lists_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::race::car_specs_store::START,
        len: crate::race::car_specs_store::END - crate::race::car_specs_store::START,
        home: Home::Address(|| crate::race::car_specs_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::race::course_store::START,
        len: crate::race::course_store::END - crate::race::course_store::START,
        home: Home::Address(|| crate::race::course_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::scene::object_tables_store::START,
        len: crate::scene::object_tables_store::END - crate::scene::object_tables_store::START,
        home: Home::Address(|| crate::scene::object_tables_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::scene::texture_places_store::START,
        len: crate::scene::texture_places_store::END - crate::scene::texture_places_store::START,
        home: Home::Address(|| crate::scene::texture_places_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::race::matrices_store::START,
        len: crate::race::matrices_store::END - crate::race::matrices_store::START,
        home: Home::Address(|| crate::race::matrices_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::race::tables_store::START,
        len: crate::race::tables_store::END - crate::race::tables_store::START,
        home: Home::Address(|| crate::race::tables_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::race::groups_store::VIEW_AND_RECORDS_RANGE.0,
        len: crate::race::groups_store::VIEW_AND_RECORDS_RANGE.1
            - crate::race::groups_store::VIEW_AND_RECORDS_RANGE.0,
        home: Home::Address(crate::race::groups_store::view_and_records_home),
    },
    Moved {
        ida: crate::physics::camera_store::START,
        len: crate::physics::camera_store::END - crate::physics::camera_store::START,
        home: Home::Address(|| crate::physics::camera_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::race::prefs_store::START,
        len: crate::race::prefs_store::END - crate::race::prefs_store::START,
        home: Home::Address(|| crate::race::prefs_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::scene::file_numbers_store::START,
        len: crate::scene::file_numbers_store::END - crate::scene::file_numbers_store::START,
        home: Home::Address(|| crate::scene::file_numbers_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::physics::tables_store::START,
        len: crate::physics::tables_store::END - crate::physics::tables_store::START,
        home: Home::Address(|| crate::physics::tables_store::tables_addr() as usize),
    },
    Moved {
        ida: crate::race::vars_store::START,
        len: crate::race::vars_store::END - crate::race::vars_store::START,
        home: Home::Address(|| crate::race::vars_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::scene::draw_store::START,
        len: crate::scene::draw_store::END - crate::scene::draw_store::START,
        home: Home::Address(|| crate::scene::draw_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::scene::light_store::START,
        len: crate::scene::light_store::END - crate::scene::light_store::START,
        home: Home::Address(|| crate::scene::light_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::race::view_store::START,
        len: crate::race::view_store::END - crate::race::view_store::START,
        home: Home::Address(|| crate::race::view_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::race::level_grid_store::START,
        len: crate::race::level_grid_store::END - crate::race::level_grid_store::START,
        home: Home::Address(|| crate::race::level_grid_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::race::frame_store::START,
        len: crate::race::frame_store::END - crate::race::frame_store::START,
        home: Home::Address(|| crate::race::frame_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::race::sound_spots_store::START,
        len: crate::race::sound_spots_store::END - crate::race::sound_spots_store::START,
        home: Home::Address(|| crate::race::sound_spots_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::physics::contact_store::START,
        len: crate::physics::contact_store::END - crate::physics::contact_store::START,
        home: Home::Address(|| crate::physics::contact_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::drive::replay_frames_store::START,
        len: crate::drive::replay_frames_store::END - crate::drive::replay_frames_store::START,
        home: Home::Address(|| crate::drive::replay_frames_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::physics::car_store::START,
        len: crate::physics::car_store::END - crate::physics::car_store::START,
        home: Home::Address(|| crate::physics::car_store::car_addr() as usize),
    },
    Moved {
        ida: crate::race::work_store::START,
        len: crate::race::work_store::END - crate::race::work_store::START,
        home: Home::Address(|| crate::race::work_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::race::text_state_store::START,
        len: crate::race::text_state_store::END - crate::race::text_state_store::START,
        home: Home::Address(|| crate::race::text_state_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::race::clock_ticks_store::START,
        len: crate::race::clock_ticks_store::END - crate::race::clock_ticks_store::START,
        home: Home::Address(|| crate::race::clock_ticks_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::race::kind_records_store::START,
        len: crate::race::kind_records_store::END - crate::race::kind_records_store::START,
        home: Home::Address(|| crate::race::kind_records_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::scene::kind_texture_store::START,
        len: crate::scene::kind_texture_store::END - crate::scene::kind_texture_store::START,
        home: Home::Address(|| crate::scene::kind_texture_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::race::frame_counter_store::START,
        len: crate::race::frame_counter_store::END - crate::race::frame_counter_store::START,
        home: Home::Address(|| crate::race::frame_counter_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::race::switches_store::START,
        len: crate::race::switches_store::END - crate::race::switches_store::START,
        home: Home::Address(|| crate::race::switches_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::scene::draw_switches_store::START,
        len: crate::scene::draw_switches_store::END - crate::scene::draw_switches_store::START,
        home: Home::Address(|| crate::scene::draw_switches_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::scene::light_work_store::START,
        len: crate::scene::light_work_store::END - crate::scene::light_work_store::START,
        home: Home::Address(|| crate::scene::light_work_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::scene::object_store::START,
        len: crate::scene::object_store::END - crate::scene::object_store::START,
        home: Home::Address(|| crate::scene::object_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::physics::ground_store::START,
        len: crate::physics::ground_store::END - crate::physics::ground_store::START,
        home: Home::Address(|| crate::physics::ground_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::race::draw_batches_store::START,
        len: crate::race::draw_batches_store::END - crate::race::draw_batches_store::START,
        home: Home::Address(|| crate::race::draw_batches_store::piece_addr() as usize),
    },
    Moved {
        ida: crate::effects::work_store::START,
        len: crate::effects::work_store::END - crate::effects::work_store::START,
        home: Home::Store(&crate::effects::work_store::MARKS_WORK),
    },
    // the menus' lists, the menus' working variables, the race's settings and results and
    // the damage model (`menu::lists_store`, `menu::state_store`, `race::state_store`,
    // `physics::store`, typed)
    Moved {
        ida: crate::menu::lists_store::START,
        len: crate::menu::lists_store::END - crate::menu::lists_store::START,
        home: Home::Address(|| crate::menu::lists_store::menu_lists_addr() as usize),
    },
    Moved {
        ida: crate::menu::state_store::START,
        len: crate::menu::state_store::END - crate::menu::state_store::START,
        home: Home::Address(|| crate::menu::state_store::menu_state_addr() as usize),
    },
    Moved {
        ida: crate::race::state_store::START,
        len: crate::race::state_store::END - crate::race::state_store::START,
        home: Home::Address(|| crate::race::state_store::race_state_addr() as usize),
    },
    Moved {
        ida: crate::physics::store::START,
        len: crate::physics::store::END - crate::physics::store::START,
        home: Home::Address(|| crate::physics::store::damage_addr() as usize),
    },
    // marks: 0x764F40 .. 0x93CCC8 (`effects::store`), less the ranges that left it (`CARVED`),
    // in the segments of `SEGS`; then the ranges that left it, each its own typed static
    Moved {
        ida: crate::effects::store::SEGS[0].0,
        len: crate::effects::store::SEGS[0].1 - crate::effects::store::SEGS[0].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[0]),
    },
    Moved {
        ida: crate::effects::store::SEGS[1].0,
        len: crate::effects::store::SEGS[1].1 - crate::effects::store::SEGS[1].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[1]),
    },
    Moved {
        ida: crate::effects::store::SEGS[2].0,
        len: crate::effects::store::SEGS[2].1 - crate::effects::store::SEGS[2].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[2]),
    },
    Moved {
        ida: crate::effects::store::SEGS[3].0,
        len: crate::effects::store::SEGS[3].1 - crate::effects::store::SEGS[3].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[3]),
    },
    Moved {
        ida: crate::effects::store::SEGS[4].0,
        len: crate::effects::store::SEGS[4].1 - crate::effects::store::SEGS[4].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[4]),
    },
    Moved {
        ida: crate::effects::store::SEGS[5].0,
        len: crate::effects::store::SEGS[5].1 - crate::effects::store::SEGS[5].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[5]),
    },
    Moved {
        ida: crate::effects::store::SEGS[6].0,
        len: crate::effects::store::SEGS[6].1 - crate::effects::store::SEGS[6].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[6]),
    },
    Moved {
        ida: crate::effects::store::SEGS[7].0,
        len: crate::effects::store::SEGS[7].1 - crate::effects::store::SEGS[7].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[7]),
    },
    Moved {
        ida: crate::effects::store::SEGS[8].0,
        len: crate::effects::store::SEGS[8].1 - crate::effects::store::SEGS[8].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[8]),
    },
    Moved {
        ida: crate::effects::store::SEGS[9].0,
        len: crate::effects::store::SEGS[9].1 - crate::effects::store::SEGS[9].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[9]),
    },
    Moved {
        ida: crate::effects::store::SEGS[10].0,
        len: crate::effects::store::SEGS[10].1 - crate::effects::store::SEGS[10].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[10]),
    },
    Moved {
        ida: crate::effects::store::SEGS[11].0,
        len: crate::effects::store::SEGS[11].1 - crate::effects::store::SEGS[11].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[11]),
    },
    Moved {
        ida: crate::effects::store::SEGS[12].0,
        len: crate::effects::store::SEGS[12].1 - crate::effects::store::SEGS[12].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[12]),
    },
    Moved {
        ida: crate::effects::store::SEGS[13].0,
        len: crate::effects::store::SEGS[13].1 - crate::effects::store::SEGS[13].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[13]),
    },
    Moved {
        ida: crate::effects::store::SEGS[14].0,
        len: crate::effects::store::SEGS[14].1 - crate::effects::store::SEGS[14].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[14]),
    },
    Moved {
        ida: crate::effects::store::SEGS[15].0,
        len: crate::effects::store::SEGS[15].1 - crate::effects::store::SEGS[15].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[15]),
    },
    Moved {
        ida: crate::effects::store::SEGS[16].0,
        len: crate::effects::store::SEGS[16].1 - crate::effects::store::SEGS[16].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[16]),
    },
    Moved {
        ida: crate::effects::store::SEGS[17].0,
        len: crate::effects::store::SEGS[17].1 - crate::effects::store::SEGS[17].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[17]),
    },
    Moved {
        ida: crate::effects::store::SEGS[18].0,
        len: crate::effects::store::SEGS[18].1 - crate::effects::store::SEGS[18].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[18]),
    },
    Moved {
        ida: crate::effects::store::SEGS[19].0,
        len: crate::effects::store::SEGS[19].1 - crate::effects::store::SEGS[19].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[19]),
    },
    Moved {
        ida: crate::effects::store::SEGS[20].0,
        len: crate::effects::store::SEGS[20].1 - crate::effects::store::SEGS[20].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[20]),
    },
    Moved {
        ida: crate::effects::store::SEGS[21].0,
        len: crate::effects::store::SEGS[21].1 - crate::effects::store::SEGS[21].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[21]),
    },
    Moved {
        ida: crate::effects::store::SEGS[22].0,
        len: crate::effects::store::SEGS[22].1 - crate::effects::store::SEGS[22].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[22]),
    },
    Moved {
        ida: crate::effects::store::SEGS[23].0,
        len: crate::effects::store::SEGS[23].1 - crate::effects::store::SEGS[23].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[23]),
    },
    Moved {
        ida: crate::effects::store::SEGS[24].0,
        len: crate::effects::store::SEGS[24].1 - crate::effects::store::SEGS[24].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[24]),
    },
    Moved {
        ida: crate::effects::store::SEGS[25].0,
        len: crate::effects::store::SEGS[25].1 - crate::effects::store::SEGS[25].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[25]),
    },
    Moved {
        ida: crate::effects::store::SEGS[26].0,
        len: crate::effects::store::SEGS[26].1 - crate::effects::store::SEGS[26].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[26]),
    },
    Moved {
        ida: crate::effects::store::SEGS[27].0,
        len: crate::effects::store::SEGS[27].1 - crate::effects::store::SEGS[27].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[27]),
    },
    Moved {
        ida: crate::effects::store::SEGS[28].0,
        len: crate::effects::store::SEGS[28].1 - crate::effects::store::SEGS[28].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[28]),
    },
    Moved {
        ida: crate::effects::store::SEGS[29].0,
        len: crate::effects::store::SEGS[29].1 - crate::effects::store::SEGS[29].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[29]),
    },
    Moved {
        ida: crate::effects::store::SEGS[30].0,
        len: crate::effects::store::SEGS[30].1 - crate::effects::store::SEGS[30].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[30]),
    },
    Moved {
        ida: crate::effects::store::SEGS[31].0,
        len: crate::effects::store::SEGS[31].1 - crate::effects::store::SEGS[31].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[31]),
    },
    Moved {
        ida: crate::effects::store::SEGS[32].0,
        len: crate::effects::store::SEGS[32].1 - crate::effects::store::SEGS[32].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[32]),
    },
    Moved {
        ida: crate::effects::store::SEGS[33].0,
        len: crate::effects::store::SEGS[33].1 - crate::effects::store::SEGS[33].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[33]),
    },
    Moved {
        ida: crate::effects::store::SEGS[34].0,
        len: crate::effects::store::SEGS[34].1 - crate::effects::store::SEGS[34].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[34]),
    },
    Moved {
        ida: crate::effects::store::SEGS[35].0,
        len: crate::effects::store::SEGS[35].1 - crate::effects::store::SEGS[35].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[35]),
    },
    Moved {
        ida: crate::effects::store::SEGS[36].0,
        len: crate::effects::store::SEGS[36].1 - crate::effects::store::SEGS[36].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[36]),
    },
    Moved {
        ida: crate::effects::store::SEGS[37].0,
        len: crate::effects::store::SEGS[37].1 - crate::effects::store::SEGS[37].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[37]),
    },
    Moved {
        ida: crate::effects::store::SEGS[38].0,
        len: crate::effects::store::SEGS[38].1 - crate::effects::store::SEGS[38].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[38]),
    },
    Moved {
        ida: crate::effects::store::SEGS[39].0,
        len: crate::effects::store::SEGS[39].1 - crate::effects::store::SEGS[39].0,
        home: Home::Store(&crate::effects::store::SEGMENTS[39]),
    },
    Moved {
        ida: crate::effects::shape_points::ShapeInputs::START,
        len: crate::effects::shape_points::ShapeInputs::LEN,
        home: Home::Store(&crate::effects::shape_points::SHAPE_INPUTS),
    },
    Moved {
        ida: crate::effects::view_vectors::ViewSources::START,
        len: crate::effects::view_vectors::ViewSources::LEN,
        home: Home::Store(&crate::effects::view_vectors::SOURCES),
    },
    Moved {
        ida: crate::effects::setup::MarksClock::START,
        len: crate::effects::setup::MarksClock::LEN,
        home: Home::Store(&crate::effects::setup::CLOCK),
    },
    Moved {
        ida: crate::effects::tuning::MarksFrame::START,
        len: crate::effects::tuning::MarksFrame::LEN,
        home: Home::Store(&crate::effects::tuning::MARKS_FRAME),
    },
    Moved {
        ida: crate::effects::tuning::MarksFloats::START,
        len: crate::effects::tuning::MarksFloats::LEN,
        home: Home::Store(&crate::effects::tuning::MARKS_FLOATS),
    },
    Moved {
        ida: crate::effects::tuning::MarksKindFloats::START,
        len: crate::effects::tuning::MarksKindFloats::LEN,
        home: Home::Store(&crate::effects::tuning::MARKS_KIND_FLOATS),
    },
    Moved {
        ida: crate::effects::setup::SettingsPointer::START,
        len: crate::effects::setup::SettingsPointer::LEN,
        home: Home::Store(&crate::effects::setup::SETTINGS),
    },
    Moved {
        ida: crate::effects::pong::PongState::START,
        len: crate::effects::pong::PongState::LEN,
        home: Home::Store(&crate::effects::pong::PONG),
    },
    Moved {
        ida: crate::effects::clip::ClipScreen::START,
        len: crate::effects::clip::ClipScreen::LEN,
        home: Home::Store(&crate::effects::clip::CLIP),
    },
    Moved {
        ida: crate::effects::pong::PongRules::START,
        len: crate::effects::pong::PongRules::LEN,
        home: Home::Store(&crate::effects::pong::PONG_RULES),
    },
    Moved {
        ida: crate::effects::kind_values::MarksKind::START,
        len: crate::effects::kind_values::MarksKind::LEN,
        home: Home::Store(&crate::effects::kind_values::MARKS_KIND),
    },
    Moved {
        ida: crate::effects::view_vectors::KeptView::START,
        len: crate::effects::view_vectors::KeptView::LEN,
        home: Home::Store(&crate::effects::view_vectors::KEPT),
    },
    Moved {
        ida: crate::effects::splines::SplineHead::START,
        len: crate::effects::splines::SplineHead::LEN,
        home: Home::Store(&crate::effects::splines::SPLINE_HEAD),
    },
    Moved {
        ida: crate::effects::splines::SplineWork::START,
        len: crate::effects::splines::SplineWork::LEN,
        home: Home::Store(&crate::effects::splines::SPLINE_WORK),
    },
    Moved {
        ida: crate::effects::splines::SplineTables::START,
        len: crate::effects::splines::SplineTables::LEN,
        home: Home::Store(&crate::effects::splines::SPLINE_TABLES),
    },
    Moved {
        ida: crate::effects::splines::SplineKinds::START,
        len: crate::effects::splines::SplineKinds::LEN,
        home: Home::Store(&crate::effects::splines::SPLINE_KINDS),
    },
    Moved {
        ida: crate::effects::splines::SplinePath::START,
        len: crate::effects::splines::SplinePath::LEN,
        home: Home::Store(&crate::effects::splines::SPLINE_PATH),
    },
    Moved {
        ida: crate::effects::splines::SplineConstants::START,
        len: crate::effects::splines::SplineConstants::LEN,
        home: Home::Store(&crate::effects::splines::SPLINE_CONSTANTS),
    },
    Moved {
        ida: crate::effects::edges::EdgeMask::START,
        len: crate::effects::edges::EdgeMask::LEN,
        home: Home::Store(&crate::effects::edges::EDGE_MASK),
    },
    Moved {
        ida: crate::effects::tilt_shape::ShapeTilt::START,
        len: crate::effects::tilt_shape::ShapeTilt::LEN,
        home: Home::Store(&crate::effects::tilt_shape::SHAPE_TILT),
    },
    Moved {
        ida: crate::effects::edges::KindBoxes::START,
        len: crate::effects::edges::KindBoxes::LEN,
        home: Home::Store(&crate::effects::edges::KIND_BOXES),
    },
    Moved {
        ida: crate::effects::windscreen_drop_state::WindscreenDropState::START,
        len: crate::effects::windscreen_drop_state::WindscreenDropState::LEN,
        home: Home::Store(&crate::effects::windscreen_drop_state::WINDSCREEN_DROP_STATE),
    },
    Moved {
        ida: crate::effects::windscreen_drop_state::WindscreenDropWork::START,
        len: crate::effects::windscreen_drop_state::WindscreenDropWork::LEN,
        home: Home::Store(&crate::effects::windscreen_drop_state::WINDSCREEN_DROP_WORK),
    },
    Moved {
        ida: crate::effects::windscreen_drop_vars::WindscreenDropVars::START,
        len: crate::effects::windscreen_drop_vars::WindscreenDropVars::LEN,
        home: Home::Store(&crate::effects::windscreen_drop_vars::HEX),
    },
    Moved {
        ida: crate::effects::mask_vars::MaskVars::START,
        len: crate::effects::mask_vars::MaskVars::LEN,
        home: Home::Store(&crate::effects::mask_vars::MASKS),
    },
    Moved {
        ida: crate::effects::mask_picks::MaskPicks::START,
        len: crate::effects::mask_picks::MaskPicks::LEN,
        home: Home::Store(&crate::effects::mask_picks::PICKS),
    },
    Moved {
        ida: crate::effects::shape_work::ShapeWork::START,
        len: crate::effects::shape_work::ShapeWork::LEN,
        home: Home::Store(&crate::effects::shape_work::SHAPE_WORK),
    },
    Moved {
        ida: crate::effects::shape_points::ShapeSetTuning::START,
        len: crate::effects::shape_points::ShapeSetTuning::LEN,
        home: Home::Store(&crate::effects::shape_points::SHAPE_SET_TUNING),
    },
    Moved {
        ida: crate::effects::mask_constants::MaskConstants::START,
        len: crate::effects::mask_constants::MaskConstants::LEN,
        home: Home::Store(&crate::effects::mask_constants::MASK_CONSTANTS),
    },
    Moved {
        ida: crate::effects::windscreen_flakes_store::WindscreenFlakeTiming::START,
        len: crate::effects::windscreen_flakes_store::WindscreenFlakeTiming::LEN,
        home: Home::Store(&crate::effects::windscreen_flakes_store::WINDSCREEN_FLAKE_TIMING),
    },
    Moved {
        ida: crate::effects::windscreen_flakes_store::WindscreenFlakeTables::START,
        len: crate::effects::windscreen_flakes_store::WindscreenFlakeTables::LEN,
        home: Home::Store(&crate::effects::windscreen_flakes_store::WINDSCREEN_FLAKE_TABLES),
    },
    Moved {
        ida: crate::effects::windscreen_flakes_store::WindscreenFlakeItems::START,
        len: crate::effects::windscreen_flakes_store::WindscreenFlakeItems::LEN,
        home: Home::Store(&crate::effects::windscreen_flakes_store::WINDSCREEN_FLAKE_ITEMS),
    },
    Moved {
        ida: crate::effects::windscreen_flakes_store::WindscreenFlakeState::START,
        len: crate::effects::windscreen_flakes_store::WindscreenFlakeState::LEN,
        home: Home::Store(&crate::effects::windscreen_flakes_store::WINDSCREEN_FLAKE_STATE),
    },
    Moved {
        ida: crate::effects::edges::EdgeSettings::START,
        len: crate::effects::edges::EdgeSettings::LEN,
        home: Home::Store(&crate::effects::edges::EDGE_SETTINGS),
    },
    Moved {
        ida: crate::effects::rain::RainState::START,
        len: crate::effects::rain::RainState::LEN,
        home: Home::Store(&crate::effects::rain::RAIN),
    },
    Moved {
        ida: crate::effects::rain::RainRules::START,
        len: crate::effects::rain::RainRules::LEN,
        home: Home::Store(&crate::effects::rain::RAIN_RULES),
    },
    Moved {
        ida: crate::effects::snow::SnowTables::START,
        len: crate::effects::snow::SnowTables::LEN,
        home: Home::Store(&crate::effects::snow::SNOW_TABLES),
    },
    Moved {
        ida: crate::effects::snow::SnowState::START,
        len: crate::effects::snow::SnowState::LEN,
        home: Home::Store(&crate::effects::snow::SNOW),
    },
    Moved {
        ida: crate::effects::snow::SnowCurve::START,
        len: crate::effects::snow::SnowCurve::LEN,
        home: Home::Store(&crate::effects::snow::SNOW_CURVE),
    },
    Moved {
        ida: crate::effects::snow::SnowCurveVars::START,
        len: crate::effects::snow::SnowCurveVars::LEN,
        home: Home::Store(&crate::effects::snow::SNOW_VARS),
    },
    Moved {
        ida: crate::effects::snow::SnowLight::START,
        len: crate::effects::snow::SnowLight::LEN,
        home: Home::Store(&crate::effects::snow::SNOW_LIGHT),
    },
    Moved {
        ida: crate::effects::blocks::PlacesBlock::START,
        len: crate::effects::blocks::PlacesBlock::LEN,
        home: Home::Store(&crate::effects::blocks::PLACES_BLOCK),
    },
    Moved {
        ida: crate::effects::profiles::ProfileRings::START,
        len: crate::effects::profiles::ProfileRings::LEN,
        home: Home::Store(&crate::effects::profiles::PROFILE_RINGS),
    },
    Moved {
        ida: crate::effects::profiles::ProfileTables::START,
        len: crate::effects::profiles::ProfileTables::LEN,
        home: Home::Store(&crate::effects::profiles::PROFILE_TABLES),
    },
    Moved {
        ida: crate::effects::frame_state::DrawWork::START,
        len: crate::effects::frame_state::DrawWork::LEN,
        home: Home::Store(&crate::effects::frame_state::DRAW_WORK),
    },
    Moved {
        ida: crate::effects::blocks::MarkCorners::START,
        len: crate::effects::blocks::MarkCorners::LEN,
        home: Home::Store(&crate::effects::blocks::MARK_CORNERS),
    },
    Moved {
        ida: crate::effects::frame_state::ViewVars::START,
        len: crate::effects::frame_state::ViewVars::LEN,
        home: Home::Store(&crate::effects::frame_state::VIEW_VARS),
    },
    Moved {
        ida: crate::effects::profiles::ProfileLists::START,
        len: crate::effects::profiles::ProfileLists::LEN,
        home: Home::Store(&crate::effects::profiles::PROFILE_LISTS),
    },
    Moved {
        ida: crate::effects::frame_state::DrawFrame::START,
        len: crate::effects::frame_state::DrawFrame::LEN,
        home: Home::Store(&crate::effects::frame_state::DRAW_FRAME),
    },
    Moved {
        ida: crate::effects::waves::MarksSines::START,
        len: crate::effects::waves::MarksSines::LEN,
        home: Home::Store(&crate::effects::waves::SINES),
    },
    Moved {
        ida: crate::effects::frame_state::FlareParts::START,
        len: crate::effects::frame_state::FlareParts::LEN,
        home: Home::Store(&crate::effects::frame_state::FLARE_PARTS),
    },
    Moved {
        ida: crate::effects::waves::WaveLevels::START,
        len: crate::effects::waves::WaveLevels::LEN,
        home: Home::Store(&crate::effects::waves::WAVE_LEVELS),
    },
    Moved {
        ida: crate::effects::waves::WaveState::START,
        len: crate::effects::waves::WaveState::LEN,
        home: Home::Store(&crate::effects::waves::WAVE),
    },
    Moved {
        ida: crate::effects::frame_state::FlareScales::START,
        len: crate::effects::frame_state::FlareScales::LEN,
        home: Home::Store(&crate::effects::frame_state::FLARE_SCALES),
    },
    Moved {
        ida: crate::effects::blocks::MarkLinks::START,
        len: crate::effects::blocks::MarkLinks::LEN,
        home: Home::Store(&crate::effects::blocks::MARK_LINKS),
    },
    Moved {
        ida: crate::effects::blocks::CornerSlopes::START,
        len: crate::effects::blocks::CornerSlopes::LEN,
        home: Home::Store(&crate::effects::blocks::CORNER_SLOPES),
    },
    Moved {
        ida: crate::effects::run_particles::TrailColours::START,
        len: crate::effects::run_particles::TrailColours::LEN,
        home: Home::Store(&crate::effects::run_particles::TRAIL_PALETTE),
    },
    Moved {
        ida: crate::effects::spread_point::SpreadRandom::START,
        len: crate::effects::spread_point::SpreadRandom::LEN,
        home: Home::Store(&crate::effects::spread_point::SPREAD_RANDOM),
    },
    Moved {
        ida: crate::effects::kind_store::Quad1::START,
        len: crate::effects::kind_store::Quad1::LEN,
        home: Home::Store(&crate::effects::kind_store::QUAD_1),
    },
    Moved {
        ida: crate::effects::kind_store::Tables1::START,
        len: crate::effects::kind_store::Tables1::LEN,
        home: Home::Store(&crate::effects::kind_store::TABLES_1),
    },
    Moved {
        ida: crate::effects::kind_store::KindExtras1::START,
        len: crate::effects::kind_store::KindExtras1::LEN,
        home: Home::Store(&crate::effects::kind_store::EXTRAS_1),
    },
    Moved {
        ida: crate::effects::kind_store::Consts1::START,
        len: crate::effects::kind_store::Consts1::LEN,
        home: Home::Store(&crate::effects::kind_store::CONSTS_1),
    },
    Moved {
        ida: crate::effects::kind_store::Quad2::START,
        len: crate::effects::kind_store::Quad2::LEN,
        home: Home::Store(&crate::effects::kind_store::QUAD_2),
    },
    Moved {
        ida: crate::effects::kind_store::Tables2::START,
        len: crate::effects::kind_store::Tables2::LEN,
        home: Home::Store(&crate::effects::kind_store::TABLES_2),
    },
    Moved {
        ida: crate::effects::kind_store::KindExtra2::START,
        len: crate::effects::kind_store::KindExtra2::LEN,
        home: Home::Store(&crate::effects::kind_store::EXTRA_2),
    },
    Moved {
        ida: crate::effects::kind_store::Consts2::START,
        len: crate::effects::kind_store::Consts2::LEN,
        home: Home::Store(&crate::effects::kind_store::CONSTS_2),
    },
    Moved {
        ida: crate::effects::kind_store::Quad3::START,
        len: crate::effects::kind_store::Quad3::LEN,
        home: Home::Store(&crate::effects::kind_store::QUAD_3),
    },
    Moved {
        ida: crate::effects::mask_store::GroundNames::START,
        len: crate::effects::mask_store::GroundNames::LEN,
        home: Home::Store(&crate::effects::mask_store::GROUND_NAMES),
    },
    Moved {
        ida: crate::effects::mask_store::MaskBlocks::START,
        len: crate::effects::mask_store::MaskBlocks::LEN,
        home: Home::Store(&crate::effects::mask_store::MASK_BLOCKS),
    },
    Moved {
        ida: crate::effects::mask_store::PictureCopy::START,
        len: crate::effects::mask_store::PictureCopy::LEN,
        home: Home::Store(&crate::effects::mask_store::PICTURE_COPY),
    },
    Moved {
        ida: crate::effects::mask_store::KindPictures::START,
        len: crate::effects::mask_store::KindPictures::LEN,
        home: Home::Store(&crate::effects::mask_store::KIND_PICTURES),
    },
    Moved {
        ida: crate::effects::draw_plane::PlaneTuning::START,
        len: crate::effects::draw_plane::PlaneTuning::LEN,
        home: Home::Store(&crate::effects::draw_plane::PLANE_TUNING),
    },
    Moved {
        ida: crate::effects::mark_slots::MarkSlots::START,
        len: crate::effects::mark_slots::MarkSlots::LEN,
        home: Home::Store(&crate::effects::mark_slots::SLOTS),
    },
    Moved {
        ida: crate::effects::mark_slots::SlotTuning::START,
        len: crate::effects::mark_slots::SlotTuning::LEN,
        home: Home::Store(&crate::effects::mark_slots::TUNING),
    },
    Moved {
        ida: crate::effects::emitters::EmitterVars::START,
        len: crate::effects::emitters::EmitterVars::LEN,
        home: Home::Store(&crate::effects::emitters::EMITTER),
    },
    Moved {
        ida: crate::effects::ring_store::RingWork::START,
        len: crate::effects::ring_store::RingWork::LEN,
        home: Home::Store(&crate::effects::ring_store::RING_WORK),
    },
    Moved {
        ida: crate::effects::ring_store::PictureNames::START,
        len: crate::effects::ring_store::PictureNames::LEN,
        home: Home::Store(&crate::effects::ring_store::PICTURE_NAMES),
    },
    Moved {
        ida: crate::effects::surface_store::SurfaceTables::START,
        len: crate::effects::surface_store::SurfaceTables::LEN,
        home: Home::Store(&crate::effects::surface_store::SURFACE),
    },
    Moved {
        ida: crate::effects::run_ribbons::Colours257::START,
        len: crate::effects::run_ribbons::Colours257::LEN,
        home: Home::Store(&crate::effects::run_ribbons::COLOURS_257),
    },
    Moved {
        ida: crate::effects::emit_particle_19::TwinState::START,
        len: crate::effects::emit_particle_19::TwinState::LEN,
        home: Home::Store(&crate::effects::emit_particle_19::TWIN_STATE),
    },
    Moved {
        ida: crate::effects::emit_kind_1::Kind1Choice::START,
        len: crate::effects::emit_kind_1::Kind1Choice::LEN,
        home: Home::Store(&crate::effects::emit_kind_1::KIND1_CHOICE),
    },
    Moved {
        ida: crate::effects::set_up_surface::MaskList::START,
        len: crate::effects::set_up_surface::MaskList::LEN,
        home: Home::Store(&crate::effects::set_up_surface::MASK_LIST),
    },
    Moved {
        ida: crate::effects::draw_turned_sprite::SpriteUv::START,
        len: crate::effects::draw_turned_sprite::SpriteUv::LEN,
        home: Home::Store(&crate::effects::draw_turned_sprite::SPRITE_UV),
    },
    Moved {
        ida: crate::effects::emit_particle_5::Kind5Set::START,
        len: crate::effects::emit_particle_5::Kind5Set::LEN,
        home: Home::Store(&crate::effects::emit_particle_5::KIND5_SET),
    },
    Moved {
        ida: crate::effects::emit_particle_11::Kind11Set::START,
        len: crate::effects::emit_particle_11::Kind11Set::LEN,
        home: Home::Store(&crate::effects::emit_particle_11::KIND11_SET),
    },
    Moved {
        ida: crate::effects::emit_particle_21::Kind21Set::START,
        len: crate::effects::emit_particle_21::Kind21Set::LEN,
        home: Home::Store(&crate::effects::emit_particle_21::KIND21_SET),
    },
    Moved {
        ida: crate::effects::emit_kind_1::Kind1Set::START,
        len: crate::effects::emit_kind_1::Kind1Set::LEN,
        home: Home::Store(&crate::effects::emit_kind_1::KIND1_SET),
    },
    Moved {
        ida: crate::effects::emit_bursts::FirstTuning::START,
        len: crate::effects::emit_bursts::FirstTuning::LEN,
        home: Home::Store(&crate::effects::emit_bursts::FIRST_TUNING),
    },
    Moved {
        ida: crate::effects::emit_bursts_2::SecondTuning::START,
        len: crate::effects::emit_bursts_2::SecondTuning::LEN,
        home: Home::Store(&crate::effects::emit_bursts_2::SECOND_TUNING),
    },
    Moved {
        ida: crate::effects::emit_particle_6::Kind6Set::START,
        len: crate::effects::emit_particle_6::Kind6Set::LEN,
        home: Home::Store(&crate::effects::emit_particle_6::KIND6_SET),
    },
    Moved {
        ida: crate::effects::emit_particle_13::Kind13Set::START,
        len: crate::effects::emit_particle_13::Kind13Set::LEN,
        home: Home::Store(&crate::effects::emit_particle_13::KIND13_SET),
    },
    Moved {
        ida: crate::effects::emit_particle_19::NineteenthTuning::START,
        len: crate::effects::emit_particle_19::NineteenthTuning::LEN,
        home: Home::Store(&crate::effects::emit_particle_19::NINETEENTH_TUNING),
    },
    Moved {
        ida: crate::effects::emit_particle_22::TwentySecondTuning::START,
        len: crate::effects::emit_particle_22::TwentySecondTuning::LEN,
        home: Home::Store(&crate::effects::emit_particle_22::TWENTY_SECOND_TUNING),
    },
    Moved {
        ida: crate::effects::run_ring::RingKinds::START,
        len: crate::effects::run_ring::RingKinds::LEN,
        home: Home::Store(&crate::effects::run_ring::RING_KINDS),
    },
    Moved {
        ida: crate::effects::blocks::RingPool::START,
        len: crate::effects::blocks::RingPool::LEN,
        home: Home::Store(&crate::effects::blocks::RING_POOL),
    },
    Moved {
        ida: crate::effects::run_particles::PuffWork::START,
        len: crate::effects::run_particles::PuffWork::LEN,
        home: Home::Store(&crate::effects::run_particles::PUFF_WORK),
    },
    Moved {
        ida: crate::effects::run_particles::BurstColours::START,
        len: crate::effects::run_particles::BurstColours::LEN,
        home: Home::Store(&crate::effects::run_particles::BURST_COLOURS),
    },
    Moved {
        ida: crate::effects::run_particles::ParticleTuning::START,
        len: crate::effects::run_particles::ParticleTuning::LEN,
        home: Home::Store(&crate::effects::run_particles::PARTICLE_TUNING),
    },
    Moved {
        ida: crate::effects::blend_corner::BlendWork::START,
        len: crate::effects::blend_corner::BlendWork::LEN,
        home: Home::Store(&crate::effects::blend_corner::BLEND_WORK),
    },
    Moved {
        ida: crate::effects::morph_object_points::MorphWork::START,
        len: crate::effects::morph_object_points::MorphWork::LEN,
        home: Home::Store(&crate::effects::morph_object_points::MORPH_WORK),
    },
    Moved {
        ida: crate::effects::lines::LinesWork::START,
        len: crate::effects::lines::LinesWork::LEN,
        home: Home::Store(&crate::effects::lines::LINES_WORK),
    },
    Moved {
        ida: crate::effects::run_frame::FrameK::START,
        len: crate::effects::run_frame::FrameK::LEN,
        home: Home::Store(&crate::effects::run_frame::FRAME_K),
    },
    Moved {
        ida: crate::effects::setup::ClockK::START,
        len: crate::effects::setup::ClockK::LEN,
        home: Home::Store(&crate::effects::setup::CLOCK_K),
    },
    Moved {
        ida: crate::effects::clip::GlowState::START,
        len: crate::effects::clip::GlowState::LEN,
        home: Home::Store(&crate::effects::clip::GLOW),
    },
];

/// Where the variable at IDA address `ida` lives now: its new home if it moved, else its
/// place in the image.
pub fn place(ida: usize) -> usize {
    // `game_addr` answers a moved variable's home.
    (game_addr(rva(ida))) as usize
}

/// Which 4 KiB pages of the image (by RVA) hold a byte of a moved range: `home_of_rva`'s
/// fast path (most addresses are on no such page).
const PAGES: usize = 0x6C0000 >> 12;
static MOVED_PAGES: [bool; PAGES] = moved_pages();

const fn moved_pages() -> [bool; PAGES] {
    let mut pages = [false; PAGES];
    let mut n = 0;
    while n < MOVED.len() {
        let first = (MOVED[n].ida - IMAGE_BASE) >> 12;
        let last = (MOVED[n].ida + MOVED[n].len - 1 - IMAGE_BASE) >> 12;
        let mut p = first;
        while p <= last {
            pages[p] = true;
            p += 1;
        }
        n += 1;
    }
    pages
}

/// The moved ranges by IDA address, sorted, with their homes (made at the first use).
fn sorted() -> &'static [(usize, usize, usize)] {
    static SORTED: std::sync::OnceLock<Vec<(usize, usize, usize)>> = std::sync::OnceLock::new();
    SORTED.get_or_init(|| {
        let mut v: Vec<_> = MOVED
            .iter()
            .map(|m| (m.ida, m.ida + m.len, m.home.address()))
            .collect();
        v.sort_unstable();
        v
    })
}

/// The moved range whose new home holds `addr`: its home's first byte and the byte past its
/// end (`pointers::region_of`).
pub fn home_range(addr: usize) -> Option<(usize, usize)> {
    static BY_HOME: std::sync::OnceLock<Vec<(usize, usize)>> = std::sync::OnceLock::new();
    let homes = BY_HOME.get_or_init(|| {
        let mut v: Vec<_> = sorted()
            .iter()
            .map(|&(start, end, home)| (home, home + (end - start)))
            .collect();
        v.sort_unstable();
        v
    });
    let i = homes.partition_point(|&(home, _)| home <= addr);
    let &(home, end) = homes.get(i.checked_sub(1)?)?;
    (addr < end).then_some((home, end))
}

/// Every moved range's bytes, for a test that writes the game's variables and has to put
/// them back (`restore`); hold `store::test_lock` around both.
#[cfg(test)]
pub fn snapshot() -> Vec<(usize, Vec<u8>)> {
    sorted()
        .iter()
        .map(|&(start, end, home)| {
            let bytes = unsafe {
                std::slice::from_raw_parts(crate::pointers::host_at(home).cast_const(), end - start)
            };
            (home, bytes.to_vec())
        })
        .collect()
}

/// The bytes `snapshot` took, put back.
#[cfg(test)]
pub fn restore(snap: &[(usize, Vec<u8>)]) {
    for (home, bytes) in snap {
        unsafe {
            std::ptr::copy_nonoverlapping(
                bytes.as_ptr(),
                crate::pointers::host_at(*home),
                bytes.len(),
            );
        };
    }
}

/// The new home of the byte at `rva` if it has left the image (`framework::game_addr`'s
/// resolver).
pub fn home_of_rva(rva: usize) -> Option<usize> {
    if !MOVED_PAGES.get(rva >> 12).copied().unwrap_or(false) {
        return None;
    }
    let ida = rva + IMAGE_BASE;
    let ranges = sorted();
    let i = ranges.partition_point(|&(start, _, _)| start <= ida);
    let &(start, end, home) = ranges.get(i.checked_sub(1)?)?;
    (ida < end).then(|| home + (ida - start))
}

/// Where a table's entry names (phase 13: the tables keep IDA's addresses as names, and
/// `point_into_homes` passes them over): IDA's `ida`, which must lie in `start .. start + len`
/// (the moved piece it names), at its new home. A name off the piece is refused with `what`.
pub fn named_in(
    ida: u32,
    start: usize,
    len: usize,
    what: &'static str,
) -> Result<u32, crate::errors::BadInput> {
    let at = ida as usize;
    if !(start..start + len).contains(&at) {
        return Err(crate::errors::BadInput(what));
    }
    home_of_rva(at - IMAGE_BASE)
        .map(|home| home as u32)
        .ok_or(crate::errors::BadInput(what))
}

/// RAL.EXE's data as `tools/extract_image` packed it (decoupling step B): only the tests
/// read it now, to check the moved constants and first values.
#[cfg(test)]
pub fn image_relocations() -> Vec<u32> {
    tests::image_relocations()
}

#[cfg(test)]
static IMAGE_DATA: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/ral_image.bin"));

/// RAL.EXE's bytes at IDA address `ida` as `data/ral_image.bin` holds them (before its
/// addresses are relocated): what the moved constants and initial values are checked
/// against while the image exists.
#[cfg(test)]
pub fn image_bytes(ida: usize, len: usize) -> Vec<u8> {
    let d = IMAGE_DATA;
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

    /// With `RC2K_LIST_MOVED` set, writes every range (`ida len` in hex) to that file: for
    /// the coordinator's scans of a merge.
    #[test]
    fn list_moved_ranges() {
        if let Ok(path) = std::env::var("RC2K_LIST_MOVED") {
            let text: String = MOVED
                .iter()
                .map(|m| format!("{:x} {:x}\n", m.ida, m.len))
                .collect();
            std::fs::write(path, text).unwrap();
        }
    }

    #[test]
    fn game_addr_finds_every_home() {
        for m in MOVED {
            let home = m.home.address();
            assert_eq!(home_of_rva(rva(m.ida)), Some(home), "{:#x}", m.ida);
            assert_eq!(game_addr(rva(m.ida + m.len - 1)) as usize, home + m.len - 1);
            if !MOVED.iter().any(|o| o.ida == m.ida + m.len) {
                assert_eq!(home_of_rva(rva(m.ida + m.len)), None, "{:#x}", m.ida);
            }
        }
        // zlib's own variables: nothing reads them, not moved (`crate::rest`).
        assert_eq!(home_of_rva(rva(0xA8C18C)), None);
    }

    #[test]
    fn every_store_home_covers_its_range_and_reads_as_its_bytes() {
        let _lock = crate::store::test_lock();
        for m in MOVED {
            let Home::Store(store) = &m.home else {
                continue;
            };
            assert!(
                m.len <= store.size(),
                "{:#x}: {:#x} bytes in a store of {:#x}",
                m.ida,
                m.len,
                store.size()
            );
            let home = crate::pointers::host_at(m.home.address());
            assert_eq!(home.cast_const(), store.first(), "{:#x}", m.ida);
            for at in (0..m.len).step_by((m.len / 512).max(1)) {
                for len in [1, 3, 4, 8].into_iter().filter(|len| at + len <= m.len) {
                    let (mut mine, mut raw) = ([0; 8], [0; 8]);
                    store.load(at, &mut mine[..len]).unwrap();
                    unsafe { crate::shared_memory::load(home.add(at), raw.as_mut_ptr(), len) };
                    assert_eq!(mine, raw, "{:#x} + {at:#x}, {len} bytes", m.ida);
                }
            }
        }
    }

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

    /// The image file's sections (rva, length, flags) and where its relocations begin.
    fn image_sections() -> (Vec<(usize, usize, usize)>, usize) {
        let d = IMAGE_DATA;
        let word = |at: usize| u32::from_le_bytes(d[at..at + 4].try_into().unwrap()) as usize;
        let mut sections = Vec::new();
        let mut at = 20;
        for _ in 0..word(16) {
            let flags = word(at + 8);
            sections.push((word(at), word(at + 4), flags));
            at += 12;
            if flags & 4 != 0 {
                let count = word(at);
                at += 4;
                for _ in 0..count {
                    at += 8 + word(at + 4);
                }
            }
            if flags & 2 != 0 {
                let raw = word(at);
                at += 4;
                let mut done = 0;
                while done < raw {
                    let (zeros, literal) = (word(at), word(at + 4));
                    at += 8 + literal;
                    done += zeros + literal;
                }
            }
        }
        (sections, at)
    }

    /// The relocation places are the image's that lie in a moved range, all of them.
    /// The places of the image's relocated pointers, sorted.
    pub fn image_relocations() -> Vec<u32> {
        let d = IMAGE_DATA;
        let word = |at: usize| u32::from_le_bytes(d[at..at + 4].try_into().unwrap()) as usize;
        let (_, at) = image_sections();
        let count = word(at);
        assert_eq!(&d[at + 4 + 4 * count..at + 8 + 4 * count], b"END!");
        let mut places: Vec<u32> = (0..count)
            .map(|k| (word(at + 4 + 4 * k) + IMAGE_BASE) as u32)
            .collect();
        places.sort_unstable();
        places
    }

    #[test]
    fn image_bytes_reads_the_image() {
        assert_eq!(image_bytes(0x4E0000, 6), b"$VER: ");
        // bss reads as zeros
        assert_eq!(image_bytes(0x93E430, 4), [0; 4]);
    }
}
