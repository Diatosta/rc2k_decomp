//! The program: the game's start-up as the list of calls it is (`run`, `start`), its ways out
//! (`finish`, `leave_on_close`) and the report of a fault, with the place and the stack's return
//! addresses.
//!
//! A refusal (`BadInput`) ends the program with its reason; a fatal error is reported by
//! `fatal::report` and ends it. What the window would tell the main loop is set by `stand_in`.

use crate::d3d::owner::D3d;
use crate::errors::Refused;
use crate::gfx::owner::Gfx;
use crate::install::outside::Game as Outside;
use crate::logfile::program_log::{LOG_PATH, log};
use crate::menu::owner::Menus;
use crate::owners::Owners;
use crate::system::owner::System;
use crate::threading::thread_slot::SlotOwners;
use std::path::Path;
use std::sync::atomic::Ordering;

// ---- calls by address that are done or answered by SDL ----

/// The 1999 addresses of the game's empty functions (a lone `retn`, and one that saves and
/// restores the registers in a draw list): a call to one is done.
const LONE_RETN: [usize; 119] = [
    0x40AC76, 0x40CB09, 0x40CF32, 0x40D27E, 0x41E2F5, 0x428D74, 0x428DB2, 0x429823, 0x429B17,
    0x429B18, 0x436832, 0x436B97, 0x436B98, 0x439329, 0x439593, 0x43DFB0, 0x44326C, 0x443471,
    0x44347E, 0x443DB9, 0x44403F, 0x444F54, 0x445AF0, 0x4467C6, 0x446993, 0x44DEA9, 0x44E1C0,
    0x44E76A, 0x44E776, 0x44E777, 0x44EB65, 0x44FF1A, 0x450222, 0x450A4A, 0x454F61, 0x454FEF,
    0x4554C9, 0x456091, 0x457F2F, 0x457F82, 0x45836E, 0x45840D, 0x45A854, 0x45BBFA, 0x45BECC,
    0x45C550, 0x45CCD3, 0x45D254, 0x465A3D, 0x465A3E, 0x465AB5, 0x465AB6, 0x465D78, 0x467BD5,
    0x46D0F7, 0x47019D, 0x47019E, 0x471391, 0x4713DD, 0x471C80, 0x471C81, 0x475C0B, 0x4907B8,
    0x4907B9, 0x4A94E9, 0x4A94EA, 0x4A94EB, 0x4A9678, 0x4B695B, 0x4B7ED9, 0x4B9789, 0x4BA849,
    0x4BA84A, 0x4BA84B, 0x4BAC86, 0x4BD018, 0x4BD121, 0x4BEF98, 0x4BEFD1, 0x4C072A, 0x4C07DA,
    0x4C08B9, 0x4C08BA, 0x4C1162, 0x4C16F8, 0x4C21EB, 0x4C25F5, 0x4C2DDA, 0x4C3901, 0x4C39D4,
    0x4C4E14, 0x4C4E15, 0x4C4E16, 0x4C4E17, 0x4C4E18, 0x4C4E19, 0x4C4E1A, 0x4C4E1B, 0x4C4E2E,
    0x4C4EA0, 0x4C4EA1, 0x4C4FFA, 0x4C65BF, 0x4C65C0, 0x4C65C1, 0x4C74D9, 0x4C8167, 0x4C9699,
    0x4C9AF5, 0x4CAE5B, 0x4CB795, 0x4D0BA7, 0x4D0BA8, 0x4D0EB3, 0x4D1710, 0x4D2C9A, 0x4D5C40,
    0x4D6AE4, 0x48558A,
];

// ---- what the faults catch ----

#[cfg(windows)]
/// The event that starts the fault reporter's time limit (`arm_time_limit`); set once the
/// watchdog thread runs.
static TIME_LIMIT_EVENT: std::sync::OnceLock<crate::platform::ffi::win32::Event> =
    std::sync::OnceLock::new();

/// How long a fault's report may take before the process is ended without it (a fault
/// inside the allocator or the loader leaves their locks held, and the report needs both).
const REPORT_TIME_LIMIT_SECONDS: u32 = 2;

#[cfg(unix)]
/// The fault reporter's time limit started: SIGALRM, whose default action ends the
/// process, comes after `REPORT_TIME_LIMIT_SECONDS`.
fn arm_time_limit() {
    crate::platform::ffi::unix::alarm(REPORT_TIME_LIMIT_SECONDS);
}

