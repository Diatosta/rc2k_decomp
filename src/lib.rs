pub mod errors;
pub mod files;
mod framework;
pub mod gfx;
pub mod logfile;
pub mod math;
pub mod records;
mod registry;
pub mod screens;
pub mod service;
pub mod strings;
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
        "commands: list | on <name|prefix|all> | off <name|prefix|all> | shadow on|off | trace on|off | blank on|off [prefix] | stats | reload | selftest",
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
            ["selftest"] => {
                for h in ALL.iter() {
                    // Game threads stay out of hooked functions while a test runs.
                    let _paused = h.self_test_pauses_game().then(framework::pause_game);
                    if let Some((cases, bad)) = h.self_test() {
                        log(&format!("{:<30} cases={cases} failures={bad}", h.name()));
                    }
                }
            }
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

fn install_all() -> Vec<bool> {
    let base = unsafe { GetModuleHandleW(std::ptr::null()) } as usize;
    BASE.store(base, Ordering::Relaxed);
    log(&format!("attached, exe base = {base:#x}"));

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

fn init() {
    if let Some(path) = dll_sibling("hook.log") {
        let _ = File::create(&path); // start each run with an empty log
        let _ = LOG_PATH.set(path);
    }
    let console = console_init();
    let mut enabled = install_all();
    let cfg = load_config().unwrap_or_default();
    apply_config(&cfg, &mut enabled);
    if let Some((input, out)) = console {
        console_loop(input, out, enabled);
    }
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
        // Don't do real work under the loader lock; hop to a thread.
        DLL_HANDLE.store(hinst as usize, Ordering::Relaxed);
        std::thread::spawn(init);
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
