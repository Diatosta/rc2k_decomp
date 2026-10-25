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

    pub fn addr(&self) -> usize {
        self.ptr() as usize
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
    // gfx / hud / d3d (phase 5, `gfx::block`)
    Moved {
        ida: crate::hud::store::START, // the HUD's variables, batch included
        len: crate::hud::store::LEN,
        home: crate::hud::store::hud_addr,
    },
    Moved {
        ida: crate::gfx::decoder::START, // The picture decoders' and glyph builder's variables (decoded picture, palette, sheet table, PCX header and stream)
        len: crate::gfx::decoder::LEN,
        home: || crate::gfx::decoder::STORE.addr(),
    },
    Moved {
        ida: crate::gfx::blit::START, // The blitter's and rescaler's variables (the clip rectangle, the skips of the visible part, the rescale steps and weights)
        len: crate::gfx::blit::LEN,
        home: || crate::gfx::blit::STORE.addr(),
    },
    Moved {
        ida: crate::gfx::strips::START, // The HUD's view strips' places (sizes, tops, lefts)
        len: crate::gfx::strips::LEN,
        home: || crate::gfx::strips::STORE.addr(),
    },
    Moved {
        ida: crate::gfx::formats::START, // The pixel formats' work variables (format numbers, row table, component format)
        len: crate::gfx::formats::LEN,
        home: || crate::gfx::formats::STORE.addr(),
    },
    Moved {
        ida: crate::gfx::modes::START, // The screen mode tables (the thirds table, pixel kinds, sizes, the modes listed)
        len: crate::gfx::modes::LEN,
        home: || crate::gfx::modes::STORE.addr(),
    },
    Moved {
        ida: crate::gfx::converters::START, // The MMX converters' masks and the converters table
        len: crate::gfx::converters::LEN,
        home: || crate::gfx::converters::STORE.addr(),
    },
    Moved {
        ida: crate::gfx::reports::START, // The 3D reports switch (dword_4F52AC)
        len: crate::gfx::reports::LEN,
        home: || crate::gfx::reports::STORE.addr(),
    },
    Moved {
        ida: crate::gfx::shots::START, // The screenshot switch, its lock and the file name's template
        len: crate::gfx::shots::LEN,
        home: || crate::gfx::shots::STORE.addr(),
    },
    Moved {
        ida: crate::gfx::picture_store::START, // The picture loader's variables (the name, flags, size, rows and buffer of the picture, the list, the grades)
        len: crate::gfx::picture_store::LEN,
        home: || crate::gfx::picture_store::STORE.addr(),
    },
    Moved {
        ida: crate::gfx::bar_store::START, // The progress bar's variables (position, width, glyphs, percent)
        len: crate::gfx::bar_store::LEN,
        home: || crate::gfx::bar_store::STORE.addr(),
    },
    Moved {
        ida: crate::gfx::png_file::START, // The PNG decoder's file (pointer, length, memory list)
        len: crate::gfx::png_file::LEN,
        home: || crate::gfx::png_file::STORE.addr(),
    },
    Moved {
        ida: crate::gfx::png_header::START, // The PNG header's fields (width, height, compression, colour type)
        len: crate::gfx::png_header::LEN,
        home: || crate::gfx::png_header::STORE.addr(),
    },
    Moved {
        ida: crate::gfx::pcx_planes::START, // The screenshot writer's plane buffer (3 * width bytes, up to 1920 wide)
        len: crate::gfx::pcx_planes::LEN,
        home: || crate::gfx::pcx_planes::STORE.addr(),
    },
    Moved {
        ida: crate::gfx::cards::START, // The 3D primary and secondary card switches
        len: crate::gfx::cards::LEN,
        home: || crate::gfx::cards::STORE.addr(),
    },
    Moved {
        ida: crate::gfx::upload::START, // The texture upload's surface description (DDSURFACEDESC) and the mode-trial switch
        len: crate::gfx::upload::LEN,
        home: || crate::gfx::upload::STORE.addr(),
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
        ida: crate::gfx::typefaces::tables::START, // the fonts, the front end's tables
        len: crate::gfx::typefaces::tables::LEN,
        home: crate::gfx::typefaces::tables::base,
    },
    Moved {
        ida: crate::gfx::records::tables::START, // the 3D card's tables, the map's state
        len: crate::gfx::records::tables::LEN,
        home: crate::gfx::records::tables::base,
    },
    Moved {
        ida: crate::gfx::front::START, // The DirectDraw front and back surface handles
        len: crate::gfx::front::LEN,
        home: || crate::gfx::front::STORE.addr(),
    },
    Moved {
        ida: crate::gfx::locked::START, // The surface-locked switch
        len: crate::gfx::locked::LEN,
        home: || crate::gfx::locked::STORE.addr(),
    },
    Moved {
        ida: crate::gfx::hexagon_batch::START, // The marks' hexagon batch drawn by d3d (the index and vertex counts, the fans, the records)
        len: crate::gfx::hexagon_batch::LEN,
        home: || crate::gfx::hexagon_batch::STORE.addr(),
    },
    // session: the screen, the window's flags, the machine, the clocks (`session`)
    Moved {
        ida: crate::session::SCREEN_IDA,
        len: crate::session::SCREEN_LEN,
        home: || &crate::session::SCREEN as *const _ as usize,
    },
    Moved {
        ida: crate::session::FLAGS_IDA,
        len: crate::session::FLAGS_LEN,
        home: || &crate::session::FLAGS as *const _ as usize,
    },
    Moved {
        ida: crate::session::MACHINE_IDA,
        len: crate::session::MACHINE_LEN,
        home: || &crate::session::MACHINE as *const _ as usize,
    },
    Moved {
        ida: crate::session::CLOCKS_IDA,
        len: crate::session::CLOCKS_LEN,
        home: || &crate::session::CLOCKS as *const _ as usize,
    },
    // audio: the audio thread's variables and the sample records (`audio::store`)
    Moved {
        ida: crate::audio::store::STATE_IDA,
        len: crate::audio::store::STATE_END - crate::audio::store::STATE_IDA,
        home: crate::audio::store::state_addr,
    },
    Moved {
        ida: crate::audio::store::SAMPLES_IDA,
        len: crate::audio::store::SAMPLES_END - crate::audio::store::SAMPLES_IDA,
        home: crate::audio::store::sample_records_addr,
    },
    Moved {
        ida: crate::audio::store::QUEUE_IDA,
        len: crate::audio::store::QUEUE_END - crate::audio::store::QUEUE_IDA,
        home: crate::audio::store::commands_addr,
    },
    Moved {
        ida: 0x6D1DD4, // g_eaxEnvironment
        len: 4,
        home: || crate::audio::store::ENVIRONMENT.addr(),
    },
    // sound: the race sounds' clock, and the spot picks .. CD audio block (`sound::store`)
    Moved {
        ida: 0x4F66DD, // the options file's four kept dwords and a gap (`saves::load_options::KEPT`)
        len: 0x14,
        home: || crate::saves::load_options::KEPT.as_ptr() as usize,
    },
    Moved {
        ida: 0x4F66F1, // the wheel surfaces (`sound::store::WHEEL_SURFACES`)
        len: 8,
        home: || crate::sound::store::WHEEL_SURFACES.as_ptr() as usize,
    },
    Moved {
        ida: 0x4F66F9, // the race record's seal key (`race::replay::SEAL_KEY`)
        len: 4,
        home: || crate::race::replay::SEAL_KEY.addr(),
    },
    Moved {
        ida: 0x512A10, // the race sounds' objects (`sound::store::RaceSounds`)
        len: 0x2580,
        home: crate::sound::store::race_sounds_addr,
    },
    Moved {
        ida: 0x5124E4, // the race sounds' clock (`sound::store::RaceClock`)
        len: 0x24,
        home: crate::sound::store::race_clock_addr,
    },
    Moved {
        ida: 0x516358, // the surface sounds' constants (`sound::store::SurfaceSounds`)
        len: 0x44,
        home: crate::sound::store::surface_sounds_addr,
    },
    Moved {
        ida: 0x51655C, // the stage sounds (`sound::store::StageSounds`)
        len: 0x364,
        home: crate::sound::store::stage_sounds_addr,
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
        home: crate::input::store::keyboard_file_addr,
    },
    Moved {
        ida: crate::input::store::DEVICES_IDA,
        len: crate::input::store::DEVICES_LEN,
        home: crate::input::store::devices_addr,
    },
    Moved {
        ida: crate::input::store::KEYBOARD_IDA,
        len: crate::input::store::KEYBOARD_LEN,
        home: crate::input::store::keyboard_addr,
    },
    Moved {
        ida: crate::input::store::MOUSE_IDA,
        len: crate::input::store::MOUSE_LEN,
        home: crate::input::store::mouse_addr,
    },
    Moved {
        ida: 0x511B0C, // g_nextMenuSoundTime
        len: 4,
        home: || crate::sound::store::NEXT_MENU_SOUND_TIME.addr(),
    },
    Moved {
        ida: 0x6218F0, // g_raceSoundsOn
        len: 4,
        home: || crate::sound::store::RACE_SOUNDS_ON.addr(),
    },
    Moved {
        ida: crate::sound::store::CAR_STREAM_IDA, // the car stream's place, list and switch
        len: 12,
        home: || crate::sound::store::CAR_STREAM.place.addr(),
    },
    // menu: the front end's variables (`menu::store`)
    Moved {
        ida: crate::menu::store::controls::START,
        len: crate::menu::store::controls::LEN,
        home: crate::menu::store::controls::base,
    },
    Moved {
        ida: crate::menu::store::state::START,
        len: crate::menu::store::state::LEN,
        home: crate::menu::store::state::base,
    },
    Moved {
        ida: crate::menu::store::players::START,
        len: crate::menu::store::players::LEN,
        home: crate::menu::store::players::base,
    },
    Moved {
        ida: crate::menu::store::menus::START,
        len: crate::menu::store::menus::LEN,
        home: crate::menu::store::menus::base,
    },
    Moved {
        ida: crate::menu::store::lists::START,
        len: crate::menu::store::lists::LEN,
        home: crate::menu::store::lists::base,
    },
    // rest: the read-only data, the texts and float constants (`crate::rest::rdata`)
    Moved {
        ida: crate::rest::rdata::texts::START,
        len: crate::rest::rdata::texts::LEN,
        home: crate::rest::rdata::texts::base,
    },
    // rest: the image's lower scattered variables (`crate::rest::low`)
    Moved {
        ida: crate::rest::low::OWN[0].0,
        len: crate::rest::low::OWN[0].1 - crate::rest::low::OWN[0].0,
        home: || crate::rest::low::home(crate::rest::low::OWN[0].0),
    },
    Moved {
        ida: crate::rest::low::OWN[1].0,
        len: crate::rest::low::OWN[1].1 - crate::rest::low::OWN[1].0,
        home: || crate::rest::low::home(crate::rest::low::OWN[1].0),
    },
    Moved {
        ida: crate::rest::low::OWN[2].0,
        len: crate::rest::low::OWN[2].1 - crate::rest::low::OWN[2].0,
        home: || crate::rest::low::home(crate::rest::low::OWN[2].0),
    },
    Moved {
        ida: crate::rest::low::OWN[3].0,
        len: crate::rest::low::OWN[3].1 - crate::rest::low::OWN[3].0,
        home: || crate::rest::low::home(crate::rest::low::OWN[3].0),
    },
    Moved {
        ida: crate::rest::low::OWN[4].0,
        len: crate::rest::low::OWN[4].1 - crate::rest::low::OWN[4].0,
        home: || crate::rest::low::home(crate::rest::low::OWN[4].0),
    },
    Moved {
        ida: crate::rest::low::OWN[5].0,
        len: crate::rest::low::OWN[5].1 - crate::rest::low::OWN[5].0,
        home: || crate::rest::low::home(crate::rest::low::OWN[5].0),
    },
    Moved {
        ida: crate::rest::low::OWN[6].0,
        len: crate::rest::low::OWN[6].1 - crate::rest::low::OWN[6].0,
        home: || crate::rest::low::home(crate::rest::low::OWN[6].0),
    },
    Moved {
        ida: crate::rest::low::OWN[7].0,
        len: crate::rest::low::OWN[7].1 - crate::rest::low::OWN[7].0,
        home: || crate::rest::low::home(crate::rest::low::OWN[7].0),
    },
    Moved {
        ida: crate::rest::low::OWN[8].0,
        len: crate::rest::low::OWN[8].1 - crate::rest::low::OWN[8].0,
        home: || crate::rest::low::home(crate::rest::low::OWN[8].0),
    },
    Moved {
        ida: crate::rest::low::OWN[9].0,
        len: crate::rest::low::OWN[9].1 - crate::rest::low::OWN[9].0,
        home: || crate::rest::low::home(crate::rest::low::OWN[9].0),
    },
    Moved {
        ida: crate::rest::low::OWN[10].0,
        len: crate::rest::low::OWN[10].1 - crate::rest::low::OWN[10].0,
        home: || crate::rest::low::home(crate::rest::low::OWN[10].0),
    },
    Moved {
        ida: crate::rest::low::OWN[11].0,
        len: crate::rest::low::OWN[11].1 - crate::rest::low::OWN[11].0,
        home: || crate::rest::low::home(crate::rest::low::OWN[11].0),
    },
    Moved {
        ida: crate::rest::low::OWN[12].0,
        len: crate::rest::low::OWN[12].1 - crate::rest::low::OWN[12].0,
        home: || crate::rest::low::home(crate::rest::low::OWN[12].0),
    },
    Moved {
        ida: crate::rest::low::OWN[13].0,
        len: crate::rest::low::OWN[13].1 - crate::rest::low::OWN[13].0,
        home: || crate::rest::low::home(crate::rest::low::OWN[13].0),
    },
    Moved {
        ida: crate::rest::low::OWN[14].0,
        len: crate::rest::low::OWN[14].1 - crate::rest::low::OWN[14].0,
        home: || crate::rest::low::home(crate::rest::low::OWN[14].0),
    },
    Moved {
        ida: crate::rest::low::OWN[15].0,
        len: crate::rest::low::OWN[15].1 - crate::rest::low::OWN[15].0,
        home: || crate::rest::low::home(crate::rest::low::OWN[15].0),
    },
    Moved {
        ida: crate::rest::low::OWN[16].0,
        len: crate::rest::low::OWN[16].1 - crate::rest::low::OWN[16].0,
        home: || crate::rest::low::home(crate::rest::low::OWN[16].0),
    },
    Moved {
        ida: crate::rest::low::OWN[17].0,
        len: crate::rest::low::OWN[17].1 - crate::rest::low::OWN[17].0,
        home: || crate::rest::low::home(crate::rest::low::OWN[17].0),
    },
    Moved {
        ida: crate::rest::low::OWN[18].0,
        len: crate::rest::low::OWN[18].1 - crate::rest::low::OWN[18].0,
        home: || crate::rest::low::home(crate::rest::low::OWN[18].0),
    },
    Moved {
        ida: crate::rest::low::OWN[19].0,
        len: crate::rest::low::OWN[19].1 - crate::rest::low::OWN[19].0,
        home: || crate::rest::low::home(crate::rest::low::OWN[19].0),
    },
    Moved {
        ida: crate::rest::low::OWN[20].0,
        len: crate::rest::low::OWN[20].1 - crate::rest::low::OWN[20].0,
        home: || crate::rest::low::home(crate::rest::low::OWN[20].0),
    },
    Moved {
        ida: crate::rest::low::OWN[21].0,
        len: crate::rest::low::OWN[21].1 - crate::rest::low::OWN[21].0,
        home: || crate::rest::low::home(crate::rest::low::OWN[21].0),
    },
    Moved {
        ida: crate::rest::low::OWN[22].0,
        len: crate::rest::low::OWN[22].1 - crate::rest::low::OWN[22].0,
        home: || crate::rest::low::home(crate::rest::low::OWN[22].0),
    },
    Moved {
        ida: crate::rest::low::OWN[23].0,
        len: crate::rest::low::OWN[23].1 - crate::rest::low::OWN[23].0,
        home: || crate::rest::low::home(crate::rest::low::OWN[23].0),
    },
    Moved {
        ida: crate::rest::low::OWN[24].0,
        len: crate::rest::low::OWN[24].1 - crate::rest::low::OWN[24].0,
        home: || crate::rest::low::home(crate::rest::low::OWN[24].0),
    },
    Moved {
        ida: crate::rest::low::OWN[25].0,
        len: crate::rest::low::OWN[25].1 - crate::rest::low::OWN[25].0,
        home: || crate::rest::low::home(crate::rest::low::OWN[25].0),
    },
    Moved {
        ida: crate::rest::low::OWN[26].0,
        len: crate::rest::low::OWN[26].1 - crate::rest::low::OWN[26].0,
        home: || crate::rest::low::home(crate::rest::low::OWN[26].0),
    },
    Moved {
        ida: crate::rest::low::OWN[27].0,
        len: crate::rest::low::OWN[27].1 - crate::rest::low::OWN[27].0,
        home: || crate::rest::low::home(crate::rest::low::OWN[27].0),
    },
    Moved {
        ida: crate::rest::low::OWN[28].0,
        len: crate::rest::low::OWN[28].1 - crate::rest::low::OWN[28].0,
        home: || crate::rest::low::home(crate::rest::low::OWN[28].0),
    },
    Moved {
        ida: crate::rest::low::OWN[29].0,
        len: crate::rest::low::OWN[29].1 - crate::rest::low::OWN[29].0,
        home: || crate::rest::low::home(crate::rest::low::OWN[29].0),
    },
    Moved {
        ida: crate::rest::low::OWN[30].0,
        len: crate::rest::low::OWN[30].1 - crate::rest::low::OWN[30].0,
        home: || crate::rest::low::home(crate::rest::low::OWN[30].0),
    },
    Moved {
        ida: crate::rest::low::OWN[31].0,
        len: crate::rest::low::OWN[31].1 - crate::rest::low::OWN[31].0,
        home: || crate::rest::low::home(crate::rest::low::OWN[31].0),
    },
    Moved {
        ida: crate::rest::low::OWN[32].0,
        len: crate::rest::low::OWN[32].1 - crate::rest::low::OWN[32].0,
        home: || crate::rest::low::home(crate::rest::low::OWN[32].0),
    },
    Moved {
        ida: crate::rest::low::OWN[33].0,
        len: crate::rest::low::OWN[33].1 - crate::rest::low::OWN[33].0,
        home: || crate::rest::low::home(crate::rest::low::OWN[33].0),
    },
    Moved {
        ida: crate::rest::low::OWN[34].0,
        len: crate::rest::low::OWN[34].1 - crate::rest::low::OWN[34].0,
        home: || crate::rest::low::home(crate::rest::low::OWN[34].0),
    },
    Moved {
        ida: crate::rest::low::OWN[35].0,
        len: crate::rest::low::OWN[35].1 - crate::rest::low::OWN[35].0,
        home: || crate::rest::low::home(crate::rest::low::OWN[35].0),
    },
    Moved {
        ida: crate::rest::low::OWN[36].0,
        len: crate::rest::low::OWN[36].1 - crate::rest::low::OWN[36].0,
        home: || crate::rest::low::home(crate::rest::low::OWN[36].0),
    },
    Moved {
        ida: crate::rest::low::OWN[37].0,
        len: crate::rest::low::OWN[37].1 - crate::rest::low::OWN[37].0,
        home: || crate::rest::low::home(crate::rest::low::OWN[37].0),
    },
    Moved {
        ida: crate::rest::low::OWN[38].0,
        len: crate::rest::low::OWN[38].1 - crate::rest::low::OWN[38].0,
        home: || crate::rest::low::home(crate::rest::low::OWN[38].0),
    },
    // rest: the scattered rest of the image's buffers (`crate::rest`)
    Moved {
        ida: crate::rest::OWN[0].0,
        len: crate::rest::OWN[0].1 - crate::rest::OWN[0].0,
        home: || crate::rest::home(crate::rest::OWN[0].0),
    },
    Moved {
        ida: crate::rest::OWN[1].0,
        len: crate::rest::OWN[1].1 - crate::rest::OWN[1].0,
        home: || crate::rest::home(crate::rest::OWN[1].0),
    },
    Moved {
        ida: crate::rest::OWN[2].0,
        len: crate::rest::OWN[2].1 - crate::rest::OWN[2].0,
        home: || crate::rest::home(crate::rest::OWN[2].0),
    },
    Moved {
        ida: crate::rest::OWN[3].0,
        len: crate::rest::OWN[3].1 - crate::rest::OWN[3].0,
        home: || crate::rest::home(crate::rest::OWN[3].0),
    },
    Moved {
        ida: crate::rest::OWN[4].0,
        len: crate::rest::OWN[4].1 - crate::rest::OWN[4].0,
        home: || crate::rest::home(crate::rest::OWN[4].0),
    },
    Moved {
        ida: crate::rest::OWN[5].0,
        len: crate::rest::OWN[5].1 - crate::rest::OWN[5].0,
        home: || crate::rest::home(crate::rest::OWN[5].0),
    },
    // race: the image's zeroed tail (`race::store::buffers`)
    Moved {
        ida: crate::race::store::buffers::START,
        len: crate::race::store::buffers::LEN,
        home: crate::race::store::buffers::base,
    },
    // race: the race core (`race::store`), the seven ranges around the holes
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
    // the menus' lists, the menus' working variables, the race's settings and results and
    // the damage model (`menu::lists_store`, `menu::state_store`, `race::state_store`,
    // `physics::store`, typed)
    Moved {
        ida: crate::menu::lists_store::START,
        len: crate::menu::lists_store::END - crate::menu::lists_store::START,
        home: crate::menu::lists_store::menu_lists_addr,
    },
    Moved {
        ida: crate::menu::state_store::START,
        len: crate::menu::state_store::END - crate::menu::state_store::START,
        home: crate::menu::state_store::menu_state_addr,
    },
    Moved {
        ida: crate::race::state_store::START,
        len: crate::race::state_store::END - crate::race::state_store::START,
        home: crate::race::state_store::race_state_addr,
    },
    Moved {
        ida: crate::physics::store::START,
        len: crate::physics::store::END - crate::physics::store::START,
        home: crate::physics::store::damage_addr,
    },
    // marks: the whole of 0x764F40 .. 0x93CCC8 (`marks::store`)
    Moved {
        ida: crate::marks::store::START,
        len: crate::marks::store::LEN,
        home: || crate::marks::store::STORE.addr(),
    },
];