#[cfg(windows)]
/// The fault reporter's time limit started: the watchdog thread (`start_watchdog`) ends
/// the process after `REPORT_TIME_LIMIT_SECONDS`.
fn arm_time_limit() {
    if let Some(event) = TIME_LIMIT_EVENT.get() {
        event.set();
    }
}

#[cfg(windows)]
/// The watchdog of the fault reporter's time limit, started before any fault: a thread made
/// inside the handler could wait for the loader lock the faulting thread holds.
fn start_watchdog() {
    let Some(event) = crate::platform::ffi::win32::create_event() else {
        log("standalone: the fault reporter has no time limit (no event)");
        return;
    };
    let started = std::thread::Builder::new()
        .name("fault watchdog".into())
        .spawn(move || {
            event.wait_forever();
            std::thread::sleep(std::time::Duration::from_secs(u64::from(
                REPORT_TIME_LIMIT_SECONDS,
            )));
            crate::platform::os::terminate(1)
        });
    if started.is_err() {
        log("standalone: the fault reporter has no time limit (no thread)");
    } else {
        let _ = TIME_LIMIT_EVENT.set(event);
    }
}

// ---- the fault report, in the handler: nothing allocated, no lock of ours taken ----
//
// Every call between a fault and the process's end (`report_fault` and below), as the
// systems' handlers (`ffi::unix::on_signal`, `ffi::win32::on_exception`) make them:
// - `arm_time_limit`: `alarm(2)` (Unix), or setting an event made before any fault (Windows);
// - the module lookups: `dladdr` (Unix), `GetModuleHandleExW` / `GetModuleFileNameW` into a
//   stack buffer (Windows); they may take the loader's lock, not async-signal-safe: a fault
//   that holds it hangs the report, and the time limit then ends the process without it;
// - `os::own_stack_word`: reads of the faulting thread's stack inside its bounds (Unix: one
//   `process_vm_readv` per word on Linux, the thread's record on macOS; Windows:
//   `GetCurrentThreadStackLimits`);
// - the text: `core::fmt` into `FaultLine`'s stack buffer (no allocation, no lock), written
//   with `write(2)` / `WriteFile` on the log opened before any fault (`FAULT_LOG`);
// - `end`: the PGO build's `__llvm_profile_write_file` (that build only: it allocates and
//   does stdio; the time limit covers it), then `_exit` / `TerminateProcess`.
// The handler always ends the process, so nothing it leaves half done is seen again.

/// What a fault's report is written with, made before any fault: the log opened for
/// appending, and the file name of the program's own module (its units as the system gives
/// them), or None when it has none.
struct FaultLog {
    file: std::fs::File,
    own_module: Option<Box<[NameUnit]>>,
}

/// A unit of a module's path as the system gives it.
#[cfg(unix)]
type NameUnit = u8;
#[cfg(windows)]
type NameUnit = u16;

static FAULT_LOG: std::sync::OnceLock<FaultLog> = std::sync::OnceLock::new();

/// `f` given the file name of the module a host address is in (its path's last part) and
/// the module's base; None when it is in none.
#[cfg(unix)]
fn with_module<T>(address: usize, f: impl FnOnce(&[NameUnit], usize) -> T) -> Option<T> {
    crate::platform::ffi::unix::with_module_of_address(address, |path, base| {
        f(path.rsplit(|&c| c == b'/').next().unwrap_or(path), base)
    })
}

#[cfg(windows)]
fn with_module<T>(address: usize, f: impl FnOnce(&[NameUnit], usize) -> T) -> Option<T> {
    let separator = |&c: &u16| c == u16::from(b'\\') || c == u16::from(b'/');
    crate::platform::ffi::win32::with_module_of_address_wide(address, |path, base| {
        f(path.rsplit(separator).next().unwrap_or(path), base)
    })
}

/// The log opened for the fault report, and the program's own module named: before any
/// fault, once `LOG_PATH` is set.
fn open_fault_log() {
    let Some(file) = LOG_PATH.get().and_then(|path| {
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .ok()
    }) else {
        return;
    };
    let own_module = with_module(
        (stack_returns as *const ()).expose_provenance(),
        |name, _| Box::from(name),
    );
    let _ = FAULT_LOG.set(FaultLog { file, own_module });
}

/// One line of the fault report, gathered in a buffer on the stack and written to the log
/// as it fills and at its end (no allocation, no lock: a handler's).
struct FaultLine<'a> {
    file: &'a std::fs::File,
    buffer: [u8; 1024],
    len: usize,
}

