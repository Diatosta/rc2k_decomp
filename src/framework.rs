use std::io::Write;
use std::fs::{File, OpenOptions};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};

pub static SHADOW_MODE: AtomicBool = AtomicBool::new(false);
pub static BASE: AtomicUsize = AtomicUsize::new(0);
pub static CONSOLE_OUT: OnceLock<Mutex<File>> = OnceLock::new();
pub static TRACE: AtomicBool = AtomicBool::new(false);
pub const MAX_LOGGED: usize = 50;


pub struct HookStats {
    pub calls: AtomicUsize,
    pub mismatches: AtomicUsize,
}

impl Default for HookStats {
    fn default() -> Self {
        Self::new()
    }
}
impl HookStats {
    pub const fn new() -> Self {
        Self {
            calls: AtomicUsize::new(0),
            mismatches: AtomicUsize::new(0),
        }
    }
}
pub struct Hook {
    pub name: &'static str,
    pub rva: usize,
    pub stats: &'static HookStats,
    pub install: unsafe fn(usize) -> Result<(), detour::Error>,
    pub set_enabled: unsafe fn(bool) -> Result<(), detour::Error>,
    pub self_test: Option<fn() -> (usize, usize)>,
}

pub fn game_addr(rva: usize) -> usize {
    BASE.load(Ordering::Relaxed) + rva
}

pub fn log(msg: &str) {
    if let Some(out) = CONSOLE_OUT.get()
        && let Ok(mut f) = out.lock()
    {
        let _ = writeln!(f, "{msg}");
    }

    if let Ok(mut f) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(r"Z:\tmp\hook.log")
    {
        let _ = writeln!(f, "{msg}");
    }
}