/// Where the variable at IDA address `ida` lives now: its new home if it moved, else its
/// place in the image.
pub fn place(ida: usize) -> usize {
    // `game_addr` answers a moved variable's home.
    game_addr(rva(ida))
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

/// Every moved range's bytes, for a test that writes the game's variables and has to put
/// them back (`restore`); hold `store::test_lock` around both.
#[cfg(test)]
pub fn snapshot() -> Vec<(usize, Vec<u8>)> {
    sorted()
        .iter()
        .map(|&(start, end, home)| {
            let bytes = unsafe { std::slice::from_raw_parts(home as *const u8, end - start) };
            (home, bytes.to_vec())
        })
        .collect()
}

/// The bytes `snapshot` took, put back.
#[cfg(test)]
pub fn restore(snap: &[(usize, Vec<u8>)]) {
    for (home, bytes) in snap {
        unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), *home as *mut u8, bytes.len()) };
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
/// profile ramps (`marks::store::from_ida`).
const RAW_KEPT: [usize; 2] = [0x5184F4, 0x764F40];

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
            Some((home, _)) => unsafe {
                let p = home as *mut u32;
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
            assert_eq!(game_addr(rva(m.ida + m.len - 1)), home + m.len - 1);
            if !MOVED.iter().any(|o| o.ida == m.ida + m.len) {
                assert_eq!(home_of_rva(rva(m.ida + m.len)), None, "{:#x}", m.ida);
            }
        }
        // zlib's own variables: nothing reads them, not moved (`crate::rest`).
        assert_eq!(home_of_rva(rva(0xA8C18C)), None);
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