impl<'a> FaultLine<'a> {
    fn new(file: &'a std::fs::File) -> Self {
        FaultLine {
            file,
            buffer: [0; 1024],
            len: 0,
        }
    }

    fn flush(&mut self) {
        use std::io::Write;
        // A failed write loses the report's text only; the process ends either way.
        let mut file = self.file;
        let _ = file.write_all(&self.buffer[..self.len]);
        self.len = 0;
    }

    /// The line ended ("\n") and written.
    fn end(mut self) {
        let _ = std::fmt::Write::write_char(&mut self, '\n');
        self.flush();
    }

    /// A module's file name, as `String::from_utf8_lossy` / `from_utf16_lossy` show it.
    fn name(&mut self, name: &[NameUnit]) {
        use std::fmt::Write;
        #[cfg(unix)]
        for chunk in name.utf8_chunks() {
            let _ = self.write_str(chunk.valid());
            if !chunk.invalid().is_empty() {
                let _ = self.write_char(char::REPLACEMENT_CHARACTER);
            }
        }
        #[cfg(windows)]
        for c in char::decode_utf16(name.iter().copied()) {
            let _ = self.write_char(c.unwrap_or(char::REPLACEMENT_CHARACTER));
        }
    }

    /// The module a host address is in, and the address relative to its base
    /// ("rc2k.exe+0x1234"); false (nothing written) when it is in none.
    fn in_module(&mut self, address: usize) -> bool {
        with_module(address, |file, base| {
            self.name(file);
            let _ = std::fmt::Write::write_fmt(self, format_args!("+{:#x}", address - base));
        })
        .is_some()
    }
}

impl std::fmt::Write for FaultLine<'_> {
    fn write_str(&mut self, text: &str) -> std::fmt::Result {
        for &byte in text.as_bytes() {
            if self.len == self.buffer.len() {
                self.flush();
            }
            self.buffer[self.len] = byte;
            self.len += 1;
        }
        Ok(())
    }
}

/// The return addresses into this program found on the stack from `esp` (at most 24), for
/// `llvm-symbolizer` with the program's PDB (or its symbols on Unix), written to `line`
/// space-separated: each a word whose module's file name begins with the program's own.
fn stack_returns(line: &mut FaultLine, own_module: Option<&[NameUnit]>, esp: usize) {
    let Some(own) = own_module else {
        return;
    };
    let mut found = 0;
    // At most 2 KB of stack, less where the stack ends sooner.
    const WORD: usize = size_of::<usize>();
    for k in 0..2048 / WORD {
        // The walk ends at the stack's end (`os::own_stack_word`).
        let Some(value) = esp
            .checked_add(WORD * k)
            .and_then(crate::platform::os::own_stack_word)
        else {
            break;
        };
        let ours = with_module(value, |file, base| {
            if !file.starts_with(own) {
                return false;
            }
            if found > 0 {
                let _ = std::fmt::Write::write_char(line, ' ');
            }
            line.name(file);
            let _ = std::fmt::Write::write_fmt(line, format_args!("+{:#x}", value - base));
            true
        });
        if ours == Some(true) {
            found += 1;
            if found == 24 {
                break;
            }
        }
    }
}

/// The game closed, in a race or on a menu: the program is ended as the game's start ends it
/// (the shutdown asked for, "Leaving main loop..." logged). The data is saved first, once the
/// front end has its tables: before that a save would write blanks.
pub fn leave_on_close(o: Owners<'static>) -> ! {
    let saves = o.saves;
    let tables = &saves.best_times;
    // A render made the program's own way of drawing (its size, its clock): it saves nothing.
    if !crate::video::active()
        && [tables.allocated.get(), tables.loaded.get()] == [1, 1]
        && let Err(e) = crate::saves::save_game_data::save_in_game(o, saves)
    {
        log(&format!(
            "standalone: the game's data was not saved on closing: {e:?}"
        ));
    }
    if let Err(e) = crate::threading::request_quit::request_quit_in(SlotOwners::of(o)) {
        log(&format!(
            "standalone: the shutdown thread did not start ({:#x})",
            e.code
        ));
    }
    log("system::main_loop: leaving (the game was closed)");
    finish("the game was closed", 0)
}

