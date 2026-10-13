use detour::RawDetour;
use std::cell::Cell;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::ptr::{copy_nonoverlapping, null, read_unaligned, write_unaligned};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};
use windows_sys::Win32::System::Diagnostics::Debug::FlushInstructionCache;
use windows_sys::Win32::System::Memory::{
    MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE, VirtualAlloc, VirtualProtect,
};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, GetCurrentThreadId, Sleep};

pub static SHADOW_MODE: AtomicBool = AtomicBool::new(false);
pub static BASE: AtomicUsize = AtomicUsize::new(0);
pub static CONSOLE_OUT: OnceLock<Mutex<File>> = OnceLock::new();
pub static TRACE: AtomicBool = AtomicBool::new(false);
/// `hook.log` next to the DLL, set by `init`. Nothing is written to a file before that.
pub static LOG_PATH: OnceLock<PathBuf> = OnceLock::new();
pub const MAX_LOGGED: usize = 50;
/// True in the program that runs the ported code without the game (`standalone.rs`).
pub static STANDALONE: AtomicBool = AtomicBool::new(false);
/// Set by `standalone.rs`: for a hook's address in the game's image, its thunk.
/// `call_game` goes there straight away; the image itself cannot be executed there.
pub static CALL_REDIRECT: OnceLock<std::collections::HashMap<usize, usize>> = OnceLock::new();

pub fn standalone() -> bool {
    STANDALONE.load(Ordering::Relaxed)
}
pub const IMAGE_BASE: usize = 0x400000;

/// RVA of an address as IDA shows it.
pub const fn rva(addr: usize) -> usize {
    addr - IMAGE_BASE
}

pub fn shadow() -> bool {
    SHADOW_MODE.load(Ordering::Relaxed)
}

pub fn trace() -> bool {
    TRACE.load(Ordering::Relaxed)
}

pub trait GameHook: Sync {
    fn name(&self) -> &'static str;
    fn rva(&self) -> usize;
    fn stats(&self) -> &HookStats;
    unsafe fn install(&self, addr: usize) -> Result<(), detour::Error>;
    unsafe fn set_enabled(&self, on: bool) -> Result<(), detour::Error>;

    /// Makes `call_original` go to `addr` from now on. For `standalone.rs`, where there is
    /// no original to run: it names an address that faults and tells which hook asked.
    fn set_original(&self, addr: usize);

    /// (cases, mismatches). None = this hook has no self-test
    fn self_test(&self) -> Option<(usize, usize)> {
        None
    }

    /// False if the game must keep running while this hook's self-test runs.
    fn self_test_pauses_game(&self) -> bool {
        true
    }
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
    unpaused_self_test: bool,
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
            name,
            rva,
            thunk,
            original,
            stats: HookStats::new(),
            private_copy_len: 0,
            self_test_fn: None,
            detour: OnceLock::new(),
            relocs: &[],
            manual: false,
            unpaused_self_test: false,
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

    /// Runs this hook's self-test without pausing the game. For a hook whose thunk is
    /// `ungated` because the game enters it holding a lock that the test also needs.
    pub const fn self_test_unpaused(mut self) -> Self {
        self.unpaused_self_test = true;
        self
    }

    pub const fn relocate_calls(mut self, offset: &'static [usize]) -> Self {
        self.relocs = offset;
        self
    }

