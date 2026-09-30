use std::ffi::c_void;
use std::io::{BufRead, BufReader, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use windows_sys::Win32::Foundation::{HINSTANCE, TRUE};
use windows_sys::Win32::System::Console::AllocConsole;
use windows_sys::Win32::System::LibraryLoader::{DisableThreadLibraryCalls, GetModuleHandleW};
use windows_sys::Win32::System::SystemServices::DLL_PROCESS_ATTACH;
use windows_sys::core::BOOL;

fn log(msg: &str) {
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("hook.log")
    {
        let _ = writeln!(f, "{msg}");
    }
}

struct Hook {
    name: &'static str,
    rva: usize,
    install: unsafe fn(usize) -> Result<(), detour::Error>,
    set_enabled: unsafe fn(bool) -> Result<(), detour::Error>,
}

static HOOKS: &[Hook] = &[Hook {
    name: "add_clamped",
    rva: 0x0001_2340,
    install: add_clamped::install,
    set_enabled: add_clamped::set_enabled,
}];

static SHADOW_MODE: AtomicBool = AtomicBool::new(false);

mod add_clamped {
    use super::*;
    use detour::static_detour;
    use std::sync::atomic::Ordering;

    type Sig = unsafe extern "C" fn(i32, i32) -> i32;

    static_detour! {
        static Detour: unsafe extern "C" fn(i32, i32) -> i32;
    }

    fn reimplemented(a: i32, b: i32) -> i32 {
        a.saturating_add(b).clamp(-1000, 1000)
    }

    fn dispatch(a: i32, b: i32) -> i32 {
        if SHADOW_MODE.load(Ordering::Relaxed) {
            let original = unsafe { Detour.call(a, b) };
            let mine = reimplemented(a, b);
            if original != mine {
                log(&format!("MISMATCH add_clamped({a}, {b}): original={original}, rust={mine}"));
            }
            original
        } else {
            reimplemented(a, b)
        }
    }

    pub unsafe fn install(addr: usize) -> Result<(), detour::Error> {
        unsafe {
            let target: Sig = std::mem::transmute(addr);
            Detour.initialize(target, dispatch)?.enable()
        }
    }

    pub unsafe fn set_enabled(enabled: bool) -> Result<(), detour::Error> {
        unsafe {
            if enabled {
                Detour.enable()
            } else {
                Detour.disable()
            }
        }
    }
}

fn apply(pat: &str, on: bool, enabled: &mut [bool], out: &mut impl Write) {
    let mut changed = 0;
    for (i, h) in HOOKS.iter().enumerate() {
        if (pat == "all" || h.name.starts_with(pat)) && enabled[i] != on {
            match unsafe { (h.set_enabled)(on) } {
                Ok(()) => {
                    enabled[i] = on;
                    changed += 1;
                }
                Err(e) => {
                    let _ = writeln!(out, "{}: {e}", h.name);
                }
            }
        }
    }

    let _ = writeln!(out, "{changed} hook(s) -> {}", if on { "on" } else { "off" });
}

fn console_loop(mut enabled: Vec<bool>) {
    unsafe { AllocConsole() };

    let (Ok(input), Ok(mut out)) = (
        std::fs::File::open("CONIN$"),
        std::fs::OpenOptions::new().write(true).open("CONOUT$"),
        ) else {
        log("could not open console");
        return;
    };

    let _ = writeln!(out, "commands: list | on <name|prefix|all> | off <name|prefix|all> | shadow on|off");

    for line in BufReader::new(input).lines().map_while(Result::ok) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        match parts.as_slice() {
            ["list"] => {
                for (i, h) in HOOKS.iter().enumerate() {
                    let _ = writeln!(out, "{:>4} {}", if enabled[i] { "on" } else { "off" }, h.name);
                }
            }
            ["on", pat] => apply(pat, true, &mut enabled, &mut out),
            ["off", pat] => apply(pat, false, &mut enabled, &mut out),
            ["shadow", v] => {
                SHADOW_MODE.store(*v == "on", Ordering::Relaxed);
                let _ = writeln!(out, "shadow = {v}");
            },
            [] => {}
            _ => {
                let _ = writeln!(out, "?");
            }
        }
    }
}

fn init() {
    let base = unsafe { GetModuleHandleW(std::ptr::null()) } as usize;
    log(&format!("attached, exe base = {base:#x}"));

    for h in HOOKS {
        let addr = base + h.rva;
        match unsafe { (h.install)(addr) } {
            Ok(()) => log(&format!("hooked {} at {addr:#x}", h.name)),
            Err(e) => log(&format!("FAILED to hook {}: {e}", h.name)),
        }
    }

    console_loop(vec![true; HOOKS.len()]);
}

/// # Safety
#[allow(nonstandard_style)]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn DllMain(hinst: HINSTANCE, reason: u32, _reserved: *mut c_void) -> BOOL {
    if reason == DLL_PROCESS_ATTACH {
        unsafe { DisableThreadLibraryCalls(hinst) };
        // Don't do real work under the loader lock; hop to a thread.
        std::thread::spawn(init);
    }
    TRUE
}