/// A function of the game's not ported, reached from Rust (`from`): the program
/// ends, naming it: nothing not ported passes unnoticed.
pub fn not_ported(at: usize, from: &str) -> ! {
    finish(&format!("not ported: {at:#x}, called from {from}"), 1)
}

pub fn finish(why: &str, code: u32) -> ! {
    finish_saying(why, why, code)
}

/// `finish` with a text of its own for the player (the log keeps `why`).
pub fn finish_saying(why: &str, shown: &str, code: u32) -> ! {
    log(&format!("standalone: ending: {why}"));
    // An error ends through the dialog, so that the player sees why (the log has it too).
    if code != 0 {
        crate::error_dialog::fatal(shown);
    }
    end(code)
}

/// The process ended with `code`, the PGO build's counters written first.
fn end(code: u32) -> ! {
    // A video's file is closed first.
    crate::video::finish();
    #[cfg(rc2k_pgo)]
    {
        crate::platform::ffi::pgo::write_profile();
    }
    // Not `std::process::exit`: this may be an exception's handler, where the libraries'
    // leave-taking can hang.
    crate::platform::os::terminate(code)
}

/// The platform's functions `start` calls that SDL stands for: (the function, what it is).
const SDL_DOES: [(usize, &str); 5] = [
    (0x4103DE, "system::open_window"),
    (0x404981, "gfx::probe_exclusive_mode"),
    (0x404CD6, "gfx::release_screens"),
    (0x40FA8B, "sound::close_cd_audio"),
    (0x410509, "system::drain_messages"),
];

/// For calls by address of a function that needs nothing done here (`menu::table_calls`): the ones
/// SDL does and the lone `retn`s. `at` is the function's 1999 address.
pub fn answers(at: usize) -> bool {
    SDL_DOES.iter().any(|w| w.0 == at) || LONE_RETN.contains(&at)
}

/// A fault reported and the program ended: `host` the faulting instruction, `data` the
/// address it read or wrote (`write`), `sp` the thread's stack pointer (host addresses).
/// Shared by the systems' handlers; allocates nothing and takes no lock of ours (the calls
/// it makes are listed above `FaultLog`).
pub(crate) fn report_fault(host: usize, data: usize, write: bool, sp: usize) -> ! {
    use std::fmt::Write;
    arm_time_limit();
    let verb = if write { "writing" } else { "reading" };
    if let Some(log) = FAULT_LOG.get() {
        let mut line = FaultLine::new(&log.file);
        let _ = line.write_str("standalone: the fault is in ");
        if !line.in_module(host) {
            let _ = line.write_str("no module");
        }
        let _ = line.write_str("; return addresses on the stack: ");
        stack_returns(&mut line, log.own_module.as_deref(), sp);
        line.end();
        let mut line = FaultLine::new(&log.file);
        let _ = write!(
            line,
            "standalone: ending: access violation at {host:#x}, {verb} address {data:#x}"
        );
        line.end();
    }
    end(1)
}

// ---- `start` ----

/// Logs that `start` calls the Rust function `name`.
fn rust(name: &str) {
    log(&format!("standalone: start calls {name}"));
}

/// A refusal in `start`: a refused input ends the program, a fatal error is
/// raised.
fn refused(name: &str, e: Refused) {
    crate::fatal::raise::refused(name, e);
}

fn stand_in(what: &str, variable: &crate::memory::session::Dword, value: u32) {
    log(&format!("standalone: stand-in: {what} (= {value})"));
    variable.set(value);
}

/// The game's `start`, call for call, in its steps. The checks it makes after calls of the
/// platform (already running, no screen mode) cannot fail here.
///
/// Once, on the main thread: the start-up functions it calls write the game's state before
/// any thread of the game's runs.
fn start(o: Owners<'static>) {
    // The fatal-error reporter finds the owners here from any thread.
    crate::threading::thread_slot::publish(SlotOwners::of(o));
    crate::platform::direct3d::set_handles(o.gfx);
    set_up_fixed_objects(o.gfx, o.menus);
    seed_clock_and_random(o.system, &o.race.shared.random);
    check_machine(o.system);
    read_settings_and_start_log(o);
    make_save_folders();
    read_command_line_and_install_state(o);
    load_language_and_game_data(o);
    probe_display(o.gfx, &o.gfx.d3d, o.system);
    start_workers(o);
    choose_screen_modes(o.gfx, &o.gfx.d3d, o.system);
    stand_in_window_state(o.system);
    run_main_loop(o);
    shut_down(o);
}

