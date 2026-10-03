use std::io::Write;
use std::fs::{File, OpenOptions};
use std::ptr::{copy_nonoverlapping, null};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};
use detour::RawDetour;
use windows_sys::Win32::System::Memory::{VirtualAlloc, MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE};

pub static SHADOW_MODE: AtomicBool = AtomicBool::new(false);
pub static BASE: AtomicUsize = AtomicUsize::new(0);
pub static CONSOLE_OUT: OnceLock<Mutex<File>> = OnceLock::new();
pub static TRACE: AtomicBool = AtomicBool::new(false);
pub const MAX_LOGGED: usize = 50;

pub trait GameHook: Sync {
    fn name(&self) -> &'static str;
    fn rva(&self) -> usize;
    fn stats(&self) -> &HookStats;
    unsafe fn install(&self, addr: usize) -> Result<(), detour::Error>;
    unsafe fn set_enabled(&self, on: bool) -> Result<(), detour::Error>;

    /// (cases, mismatches). None = this hook has no self-test
    fn self_test(&self) -> Option<(usize, usize)> { None }
}

pub struct RawHook {
    name: &'static str,
    rva: usize,
    pub stats: HookStats,
    thunk: unsafe extern "C" fn(),
    /// Where 'call_original' jumps to, filled in by 'install'
    original: &'static AtomicUsize,
    private_copy_len: usize,
    self_test_fn: Option<fn() -> (usize, usize)>,
    detour: OnceLock<RawDetour>,
}
impl RawHook {
    pub const fn new(
        name: &'static str,
        rva: usize,
        thunk: unsafe extern "C" fn(),
        original: &'static AtomicUsize,
    ) -> Self {
        Self {
            name, rva, thunk, original,
            stats: HookStats::new(),
            private_copy_len: 0,
            self_test_fn: None,
            detour: OnceLock::new(),
        }
    }

    pub const fn private_copy(mut self, len: usize) -> Self {
        self.private_copy_len = len;
        self
    }

    pub const fn self_test(mut self, f: fn() -> (usize, usize)) -> Self {
        self.self_test_fn = Some(f);
        self
    }
}
impl GameHook for RawHook {
    fn name(&self) -> &'static str { self.name }
    fn rva(&self) -> usize { self.rva }
    fn stats(&self) -> &HookStats { &self.stats }

    unsafe fn install(&self, addr: usize) -> Result<(), detour::Error> {
        unsafe {
            if self.private_copy_len > 0 {
                // Copy the untouched function before patching it
                let mem = VirtualAlloc(
                    null(), self.private_copy_len,
                    MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE,
                ) as *mut u8;

                assert!(!mem.is_null(), "VirtualAlloc failed");
                copy_nonoverlapping(addr as *const u8, mem, self.private_copy_len);
                self.original.store(mem as usize, Ordering::Relaxed);
            }

            let d = RawDetour::new(addr as *const (), self.thunk as *const ())?;
            if self.private_copy_len == 0 {
                self.original.store(d.trampoline() as *const () as usize, Ordering::Relaxed);
            }

            d.enable()?;
            let _ = self.detour.set(d);
        }

        Ok(())
    }

    unsafe fn set_enabled(&self, on: bool) -> Result<(), detour::Error> {
        let Some(d) = self.detour.get() else { return Ok(()) };
        unsafe { if on { d.enable() } else { d.disable() } }
    }

    fn self_test(&self) -> Option<(usize, usize)> {
        self.self_test_fn.map(|f| f())
    }
}

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
