//! The program: the game's `start` (0x401000) gone through as the list of calls it is, in
//! Rust, without RAL.EXE.
//!
//! What happens (`run`):
//! 1. Every variable and constant of the game's has a home in Rust (`crate::moved`); the
//!    game's own pointers stay IDA's addresses, resolved where they are read.
//! 2. No code of RAL.EXE runs. Every call the game's data makes by address (menus, draw
//!    lists, frames, thread routines) and every one Rust still makes goes to its Rust
//!    version (`menu::table_calls`, `crate::routes`); an address
//!    with none ends the program, "not ported".
//! 3. `on_exception` only reports the fault, with the place and the stack's return
//!    addresses, and ends the program.
//!
//! A refusal (`BadInput`) ends the program with its reason; a fatal error is reported
//! by `fatal::report` and ends it as the original's. Stand-ins, each to be replaced by a
//! piece of the platform: see `stand_in`.

use crate::audio::owner::audio;
use crate::errors::Refused;
use crate::framework::{LOG_PATH, log};
use crate::gfx::owner::gfx;
use crate::install::outside::Game as Outside;
use crate::saves::owner::saves;
use crate::system::owner::system;
use rc2k_core::fatal::Context;
use std::path::Path;
use std::sync::atomic::Ordering;

// ---- calls by address that are done or answered by SDL ----

/// The game's empty functions (a lone `retn`; and 0x48558A, `pusha;
/// popa; retn` in a draw list): a call to one is done.
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
static TIME_LIMIT_EVENT: std::sync::OnceLock<crate::ffi::win32::Event> = std::sync::OnceLock::new();

/// How long a fault's report may take before the process is ended without it (a fault
/// inside the allocator or the loader leaves their locks held, and the report needs both).
const REPORT_TIME_LIMIT_SECONDS: u32 = 2;

#[cfg(unix)]
/// The fault reporter's time limit started: SIGALRM, whose default action ends the
/// process, comes after `REPORT_TIME_LIMIT_SECONDS`.
fn arm_time_limit() {
    crate::ffi::unix::alarm(REPORT_TIME_LIMIT_SECONDS);
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
    let Some(event) = crate::ffi::win32::create_event() else {
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
            crate::os::terminate(1)
        });
    if started.is_err() {
        log("standalone: the fault reporter has no time limit (no thread)");
    } else {
        let _ = TIME_LIMIT_EVENT.set(event);
    }
}

#[cfg(windows)]
/// The module a host address is in, and the address relative to its base
/// ("rc2k.exe+0x1234").
fn in_module(address: usize) -> Option<String> {
    let (path, base) = crate::ffi::win32::module_of_address_wide(address)?;
    let file = path.rsplit(['\\', '/']).next().unwrap_or(&path).to_string();
    Some(format!("{file}+{:#x}", address - base))
}

#[cfg(unix)]
/// The module a host address is in, and the address relative to its base ("rc2k+0x1234").
fn in_module(address: usize) -> Option<String> {
    let (path, base) = crate::ffi::unix::module_of_address(address)?;
    let file = path.rsplit('/').next().unwrap_or(&path).to_string();
    Some(format!("{file}+{:#x}", address - base))
}

/// The word at `at` on the faulting thread's stack.
// An unchecked read of a host address, as the fault report needs (it cannot ask the stack's
// bounds in a handler); the one `unsafe` left in this file.
#[allow(unsafe_code)]
fn stack_word(at: usize) -> usize {
    // SAFETY: the faulting thread's stack, 2 KB from its stack pointer: committed (the
    // game's code runs well inside it), and that thread is stopped in the handler.
    unsafe { std::ptr::with_exposed_provenance::<usize>(at).read_unaligned() }
}

