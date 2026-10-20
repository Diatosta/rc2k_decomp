// Nearly every function here is `unsafe` for one and the same reason: it reads and writes the
// game's memory at fixed addresses, so it is only sound inside the running game, on data the
// game has loaded, under the conditions its header comment gives. A "# Safety" section on each
// of some 130 functions would repeat that sentence; the particular conditions are in each
// function's own doc comment and in the `Err(BadInput)` cases it checks.
#![allow(clippy::missing_safety_doc)]

pub mod audio;
pub mod d3d;
pub mod drive;
pub mod errors;
pub mod fatal;
pub mod files;
#[cfg(feature = "standalone")]
pub mod film;
mod framework;
pub mod gfx;
pub mod hud;
pub mod input;
pub mod install;
mod leftovers;
pub mod logfile;
pub mod marks;
pub mod math;
pub mod memory;
pub mod menu;
pub mod net;
pub mod physics;
#[cfg(feature = "standalone")]
pub mod platform;
#[cfg(feature = "standalone")]
pub mod platform_3d;
#[cfg(feature = "standalone")]
pub mod platform_input;
#[cfg(feature = "standalone")]
pub mod platform_sound;
pub mod race;
#[cfg(feature = "standalone")]
pub mod rasteriser;
pub mod records;
pub mod redirect;
mod registry;
pub mod saves;
pub mod scene;
pub mod screens;
pub mod service;
pub mod settings;
pub mod sound;
#[cfg(feature = "standalone")]
pub mod standalone;
pub mod strings;
pub mod system;
pub mod text;
pub mod threading;

use crate::framework::{BASE, CONSOLE_OUT, LOG_PATH, SHADOW_MODE, TRACE, log};
use crate::registry::ALL;
use std::collections::HashMap;
use std::ffi::c_void;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};
use windows_sys::Win32::Foundation::{HINSTANCE, HMODULE, TRUE};
use windows_sys::Win32::System::Console::AllocConsole;
use windows_sys::Win32::System::LibraryLoader::{
    DisableThreadLibraryCalls, GetModuleFileNameW, GetModuleHandleW,
};
use windows_sys::Win32::System::SystemServices::DLL_PROCESS_ATTACH;
use windows_sys::core::BOOL;

fn console_init() -> Option<(File, File)> {
    unsafe { AllocConsole() };
    let out = OpenOptions::new().write(true).open("CONOUT$").ok()?;
    let input = File::open("CONIN$").ok()?;
    let out_for_log = out.try_clone().ok()?;
    let _ = CONSOLE_OUT.set(Mutex::new(out_for_log));
    Some((input, out))
}

static DLL_HANDLE: AtomicUsize = AtomicUsize::new(0);
static INSTALLED: OnceLock<Vec<bool>> = OnceLock::new();

/// The self-tests of the hooks `pat` selects (as `on`/`off` do), each with the game paused;
/// with `leftovers`, each also logs what it left changed in the game's writable sections.
fn run_self_tests(pat: &str, leftovers: bool) {
    for h in ALL.iter().filter(|h| registry::selects(pat, h.name())) {
        let _paused = framework::pause_game();
        let before = leftovers.then(leftovers::Snapshot::take);
        framework::set_running_test(Some(h.name()));
        // A test that panics is logged and the next one runs (a panic once ended the
        // console thread, and with it every later test, 2026-10-19).
        let result = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| h.self_test()))
        {
            Ok(result) => result,
            Err(payload) => {
                let why = payload
                    .downcast_ref::<&str>()
                    .map(|s| s.to_string())
                    .or_else(|| payload.downcast_ref::<String>().cloned())
                    .unwrap_or_default();
                log(&format!(
                    "SELFTEST FAIL {}: the test panicked: {why}",
                    h.name()
                ));
                None
            }
        };
        framework::set_running_test(None);
        if let Some((cases, bad)) = result {
            log(&format!("{:<30} cases={cases} failures={bad}", h.name()));
        }
        if let Some(before) = &before {
            leftovers::report(h.name(), before);
        }
    }
}

