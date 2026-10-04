use std::io::Write;
use std::fs::{File, OpenOptions};
use std::ptr::{copy_nonoverlapping, null, read_unaligned, write_unaligned};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};
use detour::RawDetour;
use windows_sys::Win32::System::Diagnostics::Debug::FlushInstructionCache;
use windows_sys::Win32::System::Memory::{VirtualAlloc, MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE, VirtualProtect};
use windows_sys::Win32::System::Threading::GetCurrentProcess;

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
    relocs: &'static [usize],
    manual: bool,
    saved: OnceLock<[u8; 5]>,
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
            relocs: &[],
            manual: false,
            saved: OnceLock::new(),
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

    pub const fn relocate_calls(mut self, offset: &'static [usize]) -> Self {
        self.relocs = offset;
        self
    }

    unsafe fn write_code(addr: usize, bytes: &[u8]) {
        unsafe {
            let mut old = 0u32;
            VirtualProtect(addr as *const _, bytes.len(), PAGE_EXECUTE_READWRITE, &mut old);
            copy_nonoverlapping(bytes.as_ptr(), addr as *mut u8, bytes.len());
            VirtualProtect(addr as *const _, bytes.len(), old, &mut old);
            FlushInstructionCache(GetCurrentProcess(), addr as *const _, bytes.len());
        }
    }

    fn jmp_bytes(from: usize, to: usize) -> [u8; 5] {
        let rel = to.wrapping_sub(from + 5) as u32;
        let mut b = [0xE9, 0, 0, 0, 0];
        b[1..].copy_from_slice(&rel.to_le_bytes());
        b
    }

    pub const fn manual_patch(mut self) -> Self {
        self.manual = true;
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

                for &off in self.relocs {
                    // 'off' is the offset of an E8 (call) / E9 (jmp) instruction
                    assert!(matches!(*mem.add(off), 0xE8 | 0xE9));

                    let p = mem.add(off + 1) as *mut i32;
                    let old = read_unaligned(p);
                    let target = (addr + off + 5).wrapping_add(old as usize);
                    let new = target.wrapping_sub(mem as usize + off + 5) as i32;
                    write_unaligned(p, new);
                }

                self.original.store(mem as usize, Ordering::Relaxed);

                if self.manual {
                    assert!(self.private_copy_len >= 5, "manual_patch needs private_copy");
                    let mut orig = [0u8; 5];
                    copy_nonoverlapping(addr as *const u8, orig.as_mut_ptr(), 5);
                    let _ = self.saved.set(orig);
                    Self::write_code(addr, &Self::jmp_bytes(addr, self.thunk as usize));
                    return Ok(());
                }
            }

            let d = RawDetour::new(addr as *const (), self.thunk as *const ())?;
            if self.private_copy_len == 0 {
                self.original.store(d.trampoline() as usize, Ordering::Relaxed);
            }

            d.enable()?;
            let _ = self.detour.set(d);
        }

        Ok(())
    }

    unsafe fn set_enabled(&self, on: bool) -> Result<(), detour::Error> {
        if self.manual {
            let Some(orig) = self.saved.get() else { return Ok(()) };
            let addr = game_addr(self.rva);
            let patch = Self::jmp_bytes(addr, self.thunk as usize);
            let bytes: &[u8; 5] = if on { &patch } else {orig};
            unsafe { Self::write_code(addr, bytes) };
            return Ok(());
        }

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
