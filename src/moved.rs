//! Phase 5, the image gone (`notes/conversion.md`): the game's variables that have left
//! RAL.EXE's image for memory Rust owns, and the one place that knows where each went.
//!
//! Code that names a moved variable asks its owner (`logfile::write_line::state`,
//! `strings::units::units`, ...), never this file. Only the game's address tables still
//! name variables by IDA address: the settings table (`var\ral.cfg`'s key -> the address of
//! its variable, `settings::read_settings`) and copy lists such as `race::replay`'s
//! `HEADER_MOVES`. They go through `place` (an IDA address), which answers a moved
//! variable's new home and anything else's place in the image. The game's own pointers (the
//! image's relocations, `crate::relocations::PLACES`) are pointed at the new homes once at
//! start (`point_into_homes`), so a pointer read from the game's data is a real address.
//! `framework::game_addr` resolves a moved address through `home_of_rva`. The image itself
//! is gone (2026-10-24): its addresses stay reserved without access, so a read of one that
//! has no home ends the program naming it (`standalone`'s fault handler).

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
    home: fn() -> usize,
}

/// Every range moved so far, by IDA address. Kept in step with the owners' statics; ranges
/// do not overlap (`tests::moved_ranges_are_apart`).
pub(crate) const MOVED: &[Moved] = &[
    // logfile (`logfile::write_line::state`, the buffers)
    Moved {
        ida: 0x4E0094, // g_logEnabled
        len: 4,
        home: || {
            (crate::pointers::game_address(crate::logfile::write_line::state::ENABLED.as_ptr()))
                as usize
        },
    },
    Moved {
        ida: 0x4E0098, // g_logLevel (a `var\ral.cfg` setting, `logfile`)
        len: 4,
        home: || {
            (crate::pointers::game_address(crate::logfile::write_line::state::LEVEL.as_ptr()))
                as usize
        },
    },
    Moved {
        ida: 0x4E009C, // g_logLock
        len: 4,
        home: || {
            (crate::pointers::game_address(crate::logfile::write_line::state::LOCK.as_ptr()))
                as usize
        },
    },
    Moved {
        ida: 0x4E00A0, // g_logIndent
        len: 4,
        home: || {
            (crate::pointers::game_address(crate::logfile::write_line::state::INDENT.as_ptr()))
                as usize
        },
    },
    Moved {
        ida: 0x4E00A4, // g_logAtLineStart
        len: 4,
        home: || {
            (crate::pointers::game_address(crate::logfile::write_line::state::LINE_START.as_ptr()))
                as usize
        },
    },
    Moved {
        ida: 0x93E030, // dst: the numbers' text
        len: crate::logfile::numeric::SCRATCH_LEN,
        home: || crate::logfile::numeric::SCRATCH.addr() as usize,
    },
    Moved {
        ida: 0x93E080, // the log's line buffer
        len: crate::logfile::write_line::LINE_BUF_LEN,
        home: || crate::logfile::write_line::LINE_BUF.addr() as usize,
    },
    Moved {
        ida: 0x4E073A, // g_freeSpaceKb
        len: 4,
        home: || {
            (crate::pointers::game_address(crate::logfile::start_log::FREE_SPACE_KB.as_ptr()))
                as usize
        },
    },
    // math
    Moved {
        ida: 0x4E0140, // g_randomState
        len: 4,
        home: || (crate::pointers::game_address(crate::math::next_random::STATE.as_ptr())) as usize,
    },
    // system's start time, read by `logfile::date_text` and `math::seed_random`
    Moved {
        ida: 0x51839F, // byte_51839F, dword_5183A0, dword_5183A4
        len: crate::system::keep_start_time::KEPT_LEN,
        home: || crate::system::keep_start_time::KEPT.addr() as usize,
    },
    // strings
    Moved {
        ida: 0x4E0144, // dword_4E0144: names keep their case
        len: 4,
        home: || {
            (crate::pointers::game_address(crate::strings::copy_printable::KEEP_CASE.as_ptr()))
                as usize
        },
    },
    Moved {
        ida: 0x4E0148, // g_timeShowHours
        len: 4,
        home: || {
            (crate::pointers::game_address(crate::strings::append_time::SHOW_HOURS.as_ptr()))
                as usize
        },
    },
    Moved {
        ida: 0x4E0544, // g_sealCodeText
        len: crate::strings::format_seal_code::SEAL_TEXT_LEN,
        home: || crate::strings::format_seal_code::SEAL_TEXT.addr() as usize,
    },
    Moved {
        ida: 0x4E0758, // dword_4E0758: the lexicon recompiled on a key (a `var\ral.cfg` setting)
        len: 4,
        home: || {
            (crate::pointers::game_address(
                crate::strings::maybe_compile_lexicon::RECOMPILE_LEVEL.as_ptr(),
            )) as usize
        },
    },
    Moved {
        ida: 0x4E0798, // g_localeFile (g_noticeKind at +8)
        len: crate::strings::load_language::LOCALE_LEN,
        home: || crate::strings::load_language::LOCALE.addr() as usize,
    },
    Moved {
        ida: 0x4E07D4, // g_stringTable
        len: 4,
        home: || {
            (crate::pointers::game_address(crate::strings::string_table::STRING_TABLE.as_ptr()))
                as usize
        },
    },
    Moved {
        ida: 0x4E07F8, // g_lexicon
        len: rc2k_core::strings::lexicon::TABLE_ROOM,
        home: || crate::strings::string_table::LEXICON.addr() as usize,
    },
    Moved {
        ida: 0x4F0BC8, // g_lowerCase
        len: 0x100,
        home: || crate::strings::same_text::LOWER.addr() as usize,
    },
    Moved {
        ida: 0x4F0CC8, // g_plainLetters
        len: 0x100,
        home: || crate::strings::load_language::PLAIN_LETTERS.addr() as usize,
    },
    Moved {
        ida: 0x4F3034, // the record the empty messages share
        len: crate::strings::compile_lexicon::EMPTY_MESSAGE_LEN,
        home: || crate::strings::compile_lexicon::EMPTY_MESSAGE.addr() as usize,
    },
    Moved {
        ida: 0x51DE2C, // g_isImperialUnits
        len: 4,
        home: || (crate::pointers::game_address(crate::strings::units::UNITS.as_ptr())) as usize,
    },
    Moved {
        ida: 0x94EAB4, // dword_94EAB4: the lexicon compiler's work variables
        len: crate::strings::compile_lexicon::WORK_LEN,
        home: || crate::strings::compile_lexicon::WORK.addr() as usize,
    },
    // memory (the allocator's figures and own heap, `memory::state`)
    Moved {
        ida: 0x4E0578, // dword_4E0578: the most bytes held
        len: 4,
        home: || (crate::pointers::game_address(crate::memory::state::MOST_HELD.as_ptr())) as usize,
    },
    Moved {
        ida: 0x4E057C, // dword_4E057C: the bytes held now
        len: 4,
        home: || (crate::pointers::game_address(crate::memory::state::HELD.as_ptr())) as usize,
    },
    Moved {
        ida: 0x4E0580, // dword_4E0580: the turnover
        len: 4,
        home: || (crate::pointers::game_address(crate::memory::state::TURNOVER.as_ptr())) as usize,
    },
    Moved {
        ida: 0x4E0584, // g_ownHeap (a `var\ral.cfg` setting, `altmemenable`)
        len: 4,
        home: || (crate::pointers::game_address(crate::memory::state::OWN_HEAP.as_ptr())) as usize,
    },
    Moved {
        ida: 0x4E0588, // g_ownHeapKb (a `var\ral.cfg` setting, `altmemsize`)
        len: 4,
        home: || {
            (crate::pointers::game_address(crate::memory::state::OWN_HEAP_KB.as_ptr())) as usize
        },
    },
    Moved {
        ida: 0x4E058C, // g_ownHeapBase
        len: 4,
        home: || (crate::pointers::game_address(crate::memory::state::HEAP_BASE.as_ptr())) as usize,
    },
    Moved {
        ida: 0x4E0590, // g_ownHeapCursor
        len: 4,
        home: || {
            (crate::pointers::game_address(crate::memory::state::HEAP_CURSOR.as_ptr())) as usize
        },
    },
    // settings (`read_driver_names`' work areas), threading (the thread slots)
    Moved {
        ida: 0x529DA5, // aName000: the driver names' key, made in place
        len: 8,
        home: || crate::settings::read_driver_names::KEY.addr() as usize,
    },
    Moved {
        ida: 0x95ECF8, // dword_95ECF8 (a pointer a driver) and unk_95EFC8 (the names' pool), up to 0x960000
        len: 0x1308,
        home: || crate::settings::read_driver_names::NAMES.addr() as usize,
    },
    Moved {
        ida: 0x5184F4, // the thread slots: input, audio, network, shutdown
        len: crate::threading::thread_slot::SLOT_COUNT * crate::threading::thread_slot::SLOT_LEN,
        home: || crate::threading::thread_slot::SLOTS.addr() as usize,
    },
    // text (the blended font and palette, the kerning rows)
    Moved {
        ida: 0x4F3B74, // g_blendedPalette
        len: 0x400,
        home: || crate::text::blend_font::BLENDED_PALETTE.addr() as usize,
    },
    Moved {
        ida: 0x4F3F74, // g_fontBlendMode (a `var\ral.cfg` setting, `textflashmode`)
        len: 4,
        home: || {
            (crate::pointers::game_address(crate::text::blend_font::BLEND_MODE.as_ptr())) as usize
        },
    },
    Moved {
        ida: 0x51E384, // g_blendedFont
        len: 0x20,
        home: || crate::text::blend_font::BLENDED_FONT.addr() as usize,
    },
    Moved {
        ida: 0x93E830, // g_kerningRowState
        len: rc2k_core::text::KERNING_ROWS * 2,
        home: || crate::text::glyph_kerning::STATE.addr() as usize,
    },
    // files (the search record, the paths' buffers)
    Moved {
        ida: 0x4E05A4, // hFindFile
        len: 4,
        home: || {
            (crate::pointers::game_address(crate::files::state::FIND_HANDLE.as_ptr())) as usize
        },
    },
    Moved {
        ida: 0x4E05A8, // FindFileData (WIN32_FIND_DATAA)
        len: 0x13E,
        home: || crate::files::state::FIND_DATA.addr() as usize,
    },
    Moved {
        ida: 0x4E06E6, // the attributes a search passes over
        len: 4,
        home: || {
            (crate::pointers::game_address(crate::files::state::SKIPPED_ATTRIBUTES.as_ptr()))
                as usize
        },
    },
    Moved {
        ida: 0x5BE9BD, // g_packedPath
        len: crate::files::state::PACKED_PATH_LEN,
        home: || crate::files::state::PACKED_PATH.addr() as usize,
    },
    Moved {
        ida: 0x60B2CC, // g_readBytes
        len: 4,
        home: || (crate::pointers::game_address(crate::files::state::BYTES_READ.as_ptr())) as usize,
    },
    Moved {
        ida: 0x94E8B0, // byte_94E8B0 (numbered paths) and the resolver's buffer at 0x94E9B0
        len: crate::files::state::PATHS_LEN,
        home: || crate::files::state::PATHS.addr() as usize,
    },
    // fatal (the reporter's variables, `fatal::state`)
    Moved {
        ida: 0x4E0598, // dword_4E0598: the Win32 error named in a report
        len: 4,
        home: || (crate::pointers::game_address(crate::fatal::state::W32.as_ptr())) as usize,
    },
    Moved {
        ida: 0x4F52A4, // g_fatalCode
        len: 4,
        home: || (crate::pointers::game_address(crate::fatal::state::CODE.as_ptr())) as usize,
    },
    Moved {
        ida: 0x4F52A8, // g_fatalMessage
        len: 4,
        home: || (crate::pointers::game_address(crate::fatal::state::MESSAGE.as_ptr())) as usize,
    },
    Moved {
        ida: 0x4F66FD, // Caption ("Rally: Guru Meditation")
        len: crate::fatal::state::CAPTION_LEN,
        home: || crate::fatal::state::CAPTION.addr() as usize,
    },
    Moved {
        ida: 0x4F673A, // aEaxXxxxxxxxEbx: the registers' template
        len: crate::fatal::state::TEMPLATE_LEN,
        home: || crate::fatal::state::TEMPLATE.addr() as usize,
    },
    Moved {
        ida: 0x4F67BC, // g_fatalReported
        len: 1,
        home: || (crate::pointers::game_address(crate::fatal::state::REPORTED.as_ptr())) as usize,
    },
    Moved {
        ida: 0x4F67BD, // cursor: the string added to every report
        len: crate::fatal::state::EXTRA_LEN,
        home: || crate::fatal::state::EXTRA.addr() as usize,
    },
    Moved {
        ida: 0x9526F8, // Text: the report
        len: crate::fatal::state::TEXT_LEN,
        home: || crate::fatal::state::TEXT.addr() as usize,
    },
    // files (the install record's variables, `files::state`)
    Moved {
        ida: 0x5189A4, // g_mediaMode
        len: 4,
        home: || (crate::pointers::game_address(crate::files::state::MEDIA_MODE.as_ptr())) as usize,
    },
    Moved {
        ida: 0x5189A8, // g_baseDirectory, then g_localeDirectory at +0x100
        len: crate::files::state::DIRECTORIES_LEN,
        home: || crate::files::state::DIRECTORIES.addr() as usize,
    },
    // gfx / hud / d3d (phase 5; typed in phase 7)
    Moved {
        ida: crate::hud::store::START, // the HUD's variables, batch included
        len: crate::hud::store::LEN,
        home: || crate::hud::store::hud_addr() as usize,
    },
    Moved {
        ida: crate::gfx::decoder::START, // The picture decoders' and glyph builder's variables (decoded picture, palette)
        len: crate::gfx::decoder::LEN,
        home: crate::gfx::decoder::home,
    },
    Moved {
        ida: crate::strings::compile_lexicon::MEMORY_START, // The lexicon's memory (kilobytes, the list)
        len: crate::strings::compile_lexicon::MEMORY_LEN,
        home: crate::strings::compile_lexicon::memory_home,
    },
    Moved {
        ida: crate::gfx::build_glyphs::START, // The sheet builder's variables (sheet table, built table, glyph counts)
        len: crate::gfx::build_glyphs::LEN,
        home: crate::gfx::build_glyphs::home,
    },
    Moved {
        ida: crate::gfx::pcx::START, // The PCX file being decoded (header, palette, stream, limits)
        len: crate::gfx::pcx::LEN,
        home: crate::gfx::pcx::home,
    },
    Moved {
        ida: crate::gfx::read_picture_file::PNG_SETTING_START, // `nopngdecode`
        len: 4,
        home: crate::gfx::read_picture_file::png_setting_home,
    },
    Moved {
        ida: crate::gfx::glyph_tables::START, // The glyph tables' variables (`bundleheapsize`, the list)
        len: crate::gfx::glyph_tables::LEN,
        home: crate::gfx::glyph_tables::home,
    },
    Moved {
        ida: crate::gfx::show_picture_file::SHOWING_START, // g_showingPicture
        len: 4,
        home: crate::gfx::show_picture_file::showing_home,
    },
    Moved {
        ida: crate::gfx::blit::START, // The blitters' variables (the clip rectangle, the skips of the visible part)
        len: crate::gfx::blit::LEN,
        home: crate::gfx::blit::home,
    },
    Moved {
        ida: crate::gfx::rescale_picture::START, // The rescaler's variables (the copy's memory, steps, weights, the pixel's parts)
        len: crate::gfx::rescale_picture::LEN,
        home: crate::gfx::rescale_picture::home,
    },
    Moved {
        ida: crate::gfx::strips::START, // The HUD's view strips' places (sizes, tops, lefts)
        len: crate::gfx::strips::LEN,
        home: crate::gfx::strips::home,
    },
    Moved {
        ida: crate::gfx::fill_row_table::START, // The row table (the address of each row of a picture)
        len: crate::gfx::fill_row_table::LEN,
        home: crate::gfx::fill_row_table::home,
    },
    Moved {
        ida: crate::gfx::formats::START, // The pixel formats' work variables (format numbers, scratch bytes, component format)
        len: crate::gfx::formats::LEN,
        home: crate::gfx::formats::home,
    },
    Moved {
        ida: crate::gfx::fill_thirds_table::START, // The thirds table (n / 3 for the sums of three components)
        len: 0x300,
        home: crate::gfx::fill_thirds_table::home,
    },
    Moved {
        ida: crate::gfx::modes::START, // The screen mode tables (pixel kinds, sizes, the modes listed)
        len: crate::gfx::modes::LEN,
        home: crate::gfx::modes::home,
    },
    Moved {
        ida: crate::gfx::converters::START, // The MMX converters' masks and the converters table
        len: crate::gfx::converters::LEN,
        home: crate::gfx::converters::home,
    },
    Moved {
        ida: crate::gfx::reports::START, // The 3D reports switch (dword_4F52AC)
        len: crate::gfx::reports::LEN,
        home: crate::gfx::reports::home,
    },
    Moved {
        ida: crate::gfx::shots::START, // The screenshot switch, its lock and the file name's template
        len: crate::gfx::shots::LEN,
        home: crate::gfx::shots::home,
    },
    Moved {
        ida: crate::gfx::picture_store::START, // The picture loader's variables (the name, flags, size, rows and buffer of the picture, the list)
        len: crate::gfx::picture_store::LEN,
        home: crate::gfx::picture_store::home,
    },
    Moved {
        ida: crate::gfx::texture_pixels::START, // The texture grading (reading switch, scale, saturation, the grades table)
        len: crate::gfx::texture_pixels::LEN,
        home: crate::gfx::texture_pixels::home,
    },
    Moved {
        ida: crate::gfx::bar_store::START, // The progress bar's variables (position, width, glyphs, percent)
        len: crate::gfx::bar_store::LEN,
        home: crate::gfx::bar_store::home,
    },
    Moved {
        ida: crate::gfx::png_file::START, // The PNG decoder's file (pointer, length, memory list)
        len: crate::gfx::png_file::LEN,
        home: crate::gfx::png_file::home,
    },
    Moved {
        ida: crate::gfx::png_header::START, // The PNG header's fields (width, height, compression, colour type)
        len: crate::gfx::png_header::LEN,
        home: crate::gfx::png_header::home,
    },
    Moved {
        ida: crate::gfx::pcx_planes::START, // The screenshot writer's plane buffer (3 * width bytes, up to 1920 wide)
        len: crate::gfx::pcx_planes::LEN,
        home: crate::gfx::pcx_planes::home,
    },
    Moved {
        ida: crate::gfx::cards::START, // The 3D primary and secondary card switches
        len: crate::gfx::cards::LEN,
        home: crate::gfx::cards::home,
    },
    Moved {
        ida: crate::gfx::upload::START, // The texture upload's surface description (DDSURFACEDESC) and the mode-trial switch
        len: crate::gfx::upload::LEN,
        home: crate::gfx::upload::home,
    },
    Moved {
        ida: crate::install::store::InstallVars::START, // InstallVars
        len: crate::install::store::InstallVars::LEN,
        home: crate::install::store::InstallVars::home,
    },
    Moved {
        ida: crate::menu::build_notice::BuildNotice::START, // BuildNotice
        len: crate::menu::build_notice::BuildNotice::LEN,
        home: crate::menu::build_notice::BuildNotice::home,
    },
    Moved {
        ida: crate::install::store::DriveRows::START, // DriveRows
        len: crate::install::store::DriveRows::LEN,
        home: crate::install::store::DriveRows::home,
    },
    Moved {
        ida: crate::net::store::NetVars::START, // NetVars
        len: crate::net::store::NetVars::LEN,
        home: crate::net::store::NetVars::home,
    },
    Moved {
        ida: crate::install::store::LobbyKeys::START, // LobbyKeys
        len: crate::install::store::LobbyKeys::LEN,
        home: crate::install::store::LobbyKeys::home,
    },
    Moved {
        ida: crate::net::store::Button::START, // Button
        len: crate::net::store::Button::LEN,
        home: crate::net::store::Button::home,
    },
    Moved {
        ida: crate::gfx::typefaces::START, // The fonts (63 font objects)
        len: crate::gfx::typefaces::LEN,
        home: crate::gfx::typefaces::home,
    },
    Moved {
        ida: crate::strings::unit_labels::START, // The unit labels' string ids
        len: crate::strings::unit_labels::LEN,
        home: crate::strings::unit_labels::home,
    },
    Moved {
        ida: crate::saves::write_savegame_slot::START, // The saved games' file buffer and names
        len: crate::saves::write_savegame_slot::LEN,
        home: crate::saves::write_savegame_slot::home,
    },
    Moved {
        ida: crate::service::read_setup_slot::START, // The set-ups' file head and names
        len: crate::service::read_setup_slot::LEN,
        home: crate::service::read_setup_slot::home,
    },
    Moved {
        ida: crate::race::replay::START, // The replays' header and names
        len: crate::race::replay::LEN,
        home: crate::race::replay::home,
    },
    Moved {
        ida: crate::install::installer::TEXTS_START, // The installer's texts
        len: crate::install::installer::TEXTS_LEN,
        home: crate::install::installer::texts_home,
    },
    Moved {
        ida: crate::saves::load_options::PATH_START, // var\ral.dat
        len: 12,
        home: crate::saves::load_options::path_home,
    },
    Moved {
        ida: crate::saves::load_best_times::PATH_START, // var\hst.dat
        len: 12,
        home: crate::saves::load_best_times::path_home,
    },
    Moved {
        ida: crate::race::replay_path::LAST_RACE_START, // The last race's replay file name
        len: crate::race::replay_path::LAST_RACE_LEN,
        home: crate::race::replay_path::last_race_home,
    },
    Moved {
        ida: crate::saves::write_savegame_slot::TEMPLATE_START, // The saved game file name's template
        len: 27,
        home: crate::saves::write_savegame_slot::template_home,
    },
    Moved {
        ida: crate::service::read_setup_slot::TEMPLATE_START, // The set-up file name's template
        len: 27,
        home: crate::service::read_setup_slot::template_home,
    },
    Moved {
        ida: crate::race::find_snapshot_number::SNAPSHOT_START, // The snapshot picture's name
        len: 26,
        home: crate::race::find_snapshot_number::snapshot_home,
    },
    Moved {
        ida: crate::race::replay_path::TEMPLATES_START, // The replay file names' templates
        len: crate::race::replay_path::TEMPLATES_LEN,
        home: crate::race::replay_path::templates_home,
    },
    Moved {
        ida: crate::menu::play_films::FILMS_START, // The films' file names
        len: crate::menu::play_films::FILMS_LEN,
        home: crate::menu::play_films::films_home,
    },
    Moved {
        ida: crate::screens::draw_message_box::BOX_START, // The message box's rectangle
        len: 16,
        home: crate::screens::draw_message_box::box_home,
    },
    Moved {
        ida: crate::race::frame_text::CREDITS_START, // The credits' state and script
        len: crate::race::frame_text::CREDITS_LEN,
        home: crate::race::frame_text::credits_home,
    },
    Moved {
        ida: crate::system::check_developer_machine::DEVELOPER_START, // g_developerMachine
        len: 4,
        home: crate::system::check_developer_machine::developer_home,
    },
    Moved {
        ida: crate::gfx::texture_upload::START, // The 3D card's tables and the textures' records
        len: crate::gfx::texture_upload::LEN,
        home: crate::gfx::texture_upload::home,
    },
    Moved {
        ida: crate::platform_3d::GUIDS_A_START, // DirectX interface ids (nothing reads them)
        len: crate::platform_3d::GUIDS_A_LEN,
        home: crate::platform_3d::guids_a_home,
    },
    Moved {
        ida: crate::film::NO_FMV_START, // g_noFmv
        len: 4,
        home: crate::film::no_fmv_home,
    },
    Moved {
        ida: crate::platform_3d::GUIDS_B_START, // DirectX interface ids (nothing reads them)
        len: crate::platform_3d::GUIDS_B_LEN,
        home: crate::platform_3d::guids_b_home,
    },
    Moved {
        ida: crate::gfx::build_game_fonts::START, // The game fonts' memory (`gamegfxheapsize`, the list)
        len: crate::gfx::build_game_fonts::LEN,
        home: crate::gfx::build_game_fonts::home,
    },
    Moved {
        ida: crate::scene::upload_car_icon::START, // The car icon upload's record
        len: 0x1C,
        home: crate::scene::upload_car_icon::home,
    },
    Moved {
        ida: crate::race::game_map::START, // The in-race map's variables
        len: crate::race::game_map::LEN,
        home: crate::race::game_map::home,
    },
    Moved {
        ida: crate::gfx::front::START, // The DirectDraw front and back surface handles
        len: crate::gfx::front::LEN,
        home: crate::gfx::front::home,
    },
    Moved {
        ida: crate::gfx::locked::START, // The surface-locked switch
        len: crate::gfx::locked::LEN,
        home: crate::gfx::locked::home,
    },
    Moved {
        ida: crate::gfx::windscreen_drop_batch::START, // The marks' windscreen drop batch drawn by d3d (the index and vertex counts, the fans, the records)
        len: crate::gfx::windscreen_drop_batch::LEN,
        home: crate::gfx::windscreen_drop_batch::home,
    },
    // session: the screen, the window's flags, the machine, the clocks (`session`)
    Moved {
        ida: crate::session::SCREEN_IDA,
        len: crate::session::SCREEN_LEN,
        home: || crate::pointers::game_address(&crate::session::SCREEN as *const _) as usize,
    },
    Moved {
        ida: crate::session::FLAGS_IDA,
        len: crate::session::FLAGS_LEN,
        home: || crate::pointers::game_address(&crate::session::FLAGS as *const _) as usize,
    },
    Moved {
        ida: crate::session::MACHINE_IDA,
        len: crate::session::MACHINE_LEN,
        home: || crate::pointers::game_address(&crate::session::MACHINE as *const _) as usize,
    },
    Moved {
        ida: crate::session::CLOCKS_IDA,
        len: crate::session::CLOCKS_LEN,
        home: || crate::pointers::game_address(&crate::session::CLOCKS as *const _) as usize,
    },
    // audio: the audio thread's variables and the sample records (`audio::store`)
    Moved {
        ida: crate::audio::store::STATE_IDA,
        len: crate::audio::store::STATE_END - crate::audio::store::STATE_IDA,
        home: || crate::audio::store::state_addr() as usize,
    },
    Moved {
        ida: crate::audio::store::SAMPLES_IDA,
        len: crate::audio::store::SAMPLES_END - crate::audio::store::SAMPLES_IDA,
        home: || crate::audio::store::sample_records_addr() as usize,
    },
    Moved {
        ida: crate::audio::store::QUEUE_IDA,
        len: crate::audio::store::QUEUE_END - crate::audio::store::QUEUE_IDA,
        home: || crate::audio::store::commands_addr() as usize,
    },
    Moved {
        ida: 0x6D1DD4, // g_eaxEnvironment
        len: 4,
        home: || crate::audio::store::ENVIRONMENT.addr() as usize,
    },
    // sound: the race sounds' clock, and the spot picks .. CD audio block (`sound::store`)
    Moved {
        ida: 0x4F66DD, // the options file's four kept dwords and a gap (`saves::load_options::KEPT`)
        len: 0x14,
        home: || {
            (crate::pointers::game_address(crate::saves::load_options::KEPT.as_ptr())) as usize
        },
    },
    Moved {
        ida: 0x4F66F1, // the wheel surfaces (`sound::store::WHEEL_SURFACES`)
        len: 8,
        home: || {
            (crate::pointers::game_address(crate::sound::store::WHEEL_SURFACES.as_ptr())) as usize
        },
    },
    Moved {
        ida: 0x4F66F9, // the race record's seal key (`race::replay::SEAL_KEY`)
        len: 4,
        home: || crate::race::replay::SEAL_KEY.addr() as usize,
    },
    Moved {
        ida: 0x512A10, // the race sounds' objects (`sound::store::RaceSounds`)
        len: 0x2580,
        home: || crate::sound::store::race_sounds_addr() as usize,
    },
    Moved {
        ida: 0x5124E4, // the race sounds' clock (`sound::store::RaceClock`)
        len: 0x24,
        home: || crate::sound::store::race_clock_addr() as usize,
    },
    Moved {
        ida: 0x516358, // the surface sounds' constants (`sound::store::SurfaceSounds`)
        len: 0x44,
        home: || crate::sound::store::surface_sounds_addr() as usize,
    },
    Moved {
        ida: 0x51655C, // the stage sounds (`sound::store::StageSounds`)
        len: 0x364,
        home: || crate::sound::store::stage_sounds_addr() as usize,
    },
    Moved {
        ida: crate::sound::store::SpotPicks::START, // The spot voices' pickers' work and the sound kinds per spot
        len: crate::sound::store::SpotPicks::LEN,
        home: crate::sound::store::SpotPicks::home,
    },
    Moved {
        ida: crate::sound::store::SettingsApplied::START, // Set once the kept sound settings are applied
        len: crate::sound::store::SettingsApplied::LEN,
        home: crate::sound::store::SettingsApplied::home,
    },
    Moved {
        ida: crate::input::store::ForceFeedback::START, // The force-feedback effects' clock, surface kinds and constants
        len: crate::input::store::ForceFeedback::LEN,
        home: crate::input::store::ForceFeedback::home,
    },
    Moved {
        ida: crate::sound::store::SoundOptions::START, // The kept sound options
        len: crate::sound::store::SoundOptions::LEN,
        home: crate::sound::store::SoundOptions::home,
    },
    Moved {
        ida: crate::sound::store::CdAudio::START, // The CD audio's state
        len: crate::sound::store::CdAudio::LEN,
        home: crate::sound::store::CdAudio::home,
    },
    // input: the keyboard file, devices, keyboard and mouse (`input::store`, typed)
    Moved {
        ida: crate::input::store::KEYBOARD_FILE_IDA,
        len: crate::input::store::KEYBOARD_FILE_LEN,
        home: || crate::input::store::keyboard_file_addr() as usize,
    },
    Moved {
        ida: crate::input::store::DEVICES_IDA,
        len: crate::input::store::DEVICES_LEN,
        home: || crate::input::store::devices_addr() as usize,
    },
    Moved {
        ida: crate::input::store::KEYBOARD_IDA,
        len: crate::input::store::KEYBOARD_LEN,
        home: || crate::input::store::keyboard_addr() as usize,
    },
    Moved {
        ida: crate::input::store::MOUSE_IDA,
        len: crate::input::store::MOUSE_LEN,
        home: || crate::input::store::mouse_addr() as usize,
    },
    Moved {
        ida: 0x511B0C, // g_nextMenuSoundTime
        len: 4,
        home: || crate::sound::store::NEXT_MENU_SOUND_TIME.addr() as usize,
    },
    Moved {
        ida: 0x6218F0, // g_raceSoundsOn
        len: 4,
        home: || crate::sound::store::RACE_SOUNDS_ON.addr() as usize,
    },
    Moved {
        ida: crate::sound::store::CAR_STREAM_IDA, // the car stream's place, list and switch
        len: 12,
        home: || crate::sound::store::CAR_STREAM.place.addr() as usize,
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
        home: || crate::saves::preview_store::savegame_preview_addr() as usize,
    },
    Moved {
        ida: crate::screens::controls_store::START,
        len: crate::screens::controls_store::END - crate::screens::controls_store::START,
        home: || crate::screens::controls_store::controls_screen_addr() as usize,
    },
    Moved {
        ida: crate::menu::players_store::START,
        len: crate::menu::players_store::END - crate::menu::players_store::START,
        home: || crate::menu::players_store::players_addr() as usize,
    },
    Moved {
        ida: crate::menu::cars_store::START,
        len: crate::menu::cars_store::END - crate::menu::cars_store::START,
        home: || crate::menu::cars_store::cars_addr() as usize,
    },
    Moved {
        ida: crate::menu::cheats_store::START,
        len: crate::menu::cheats_store::END - crate::menu::cheats_store::START,
        home: || crate::menu::cheats_store::cheats_addr() as usize,
    },
    Moved {
        ida: crate::menu::results_store::START,
        len: crate::menu::results_store::END - crate::menu::results_store::START,
        home: || crate::menu::results_store::results_addr() as usize,
    },
    Moved {
        ida: crate::menu::arcade_store::START,
        len: crate::menu::arcade_store::END - crate::menu::arcade_store::START,
        home: || crate::menu::arcade_store::arcade_addr() as usize,
    },
    Moved {
        ida: crate::menu::captions_store::START,
        len: crate::menu::captions_store::END - crate::menu::captions_store::START,
        home: || crate::menu::captions_store::captions_addr() as usize,
    },
    Moved {
        ida: crate::menu::pages_store::START,
        len: crate::menu::pages_store::END - crate::menu::pages_store::START,
        home: || crate::menu::pages_store::menu_pages_addr() as usize,
    },
    Moved {
        ida: crate::menu::car_numbers_store::START,
        len: crate::menu::car_numbers_store::END - crate::menu::car_numbers_store::START,
        home: || crate::menu::car_numbers_store::car_numbers_addr() as usize,
    },
    Moved {
        ida: crate::menu::flow_store::START,
        len: crate::menu::flow_store::END - crate::menu::flow_store::START,
        home: || crate::menu::flow_store::screen_flow_addr() as usize,
    },
    Moved {
        ida: crate::menu::loop_store::START,
        len: crate::menu::loop_store::END - crate::menu::loop_store::START,
        home: || crate::menu::loop_store::loop_flags_addr() as usize,
    },
    Moved {
        ida: crate::menu::keys_store::START,
        len: crate::menu::keys_store::END - crate::menu::keys_store::START,
        home: || crate::menu::keys_store::key_tables_addr() as usize,
    },
    Moved {
        ida: crate::menu::option_work_store::START,
        len: crate::menu::option_work_store::END - crate::menu::option_work_store::START,
        home: || crate::menu::option_work_store::option_work_addr() as usize,
    },
    Moved {
        ida: crate::menu::champ_store::START,
        len: crate::menu::champ_store::END - crate::menu::champ_store::START,
        home: || crate::menu::champ_store::champ_view_addr() as usize,
    },
    Moved {
        ida: crate::menu::names_store::START,
        len: crate::menu::names_store::END - crate::menu::names_store::START,
        home: || crate::menu::names_store::names_addr() as usize,
    },
    Moved {
        ida: crate::menu::event_store::START,
        len: crate::menu::event_store::END - crate::menu::event_store::START,
        home: || crate::menu::event_store::event_addr() as usize,
    },
    Moved {
        ida: crate::menu::draw_work_store::START,
        len: crate::menu::draw_work_store::END - crate::menu::draw_work_store::START,
        home: || crate::menu::draw_work_store::draw_work_addr() as usize,
    },
    // the race block's tail (phase 7): the menus' texts (`menu::texts_store`), the stages' tables (`records::stages_store`), the drivers (`menu::drivers_store`), the rallies' best times and the best times' memory (`saves::best_times_store`),
    // the screens' kept items (`menu::screen_items_store`), the items' sprite lists
    // (`menu::item_sprites_store`)
    Moved {
        ida: crate::menu::texts_store::START,
        len: crate::menu::texts_store::END - crate::menu::texts_store::START,
        home: || crate::menu::texts_store::menu_texts_addr() as usize,
    },
    Moved {
        ida: crate::records::stages_store::START,
        len: crate::records::stages_store::END - crate::records::stages_store::START,
        home: || crate::records::stages_store::stage_tables_addr() as usize,
    },
    Moved {
        ida: crate::menu::drivers_store::START,
        len: crate::menu::drivers_store::END - crate::menu::drivers_store::START,
        home: || crate::menu::drivers_store::drivers_addr() as usize,
    },
    Moved {
        ida: crate::saves::best_times_store::RallyTimes::START,
        len: crate::saves::best_times_store::RallyTimes::END
            - crate::saves::best_times_store::RallyTimes::START,
        home: crate::saves::best_times_store::RallyTimes::home,
    },
    Moved {
        ida: crate::saves::best_times_store::START,
        len: crate::saves::best_times_store::END - crate::saves::best_times_store::START,
        home: || crate::saves::best_times_store::best_times_addr() as usize,
    },
    Moved {
        ida: crate::menu::screen_items_store::START,
        len: crate::menu::screen_items_store::END - crate::menu::screen_items_store::START,
        home: || crate::menu::screen_items_store::screen_items_addr() as usize,
    },
    Moved {
        ida: crate::menu::item_sprites_store::START,
        len: crate::menu::item_sprites_store::END - crate::menu::item_sprites_store::START,
        home: || crate::menu::item_sprites_store::item_sprites_addr() as usize,
    },
    Moved {
        ida: crate::menu::menus_store::START,
        len: crate::menu::menus_store::END - crate::menu::menus_store::START,
        home: || crate::menu::menus_store::menus_addr() as usize,
    },
    Moved {
        ida: crate::menu::draw_lists_store::START,
        len: crate::menu::draw_lists_store::END - crate::menu::draw_lists_store::START,
        home: || crate::menu::draw_lists_store::draw_lists_addr() as usize,
    },
    // the former `rest` module's pieces: typed, each in its owner's module
    Moved {
        ida: crate::race::tyre_store::TyreChange::START,
        len: crate::race::tyre_store::TyreChange::LEN,
        home: crate::race::tyre_store::TyreChange::home,
    },
    Moved {
        ida: crate::race::seal_store::SealWork::START,
        len: crate::race::seal_store::SealWork::LEN,
        home: crate::race::seal_store::SealWork::home,
    },
    Moved {
        ida: crate::system::set_error_mode::ErrorMode::START,
        len: crate::system::set_error_mode::ErrorMode::LEN,
        home: crate::system::set_error_mode::ErrorMode::home,
    },
    Moved {
        ida: crate::system::start_texts::StartTexts::START,
        len: crate::system::start_texts::StartTexts::LEN,
        home: crate::system::start_texts::StartTexts::home,
    },
    Moved {
        ida: crate::input::guids::InputGuids::START,
        len: crate::input::guids::InputGuids::LEN,
        home: crate::input::guids::InputGuids::home,
    },
    Moved {
        ida: crate::sound::sets_store::MenuSounds::START,
        len: crate::sound::sets_store::MenuSounds::LEN,
        home: crate::sound::sets_store::MenuSounds::home,
    },
    Moved {
        ida: crate::sound::sets_store::RaceSoundSets::START,
        len: crate::sound::sets_store::RaceSoundSets::LEN,
        home: crate::sound::sets_store::RaceSoundSets::home,
    },
    Moved {
        ida: crate::sound::sets_store::CueLists::START,
        len: crate::sound::sets_store::CueLists::LEN,
        home: crate::sound::sets_store::CueLists::home,
    },
    Moved {
        ida: crate::sound::sets_store::SoundKinds::START,
        len: crate::sound::sets_store::SoundKinds::LEN,
        home: crate::sound::sets_store::SoundKinds::home,
    },
    Moved {
        ida: crate::sound::sets_store::SurfaceSets::START,
        len: crate::sound::sets_store::SurfaceSets::LEN,
        home: crate::sound::sets_store::SurfaceSets::home,
    },
    Moved {
        ida: crate::sound::sets_store::SpotVoices::START,
        len: crate::sound::sets_store::SpotVoices::LEN,
        home: crate::sound::sets_store::SpotVoices::home,
    },
    Moved {
        ida: crate::install::vars::InstallerRun::START,
        len: crate::install::vars::InstallerRun::LEN,
        home: crate::install::vars::InstallerRun::home,
    },
    Moved {
        ida: crate::install::vars::InstallerNames::START,
        len: crate::install::vars::InstallerNames::LEN,
        home: crate::install::vars::InstallerNames::home,
    },
    Moved {
        ida: crate::install::vars::InstallerPages::START,
        len: crate::install::vars::InstallerPages::LEN,
        home: crate::install::vars::InstallerPages::home,
    },
    Moved {
        ida: crate::install::vars::InstallerPaths::START,
        len: crate::install::vars::InstallerPaths::LEN,
        home: crate::install::vars::InstallerPaths::home,
    },
    Moved {
        ida: crate::install::vars::InstallerPalette::START,
        len: crate::install::vars::InstallerPalette::LEN,
        home: crate::install::vars::InstallerPalette::home,
    },
    Moved {
        ida: crate::system::read_command_line::CommandLine::START,
        len: crate::system::read_command_line::CommandLine::LEN,
        home: crate::system::read_command_line::CommandLine::home,
    },
    Moved {
        ida: crate::system::load_game_data::RunCount::START,
        len: crate::system::load_game_data::RunCount::LEN,
        home: crate::system::load_game_data::RunCount::home,
    },
    Moved {
        ida: crate::install::read_sku::Sku::START,
        len: crate::install::read_sku::Sku::LEN,
        home: crate::install::read_sku::Sku::home,
    },
    Moved {
        ida: crate::strings::keyboard_tables::KeyboardTables::START,
        len: crate::strings::keyboard_tables::KeyboardTables::LEN,
        home: crate::strings::keyboard_tables::KeyboardTables::home,
    },
    Moved {
        ida: crate::strings::text_buffer::TextBuffer::START,
        len: crate::strings::text_buffer::TextBuffer::LEN,
        home: crate::strings::text_buffer::TextBuffer::home,
    },
    Moved {
        ida: crate::files::scratch::Scratch::START,
        len: crate::files::scratch::Scratch::LEN,
        home: crate::files::scratch::Scratch::home,
    },
    Moved {
        ida: crate::gfx::screen_state::ScreenMode::START,
        len: crate::gfx::screen_state::ScreenMode::LEN,
        home: crate::gfx::screen_state::ScreenMode::home,
    },
    Moved {
        ida: crate::gfx::screen_state::DrawSurface::START,
        len: crate::gfx::screen_state::DrawSurface::LEN,
        home: crate::gfx::screen_state::DrawSurface::home,
    },
    Moved {
        ida: crate::d3d::cards_store::Cards3d::START,
        len: crate::d3d::cards_store::Cards3d::LEN,
        home: crate::d3d::cards_store::Cards3d::home,
    },
    Moved {
        ida: crate::gfx::fonts_store::Fonts::START,
        len: crate::gfx::fonts_store::Fonts::LEN,
        home: crate::gfx::fonts_store::Fonts::home,
    },
    Moved {
        ida: crate::audio::names_store::SampleNames::START,
        len: crate::audio::names_store::SampleNames::LEN,
        home: crate::audio::names_store::SampleNames::home,
    },
    Moved {
        ida: crate::gfx::choices_store::ScreenChoices::START,
        len: crate::gfx::choices_store::ScreenChoices::LEN,
        home: crate::gfx::choices_store::ScreenChoices::home,
    },
    Moved {
        ida: crate::gfx::jade_store::JadeFont::START,
        len: crate::gfx::jade_store::JadeFont::LEN,
        home: crate::gfx::jade_store::JadeFont::home,
    },
    // race: the image's zeroed tail (`race::buffers_store`)
    Moved {
        ida: crate::race::buffers_store::START,
        len: crate::race::buffers_store::END - crate::race::buffers_store::START,
        home: crate::race::buffers_store::home,
    },
    // race: the race core (`race::store`), the ranges around the holes and the left pieces
    Moved {
        ida: crate::race::store::OWN[0].0,
        len: crate::race::store::OWN[0].1 - crate::race::store::OWN[0].0,
        home: || crate::race::store::home(crate::race::store::OWN[0].0),
    },
    Moved {
        ida: crate::race::store::OWN[1].0,
        len: crate::race::store::OWN[1].1 - crate::race::store::OWN[1].0,
        home: || crate::race::store::home(crate::race::store::OWN[1].0),
    },
    Moved {
        ida: crate::race::store::OWN[2].0,
        len: crate::race::store::OWN[2].1 - crate::race::store::OWN[2].0,
        home: || crate::race::store::home(crate::race::store::OWN[2].0),
    },
    Moved {
        ida: crate::race::store::OWN[3].0,
        len: crate::race::store::OWN[3].1 - crate::race::store::OWN[3].0,
        home: || crate::race::store::home(crate::race::store::OWN[3].0),
    },
    Moved {
        ida: crate::race::store::OWN[4].0,
        len: crate::race::store::OWN[4].1 - crate::race::store::OWN[4].0,
        home: || crate::race::store::home(crate::race::store::OWN[4].0),
    },
    Moved {
        ida: crate::race::store::OWN[5].0,
        len: crate::race::store::OWN[5].1 - crate::race::store::OWN[5].0,
        home: || crate::race::store::home(crate::race::store::OWN[5].0),
    },
    Moved {
        ida: crate::race::store::OWN[6].0,
        len: crate::race::store::OWN[6].1 - crate::race::store::OWN[6].0,
        home: || crate::race::store::home(crate::race::store::OWN[6].0),
    },
    Moved {
        ida: crate::race::store::OWN[7].0,
        len: crate::race::store::OWN[7].1 - crate::race::store::OWN[7].0,
        home: || crate::race::store::home(crate::race::store::OWN[7].0),
    },
    Moved {
        ida: crate::race::store::OWN[8].0,
        len: crate::race::store::OWN[8].1 - crate::race::store::OWN[8].0,
        home: || crate::race::store::home(crate::race::store::OWN[8].0),
    },
    Moved {
        ida: crate::race::store::OWN[9].0,
        len: crate::race::store::OWN[9].1 - crate::race::store::OWN[9].0,
        home: || crate::race::store::home(crate::race::store::OWN[9].0),
    },
    Moved {
        ida: crate::race::store::OWN[10].0,
        len: crate::race::store::OWN[10].1 - crate::race::store::OWN[10].0,
        home: || crate::race::store::home(crate::race::store::OWN[10].0),
    },
    Moved {
        ida: crate::race::store::OWN[11].0,
        len: crate::race::store::OWN[11].1 - crate::race::store::OWN[11].0,
        home: || crate::race::store::home(crate::race::store::OWN[11].0),
    },
    Moved {
        ida: crate::race::store::OWN[12].0,
        len: crate::race::store::OWN[12].1 - crate::race::store::OWN[12].0,
        home: || crate::race::store::home(crate::race::store::OWN[12].0),
    },
    Moved {
        ida: crate::race::store::OWN[13].0,
        len: crate::race::store::OWN[13].1 - crate::race::store::OWN[13].0,
        home: || crate::race::store::home(crate::race::store::OWN[13].0),
    },
    Moved {
        ida: crate::race::store::OWN[14].0,
        len: crate::race::store::OWN[14].1 - crate::race::store::OWN[14].0,
        home: || crate::race::store::home(crate::race::store::OWN[14].0),
    },
    Moved {
        ida: crate::race::store::OWN[15].0,
        len: crate::race::store::OWN[15].1 - crate::race::store::OWN[15].0,
        home: || crate::race::store::home(crate::race::store::OWN[15].0),
    },
    Moved {
        ida: crate::race::store::OWN[16].0,
        len: crate::race::store::OWN[16].1 - crate::race::store::OWN[16].0,
        home: || crate::race::store::home(crate::race::store::OWN[16].0),
    },
    Moved {
        ida: crate::race::store::OWN[17].0,
        len: crate::race::store::OWN[17].1 - crate::race::store::OWN[17].0,
        home: || crate::race::store::home(crate::race::store::OWN[17].0),
    },
    Moved {
        ida: crate::race::store::OWN[18].0,
        len: crate::race::store::OWN[18].1 - crate::race::store::OWN[18].0,
        home: || crate::race::store::home(crate::race::store::OWN[18].0),
    },
    Moved {
        ida: crate::race::store::OWN[19].0,
        len: crate::race::store::OWN[19].1 - crate::race::store::OWN[19].0,
        home: || crate::race::store::home(crate::race::store::OWN[19].0),
    },
    Moved {
        ida: crate::race::store::OWN[20].0,
        len: crate::race::store::OWN[20].1 - crate::race::store::OWN[20].0,
        home: || crate::race::store::home(crate::race::store::OWN[20].0),
    },
    Moved {
        ida: crate::race::store::OWN[21].0,
        len: crate::race::store::OWN[21].1 - crate::race::store::OWN[21].0,
        home: || crate::race::store::home(crate::race::store::OWN[21].0),
    },
    Moved {
        ida: crate::race::store::OWN[22].0,
        len: crate::race::store::OWN[22].1 - crate::race::store::OWN[22].0,
        home: || crate::race::store::home(crate::race::store::OWN[22].0),
    },
    // race: the groups that left the race core together (`race::groups_store`)
    Moved {
        ida: crate::race::groups_store::VIEWER_AND_KINDS_RANGE.0,
        len: crate::race::groups_store::VIEWER_AND_KINDS_RANGE.1
            - crate::race::groups_store::VIEWER_AND_KINDS_RANGE.0,
        home: crate::race::groups_store::viewer_and_kinds_home,
    },
    Moved {
        ida: crate::race::groups_store::LIGHTS_AND_GEAR_RANGE.0,
        len: crate::race::groups_store::LIGHTS_AND_GEAR_RANGE.1
            - crate::race::groups_store::LIGHTS_AND_GEAR_RANGE.0,
        home: crate::race::groups_store::lights_and_gear_home,
    },
    Moved {
        ida: crate::race::groups_store::CARS_AND_REPLAY_RANGE.0,
        len: crate::race::groups_store::CARS_AND_REPLAY_RANGE.1
            - crate::race::groups_store::CARS_AND_REPLAY_RANGE.0,
        home: crate::race::groups_store::cars_and_replay_home,
    },
    Moved {
        ida: crate::race::groups_store::PANEL_AND_OVERLAY_RANGE.0,
        len: crate::race::groups_store::PANEL_AND_OVERLAY_RANGE.1
            - crate::race::groups_store::PANEL_AND_OVERLAY_RANGE.0,
        home: crate::race::groups_store::panel_and_overlay_home,
    },
    // race: the pieces that left the race core for statics of their own (`race::store::LEFT`)
    Moved {
        ida: crate::physics::split_tables_store::START,
        len: crate::physics::split_tables_store::END - crate::physics::split_tables_store::START,
        home: || crate::physics::split_tables_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::race::league_store::START,
        len: crate::race::league_store::END - crate::race::league_store::START,
        home: || crate::race::league_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::race::lights_store::START,
        len: crate::race::lights_store::END - crate::race::lights_store::START,
        home: || crate::race::lights_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::race::level_paths_store::START,
        len: crate::race::level_paths_store::END - crate::race::level_paths_store::START,
        home: || crate::race::level_paths_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::physics::pace_store::START,
        len: crate::physics::pace_store::END - crate::physics::pace_store::START,
        home: || crate::physics::pace_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::physics::ground_probe_store::START,
        len: crate::physics::ground_probe_store::END - crate::physics::ground_probe_store::START,
        home: || crate::physics::ground_probe_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::race::panel_clock_store::START,
        len: crate::race::panel_clock_store::END - crate::race::panel_clock_store::START,
        home: || crate::race::panel_clock_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::scene::object_slots_store::START,
        len: crate::scene::object_slots_store::END - crate::scene::object_slots_store::START,
        home: || crate::scene::object_slots_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::scene::light_tables_store::START,
        len: crate::scene::light_tables_store::END - crate::scene::light_tables_store::START,
        home: || crate::scene::light_tables_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::race::error_list_store::START,
        len: crate::race::error_list_store::END - crate::race::error_list_store::START,
        home: || crate::race::error_list_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::race::flares_store::START,
        len: crate::race::flares_store::END - crate::race::flares_store::START,
        home: || crate::race::flares_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::race::shadow_skids_store::START,
        len: crate::race::shadow_skids_store::END - crate::race::shadow_skids_store::START,
        home: || crate::race::shadow_skids_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::drive::finish_store::START,
        len: crate::drive::finish_store::END - crate::drive::finish_store::START,
        home: || crate::drive::finish_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::race::replay_file_store::START,
        len: crate::race::replay_file_store::END - crate::race::replay_file_store::START,
        home: || crate::race::replay_file_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::race::viewer_state_store::START,
        len: crate::race::viewer_state_store::END - crate::race::viewer_state_store::START,
        home: || crate::race::viewer_state_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::race::numbers_store::START,
        len: crate::race::numbers_store::END - crate::race::numbers_store::START,
        home: || crate::race::numbers_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::race::scratch_store::START,
        len: crate::race::scratch_store::END - crate::race::scratch_store::START,
        home: || crate::race::scratch_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::race::course_lists_store::START,
        len: crate::race::course_lists_store::END - crate::race::course_lists_store::START,
        home: || crate::race::course_lists_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::race::car_specs_store::START,
        len: crate::race::car_specs_store::END - crate::race::car_specs_store::START,
        home: || crate::race::car_specs_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::race::course_store::START,
        len: crate::race::course_store::END - crate::race::course_store::START,
        home: || crate::race::course_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::scene::object_tables_store::START,
        len: crate::scene::object_tables_store::END - crate::scene::object_tables_store::START,
        home: || crate::scene::object_tables_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::scene::texture_places_store::START,
        len: crate::scene::texture_places_store::END - crate::scene::texture_places_store::START,
        home: || crate::scene::texture_places_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::race::matrices_store::START,
        len: crate::race::matrices_store::END - crate::race::matrices_store::START,
        home: || crate::race::matrices_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::race::tables_store::START,
        len: crate::race::tables_store::END - crate::race::tables_store::START,
        home: || crate::race::tables_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::race::groups_store::VIEW_AND_RECORDS_RANGE.0,
        len: crate::race::groups_store::VIEW_AND_RECORDS_RANGE.1
            - crate::race::groups_store::VIEW_AND_RECORDS_RANGE.0,
        home: crate::race::groups_store::view_and_records_home,
    },
    Moved {
        ida: crate::physics::camera_store::START,
        len: crate::physics::camera_store::END - crate::physics::camera_store::START,
        home: || crate::physics::camera_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::race::prefs_store::START,
        len: crate::race::prefs_store::END - crate::race::prefs_store::START,
        home: || crate::race::prefs_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::scene::file_numbers_store::START,
        len: crate::scene::file_numbers_store::END - crate::scene::file_numbers_store::START,
        home: || crate::scene::file_numbers_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::physics::tables_store::START,
        len: crate::physics::tables_store::END - crate::physics::tables_store::START,
        home: || crate::physics::tables_store::tables_addr() as usize,
    },
    Moved {
        ida: crate::race::vars_store::START,
        len: crate::race::vars_store::END - crate::race::vars_store::START,
        home: || crate::race::vars_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::scene::draw_store::START,
        len: crate::scene::draw_store::END - crate::scene::draw_store::START,
        home: || crate::scene::draw_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::scene::light_store::START,
        len: crate::scene::light_store::END - crate::scene::light_store::START,
        home: || crate::scene::light_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::race::view_store::START,
        len: crate::race::view_store::END - crate::race::view_store::START,
        home: || crate::race::view_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::race::level_grid_store::START,
        len: crate::race::level_grid_store::END - crate::race::level_grid_store::START,
        home: || crate::race::level_grid_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::race::frame_store::START,
        len: crate::race::frame_store::END - crate::race::frame_store::START,
        home: || crate::race::frame_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::race::sound_spots_store::START,
        len: crate::race::sound_spots_store::END - crate::race::sound_spots_store::START,
        home: || crate::race::sound_spots_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::physics::contact_store::START,
        len: crate::physics::contact_store::END - crate::physics::contact_store::START,
        home: || crate::physics::contact_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::drive::replay_frames_store::START,
        len: crate::drive::replay_frames_store::END - crate::drive::replay_frames_store::START,
        home: || crate::drive::replay_frames_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::physics::car_store::START,
        len: crate::physics::car_store::END - crate::physics::car_store::START,
        home: || crate::physics::car_store::car_addr() as usize,
    },
    Moved {
        ida: crate::race::work_store::START,
        len: crate::race::work_store::END - crate::race::work_store::START,
        home: || crate::race::work_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::race::text_state_store::START,
        len: crate::race::text_state_store::END - crate::race::text_state_store::START,
        home: || crate::race::text_state_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::race::clock_ticks_store::START,
        len: crate::race::clock_ticks_store::END - crate::race::clock_ticks_store::START,
        home: || crate::race::clock_ticks_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::race::kind_records_store::START,
        len: crate::race::kind_records_store::END - crate::race::kind_records_store::START,
        home: || crate::race::kind_records_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::scene::kind_texture_store::START,
        len: crate::scene::kind_texture_store::END - crate::scene::kind_texture_store::START,
        home: || crate::scene::kind_texture_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::race::frame_counter_store::START,
        len: crate::race::frame_counter_store::END - crate::race::frame_counter_store::START,
        home: || crate::race::frame_counter_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::race::switches_store::START,
        len: crate::race::switches_store::END - crate::race::switches_store::START,
        home: || crate::race::switches_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::scene::draw_switches_store::START,
        len: crate::scene::draw_switches_store::END - crate::scene::draw_switches_store::START,
        home: || crate::scene::draw_switches_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::scene::light_work_store::START,
        len: crate::scene::light_work_store::END - crate::scene::light_work_store::START,
        home: || crate::scene::light_work_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::scene::object_store::START,
        len: crate::scene::object_store::END - crate::scene::object_store::START,
        home: || crate::scene::object_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::physics::ground_store::START,
        len: crate::physics::ground_store::END - crate::physics::ground_store::START,
        home: || crate::physics::ground_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::race::draw_batches_store::START,
        len: crate::race::draw_batches_store::END - crate::race::draw_batches_store::START,
        home: || crate::race::draw_batches_store::piece_addr() as usize,
    },
    Moved {
        ida: crate::effects::work_store::START,
        len: crate::effects::work_store::END - crate::effects::work_store::START,
        home: || crate::effects::work_store::piece_addr() as usize,
    },
    // the menus' lists, the menus' working variables, the race's settings and results and
    // the damage model (`menu::lists_store`, `menu::state_store`, `race::state_store`,
    // `physics::store`, typed)
    Moved {
        ida: crate::menu::lists_store::START,
        len: crate::menu::lists_store::END - crate::menu::lists_store::START,
        home: || crate::menu::lists_store::menu_lists_addr() as usize,
    },
    Moved {
        ida: crate::menu::state_store::START,
        len: crate::menu::state_store::END - crate::menu::state_store::START,
        home: || crate::menu::state_store::menu_state_addr() as usize,
    },
    Moved {
        ida: crate::race::state_store::START,
        len: crate::race::state_store::END - crate::race::state_store::START,
        home: || crate::race::state_store::race_state_addr() as usize,
    },
    Moved {
        ida: crate::physics::store::START,
        len: crate::physics::store::END - crate::physics::store::START,
        home: || crate::physics::store::damage_addr() as usize,
    },
    // marks: 0x764F40 .. 0x93CCC8 (`effects::store`), less the ranges that left it (`CARVED`),
    // in the segments of `SEGS`; then the ranges that left it, each its own typed static
    Moved {
        ida: crate::effects::store::SEGS[0].0,
        len: crate::effects::store::SEGS[0].1 - crate::effects::store::SEGS[0].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[0].0 }>,
    },
    Moved {
        ida: crate::effects::store::SEGS[1].0,
        len: crate::effects::store::SEGS[1].1 - crate::effects::store::SEGS[1].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[1].0 }>,
    },
    Moved {
        ida: crate::effects::store::SEGS[2].0,
        len: crate::effects::store::SEGS[2].1 - crate::effects::store::SEGS[2].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[2].0 }>,
    },
    Moved {
        ida: crate::effects::store::SEGS[3].0,
        len: crate::effects::store::SEGS[3].1 - crate::effects::store::SEGS[3].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[3].0 }>,
    },
    Moved {
        ida: crate::effects::store::SEGS[4].0,
        len: crate::effects::store::SEGS[4].1 - crate::effects::store::SEGS[4].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[4].0 }>,
    },
    Moved {
        ida: crate::effects::store::SEGS[5].0,
        len: crate::effects::store::SEGS[5].1 - crate::effects::store::SEGS[5].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[5].0 }>,
    },
    Moved {
        ida: crate::effects::store::SEGS[6].0,
        len: crate::effects::store::SEGS[6].1 - crate::effects::store::SEGS[6].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[6].0 }>,
    },
    Moved {
        ida: crate::effects::store::SEGS[7].0,
        len: crate::effects::store::SEGS[7].1 - crate::effects::store::SEGS[7].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[7].0 }>,
    },
    Moved {
        ida: crate::effects::store::SEGS[8].0,
        len: crate::effects::store::SEGS[8].1 - crate::effects::store::SEGS[8].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[8].0 }>,
    },
    Moved {
        ida: crate::effects::store::SEGS[9].0,
        len: crate::effects::store::SEGS[9].1 - crate::effects::store::SEGS[9].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[9].0 }>,
    },
    Moved {
        ida: crate::effects::store::SEGS[10].0,
        len: crate::effects::store::SEGS[10].1 - crate::effects::store::SEGS[10].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[10].0 }>,
    },
    Moved {
        ida: crate::effects::store::SEGS[11].0,
        len: crate::effects::store::SEGS[11].1 - crate::effects::store::SEGS[11].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[11].0 }>,
    },
    Moved {
        ida: crate::effects::store::SEGS[12].0,
        len: crate::effects::store::SEGS[12].1 - crate::effects::store::SEGS[12].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[12].0 }>,
    },
    Moved {
        ida: crate::effects::store::SEGS[13].0,
        len: crate::effects::store::SEGS[13].1 - crate::effects::store::SEGS[13].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[13].0 }>,
    },
    Moved {
        ida: crate::effects::store::SEGS[14].0,
        len: crate::effects::store::SEGS[14].1 - crate::effects::store::SEGS[14].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[14].0 }>,
    },
    Moved {
        ida: crate::effects::store::SEGS[15].0,
        len: crate::effects::store::SEGS[15].1 - crate::effects::store::SEGS[15].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[15].0 }>,
    },
    Moved {
        ida: crate::effects::store::SEGS[16].0,
        len: crate::effects::store::SEGS[16].1 - crate::effects::store::SEGS[16].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[16].0 }>,
    },
    Moved {
        ida: crate::effects::store::SEGS[17].0,
        len: crate::effects::store::SEGS[17].1 - crate::effects::store::SEGS[17].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[17].0 }>,
    },
    Moved {
        ida: crate::effects::store::SEGS[18].0,
        len: crate::effects::store::SEGS[18].1 - crate::effects::store::SEGS[18].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[18].0 }>,
    },
    Moved {
        ida: crate::effects::store::SEGS[19].0,
        len: crate::effects::store::SEGS[19].1 - crate::effects::store::SEGS[19].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[19].0 }>,
    },
    Moved {
        ida: crate::effects::store::SEGS[20].0,
        len: crate::effects::store::SEGS[20].1 - crate::effects::store::SEGS[20].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[20].0 }>,
    },
    Moved {
        ida: crate::effects::store::SEGS[21].0,
        len: crate::effects::store::SEGS[21].1 - crate::effects::store::SEGS[21].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[21].0 }>,
    },
    Moved {
        ida: crate::effects::store::SEGS[22].0,
        len: crate::effects::store::SEGS[22].1 - crate::effects::store::SEGS[22].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[22].0 }>,
    },
    Moved {
        ida: crate::effects::store::SEGS[23].0,
        len: crate::effects::store::SEGS[23].1 - crate::effects::store::SEGS[23].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[23].0 }>,
    },
    Moved {
        ida: crate::effects::store::SEGS[24].0,
        len: crate::effects::store::SEGS[24].1 - crate::effects::store::SEGS[24].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[24].0 }>,
    },
    Moved {
        ida: crate::effects::store::SEGS[25].0,
        len: crate::effects::store::SEGS[25].1 - crate::effects::store::SEGS[25].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[25].0 }>,
    },
    Moved {
        ida: crate::effects::store::SEGS[26].0,
        len: crate::effects::store::SEGS[26].1 - crate::effects::store::SEGS[26].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[26].0 }>,
    },
    Moved {
        ida: crate::effects::store::SEGS[27].0,
        len: crate::effects::store::SEGS[27].1 - crate::effects::store::SEGS[27].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[27].0 }>,
    },
    Moved {
        ida: crate::effects::store::SEGS[28].0,
        len: crate::effects::store::SEGS[28].1 - crate::effects::store::SEGS[28].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[28].0 }>,
    },
    Moved {
        ida: crate::effects::store::SEGS[29].0,
        len: crate::effects::store::SEGS[29].1 - crate::effects::store::SEGS[29].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[29].0 }>,
    },
    Moved {
        ida: crate::effects::store::SEGS[30].0,
        len: crate::effects::store::SEGS[30].1 - crate::effects::store::SEGS[30].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[30].0 }>,
    },
    Moved {
        ida: crate::effects::store::SEGS[31].0,
        len: crate::effects::store::SEGS[31].1 - crate::effects::store::SEGS[31].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[31].0 }>,
    },
    Moved {
        ida: crate::effects::store::SEGS[32].0,
        len: crate::effects::store::SEGS[32].1 - crate::effects::store::SEGS[32].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[32].0 }>,
    },
    Moved {
        ida: crate::effects::store::SEGS[33].0,
        len: crate::effects::store::SEGS[33].1 - crate::effects::store::SEGS[33].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[33].0 }>,
    },
    Moved {
        ida: crate::effects::store::SEGS[34].0,
        len: crate::effects::store::SEGS[34].1 - crate::effects::store::SEGS[34].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[34].0 }>,
    },
    Moved {
        ida: crate::effects::store::SEGS[35].0,
        len: crate::effects::store::SEGS[35].1 - crate::effects::store::SEGS[35].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[35].0 }>,
    },
    Moved {
        ida: crate::effects::store::SEGS[36].0,
        len: crate::effects::store::SEGS[36].1 - crate::effects::store::SEGS[36].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[36].0 }>,
    },
    Moved {
        ida: crate::effects::store::SEGS[37].0,
        len: crate::effects::store::SEGS[37].1 - crate::effects::store::SEGS[37].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[37].0 }>,
    },
    Moved {
        ida: crate::effects::store::SEGS[38].0,
        len: crate::effects::store::SEGS[38].1 - crate::effects::store::SEGS[38].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[38].0 }>,
    },
    Moved {
        ida: crate::effects::store::SEGS[39].0,
        len: crate::effects::store::SEGS[39].1 - crate::effects::store::SEGS[39].0,
        home: crate::effects::store::home_of::<{ crate::effects::store::SEGS[39].0 }>,
    },
    Moved {
        ida: crate::effects::shape_points::ShapeInputs::START,
        len: crate::effects::shape_points::ShapeInputs::LEN,
        home: crate::effects::shape_points::ShapeInputs::home,
    },
    Moved {
        ida: crate::effects::view_vectors::ViewSources::START,
        len: crate::effects::view_vectors::ViewSources::LEN,
        home: crate::effects::view_vectors::ViewSources::home,
    },
    Moved {
        ida: crate::effects::setup::MarksClock::START,
        len: crate::effects::setup::MarksClock::LEN,
        home: crate::effects::setup::MarksClock::home,
    },
    Moved {
        ida: crate::effects::tuning::MarksFrame::START,
        len: crate::effects::tuning::MarksFrame::LEN,
        home: crate::effects::tuning::MarksFrame::home,
    },
    Moved {
        ida: crate::effects::tuning::MarksFloats::START,
        len: crate::effects::tuning::MarksFloats::LEN,
        home: crate::effects::tuning::MarksFloats::home,
    },
    Moved {
        ida: crate::effects::tuning::MarksKindFloats::START,
        len: crate::effects::tuning::MarksKindFloats::LEN,
        home: crate::effects::tuning::MarksKindFloats::home,
    },
    Moved {
        ida: crate::effects::setup::MarksSettings::START,
        len: crate::effects::setup::MarksSettings::LEN,
        home: crate::effects::setup::MarksSettings::home,
    },
    Moved {
        ida: crate::effects::pong::PongState::START,
        len: crate::effects::pong::PongState::LEN,
        home: crate::effects::pong::PongState::home,
    },
    Moved {
        ida: crate::effects::clip::ClipScreen::START,
        len: crate::effects::clip::ClipScreen::LEN,
        home: crate::effects::clip::ClipScreen::home,
    },
    Moved {
        ida: crate::effects::pong::PongRules::START,
        len: crate::effects::pong::PongRules::LEN,
        home: crate::effects::pong::PongRules::home,
    },
    Moved {
        ida: crate::effects::kind_values::MarksKind::START,
        len: crate::effects::kind_values::MarksKind::LEN,
        home: crate::effects::kind_values::MarksKind::home,
    },
    Moved {
        ida: crate::effects::view_vectors::KeptView::START,
        len: crate::effects::view_vectors::KeptView::LEN,
        home: crate::effects::view_vectors::KeptView::home,
    },
    Moved {
        ida: crate::effects::splines::SplineHead::START,
        len: crate::effects::splines::SplineHead::LEN,
        home: crate::effects::splines::SplineHead::home,
    },
    Moved {
        ida: crate::effects::splines::SplineWork::START,
        len: crate::effects::splines::SplineWork::LEN,
        home: crate::effects::splines::SplineWork::home,
    },
    Moved {
        ida: crate::effects::splines::SplineTables::START,
        len: crate::effects::splines::SplineTables::LEN,
        home: crate::effects::splines::SplineTables::home,
    },
    Moved {
        ida: crate::effects::splines::SplineKinds::START,
        len: crate::effects::splines::SplineKinds::LEN,
        home: crate::effects::splines::SplineKinds::home,
    },
    Moved {
        ida: crate::effects::splines::SplinePath::START,
        len: crate::effects::splines::SplinePath::LEN,
        home: crate::effects::splines::SplinePath::home,
    },
    Moved {
        ida: crate::effects::splines::SplineConstants::START,
        len: crate::effects::splines::SplineConstants::LEN,
        home: crate::effects::splines::SplineConstants::home,
    },
    Moved {
        ida: crate::effects::edges::EdgeMask::START,
        len: crate::effects::edges::EdgeMask::LEN,
        home: crate::effects::edges::EdgeMask::home,
    },
    Moved {
        ida: crate::effects::tilt_shape::ShapeTilt::START,
        len: crate::effects::tilt_shape::ShapeTilt::LEN,
        home: crate::effects::tilt_shape::ShapeTilt::home,
    },
    Moved {
        ida: crate::effects::edges::KindBoxes::START,
        len: crate::effects::edges::KindBoxes::LEN,
        home: crate::effects::edges::KindBoxes::home,
    },
    Moved {
        ida: crate::effects::windscreen_drop_state::WindscreenDropState::START,
        len: crate::effects::windscreen_drop_state::WindscreenDropState::LEN,
        home: crate::effects::windscreen_drop_state::WindscreenDropState::home,
    },
    Moved {
        ida: crate::effects::windscreen_drop_state::WindscreenDropWork::START,
        len: crate::effects::windscreen_drop_state::WindscreenDropWork::LEN,
        home: crate::effects::windscreen_drop_state::WindscreenDropWork::home,
    },
    Moved {
        ida: crate::effects::windscreen_drop_vars::WindscreenDropVars::START,
        len: crate::effects::windscreen_drop_vars::WindscreenDropVars::LEN,
        home: crate::effects::windscreen_drop_vars::WindscreenDropVars::home,
    },
    Moved {
        ida: crate::effects::mask_vars::MaskVars::START,
        len: crate::effects::mask_vars::MaskVars::LEN,
        home: crate::effects::mask_vars::MaskVars::home,
    },
    Moved {
        ida: crate::effects::mask_picks::MaskPicks::START,
        len: crate::effects::mask_picks::MaskPicks::LEN,
        home: crate::effects::mask_picks::MaskPicks::home,
    },
    Moved {
        ida: crate::effects::shape_work::ShapeWork::START,
        len: crate::effects::shape_work::ShapeWork::LEN,
        home: crate::effects::shape_work::ShapeWork::home,
    },
    Moved {
        ida: crate::effects::shape_points::ShapeSetTuning::START,
        len: crate::effects::shape_points::ShapeSetTuning::LEN,
        home: crate::effects::shape_points::ShapeSetTuning::home,
    },
    Moved {
        ida: crate::effects::mask_constants::MaskConstants::START,
        len: crate::effects::mask_constants::MaskConstants::LEN,
        home: crate::effects::mask_constants::MaskConstants::home,
    },
    Moved {
        ida: crate::effects::windscreen_flakes_store::WindscreenFlakeTiming::START,
        len: crate::effects::windscreen_flakes_store::WindscreenFlakeTiming::LEN,
        home: crate::effects::windscreen_flakes_store::WindscreenFlakeTiming::home,
    },
    Moved {
        ida: crate::effects::windscreen_flakes_store::WindscreenFlakeTables::START,
        len: crate::effects::windscreen_flakes_store::WindscreenFlakeTables::LEN,
        home: crate::effects::windscreen_flakes_store::WindscreenFlakeTables::home,
    },
    Moved {
        ida: crate::effects::windscreen_flakes_store::WindscreenFlakeItems::START,
        len: crate::effects::windscreen_flakes_store::WindscreenFlakeItems::LEN,
        home: crate::effects::windscreen_flakes_store::WindscreenFlakeItems::home,
    },
    Moved {
        ida: crate::effects::windscreen_flakes_store::WindscreenFlakeState::START,
        len: crate::effects::windscreen_flakes_store::WindscreenFlakeState::LEN,
        home: crate::effects::windscreen_flakes_store::WindscreenFlakeState::home,
    },
    Moved {
        ida: crate::effects::edges::EdgeSettings::START,
        len: crate::effects::edges::EdgeSettings::LEN,
        home: crate::effects::edges::EdgeSettings::home,
    },
    Moved {
        ida: crate::effects::rain::RainState::START,
        len: crate::effects::rain::RainState::LEN,
        home: crate::effects::rain::RainState::home,
    },
    Moved {
        ida: crate::effects::rain::RainRules::START,
        len: crate::effects::rain::RainRules::LEN,
        home: crate::effects::rain::RainRules::home,
    },
    Moved {
        ida: crate::effects::snow::SnowTables::START,
        len: crate::effects::snow::SnowTables::LEN,
        home: crate::effects::snow::SnowTables::home,
    },
    Moved {
        ida: crate::effects::snow::SnowState::START,
        len: crate::effects::snow::SnowState::LEN,
        home: crate::effects::snow::SnowState::home,
    },
    Moved {
        ida: crate::effects::snow::SnowCurve::START,
        len: crate::effects::snow::SnowCurve::LEN,
        home: crate::effects::snow::SnowCurve::home,
    },
    Moved {
        ida: crate::effects::snow::SnowCurveVars::START,
        len: crate::effects::snow::SnowCurveVars::LEN,
        home: crate::effects::snow::SnowCurveVars::home,
    },
    Moved {
        ida: crate::effects::snow::SnowLight::START,
        len: crate::effects::snow::SnowLight::LEN,
        home: crate::effects::snow::SnowLight::home,
    },
    Moved {
        ida: crate::effects::blocks::PlacesBlock::START,
        len: crate::effects::blocks::PlacesBlock::LEN,
        home: crate::effects::blocks::PlacesBlock::home,
    },
    Moved {
        ida: crate::effects::profiles::ProfileRings::START,
        len: crate::effects::profiles::ProfileRings::LEN,
        home: crate::effects::profiles::ProfileRings::home,
    },
    Moved {
        ida: crate::effects::profiles::ProfileTables::START,
        len: crate::effects::profiles::ProfileTables::LEN,
        home: crate::effects::profiles::ProfileTables::home,
    },
    Moved {
        ida: crate::effects::frame_state::DrawWork::START,
        len: crate::effects::frame_state::DrawWork::LEN,
        home: crate::effects::frame_state::DrawWork::home,
    },
    Moved {
        ida: crate::effects::blocks::MarkCorners::START,
        len: crate::effects::blocks::MarkCorners::LEN,
        home: crate::effects::blocks::MarkCorners::home,
    },
    Moved {
        ida: crate::effects::frame_state::ViewVars::START,
        len: crate::effects::frame_state::ViewVars::LEN,
        home: crate::effects::frame_state::ViewVars::home,
    },
    Moved {
        ida: crate::effects::profiles::ProfileLists::START,
        len: crate::effects::profiles::ProfileLists::LEN,
        home: crate::effects::profiles::ProfileLists::home,
    },
    Moved {
        ida: crate::effects::frame_state::DrawFrame::START,
        len: crate::effects::frame_state::DrawFrame::LEN,
        home: crate::effects::frame_state::DrawFrame::home,
    },
    Moved {
        ida: crate::effects::waves::MarksSines::START,
        len: crate::effects::waves::MarksSines::LEN,
        home: crate::effects::waves::MarksSines::home,
    },
    Moved {
        ida: crate::effects::frame_state::FlareParts::START,
        len: crate::effects::frame_state::FlareParts::LEN,
        home: crate::effects::frame_state::FlareParts::home,
    },
    Moved {
        ida: crate::effects::waves::WaveLevels::START,
        len: crate::effects::waves::WaveLevels::LEN,
        home: crate::effects::waves::WaveLevels::home,
    },
    Moved {
        ida: crate::effects::waves::WaveState::START,
        len: crate::effects::waves::WaveState::LEN,
        home: crate::effects::waves::WaveState::home,
    },
    Moved {
        ida: crate::effects::frame_state::FlareScales::START,
        len: crate::effects::frame_state::FlareScales::LEN,
        home: crate::effects::frame_state::FlareScales::home,
    },
    Moved {
        ida: crate::effects::blocks::MarkLinks::START,
        len: crate::effects::blocks::MarkLinks::LEN,
        home: crate::effects::blocks::MarkLinks::home,
    },
    Moved {
        ida: crate::effects::blocks::CornerSlopes::START,
        len: crate::effects::blocks::CornerSlopes::LEN,
        home: crate::effects::blocks::CornerSlopes::home,
    },
    Moved {
        ida: crate::effects::run_particles::TrailColours::START,
        len: crate::effects::run_particles::TrailColours::LEN,
        home: crate::effects::run_particles::TrailColours::home,
    },
    Moved {
        ida: crate::effects::spread_point::SpreadRandom::START,
        len: crate::effects::spread_point::SpreadRandom::LEN,
        home: crate::effects::spread_point::SpreadRandom::home,
    },
    Moved {
        ida: crate::effects::kind_store::Quad1::START,
        len: crate::effects::kind_store::Quad1::LEN,
        home: crate::effects::kind_store::Quad1::home,
    },
    Moved {
        ida: crate::effects::kind_store::Tables1::START,
        len: crate::effects::kind_store::Tables1::LEN,
        home: crate::effects::kind_store::Tables1::home,
    },
    Moved {
        ida: crate::effects::kind_store::KindExtras1::START,
        len: crate::effects::kind_store::KindExtras1::LEN,
        home: crate::effects::kind_store::KindExtras1::home,
    },
    Moved {
        ida: crate::effects::kind_store::Consts1::START,
        len: crate::effects::kind_store::Consts1::LEN,
        home: crate::effects::kind_store::Consts1::home,
    },
    Moved {
        ida: crate::effects::kind_store::Quad2::START,
        len: crate::effects::kind_store::Quad2::LEN,
        home: crate::effects::kind_store::Quad2::home,
    },
    Moved {
        ida: crate::effects::kind_store::Tables2::START,
        len: crate::effects::kind_store::Tables2::LEN,
        home: crate::effects::kind_store::Tables2::home,
    },
    Moved {
        ida: crate::effects::kind_store::KindExtra2::START,
        len: crate::effects::kind_store::KindExtra2::LEN,
        home: crate::effects::kind_store::KindExtra2::home,
    },
    Moved {
        ida: crate::effects::kind_store::Consts2::START,
        len: crate::effects::kind_store::Consts2::LEN,
        home: crate::effects::kind_store::Consts2::home,
    },
    Moved {
        ida: crate::effects::kind_store::Quad3::START,
        len: crate::effects::kind_store::Quad3::LEN,
        home: crate::effects::kind_store::Quad3::home,
    },
    Moved {
        ida: crate::effects::mask_store::GroundNames::START,
        len: crate::effects::mask_store::GroundNames::LEN,
        home: crate::effects::mask_store::GroundNames::home,
    },
    Moved {
        ida: crate::effects::mask_store::MaskBlocks::START,
        len: crate::effects::mask_store::MaskBlocks::LEN,
        home: crate::effects::mask_store::MaskBlocks::home,
    },
    Moved {
        ida: crate::effects::mask_store::PictureCopy::START,
        len: crate::effects::mask_store::PictureCopy::LEN,
        home: crate::effects::mask_store::PictureCopy::home,
    },
    Moved {
        ida: crate::effects::mask_store::KindPictures::START,
        len: crate::effects::mask_store::KindPictures::LEN,
        home: crate::effects::mask_store::KindPictures::home,
    },
    Moved {
        ida: crate::effects::draw_plane::PlaneTuning::START,
        len: crate::effects::draw_plane::PlaneTuning::LEN,
        home: crate::effects::draw_plane::PlaneTuning::home,
    },
    Moved {
        ida: crate::effects::mark_slots::MarkSlots::START,
        len: crate::effects::mark_slots::MarkSlots::LEN,
        home: crate::effects::mark_slots::MarkSlots::home,
    },
    Moved {
        ida: crate::effects::mark_slots::SlotTuning::START,
        len: crate::effects::mark_slots::SlotTuning::LEN,
        home: crate::effects::mark_slots::SlotTuning::home,
    },
    Moved {
        ida: crate::effects::emitters::EmitterVars::START,
        len: crate::effects::emitters::EmitterVars::LEN,
        home: crate::effects::emitters::EmitterVars::home,
    },
    Moved {
        ida: crate::effects::ring_store::RingWork::START,
        len: crate::effects::ring_store::RingWork::LEN,
        home: crate::effects::ring_store::RingWork::home,
    },
    Moved {
        ida: crate::effects::ring_store::PictureNames::START,
        len: crate::effects::ring_store::PictureNames::LEN,
        home: crate::effects::ring_store::PictureNames::home,
    },
    Moved {
        ida: crate::effects::surface_store::SurfaceTables::START,
        len: crate::effects::surface_store::SurfaceTables::LEN,
        home: crate::effects::surface_store::SurfaceTables::home,
    },
    Moved {
        ida: crate::effects::run_ribbons::Colours257::START,
        len: crate::effects::run_ribbons::Colours257::LEN,
        home: crate::effects::run_ribbons::Colours257::home,
    },
    Moved {
        ida: crate::effects::emit_particle_19::TwinState::START,
        len: crate::effects::emit_particle_19::TwinState::LEN,
        home: crate::effects::emit_particle_19::TwinState::home,
    },
    Moved {
        ida: crate::effects::emit_kind_1::Kind1Choice::START,
        len: crate::effects::emit_kind_1::Kind1Choice::LEN,
        home: crate::effects::emit_kind_1::Kind1Choice::home,
    },
    Moved {
        ida: crate::effects::set_up_surface::MaskList::START,
        len: crate::effects::set_up_surface::MaskList::LEN,
        home: crate::effects::set_up_surface::MaskList::home,
    },
    Moved {
        ida: crate::effects::draw_turned_sprite::SpriteUv::START,
        len: crate::effects::draw_turned_sprite::SpriteUv::LEN,
        home: crate::effects::draw_turned_sprite::SpriteUv::home,
    },
    Moved {
        ida: crate::effects::emit_particle_5::Kind5Set::START,
        len: crate::effects::emit_particle_5::Kind5Set::LEN,
        home: crate::effects::emit_particle_5::Kind5Set::home,
    },
    Moved {
        ida: crate::effects::emit_particle_11::Kind11Set::START,
        len: crate::effects::emit_particle_11::Kind11Set::LEN,
        home: crate::effects::emit_particle_11::Kind11Set::home,
    },
    Moved {
        ida: crate::effects::emit_particle_21::Kind21Set::START,
        len: crate::effects::emit_particle_21::Kind21Set::LEN,
        home: crate::effects::emit_particle_21::Kind21Set::home,
    },
    Moved {
        ida: crate::effects::emit_kind_1::Kind1Set::START,
        len: crate::effects::emit_kind_1::Kind1Set::LEN,
        home: crate::effects::emit_kind_1::Kind1Set::home,
    },
    Moved {
        ida: crate::effects::emit_bursts::FirstTuning::START,
        len: crate::effects::emit_bursts::FirstTuning::LEN,
        home: crate::effects::emit_bursts::FirstTuning::home,
    },
    Moved {
        ida: crate::effects::emit_bursts_2::SecondTuning::START,
        len: crate::effects::emit_bursts_2::SecondTuning::LEN,
        home: crate::effects::emit_bursts_2::SecondTuning::home,
    },
    Moved {
        ida: crate::effects::emit_particle_6::Kind6Set::START,
        len: crate::effects::emit_particle_6::Kind6Set::LEN,
        home: crate::effects::emit_particle_6::Kind6Set::home,
    },
    Moved {
        ida: crate::effects::emit_particle_13::Kind13Set::START,
        len: crate::effects::emit_particle_13::Kind13Set::LEN,
        home: crate::effects::emit_particle_13::Kind13Set::home,
    },
    Moved {
        ida: crate::effects::emit_particle_19::NineteenthTuning::START,
        len: crate::effects::emit_particle_19::NineteenthTuning::LEN,
        home: crate::effects::emit_particle_19::NineteenthTuning::home,
    },
    Moved {
        ida: crate::effects::emit_particle_22::TwentySecondTuning::START,
        len: crate::effects::emit_particle_22::TwentySecondTuning::LEN,
        home: crate::effects::emit_particle_22::TwentySecondTuning::home,
    },
    Moved {
        ida: crate::effects::run_ring::RingKinds::START,
        len: crate::effects::run_ring::RingKinds::LEN,
        home: crate::effects::run_ring::RingKinds::home,
    },
    Moved {
        ida: crate::effects::blocks::RingPool::START,
        len: crate::effects::blocks::RingPool::LEN,
        home: crate::effects::blocks::RingPool::home,
    },
    Moved {
        ida: crate::effects::run_particles::PuffWork::START,
        len: crate::effects::run_particles::PuffWork::LEN,
        home: crate::effects::run_particles::PuffWork::home,
    },
    Moved {
        ida: crate::effects::run_particles::BurstColours::START,
        len: crate::effects::run_particles::BurstColours::LEN,
        home: crate::effects::run_particles::BurstColours::home,
    },
    Moved {
        ida: crate::effects::run_particles::ParticleTuning::START,
        len: crate::effects::run_particles::ParticleTuning::LEN,
        home: crate::effects::run_particles::ParticleTuning::home,
    },
    Moved {
        ida: crate::effects::blend_corner::BlendWork::START,
        len: crate::effects::blend_corner::BlendWork::LEN,
        home: crate::effects::blend_corner::BlendWork::home,
    },
    Moved {
        ida: crate::effects::morph_object_points::MorphWork::START,
        len: crate::effects::morph_object_points::MorphWork::LEN,
        home: crate::effects::morph_object_points::MorphWork::home,
    },
    Moved {
        ida: crate::effects::lines::LinesWork::START,
        len: crate::effects::lines::LinesWork::LEN,
        home: crate::effects::lines::LinesWork::home,
    },
    Moved {
        ida: crate::effects::run_frame::FrameK::START,
        len: crate::effects::run_frame::FrameK::LEN,
        home: crate::effects::run_frame::FrameK::home,
    },
    Moved {
        ida: crate::effects::setup::ClockK::START,
        len: crate::effects::setup::ClockK::LEN,
        home: crate::effects::setup::ClockK::home,
    },
    Moved {
        ida: crate::effects::clip::GlowState::START,
        len: crate::effects::clip::GlowState::LEN,
        home: crate::effects::clip::GlowState::home,
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
            .map(|m| (m.ida, m.ida + m.len, (m.home)()))
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
                std::slice::from_raw_parts(crate::pointers::host_at(home) as *const u8, end - start)
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
            )
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

