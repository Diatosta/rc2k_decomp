//! Phase 5, the image gone (`notes/conversion.md`): the game's variables that have left
//! RAL.EXE's image for memory Rust owns, and the one place that knows where each went.
//!
//! Code that names a moved variable asks its owner (`logfile::write_line::state`,
//! `strings::units::units`, ...), never this file. Only the game's address tables still
//! name variables by IDA address: the settings table (`var\ral.cfg`'s key -> the address of
//! its variable, `settings::read_settings`) and copy lists such as `race::replay`'s
//! `HEADER_MOVES`. They go through `place` (an IDA address), which answers a moved
//! variable's new home and anything else's place in the image. The game's own pointers (the
//! image's relocations) are pointed at the new homes once at start (`point_into_homes`), so
//! a pointer read from the game's data is a real address. `framework::game_addr` does not
//! consult this: a stale
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
        home: || crate::hud::store::STORE.addr(),
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
        ida: crate::audio::store::state::START,
        len: crate::audio::store::state::LEN,
        home: crate::audio::store::state::base,
    },
    Moved {
        ida: crate::audio::store::samples::START,
        len: crate::audio::store::samples::LEN,
        home: crate::audio::store::samples::base,
    },
    Moved {
        ida: crate::audio::store::queue::START,
        len: crate::audio::store::queue::LEN,
        home: crate::audio::store::queue::base,
    },
    Moved {
        ida: 0x6D1DD4, // g_eaxEnvironment
        len: 4,
        home: || crate::audio::store::ENVIRONMENT.addr(),
    },
    // sound: the race sounds' clock, and the spot picks .. CD audio block (`sound::store`)
    Moved {
        ida: crate::sound::store::wheels::START,
        len: crate::sound::store::wheels::LEN,
        home: crate::sound::store::wheels::base,
    },
    Moved {
        ida: crate::sound::store::objects::START,
        len: crate::sound::store::objects::LEN,
        home: crate::sound::store::objects::base,
    },
    Moved {
        ida: crate::sound::store::clock::START,
        len: crate::sound::store::clock::LEN,
        home: crate::sound::store::clock::base,
    },
    Moved {
        ida: crate::sound::store::surface::START,
        len: crate::sound::store::surface::LEN,
        home: crate::sound::store::surface::base,
    },
    Moved {
        ida: crate::sound::store::stage::START,
        len: crate::sound::store::stage::LEN,
        home: crate::sound::store::stage::base,
    },
    Moved {
        ida: crate::sound::store::effects::START,
        len: crate::sound::store::effects::LEN,
        home: crate::sound::store::effects::base,
    },
    // input: the layout, devices, keyboard and mouse blocks (`input::store`)
    Moved {
        ida: crate::input::store::layout::START,
        len: crate::input::store::layout::LEN,
        home: crate::input::store::layout::base,
    },
    Moved {
        ida: crate::input::store::devices::START,
        len: crate::input::store::devices::LEN,
        home: crate::input::store::devices::base,
    },
    Moved {
        ida: crate::input::store::keyboard::START,
        len: crate::input::store::keyboard::LEN,
        home: crate::input::store::keyboard::base,
    },
    Moved {
        ida: crate::input::store::mouse::START,
        len: crate::input::store::mouse::LEN,
        home: crate::input::store::mouse::base,
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
    // physics: the car lists, race settings and damage model (`physics::store`)
    Moved {
        ida: crate::physics::store::vars::START,
        len: crate::physics::store::vars::LEN,
        home: crate::physics::store::vars::base,
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

/// Points the game's pointers (`places`: the image's relocations, by RVA) at the new homes
/// of the variables they name, as the image's relocation pointed them at the image: a
/// pointer of the image to a moved variable is changed to its home; a pointer that moved
/// with its block (whose first values are the image's bytes before relocation, IDA
/// addresses) is made the address of its target, wherever that lives now. The ranges of
/// `RAW_KEPT` stay as they are. Run once, while the image is built (`base` its address;
/// `framework::BASE` is not set yet), after its relocation and before its pages are
/// protected. Returns how many pointers it changed.
pub unsafe fn point_into_homes(base: usize, places: &[usize]) -> usize {
    let mut changed = 0;
    for &at in places {
        let ida = at + IMAGE_BASE;
        unsafe {
            match moved_home(ida) {
                Some((_, block)) if RAW_KEPT.contains(&block) => {}
                Some((home, _)) => {
                    let p = home as *mut u32;
                    let target = p.read_unaligned() as usize;
                    let now = match moved_home(target) {
                        Some((h, _)) => h,
                        None => base + rva(target),
                    };
                    p.write_unaligned(now as u32);
                    changed += 1;
                }
                None => {
                    let p = (base + at) as *mut u32;
                    let target = (p.read_unaligned() as usize)
                        .wrapping_sub(base)
                        .wrapping_add(IMAGE_BASE);
                    if let Some((h, _)) = moved_home(target) {
                        p.write_unaligned(h as u32);
                        changed += 1;
                    }
                }
            }
        }
    }
    changed
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
        assert_eq!(home_of_rva(rva(0x4E0000)), None);
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

    #[test]
    fn image_bytes_reads_the_image() {
        assert_eq!(image_bytes(0x4E0000, 6), b"$VER: ");
        // bss reads as zeros
        assert_eq!(image_bytes(0x93E430, 4), [0; 4]);
    }
}