    unsafe fn write_code(addr: usize, bytes: &[u8]) {
        unsafe {
            let mut old = 0u32;
            VirtualProtect(
                addr as *const _,
                bytes.len(),
                PAGE_EXECUTE_READWRITE,
                &mut old,
            );
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

    pub fn is_enabled(&self) -> bool {
        if self.manual {
            let Some(orig) = self.saved.get() else {
                return false;
            };
            let now = unsafe { *(game_addr(self.rva) as *const [u8; 5]) };
            return now != *orig;
        }
        self.detour.get().is_some_and(|d| d.is_enabled())
    }

    /// False until `install` has stored where `call_original` jumps.
    pub fn has_original(&self) -> bool {
        self.original.load(Ordering::Relaxed) != 0
    }

    /// Counts one call from the game. First line of every `dispatch`.
    pub fn count_call(&self) {
        self.stats.calls.fetch_add(1, Ordering::Relaxed);
    }

    /// Counts a shadow mismatch and logs it, at most `MAX_LOGGED` times per hook.
    pub fn mismatch(&self, detail: impl FnOnce() -> String) {
        if self.stats.mismatches.fetch_add(1, Ordering::Relaxed) < MAX_LOGGED {
            log(&format!("MISMATCH {}: {}", self.name, detail()));
        }
    }

    /// Runs `f` with this hook switched off, then restores its previous state.
    pub fn run_unhooked<R>(&self, f: impl FnOnce() -> R) -> R {
        let was = self.is_enabled();
        if was {
            let _ = unsafe { self.set_enabled(false) };
        }
        let r = f();
        if was {
            let _ = unsafe { self.set_enabled(true) };
        }
        r
    }
}
impl GameHook for RawHook {
    fn name(&self) -> &'static str {
        self.name
    }
    fn rva(&self) -> usize {
        self.rva
    }
    fn stats(&self) -> &HookStats {
        &self.stats
    }
    fn set_original(&self, addr: usize) {
        self.original.store(addr, Ordering::SeqCst);
    }

    unsafe fn install(&self, addr: usize) -> Result<(), detour::Error> {
        unsafe {
            if self.private_copy_len > 0 {
                // Copy the untouched function before patching it
                let mem = VirtualAlloc(
                    null(),
                    self.private_copy_len,
                    MEM_COMMIT | MEM_RESERVE,
                    PAGE_EXECUTE_READWRITE,
                ) as *mut u8;

                assert!(!mem.is_null(), "VirtualAlloc failed");
                copy_nonoverlapping(addr as *const u8, mem, self.private_copy_len);

                for &off in self.relocs {
                    // 'off' is the offset of an E8 (call) or E9 (jmp) instruction, or of a
                    // conditional near jump (0F 8x) out of the function, as the screen
                    // loops have to their fatal-error stubs.
                    let len = match (*mem.add(off), *mem.add(off + 1)) {
                        (0xE8 | 0xE9, _) => 5,
                        (0x0F, 0x80..=0x8F) => 6,
                        _ => panic!("relocation at {off:#x} is not a call or a near jump"),
                    };

                    let p = mem.add(off + len - 4) as *mut i32;
                    let old = read_unaligned(p);
                    let target = (addr + off + len).wrapping_add(old as usize);
                    let new = target.wrapping_sub(mem as usize + off + len) as i32;
                    write_unaligned(p, new);
                }

                self.original.store(mem as usize, Ordering::Relaxed);

                if self.manual {
                    assert!(
                        self.private_copy_len >= 5,
                        "manual_patch needs private_copy"
                    );
                    let mut orig = [0u8; 5];
                    copy_nonoverlapping(addr as *const u8, orig.as_mut_ptr(), 5);
                    let _ = self.saved.set(orig);
                    Self::write_code(addr, &Self::jmp_bytes(addr, self.thunk as usize));
                    return Ok(());
                }
            }

            let d = RawDetour::new(addr as *const (), self.thunk as *const ())?;
            if self.private_copy_len == 0 {
                self.original
                    .store(d.trampoline() as usize, Ordering::Relaxed);
            }

            d.enable()?;
            let _ = self.detour.set(d);
        }

        Ok(())
    }

    unsafe fn set_enabled(&self, on: bool) -> Result<(), detour::Error> {
        if self.manual {
            let Some(orig) = self.saved.get() else {
                return Ok(());
            };
            let addr = game_addr(self.rva);
            let patch = Self::jmp_bytes(addr, self.thunk as usize);
            let bytes: &[u8; 5] = if on { &patch } else { orig };
            unsafe { Self::write_code(addr, bytes) };
            return Ok(());
        }

        let Some(d) = self.detour.get() else {
            return Ok(());
        };
        unsafe { if on { d.enable() } else { d.disable() } }
    }

    fn self_test(&self) -> Option<(usize, usize)> {
        self.self_test_fn.map(|f| f())
    }

    fn self_test_pauses_game(&self) -> bool {
        !self.unpaused_self_test
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

// ---- pausing the game while a self-test runs ----
//
// Self-tests and the game's own code share game memory (text buffers, the kerning table, work
// variables), so a test must not run while a game thread is inside a hooked function, and the
// game must not enter one while a test runs. Every thunk calls `hook_enter` and `hook_leave`.
// A game thread holds nothing while it is inside a hook: it is only counted, so that
// `pause_game` can wait for it to come out, and it waits at its next outermost hook entry for
// as long as a test is running.

/// Thread running a self-test that pauses the game; 0 when there is none.
static TEST_THREAD: AtomicU32 = AtomicU32::new(0);
/// Threads that are inside a hook right now.
static THREADS_IN_HOOKS: AtomicUsize = AtomicUsize::new(0);

thread_local! {
    /// How many hooks this thread is inside: hooked functions call hooked functions.
    static HOOK_DEPTH: Cell<u32> = const { Cell::new(0) };
}

fn other_thread_is_testing() -> bool {
    let t = TEST_THREAD.load(Ordering::SeqCst);
    t != 0 && t != unsafe { GetCurrentThreadId() }
}

/// Called by every gated thunk before `dispatch`.
pub extern "C" fn hook_enter() {
    let depth = HOOK_DEPTH.get();
    HOOK_DEPTH.set(depth + 1);
    if depth > 0 {
        return; // already inside a hook: never wait half way through one
    }
    loop {
        while other_thread_is_testing() {
            unsafe { Sleep(1) };
        }
        THREADS_IN_HOOKS.fetch_add(1, Ordering::SeqCst);
        if !other_thread_is_testing() {
            return;
        }
        // A test started in between: step back out and wait for it.
        THREADS_IN_HOOKS.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Called by every gated thunk after `dispatch`.
pub extern "C" fn hook_leave() {
    let depth = HOOK_DEPTH.get() - 1;
    HOOK_DEPTH.set(depth);
    if depth == 0 {
        THREADS_IN_HOOKS.fetch_sub(1, Ordering::SeqCst);
    }
}

/// While this exists, no other thread enters a hooked function.
pub struct GamePaused(());

/// Keeps game threads out of hooked functions until the result is dropped, after waiting for
/// those already inside one to come out. The wait is limited: a thread that went into the
/// fatal-error reporter never comes out.
pub fn pause_game() -> GamePaused {
    TEST_THREAD.store(unsafe { GetCurrentThreadId() }, Ordering::SeqCst);
    for _ in 0..2000 {
        if THREADS_IN_HOOKS.load(Ordering::SeqCst) == 0 {
            break;
        }
        unsafe { Sleep(1) };
    }
    GamePaused(())
}

impl Drop for GamePaused {
    fn drop(&mut self) {
        TEST_THREAD.store(0, Ordering::SeqCst);
    }
}

/// Calls game code at `addr` with the registers in `regs`, and stores the registers and flags
/// it returns with back into `regs`: `call_original` for any address. For Rust versions that
/// have to call a game function that is not ported yet, and for self-tests that run a copy of
/// an original. `esp` in `regs` is ignored.
/// In the standalone program a hook's address is exchanged for its thunk first.
pub unsafe fn call_game(regs: *mut Regs, addr: usize) {
    let addr = CALL_REDIRECT
        .get()
        .and_then(|thunks| thunks.get(&addr).copied())
        .unwrap_or(addr);
    unsafe { call_at(regs, addr) }
}

/// `call_game` without the exchange.
#[unsafe(naked)]
unsafe extern "C" fn call_at(_regs: *mut Regs, _addr: usize) {
    core::arch::naked_asm!(
        "push ebx", "push esi", "push edi", "push ebp",
        "mov eax, [esp + 20]",           // eax = regs
        "push eax",                      // remember regs across the call
        "mov ecx, [eax + {o_ecx}]",
        "mov edx, [eax + {o_edx}]",
        "mov ebx, [eax + {o_ebx}]",
        "mov esi, [eax + {o_esi}]",
        "mov edi, [eax + {o_edi}]",
        "mov ebp, [eax + {o_ebp}]",
        "mov eax, [eax + {o_eax}]",
        "call dword ptr [esp + 28]",     // addr
        "pushfd",                        // the flags it returned with
        "cld",
        "push eax",
        "mov eax, [esp + 8]",            // eax = regs again
        "mov [eax + {o_ecx}], ecx",
        "mov [eax + {o_edx}], edx",
        "mov [eax + {o_ebx}], ebx",
        "mov [eax + {o_esi}], esi",
        "mov [eax + {o_edi}], edi",
        "mov [eax + {o_ebp}], ebp",
        "pop dword ptr [eax + {o_eax}]",
        "pop dword ptr [eax + {o_eflags}]",
        "add esp, 4",
        "pop ebp", "pop edi", "pop esi", "pop ebx",
        "ret",
        o_eax = const core::mem::offset_of!(Regs, eax),
        o_eflags = const core::mem::offset_of!(Regs, eflags),
        o_ecx = const core::mem::offset_of!(Regs, ecx),
        o_edx = const core::mem::offset_of!(Regs, edx),
        o_ebx = const core::mem::offset_of!(Regs, ebx),
        o_esi = const core::mem::offset_of!(Regs, esi),
        o_edi = const core::mem::offset_of!(Regs, edi),
        o_ebp = const core::mem::offset_of!(Regs, ebp),
    )
}

pub fn game_addr(rva: usize) -> usize {
    BASE.load(Ordering::Relaxed) + rva
}

/// Pointer to game memory at `rva`.
pub fn game_ptr<T>(rva: usize) -> *mut T {
    game_addr(rva) as *mut T
}

/// Reads a game global. Call it on every use: the game changes these at runtime.
pub unsafe fn game_read<T: Copy>(rva: usize) -> T {
    let p = game_ptr::<T>(rva);
    // Some game tables sit on odd addresses.
    unsafe {
        if p.is_aligned() {
            p.read_volatile()
        } else {
            p.read_unaligned()
        }
    }
}

/// Writes a game global.
pub unsafe fn game_write<T: Copy>(rva: usize, value: T) {
    let p = game_ptr::<T>(rva);
    unsafe {
        if p.is_aligned() {
            p.write_volatile(value)
        } else {
            p.write_unaligned(value)
        }
    }
}

pub fn log(msg: &str) {
    if let Some(out) = CONSOLE_OUT.get()
        && let Ok(mut f) = out.lock()
    {
        let _ = writeln!(f, "{msg}");
    }

    if let Some(path) = LOG_PATH.get()
        && let Ok(mut f) = OpenOptions::new().create(true).append(true).open(path)
    {
        let _ = writeln!(f, "{msg}");
    }
}

/// The registers at a hook: what the thunk's `pushad` and `pushfd` leave on the stack.
#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct Regs {
    pub edi: u32,
    pub esi: u32,
    pub ebp: u32,
    pub esp: u32,
    pub ebx: u32,
    pub edx: u32,
    pub ecx: u32,
    pub eax: u32,
    /// In `dispatch`: the caller's flags, restored on return unless changed. After
    /// `call_original`: the flags the original returned with. Never loaded before a call.
    pub eflags: u32,
}

pub const CARRY_FLAG: u32 = 1 << 0;
pub const ZERO_FLAG: u32 = 1 << 6;

pub const EAX: u8 = 1 << 0;
pub const ECX: u8 = 1 << 1;
pub const EDX: u8 = 1 << 2;
pub const EBX: u8 = 1 << 3;
pub const EBP: u8 = 1 << 4;
pub const ESI: u8 = 1 << 5;
pub const EDI: u8 = 1 << 6;

impl Regs {
    /// A recognisable value in every register, so a self-test can see which ones survive.
    pub const SENTINEL: Regs = Regs {
        edi: 0x1111_1111,
        esi: 0x2222_2222,
        ebp: 0x3333_3333,
        esp: 0,
        ebx: 0x4444_4444,
        edx: 0x5555_5555,
        ecx: 0x6666_6666,
        eax: 0x7777_7777,
        eflags: 0,
    };

    pub fn carry(&self) -> bool {
        self.eflags & CARRY_FLAG != 0
    }

    /// Sets the carry flag the game sees when the hook returns.
    pub fn set_carry(&mut self, on: bool) {
        self.eflags = (self.eflags & !CARRY_FLAG) | if on { CARRY_FLAG } else { 0 };
    }

    /// Sets the zero flag the game sees when the hook returns.
    pub fn set_zero(&mut self, on: bool) {
        self.eflags = (self.eflags & !ZERO_FLAG) | if on { ZERO_FLAG } else { 0 };
    }

    /// True if every register equals `before`, apart from those in `may_change`
    /// (a mask of `EAX`, `EBX`, ...). `esp` is never compared.
    pub fn preserved_except(&self, before: &Regs, may_change: u8) -> bool {
        [
            (EAX, self.eax, before.eax),
            (ECX, self.ecx, before.ecx),
            (EDX, self.edx, before.edx),
            (EBX, self.ebx, before.ebx),
            (EBP, self.ebp, before.ebp),
            (ESI, self.esi, before.esi),
            (EDI, self.edi, before.edi),
        ]
        .iter()
        .all(|&(bit, now, was)| may_change & bit != 0 || now == was)
    }
}

/// Case and failure counting for a hook's self-test. Only failures are logged.
pub struct SelfTest {
    name: &'static str,
    cases: usize,
    failures: usize,
}
impl SelfTest {
    /// None (after logging one SKIPPED line) if the hook has no original to compare against.
    pub fn begin(hook: &RawHook) -> Option<Self> {
        let t = Self {
            name: hook.name,
            cases: 0,
            failures: 0,
        };
        if hook.has_original() {
            Some(t)
        } else {
            t.skip("original not available");
            None
        }
    }

    /// Logs one SKIPPED line and reports no cases.
    pub fn skip(self, reason: &str) -> (usize, usize) {
        log(&format!("SELFTEST SKIPPED {}: {reason}", self.name));
        (0, 0)
    }

    /// Counts one case; on failure logs `detail` (at most `MAX_LOGGED` times).
    pub fn check(&mut self, ok: bool, detail: impl FnOnce() -> String) {
        self.cases += 1;
        if !ok {
            self.failures += 1;
            if self.failures <= MAX_LOGGED {
                log(&format!("SELFTEST FAIL {}: {}", self.name, detail()));
            }
        }
    }

    /// (cases, failures)
    pub fn finish(self) -> (usize, usize) {
        (self.cases, self.failures)
    }
}

macro_rules! hook_asm {
    // For a hook the game enters while holding a lock its own self-test needs (the log tail):
    // the thunk does not wait for self-tests. Pair it with `.self_test_unpaused()`.
    ($thunk:ident, $call_original:ident, dispatch = $dispatch:ident, original = $orig:ident, ungated) => {
        /// Detour target: the game calls this believing it is the original function.
        #[unsafe(naked)]
        pub(crate) unsafe extern "C" fn $thunk() {
            core::arch::naked_asm!(
                "pushfd",              // Regs.eflags
                "pushad",
                "mov eax, esp",
                "push eax",            // arg: *mut Regs
                "cld",                 // Rust code expects the direction flag clear
                "call {dispatch}",
                "add esp, 4",
                "popad",
                "popfd",               // the caller's flags, or what dispatch set
                "ret",
                dispatch = sym $dispatch,
            )
        }

        $crate::framework::hook_asm!(@call_original $call_original, $orig);
    };

    ($thunk:ident, $call_original:ident, dispatch = $dispatch:ident, original = $orig:ident
     $(, epilogue = $epilogue:literal)?) => {
        /// Detour target: the game calls this believing it is the original function.
        #[unsafe(naked)]
        pub(crate) unsafe extern "C" fn $thunk() {
            core::arch::naked_asm!(
                "pushfd",              // Regs.eflags
                "pushad",
                "cld",                 // Rust code expects the direction flag clear
                "call {enter}",        // waits here while a self-test runs
                "mov eax, esp",
                "push eax",            // arg: *mut Regs
                "call {dispatch}",
                "add esp, 4",
                "call {leave}",
                "popad",
                "popfd",               // the caller's flags, or what dispatch set
                $($epilogue,)?         // e.g. "test esi, esi" when the caller reads ZF
                "ret",
                dispatch = sym $dispatch,
                enter = sym $crate::framework::hook_enter,
                leave = sym $crate::framework::hook_leave,
            )
        }

        $crate::framework::hook_asm!(@call_original $call_original, $orig);
    };

    (@call_original $call_original:ident, $orig:ident) => {
        /// Runs the original with the registers in `regs`; stores the resulting registers back.
        #[unsafe(naked)]
        pub(crate) unsafe extern "C" fn $call_original(_regs: *mut $crate::framework::Regs) {
            core::arch::naked_asm!(
                "push ebx", "push esi", "push edi", "push ebp",
                "mov eax, [esp + 20]",           // eax = regs
                "push eax",                      // remember regs across the call
                "mov ecx, [eax + {o_ecx}]",
                "mov edx, [eax + {o_edx}]",
                "mov ebx, [eax + {o_ebx}]",
                "mov esi, [eax + {o_esi}]",
                "mov edi, [eax + {o_edi}]",
                "mov ebp, [eax + {o_ebp}]",
                "mov eax, [eax + {o_eax}]",
                "call dword ptr [{t}]",
                "pushfd",                        // the flags the original returned with
                "cld",
                "push eax",
                "mov eax, [esp + 8]",            // eax = regs again
                "mov [eax + {o_ecx}], ecx",
                "mov [eax + {o_edx}], edx",
                "mov [eax + {o_ebx}], ebx",
                "mov [eax + {o_esi}], esi",
                "mov [eax + {o_edi}], edi",
                "mov [eax + {o_ebp}], ebp",
                "pop dword ptr [eax + {o_eax}]", // the original's eax
                "pop dword ptr [eax + {o_eflags}]",
                "add esp, 4",
                "pop ebp", "pop edi", "pop esi", "pop ebx",
                "ret",
                t = sym $orig,
                o_eax = const core::mem::offset_of!($crate::framework::Regs, eax),
                o_eflags = const core::mem::offset_of!($crate::framework::Regs, eflags),
                o_ecx = const core::mem::offset_of!($crate::framework::Regs, ecx),
                o_edx = const core::mem::offset_of!($crate::framework::Regs, edx),
                o_ebx = const core::mem::offset_of!($crate::framework::Regs, ebx),
                o_esi = const core::mem::offset_of!($crate::framework::Regs, esi),
                o_edi = const core::mem::offset_of!($crate::framework::Regs, edi),
                o_ebp = const core::mem::offset_of!($crate::framework::Regs, ebp),
            )
        }
    };
}

pub(crate) use hook_asm;