/// The moved ranges (by IDA address) whose readers take the image's pointers in them as IDA
/// addresses on purpose: the thread slots (`threading::start_slot_thread`) and the marks'
/// profile ramps (`effects::profiles::ProfileLists`).
const RAW_KEPT: [usize; 2] = [0x5184F4, crate::effects::profiles::ProfileLists::START];

fn moved_home(ida: usize) -> Option<(usize, usize)> {
    MOVED
        .iter()
        .find(|m| ida.wrapping_sub(m.ida) < m.len)
        .map(|m| ((m.home)() + (ida - m.ida), m.ida))
}

/// Points the game's pointers that moved with their blocks (`relocations::PLACES`, whose
/// first values are the image's bytes before relocation, IDA addresses) at their targets,
/// as the image's relocation pointed them: a moved target's home, or else its place in the
/// reserved image (a code address: the menus' and the routes' tables run it by its value). The ranges of `RAW_KEPT` stay as
/// they are. Run once at start (`base` the reserved image's address; `framework::BASE` is
/// not set yet). Returns how many pointers it changed, or the first place outside every
/// moved range.
pub unsafe fn point_into_homes(base: usize) -> Result<usize, usize> {
    let mut changed = 0;
    for &ida in &crate::relocations::PLACES {
        let ida = ida as usize;
        match moved_home(ida) {
            Some((_, block)) if RAW_KEPT.contains(&block) => {}
            // Kept as handles, not addresses (phase 13).
            Some(_) if crate::menu::menus_store::holds_handle(ida) => {}
            Some(_) if crate::audio::store::holds_handle(ida) => {}
            Some((home, _)) => unsafe {
                let p = crate::pointers::host_at(home) as *mut u32;
                let target = p.read_unaligned() as usize;
                let now = match moved_home(target) {
                    Some((h, _)) => h,
                    None => base + rva(target),
                };
                p.write_unaligned(now as u32);
                changed += 1;
            },
            None => return Err(ida),
        }
    }
    Ok(changed)
}