/// The objects that point at each other: the players' records, font 56's palette (its place in
/// `gfx::glyph_store`; `gfx::build_fonts` hangs the rest) and the surface's pixel format (the
/// format record of `session::SCREEN`, named by its handle; the platform sets it again at every
/// lock).
fn set_up_fixed_objects(gfx: &'static Gfx, menus: &'static Menus) {
    crate::menu::players_store::point_players(menus);
    gfx.fonts.typefaces.objects[56]
        .palette
        .set(crate::gfx::glyph_store::SHEET_PLACE);
    gfx.drawing
        .draw_surface
        .format
        .set(crate::gfx::pixel_format::SCREEN_FORMAT);
}

/// The start time and the random generator's seed (from that time alone).
fn seed_clock_and_random(system: &'static System, random: &crate::math::next_random::Randoms) {
    rust("system::keep_start_time");
    crate::system::keep_start_time::keep_start_time(system);
    rust("math::seed_random");
    crate::math::seed_random::seed_random(random, system);
    // One more step of the generator, with nothing stirred in.
    rust("math::stir_random");
    crate::math::stir_random::stir_random(random, 0);
}

/// The processor's test and the system's version.
fn check_machine(system: &'static System) {
    rust("system::test_processor");
    crate::system::test_processor::test_processor(&crate::system::test_processor::Real, system);
    rust("system::read_windows_version");
    crate::system::read_windows_version::read_windows_version(system);
}

/// `var\ral.cfg`'s settings, the feature switches and the game's log.
fn read_settings_and_start_log(o: Owners<'static>) {
    rust("settings::read_settings");
    if let Err(why) = crate::settings::read_settings::read_settings(o) {
        finish(&format!("settings::read_settings refused: {}", why.0), 1);
    }
    rust("logfile::start_log");
    if let Err(why) = crate::logfile::start_log::start_log(o.system) {
        refused("logfile::start_log", why.into());
    }
}

/// The folders the game saves into but never makes (`install::make_folders` makes them only
/// from the installer's script): made here when missing, in the game's folder, as the game's
/// path templates name them; not in the download folder, where the program is the installer.
fn make_save_folders() {
    let folders: &[&str] = if crate::install::portable::is_source_folder() {
        &[]
    } else {
        &[
            "savedata\\savegame",
            "savedata\\carsetup",
            "savedata\\snapshot",
            "savedata\\replay",
        ]
    };
    for &folder in folders {
        let at = crate::files::host_path(folder.as_bytes());
        if !at.is_dir() {
            match std::fs::create_dir_all(&at) {
                Ok(()) => log(&format!("standalone: made the folder {folder}")),
                Err(e) => log(&format!(
                    "standalone: the folder {folder} cannot be made: {e}"
                )),
            }
        }
    }
}

/// The FPU check, the command line and what the installation left: the disc check, the
/// start check, the languages and the install record.
fn read_command_line_and_install_state(o: Owners<'static>) {
    let system = o.system;
    // `system::test_processor` (called above) has set it from `cpuid`.
    if system.session.host.has_fpu.get() != 1 {
        finish("fatal error 0x900010FF, FPU is required", 1);
    }
    rust("system::read_command_line");
    if let Err(why) = crate::system::read_command_line::read_own_command_line(o) {
        refused("system::read_command_line", why.into());
    }
    let mut outside = Outside {
        install: o.install,
        system,
    };
    rust("install::check_clio_disc");
    if let Err(e) = crate::install::check_clio_disc::check_clio_disc(o.install, &mut outside) {
        refused("install::check_clio_disc", e);
    }
    rust("install::check_at_start");
    let no_audio = &o.audio.state.no_audio;
    if let Err(e) = crate::install::check_at_start::check_at_start(
        o.install,
        &o.menus.text_buffers,
        system,
        &mut outside,
        no_audio,
    ) {
        refused("install::check_at_start", e);
    }
    rust("install::read_languages");
    if let Err(e) = crate::install::read_languages::load_languages(
        o.install,
        o.system,
        &o.install.names.lang_cfg.text(),
    )
    .map_err(Refused::from)
    {
        refused("install::read_languages", e);
    }
    rust("install::read_record");
    if let Err(e) =
        crate::install::read_record::load_record(o.system, &o.install.texts.record_path.text())
            .map_err(Refused::from)
    {
        refused("install::read_record", e);
    }
}