/// The return addresses into this program found on the stack from `esp` (at most 24), for
/// `llvm-symbolizer` with the program's PDB (or its symbols on Unix).
fn stack_returns(esp: usize) -> String {
    let own = in_module((stack_returns as *const ()).expose_provenance())
        .map(|m| m.split('+').next().unwrap_or("").to_string());
    let mut out = Vec::new();
    // 2 KB of stack: well inside it at any fault the game code can reach.
    const WORD: usize = size_of::<usize>();
    for k in 0..2048 / WORD {
        let at = esp + WORD * k;
        let value = stack_word(at);
        if let (Some(m), Some(own)) = (in_module(value), own.as_ref())
            && m.starts_with(own.as_str())
        {
            out.push(m);
            if out.len() == 24 {
                break;
            }
        }
    }
    out.join(" ")
}

/// The game closed (`g_quitting`), in a race or on a menu: what the original's way out does
/// (0x4154B5 back into the main loop, which asks for the shutdown, logs "Leaving main
/// loop..." and returns to `start`), the program ended as `start` ends it. The data is saved
/// first, once the front end has its tables: before that a save would write blanks.
pub fn leave_on_close() -> ! {
    let saves = saves();
    let tables = &saves.best_times;
    if [tables.allocated.get(), tables.loaded.get()] == [1, 1]
        && let Err(e) = crate::saves::save_game_data::save_in_game(saves)
    {
        log(&format!(
            "standalone: the game's data was not saved on closing: {e:?}"
        ));
    }
    if let Err(e) = crate::threading::request_quit::request_quit() {
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
    log(&format!("standalone: ending: {why}"));
    crate::write_watch::dump_fields();
    #[cfg(rc2k_pgo)]
    {
        crate::ffi::pgo::write_profile();
    }
    // Not `std::process::exit`: this may be an exception's handler, where the libraries'
    // leave-taking can hang.
    crate::os::terminate(code)
}

/// The platform's functions `start` calls that SDL stands for: (the function, what it is).
const SDL_DOES: [(usize, &str); 5] = [
    (0x4103DE, "system::open_window"),
    (0x404981, "gfx::probe_exclusive_mode"),
    (0x404CD6, "gfx::release_screens"),
    (0x40FA8B, "sound::close_cd_audio"),
    (0x410509, "system::drain_messages"),
];

/// For calls by address of a function that needs nothing done here
/// (`menu::table_calls`): the ones SDL does and the lone `retn`s.
/// `at` is the address as IDA shows it. (SDL's leave the carry clear, which a menu
/// item's caller does not read.)
pub fn answers(at: usize) -> bool {
    SDL_DOES.iter().any(|w| w.0 == at) || LONE_RETN.contains(&at)
}

/// A fault reported and the program ended: `host` the faulting instruction, `data` the
/// address it read or wrote (`write`), `sp` the thread's stack pointer (host addresses).
/// Shared by the systems' handlers.
pub(crate) fn report_fault(host: usize, data: usize, write: bool, sp: usize) -> ! {
    arm_time_limit();
    let verb = if write { "writing" } else { "reading" };
    log(&format!(
        "standalone: the fault is in {}; return addresses on the stack: {}",
        in_module(host).unwrap_or_else(|| "no module".into()),
        stack_returns(sp)
    ));
    finish(
        &format!("access violation at {host:#x}, {verb} address {data:#x}"),
        1,
    );
}

// ---- `start` (0x401000) ----

/// Logs that `start` calls the Rust function `name`.
fn rust(name: &str) {
    log(&format!("standalone: start calls {name}"));
}

/// A refusal in `start`: a refused input ends the program, a fatal error is
/// raised.
fn refused(name: &str, e: Refused, r: &Context) {
    crate::fatal::raise::refused_in(name, e, r);
}

fn stand_in(what: &str, variable: &crate::session::Dword, value: u32) {
    log(&format!(
        "standalone: stand-in: {what} ({:#x} = {value})",
        variable.addr()
    ));
    variable.set(value);
}

/// The game's `start`, call for call, in its steps. The checks it makes after calls of the
/// platform (already running, no screen mode) cannot fail here.
///
/// Once, on the main thread, after the image is built: the start-up functions it calls write
/// the game's state before any thread of the game's runs.
fn start() {
    let r = Context::default();
    set_up_fixed_objects();
    seed_clock_and_random();
    check_machine(&r);
    read_settings_and_start_log(&r);
    make_save_folders();
    read_command_line_and_install_state(&r);
    load_language_and_game_data(&r);
    probe_display(&r);
    start_workers(&r);
    choose_screen_modes(&r);
    stand_in_window_state();
    run_main_loop(&r);
    shut_down(&r);
}

/// The objects the image pointed at each other: the players' records, font 56's palette (the
/// image names the palette sheet: its place in `gfx::glyph_store`; `gfx::build_fonts` hangs
/// the rest) and `g_surface.format`
/// (the image points it at the format record, `session::SCREEN`'s, named by its handle; the
/// platform sets it again at every lock).
fn set_up_fixed_objects() {
    let gfx = gfx();
    crate::menu::players_store::point_players(crate::menu::owner::menus());
    gfx.fonts.typefaces.objects[56]
        .palette
        .set(crate::gfx::glyph_store::SHEET_PLACE);
    gfx.drawing
        .draw_surface
        .format
        .set(crate::gfx::pixel_format::SCREEN_FORMAT);
}

/// The start time, the random generator's seed and the volume serial that stirs it.
fn seed_clock_and_random() {
    let system = system();
    rust("system::keep_start_time");
    crate::system::keep_start_time::keep_start_time(system);
    rust("math::seed_random");
    crate::math::seed_random::seed_random();
    rust("system::read_volume_serial");
    crate::system::read_volume_serial::read_volume_serial(system);
    // eax: what `system::read_volume_serial` leaves there, `start`'s esi at entry (its
    // `pop eax`; the serial is lost: `notes/bugs.md`), 0 here.
    rust("math::stir_random");
    crate::math::stir_random::stir_random(0);
}

/// The processor, keyboard and Windows version tests, and the error mode.
fn check_machine(r: &Context) {
    let system = system();
    // `system::set_exception_filter` (0x410D4C) sets nothing in the standalone.
    rust("system::test_processor");
    if let Err(e) =
        crate::system::test_processor::test_processor(&crate::system::test_processor::Real, system)
    {
        refused("system::test_processor", e, r);
    }
    rust("system::read_keyboard_type");
    crate::system::read_keyboard_type::read_keyboard_type(system);
    rust("system::read_windows_version");
    crate::system::read_windows_version::read_windows_version(system);
    // `CoInitialize` (no COM is used on this thread).
    // `memory::open_own_heap` (0x40298A): the game's own heap is not used.
    rust("system::set_error_mode");
    crate::system::set_error_mode::set_error_mode();
}

/// `var\ral.cfg`'s settings, the feature switches and the game's log.
fn read_settings_and_start_log(r: &Context) {
    rust("settings::read_settings");
    if let Err(why) = crate::settings::read_settings::read_settings() {
        finish(&format!("settings::read_settings refused: {}", why.0), 1);
    }
    rust("settings::apply_feature_switches");
    crate::settings::apply_feature_switches::apply_feature_switches();
    rust("logfile::start_log");
    if let Err(why) = crate::logfile::start_log::start_log() {
        refused("logfile::start_log", why.into(), r);
    }
}

/// The folders the game saves into but never makes (`notes`: `install::make_folders` makes
/// them only from the installer's script): made here when missing, in the game's folder, as
/// the path templates at 0x5296FB .. 0x52974B name them; not in the download folder, where the
/// program is the installer.
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
/// start check, the developer machine, the languages and the install record.
fn read_command_line_and_install_state(r: &Context) {
    let system = system();
    // `system::test_processor` (0x410883, called above) has set it from `cpuid`.
    if system.session.machine.has_fpu.get() != 1 {
        finish("fatal error 0x900010FF, FPU is required", 1);
    }
    rust("system::read_command_line");
    if let Err(why) = crate::system::read_command_line::read_own_command_line(system) {
        refused("system::read_command_line", why.into(), r);
    }
    rust("install::check_clio_disc");
    if let Err(e) = crate::install::check_clio_disc::check_clio_disc(
        crate::install::owner::install(),
        &mut Outside,
    ) {
        refused("install::check_clio_disc", e, r);
    }
    rust("install::check_at_start");
    let no_audio = &audio().state.no_audio;
    if let Err(e) = crate::install::check_at_start::check_at_start(
        crate::install::owner::install(),
        &mut Outside,
        no_audio,
    ) {
        refused("install::check_at_start", e, r);
    }
    rust("system::check_developer_machine");
    crate::system::check_developer_machine::check_developer_machine();
    rust("install::read_languages");
    if let Err(e) = crate::install::read_languages::read_languages(
        crate::install::owner::install(),
        &crate::install::owner::install().names.lang_cfg.text(),
    )
    .map_err(Refused::from)
    {
        refused("install::read_languages", e, r);
    }
    rust("install::read_record");
    if let Err(e) = crate::install::read_record::read_record(
        &crate::install::owner::install().texts.record_path.text(),
    )
    .map_err(Refused::from)
    {
        refused("install::read_record", e, r);
    }
}

/// The language's texts (`strings::load_language`, 0x40331F), the game's data files, and the
/// check that no other copy of the game runs.
fn load_language_and_game_data(r: &Context) {
    rust("strings::load_language");
    let compile = &mut || crate::strings::compile_lexicon::compile_lexicon();
    if let Err(e) = crate::strings::load_language::load_language(compile) {
        refused("strings::load_language", e, r);
    }
    rust("system::load_game_data");
    if let Err(e) = crate::system::load_game_data::load_in_game(system()) {
        refused("system::load_game_data", e, r);
    }
    rust("system::find_running_game");
    if crate::system::find_running_game::find_running_game() {
        finish("the game is running already (0x4103BE)", 0);
    }
}

/// The 3D cards and the display modes the screen can take. `system::open_window` (0x4103DE)
/// and `gfx::probe_exclusive_mode` (0x404981) are SDL's (`SDL_DOES`).
fn probe_display(r: &Context) {
    let gfx = gfx();
    rust("gfx::find_3d_cards");
    crate::gfx::find_3d_cards::find_3d_cards(&gfx.display);
    rust("gfx::list_primary_modes");
    if let Err(e) = crate::gfx::list_primary_modes::list_primary_modes(&gfx.display) {
        refused("gfx::list_primary_modes", e, r);
    }
    rust("gfx::list_secondary_modes");
    if let Err(e) = crate::gfx::list_secondary_modes::list_secondary_modes(&gfx.display) {
        refused("gfx::list_secondary_modes", e, r);
    }
}

/// The network's set-up, the game's threads and the audio thread's commands.
fn start_workers(r: &Context) {
    rust("net::set_up");
    if let Err(e) = crate::net::set_up::set_up() {
        refused("net::set_up", e, r);
    }
    rust("system::set_priority_class");
    crate::system::set_priority_class::set_priority_class(0x80);
    rust("threading::start_threads");
    if let Err(e) = crate::threading::start_threads::start_threads_in_game() {
        refused("threading::start_threads", e, r);
    }
    rust("sound::wait_for_commands");
    crate::sound::wait_for_commands::wait_in_game();
    rust("system::sleep");
    crate::system::sleep::sleep(1000);
}

/// The screen modes, the picture buffer and the pixel converters.
/// `sound::open_cd_audio` (0x40F9DE) is cut out of this build (`pusha; jmp` to its end), so
/// not called.
fn choose_screen_modes(r: &Context) {
    let gfx = gfx();
    rust("gfx::choose_screen_modes");
    if let Err(e) = crate::platform::choose_screen_modes(gfx) {
        refused("gfx::choose_screen_modes", e.into(), r);
    }
    rust("gfx::allocate_picture_buffer");
    if let Err(e) = crate::gfx::allocate_picture_buffer::allocate_picture_buffer(gfx) {
        refused("gfx::allocate_picture_buffer", e, r);
    }
    rust("gfx::set_component_format");
    if let Err(e) = crate::gfx::set_component_format::set_component_format(&gfx.pixel.formats) {
        refused("gfx::set_component_format", e, r);
    }
    rust("gfx::fill_thirds_table");
    crate::gfx::fill_thirds_table::fill_thirds_table(gfx);
    rust("gfx::install_converters");
    crate::gfx::install_converters::install_converters(&gfx.pixel.converters, system());
}

/// What the window's procedure and the key thread tell the main loop (the films play from `film.rs`).
fn stand_in_window_state() {
    let flags = &system().session.flags;
    stand_in("the window is the active one", &flags.window_active, 1);
    stand_in(
        "the window has the focus (until SDL says otherwise)",
        &flags.window_focused,
        1,
    );
    stand_in("the key thread runs", &system().session.clocks.key_ticks, 1);
}

fn run_main_loop(r: &Context) {
    rust("system::main_loop");
    if let Err(e) = crate::system::main_loop::main_loop_in_game(system()) {
        refused("system::main_loop", e, r);
    }
}

/// What follows the main loop: the network, the picture buffer, the priority class, the
/// installer's after-exit script, the error mode and the fatal report. `gfx::release_screens`
/// (0x404CD6), `sound::close_cd_audio` (0x40FA8B) and `system::drain_messages` (0x410509) are
/// SDL's (`SDL_DOES`).
fn shut_down(r: &Context) {
    rust("net::shut_down");
    if let Err(e) = crate::net::shut_down::shut_down() {
        refused("net::shut_down", e, r);
    }
    rust("gfx::free_picture_buffer");
    if let Err(why) = crate::gfx::free_picture_buffer::free_picture_buffer(gfx()) {
        finish(&format!("gfx::free_picture_buffer refused: {}", why.0), 1);
    }
    rust("system::set_priority_class");
    crate::system::set_priority_class::set_priority_class(0x20);
    rust("install::run_after_exit");
    if let Err(e) = crate::install::run_after_exit::run_after_exit(crate::install::owner::install())
    {
        refused("install::run_after_exit", e, r);
    }
    rust("system::restore_error_mode");
    crate::system::restore_error_mode::restore_error_mode(system());
    // `memory::free_own_heap` (0x4029DB): the game's own heap is not used.
    // `CoUninitialize` (see `CoInitialize`).
    rust("fatal::show_report_at_exit");
    crate::fatal::show_report_at_exit::show_report_at_exit();
}

/// Starts the platform and goes through `start`. The current
/// directory has to be the game's. Does not come back.
/// `frames`, if not 0: the program ends itself after that many frames were shown.
pub fn run(log_file: &Path, frames: u32) -> ! {
    crate::session::set_game_thread();
    crate::platform::FRAMES_TO_SHOW.store(frames, Ordering::SeqCst);
    crate::platform_timing::start_threads();
    crate::platform_input::SdlBackend::start();
    // Absolute: the installer changes the current folder (`install::make_folders`).
    let log_file = &std::path::absolute(log_file).unwrap_or_else(|_| log_file.to_path_buf());
    let _ = std::fs::File::create(log_file);
    let _ = LOG_PATH.set(log_file.to_path_buf());
    // A panic ends the program at once (it cannot unwind out of an `extern "C"` function):
    // say where first.
    std::panic::set_hook(Box::new(|info| {
        log(&format!("standalone: panic: {info}"));
    }));
    #[cfg(unix)]
    crate::ffi::unix::install_fault_reporter();
    #[cfg(windows)]
    {
        start_watchdog();
        crate::ffi::win32::install_fault_reporter();
    }
    // The audio thread runs on DirectSound over SDL
    // (`platform_sound`).
    // SAFETY: once, here, on the main thread.
    start();
    finish("start came to its end", 0)
}