fn apply(pat: &str, on: bool, enabled: &mut [bool], out: &mut impl Write) {
    let mut changed = 0;
    for (i, h) in ALL.iter().enumerate() {
        if registry::selects(pat, h.name()) && enabled[i] != on {
            match unsafe { h.set_enabled(on) } {
                Ok(()) => {
                    enabled[i] = on;
                    changed += 1;
                }
                Err(e) => {
                    let _ = writeln!(out, "{}: {e}", h.name());
                }
            }
        }
    }

    let _ = writeln!(
        out,
        "{changed} hook(s) -> {}",
        if on { "on" } else { "off" }
    );
}

fn console_loop(input: File, mut out: File, mut enabled: Vec<bool>) {
    log(
        "commands: list | on <name|prefix|all> | off <name|prefix|all> | shadow on|off | trace on|off | blank on|off [prefix] | stats | reload | selftest [name|prefix [leftovers]] | fatal",
    );

    for line in BufReader::new(input).lines().map_while(Result::ok) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        match parts.as_slice() {
            ["list"] => {
                for (i, h) in ALL.iter().enumerate() {
                    log(&format!(
                        "{:>4} {}",
                        if enabled[i] { "on" } else { "off" },
                        h.name()
                    ));
                }
            }
            ["on", pat] => apply(pat, true, &mut enabled, &mut out),
            ["off", pat] => apply(pat, false, &mut enabled, &mut out),
            ["shadow", v] => {
                SHADOW_MODE.store(*v == "on", Ordering::Relaxed);
                log(&format!("shadow = {v}"));
            }
            ["stats"] => {
                for h in ALL.iter() {
                    log(&format!(
                        "{:<30} calls={} mismatches={}",
                        h.name(),
                        h.stats().calls.load(Ordering::Relaxed),
                        h.stats().mismatches.load(Ordering::Relaxed)
                    ));
                }
            }
            ["reload"] => {
                if let Some(cfg) = load_config() {
                    apply_config(&cfg, &mut enabled);
                }
                log("reload done");
            }
            ["selftest"] => run_self_tests("all", false),
            ["selftest", pat] => run_self_tests(pat, false),
            ["selftest", pat, "leftovers"] => run_self_tests(pat, true),
            ["blank", v] => {
                screens::set_blank(*v == "on", "");
                log(&format!(
                    "blank = {v} (every hook in screens:: draws nothing while on)"
                ));
            }
            ["blank", v, prefix] => {
                screens::set_blank(*v == "on", prefix);
                log(&format!(
                    "blank = {v} for hooks starting with {prefix} (they draw nothing while on)"
                ));
            }
            ["fatal"] => {
                fatal::raise::arm_test();
                log(
                    "fatal test armed: the next key poll raises a test error and the game ends (shadow on: the original reporter runs; shadow off: the Rust one)",
                );
            }
            ["trace", v] => {
                TRACE.store(*v == "on", Ordering::Relaxed);
                log(&format!("trace = {v}"));
            }
            [] => {}
            _ => {
                log("?");
            }
        }
    }
}

/// Installs every hook on the game's code at `base`: the running game's own image, or
/// the copy of it `standalone` has mapped.
/// The standalone program's: every hook counted as in place at `base` (its image holds no
/// code to patch: `standalone.rs` sends each hooked address to the hook's thunk).
pub fn mark_all_installed(base: usize) -> Vec<bool> {
    BASE.store(base, Ordering::Relaxed);
    let installed = vec![true; ALL.len()];
    let _ = INSTALLED.set(installed.clone());
    installed
}

fn install_all(base: usize) -> Vec<bool> {
    BASE.store(base, Ordering::Relaxed);
    log(&format!(
        "attached, exe base = {base:#x}, DLL base = {:#x}",
        framework::dll_base()
    ));

    let installed: Vec<bool> = ALL
        .iter()
        .map(|h| {
            let addr = base + h.rva();
            match unsafe { h.install(addr) } {
                Ok(()) => {
                    log(&format!("hooked {} at {addr:#x}", h.name()));
                    true
                }
                Err(e) => {
                    log(&format!("FAILED to hook {}: {e}", h.name()));
                    false
                }
            }
        })
        .collect();

    let _ = INSTALLED.set(installed.clone());
    installed
}