/// The language's texts (`strings::load_language`) and the game's data files.
fn load_language_and_game_data(o: Owners<'static>) {
    rust("strings::load_language");
    let compile = &mut || crate::strings::compile_lexicon::compile_lexicon(o.strings, o.system);
    if let Err(e) = crate::strings::load_language::load_language(o, compile) {
        refused("strings::load_language", e);
    }
    rust("system::load_game_data");
    if let Err(e) = crate::system::load_game_data::load_in_game(o) {
        refused("system::load_game_data", e);
    }
}

/// The 3D cards and the display modes the screen can take. `system::open_window` and
/// `gfx::probe_exclusive_mode` are SDL's (`SDL_DOES`).
fn probe_display(gfx: &'static Gfx, d3d: &'static D3d, system: &'static System) {
    rust("gfx::list_primary_modes");
    if let Err(e) = crate::gfx::list_primary_modes::list_primary_modes(&gfx.display, system, d3d) {
        refused("gfx::list_primary_modes", e);
    }
}

/// The network's set-up, the game's threads and the audio thread's commands.
fn start_workers(o: Owners<'static>) {
    rust("net::set_up");
    crate::net::set_up::set_up(o.net, o.strings);
    rust("threading::start_workers");
    if let Err(e) = crate::threading::thread_slot::start_workers(SlotOwners::of(o)) {
        refused("threading::start_workers", e.into());
    }
    rust("sound::wait_for_commands");
    crate::sound::wait_for_commands::wait_in_game(o);
    // The original slept a second more here, for the sound card of its day; SDL's sound is
    // ready once the audio thread has taken its commands.
}

/// The screen modes, the picture buffer and the pixel converters. `sound::open_cd_audio` is not
/// called: there is no CD audio here.
fn choose_screen_modes(gfx: &'static Gfx, d3d: &'static D3d, system: &'static System) {
    rust("gfx::choose_screen_modes");
    if let Err(e) = crate::platform::choose_screen_modes(gfx, d3d, system) {
        refused("gfx::choose_screen_modes", e.into());
    }
    rust("gfx::allocate_picture_buffer");
    if let Err(e) = crate::gfx::allocate_picture_buffer::allocate_picture_buffer(gfx) {
        refused("gfx::allocate_picture_buffer", e);
    }
    rust("gfx::set_component_format");
    if let Err(e) = crate::gfx::set_component_format::set_component_format(&gfx.pixel.formats) {
        refused("gfx::set_component_format", e);
    }
    rust("gfx::fill_thirds_table");
    crate::gfx::fill_thirds_table::fill_thirds_table(gfx);
}

/// What the window's procedure and the key thread tell the main loop (the films play from
/// `platform/film.rs`).
fn stand_in_window_state(system: &'static System) {
    let flags = &system.session.flags;
    stand_in("the window is the active one", &flags.window_active, 1);
    stand_in(
        "the window has the focus (until SDL says otherwise)",
        &flags.window_focused,
        1,
    );
    stand_in("the key thread runs", &system.session.key_clock.ticks, 1);
}

fn run_main_loop(o: Owners<'static>) {
    rust("system::main_loop");
    if let Err(e) = crate::system::main_loop::main_loop_in_game(o) {
        refused("system::main_loop", e);
    }
}

/// What follows the main loop: the network, the picture buffer, the priority class, the installer's
/// after-exit script, the error mode and the fatal report. `gfx::release_screens`,
/// `sound::close_cd_audio` and `system::drain_messages` are SDL's (`SDL_DOES`).
fn shut_down(o: Owners<'static>) {
    rust("net::shut_down");
    if let Err(e) =
        crate::net::shut_down::shut_down(o, &crate::race::viewer_screen_store::NET_TEXTS)
    {
        refused("net::shut_down", e);
    }
    rust("gfx::free_picture_buffer");
    if let Err(why) = crate::gfx::free_picture_buffer::free_picture_buffer(o.gfx) {
        finish(&format!("gfx::free_picture_buffer refused: {}", why.0), 1);
    }
    rust("install::run_after_exit");
    if let Err(e) = crate::install::run_after_exit::run_after_exit(o.install, o.system) {
        refused("install::run_after_exit", e);
    }
    rust("fatal::show_report_at_exit");
    crate::fatal::show_report_at_exit::show_report_at_exit();
}

