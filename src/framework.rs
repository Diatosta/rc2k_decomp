use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};

pub static BASE: AtomicUsize = AtomicUsize::new(0);
/// The program's log (`rc2k.log`), set by `standalone::run`. Nothing is written before that.
pub static LOG_PATH: OnceLock<PathBuf> = OnceLock::new();

pub const IMAGE_BASE: usize = 0x400000;

/// RVA of an address as IDA shows it.
pub const fn rva(addr: usize) -> usize {
    addr - IMAGE_BASE
}

pub trait GameHook: Sync {
    fn name(&self) -> &'static str;
    fn rva(&self) -> usize;
    fn stats(&self) -> &HookStats;
}

/// A function of the game's that the platform (or the fatal reporter) stands for: its
/// name, its address (for the menus' and Rust's calls by address, `platform::by_address`)
/// and its calls counted. Nothing runs at the address itself (2026-10-24).
pub struct RawHook {
    name: &'static str,
    rva: usize,
    pub stats: HookStats,
}
impl RawHook {
    pub const fn new(name: &'static str, rva: usize) -> Self {
        Self {
            name,
            rva,
            stats: HookStats::new(),
        }
    }

    /// Counts one call. First line of every `dispatch`.
    pub fn count_call(&self) {
        self.stats.calls.fetch_add(1, Ordering::Relaxed);
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
}

pub struct HookStats {
    pub calls: AtomicUsize,
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
        }
    }
}

/// The address of the game's variable at `rva`: in the image, or for a variable that has
/// left it (phase 5, `crate::moved`) its new home, so an address the code still names by its
/// RVA reaches the variable wherever it lives (owner, 2026-10-24: the resolver).
pub fn game_addr(rva: usize) -> usize {
    match crate::moved::home_of_rva(rva) {
        Some(home) => home,
        None => BASE.load(Ordering::Relaxed) + rva,
    }
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
}

/// The carry and zero flags a function leaves, for Rust callers that still read them
/// (conversion phase 3: a function whose callers hold registers returns these, and they
/// go into the callers' `Regs` with `apply`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Flags {
    pub carry: bool,
    pub zero: bool,
}

impl Flags {}
