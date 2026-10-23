use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};

pub static BASE: AtomicUsize = AtomicUsize::new(0);
/// The program's log (`rc2k.log`), set by `standalone::run`. Nothing is written before that.
pub static LOG_PATH: OnceLock<PathBuf> = OnceLock::new();
/// Set by `standalone.rs`: for a hook's address in the game's image, its thunk.
/// `call_game` goes there straight away; the image itself cannot be executed there.
pub static CALL_REDIRECT: OnceLock<std::collections::HashMap<usize, usize>> = OnceLock::new();

pub const IMAGE_BASE: usize = 0x400000;

/// RVA of an address as IDA shows it.
pub const fn rva(addr: usize) -> usize {
    addr - IMAGE_BASE
}

pub trait GameHook: Sync {
    fn name(&self) -> &'static str;
    fn rva(&self) -> usize;
    fn stats(&self) -> &HookStats;

    /// Makes `call_original` go to `addr` from now on. For `standalone.rs`, where there is
    /// no original to run: it names an address that faults and tells which hook asked.
    fn set_original(&self, addr: usize);

    /// The hook's thunk: `standalone.rs` sends calls of the hooked address there.
    fn thunk(&self) -> usize;
}

/// A function of the game's image whose calls run Rust code: `standalone.rs` sends a
/// call of its address to its thunk (no code is patched; the image is data only).
pub struct RawHook {
    name: &'static str,
    rva: usize,
    pub stats: HookStats,
    thunk: unsafe extern "C" fn(),
    /// Where `call_original` jumps to, set by `standalone.rs`.
    original: &'static AtomicUsize,
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
    fn set_original(&self, addr: usize) {
        self.original.store(addr, Ordering::SeqCst);
    }
    fn thunk(&self) -> usize {
        self.thunk as usize
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

/// Calls game code at `addr` with the registers in `regs`, and stores the registers and flags
/// it returns with back into `regs`: `call_original` for any address. `esp` in `regs` is
/// ignored. A hook's address is exchanged for its thunk first; any other address of the
/// game's image faults (no code runs there) and ends the program.
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

/// A hook's thunk (`hook_asm!(thunk, dispatch = dispatch)`): where `standalone.rs` sends a
/// call of the hooked address. It saves the registers and flags as `Regs`, calls `dispatch`
/// with them and returns with what `dispatch` left there (`epilogue = "..."`: instructions
/// run before the `ret`, e.g. "test esi, esi" when the caller reads ZF).
macro_rules! hook_asm {
    ($thunk:ident, dispatch = $dispatch:ident $(, epilogue = $epilogue:literal)?) => {
        #[unsafe(naked)]
        pub(crate) unsafe extern "C" fn $thunk() {
            core::arch::naked_asm!(
                "pushfd",              // Regs.eflags
                "pushad",
                "cld",                 // Rust code expects the direction flag clear
                "mov eax, esp",
                "push eax",            // arg: *mut Regs
                "call {dispatch}",
                "add esp, 4",
                "popad",
                "popfd",               // the caller's flags, or what dispatch set
                $($epilogue,)?
                "ret",
                dispatch = sym $dispatch,
            )
        }
    };
}

pub(crate) use hook_asm;