/// What the command line asks of the program before it starts, if anything: `--help` prints the
/// list of switches (to the terminal it was started from, else in a message box) and ends; a
/// `--` switch the program does not know, or a value of the program's own switches it cannot
/// take, is refused the same way, before anything loads. The code the program ends with, or
/// None to go on.
pub fn answer_command_line(args: &[String]) -> Option<i32> {
    use crate::platform::os::{Icon, message_box, print_to_terminal};
    let (text, code, icon) = if rc2k_core::cli::wants_help(args) {
        (
            rc2k_core::cli::help_text(env!("CARGO_PKG_VERSION")),
            0,
            Icon::Information,
        )
    } else {
        let refusal = rc2k_core::cli::unknown(args).or_else(|| program_switches_refused(args));
        (refusal? + "\n", 2, Icon::Warning)
    };
    if !print_to_terminal(&text) {
        message_box(icon, b"Rally Championship", text.as_bytes());
    }
    Some(code)
}

/// Why the switches that say how the program runs (`--renderer`) cannot be taken, if they
/// cannot.
fn program_switches_refused(args: &[String]) -> Option<String> {
    let switches = match rc2k_core::system::program_switches::Switches::from_args(args) {
        Ok(switches) => switches,
        Err(why) => return Some(why),
    };
    if switches.render_replay.is_some() && switches.out.is_none() {
        return Some("--render-replay needs --out <video file>".into());
    }
    if switches.render_replay.is_none()
        && (switches.out.is_some()
            || switches.size.is_some()
            || switches.fps.is_some()
            || switches.quality.is_some()
            || switches.no_sound
            || switches.from_cs.is_some()
            || switches.to_cs.is_some())
    {
        return Some(
            "--out, --size, --fps, --quality, --no-sound, --from and --to go with --render-replay"
                .into(),
        );
    }
    let renderer = switches.renderer?;
    match crate::render::choice::parse(&renderer) {
        Some(_) => None,
        None => Some(format!(
            "--renderer {renderer}: not a renderer; the choices are {}",
            crate::render::choice::names()
        )),
    }
}

/// The program ends at its start: another copy runs in this folder, said where the player
/// reads it (the terminal, else a message box; no box in the test builds, which run without a
/// screen to answer it).
fn refuse_second_copy(why: &str) -> ! {
    let text = format!("{why}\n");
    if !crate::platform::os::print_to_terminal(&text) {
        #[cfg(not(feature = "remote"))]
        crate::platform::os::message_box(
            crate::platform::os::Icon::Warning,
            b"Rally Championship",
            text.as_bytes(),
        );
    }
    std::process::exit(3)
}