/// RAL.EXE's data as `tools/extract_image` packed it (decoupling step B): only the tests
/// read it now, to check the moved constants and first values.
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
            let home = (m.home)();
            assert_eq!(home_of_rva(rva(m.ida)), Some(home), "{:#x}", m.ida);
            assert_eq!(game_addr(rva(m.ida + m.len - 1)) as usize, home + m.len - 1);
            if !MOVED.iter().any(|o| o.ida == m.ida + m.len) {
                assert_eq!(home_of_rva(rva(m.ida + m.len)), None, "{:#x}", m.ida);
            }
        }
        // zlib's own variables: nothing reads them, not moved (`crate::rest`).
        assert_eq!(home_of_rva(rva(0xA8C18C)), None);
    }

    /// `point_into_homes` ends the start-up at a relocated pointer outside every moved
    /// range: each piece split or dropped must keep those places homed.
    #[test]
    fn every_relocated_pointer_has_a_home() {
        let lost: Vec<String> = crate::relocations::PLACES
            .iter()
            .filter(|&&ida| moved_home(ida as usize).is_none())
            .map(|ida| format!("{ida:#x}"))
            .collect();
        assert!(lost.is_empty(), "relocated pointers with no home: {lost:?}");
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
    #[test]
    fn relocations_are_the_images() {
        let d = IMAGE_DATA;
        let word = |at: usize| u32::from_le_bytes(d[at..at + 4].try_into().unwrap()) as usize;
        let (_, at) = image_sections();
        let count = word(at);
        let mut wanted: Vec<u32> = (0..count)
            .map(|k| (word(at + 4 + 4 * k) + IMAGE_BASE) as u32)
            .filter(|&ida| moved_home(ida as usize).is_some())
            .collect();
        wanted.sort_unstable();
        assert_eq!(&crate::relocations::PLACES[..], &wanted[..]);
        assert_eq!(&d[at + 4 + 4 * count..at + 8 + 4 * count], b"END!");
    }

    /// `standalone`'s reserved image has the image's size and its code where the image had it.
    #[test]
    fn the_image_layout_is_the_images() {
        let word = |at: usize| u32::from_le_bytes(IMAGE_DATA[at..at + 4].try_into().unwrap());
        assert_eq!(word(8) as usize, crate::standalone::IMAGE_SIZE);
        assert_eq!(word(12) as usize, IMAGE_BASE);
        let code: Vec<(usize, usize)> = image_sections()
            .0
            .into_iter()
            .filter(|&(_, _, flags)| flags & 1 != 0)
            .map(|(rva, len, _)| (rva, len))
            .collect();
        assert_eq!(&code[..], &crate::standalone::CODE[..]);
    }

    #[test]
    fn image_bytes_reads_the_image() {
        assert_eq!(image_bytes(0x4E0000, 6), b"$VER: ");
        // bss reads as zeros
        assert_eq!(image_bytes(0x93E430, 4), [0; 4]);
    }
}