/// Everything that has to be in place before the game's first instruction: the log file,
/// the hooks and what `hook.toml` says about them. Runs in `DllMain`, so that no call of
/// the game's start is missed and no code is patched while a thread is running it; only
/// what is safe under the loader lock is done here (files, memory, patching code).
///
/// The console window is opened here too, before the game has a window: opened later, from
/// its thread, it came up over the game's window on a fast start and took the focus, and
/// the game waits at "Entering main loop..." for its window to be the active one
/// (2026-10-12, the first start with `shadow = false` after the hooks moved here).
fn install_now() -> (Vec<bool>, Option<(File, File)>) {
    if let Some(path) = dll_sibling("hook.log") {
        let _ = File::create(&path); // start each run with an empty log
        let _ = LOG_PATH.set(path);
    }
    let console = console_init();
    let mut enabled = install_all(unsafe { GetModuleHandleW(std::ptr::null()) } as usize);
    let cfg = load_config().unwrap_or_default();
    apply_config(&cfg, &mut enabled);
    (enabled, console)
}

/// # Safety
#[allow(nonstandard_style)]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn DllMain(
    hinst: HINSTANCE,
    reason: u32,
    _reserved: *mut c_void,
) -> BOOL {
    if reason == DLL_PROCESS_ATTACH {
        unsafe { DisableThreadLibraryCalls(hinst) };
        DLL_HANDLE.store(hinst as usize, Ordering::Relaxed);
        let (enabled, console) = install_now();
        // The console waits for input: that is for a thread (which starts once the
        // loader lock is free).
        if let Some((input, out)) = console {
            std::thread::spawn(move || console_loop(input, out, enabled));
        }
    }
    TRUE
}

#[derive(serde::Deserialize, Default)]
#[serde(default)]
struct Config {
    shadow: bool,
    default_enabled: Option<bool>,
    hooks: HashMap<String, bool>,
    trace: bool,
}

/// A file in the same folder as this DLL.
fn dll_sibling(name: &str) -> Option<PathBuf> {
    let mut buf = [0u16; 1024];
    let n = unsafe {
        GetModuleFileNameW(
            DLL_HANDLE.load(Ordering::Relaxed) as HMODULE,
            buf.as_mut_ptr(),
            buf.len() as u32,
        )
    } as usize;

    if n == 0 {
        return None;
    }
    Some(PathBuf::from(String::from_utf16_lossy(&buf[..n])).with_file_name(name))
}

fn config_path() -> Option<PathBuf> {
    dll_sibling("hook.toml")
}

fn load_config() -> Option<Config> {
    let path = config_path()?;
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            log(&format!("config: {} not read ({e})", path.display()));
            return None;
        }
    };

    match toml::from_str::<Config>(&text) {
        Ok(c) => {
            log(&format!("config: loaded {}", path.display()));
            Some(c)
        }
        Err(e) => {
            log(&format!("config: parse error in {}:\n{e}", path.display()));
            None
        }
    }
}

fn apply_config(cfg: &Config, enabled: &mut [bool]) {
    SHADOW_MODE.store(cfg.shadow, Ordering::Relaxed);
    TRACE.store(cfg.trace, Ordering::Relaxed);

    for name in cfg.hooks.keys() {
        if !ALL.iter().any(|h| h.name() == name.as_str()) {
            log(&format!("config: hook '{name}' not found"));
        }
    }

    let default = cfg.default_enabled.unwrap_or(true);
    for (i, h) in ALL.iter().enumerate() {
        if !INSTALLED.get().is_some_and(|v| v[i]) {
            continue;
        }

        let want = cfg.hooks.get(h.name()).copied().unwrap_or(default);
        if want != enabled[i] {
            match unsafe { h.set_enabled(want) } {
                Ok(()) => enabled[i] = want,
                Err(e) => {
                    log(&format!("config: {}: {e}", h.name()));
                }
            }
        }
    }

    log(&format!(
        "config applied: shadow={} trace={}",
        cfg.shadow, cfg.trace
    ));
}