/// Starts the platform and goes through `start`. The current
/// directory has to be the game's. Does not come back.
/// `frames`, if not 0: the program ends itself after that many frames were shown.
pub fn run(log_file: &Path, frames: u32) -> ! {
    crate::memory::session::set_game_thread();
    crate::platform::FRAMES_TO_SHOW.store(frames, Ordering::SeqCst);
    crate::platform::timing::start_threads();
    crate::platform::input::SdlBackend::start();
    // Before any path is read: a program in a macOS bundle runs in the bundle's folder.
    #[cfg(target_os = "macos")]
    let bundle_folder = crate::platform::os::enter_bundle_folder();
    // Absolute: the installer changes the current folder (`install::make_folders`).
    let switches = rc2k_core::system::program_switches::Switches::from_args(
        &crate::platform::os::program_args()[1..],
    )
    .unwrap_or_default();
    if let Err(why) = crate::video::begin(&switches) {
        refuse_second_copy(&why);
    }
    // Before the folder's lock and log: a folder with no game data is replaced by the one the
    // player picks.
    crate::first_run::find_game_data(switches.game_data.as_deref());
    // Held until the program ends. A refusal comes before the log is begun: the log of the
    // copy that runs is not emptied.
    let claimed = match switches.instance {
        Some(number) => crate::single_copy::claim_instance(number),
        None => crate::single_copy::claim(switches.allow_another),
    };
    let copy = match claimed {
        Ok(claim) => claim,
        Err(why) => refuse_second_copy(&why),
    };
    if switches.instance.is_none() {
        crate::single_copy::note_shared_copy(copy.number);
    }
    let log_file = crate::single_copy::log_for(log_file, copy.number);
    let log_file = &std::path::absolute(&log_file).unwrap_or(log_file);
    crate::logfile::program_log::start(log_file);
    if switches.instance.is_some_and(|n| n > 1) {
        log(&format!(
            "standalone: instance {} of this folder: its own var and savedata are in instances/{}/",
            copy.number, copy.number
        ));
    } else if copy.number > 1 {
        log(&format!(
            "standalone: copy {} in this folder (--allow-another): the options, saved games and the game's log are shared with the others",
            copy.number
        ));
    }
    crate::gfx::draw2d::enable_from_environment();
    #[cfg(target_os = "macos")]
    match bundle_folder {
        Ok(Some(folder)) => log(&format!(
            "standalone: started from a bundle: running in {}",
            folder.display()
        )),
        Ok(None) => {}
        Err(e) => {
            log(&format!("standalone: the bundle's folder: {e}"));
            crate::fatal::raise::end(
                "standalone::run",
                crate::errors::BadInput(
                    "the folder holding the program's bundle cannot be entered",
                ),
            )
        }
    }
    // A panic ends the program at once (it cannot unwind out of an `extern "C"` function):
    // say where first.
    std::panic::set_hook(Box::new(|info| {
        let thread = std::thread::current();
        let name = thread.name().unwrap_or("unnamed");
        let trace = std::backtrace::Backtrace::force_capture();
        log(&format!(
            "standalone: panic on thread {name} ({:?}): {info}\n{trace}",
            thread.id()
        ));
    }));
    open_fault_log();
    #[cfg(unix)]
    crate::platform::ffi::unix::install_fault_reporter();
    #[cfg(windows)]
    {
        start_watchdog();
        crate::platform::ffi::win32::install_fault_reporter();
    }
    // The audio thread runs on DirectSound over SDL
    // (`platform::sound`).
    // SAFETY: once, here, on the main thread.
    let owners = Owners::build();
    crate::error_dialog::publish(owners);
    crate::platform::direct3d::register_game_scratch(&owners.scene.scratch);
    start(owners);
    finish("start came to its end", 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A module's file name as the report wrote it before it ran without allocating.
    #[cfg(unix)]
    fn lossy(name: &[NameUnit]) -> String {
        String::from_utf8_lossy(name).into_owned()
    }

    #[cfg(windows)]
    fn lossy(name: &[NameUnit]) -> String {
        String::from_utf16_lossy(name)
    }

    /// The report's place texts as `format!` wrote them: "file+0x..", or None.
    fn in_module_text(address: usize) -> Option<String> {
        with_module(address, |file, base| {
            format!("{}+{:#x}", lossy(file), address - base)
        })
    }

    /// The fault line's module place and stack returns, written without allocating, are the
    /// texts the report's former `format!` / `join` made of the same stack.
    #[test]
    fn fault_line_writes_the_report_as_before() {
        let own_fn = (report_fault as *const ()).expose_provenance();
        let other_fn = (stack_returns as *const ()).expose_provenance();
        let mut stack = [0x10usize; 2048 / size_of::<usize>()];
        for (k, word) in stack.iter_mut().enumerate() {
            *word = match k % 7 {
                0 => own_fn + k,
                3 => other_fn + 2 * k,
                _ => 0x10 + k,
            };
        }
        let own =
            with_module(other_fn, |name, _| Box::<[NameUnit]>::from(name)).expect("own module");
        let path = std::env::temp_dir().join(format!("rc2k_fault_line_{}", std::process::id()));
        let file = std::fs::File::create(&path).unwrap();
        let mut line = FaultLine::new(&file);
        let _ = std::fmt::Write::write_str(&mut line, "in ");
        assert!(line.in_module(own_fn));
        let _ = std::fmt::Write::write_str(&mut line, "; ");
        stack_returns(&mut line, Some(&own), stack.as_ptr().expose_provenance());
        line.end();
        drop(file);
        let written = std::fs::read_to_string(&path).unwrap();
        let _ = std::fs::remove_file(&path);

        let own_text = in_module_text(other_fn).map(|m| m.split('+').next().unwrap().to_string());
        let mut returns = Vec::new();
        for &value in &stack {
            if let (Some(m), Some(own)) = (in_module_text(value), own_text.as_ref())
                && m.starts_with(own.as_str())
            {
                returns.push(m);
                if returns.len() == 24 {
                    break;
                }
            }
        }
        let expected = format!(
            "in {}; {}\n",
            in_module_text(own_fn).unwrap(),
            returns.join(" ")
        );
        assert_eq!(returns.len(), 24);
        assert_eq!(written, expected);
    }
}
