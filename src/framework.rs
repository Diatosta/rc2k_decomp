use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;
#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};

/// Where a test's memory stands for addresses with no home (`game_addr`).
#[cfg(test)]
pub static BASE: AtomicUsize = AtomicUsize::new(0);
/// The program's log (`rc2k.log`), set by `standalone::run`. Nothing is written before that.
pub static LOG_PATH: OnceLock<PathBuf> = OnceLock::new();

pub const IMAGE_BASE: usize = 0x400000;

/// RVA of an address as IDA shows it.
pub const fn rva(addr: usize) -> usize {
    addr - IMAGE_BASE
}

/// The address of the game's variable at `rva`: in the image, or for a variable that has
/// left it (phase 5, `crate::moved`) its new home, so an address the code still names by its
/// RVA reaches the variable wherever it lives (owner, 2026-10-24: the resolver).
pub fn game_addr(rva: usize) -> u32 {
    match crate::moved::home_of_rva(rva) {
        Some(home) => home as u32,
        // No home: the number as IDA shows it, which no allocation holds, so a read of it ends
        // the program naming it.
        #[cfg(not(test))]
        None => (IMAGE_BASE + rva) as u32,
        #[cfg(test)]
        None => (BASE.load(Ordering::Relaxed) + rva) as u32,
    }
}

/// Pointer to game memory at `rva`.
pub fn game_ptr<T>(rva: usize) -> *mut T {
    crate::pointers::host(game_addr(rva)) as *mut T
}

/// Reads a game global. Call it on every use: the game changes these at runtime.
pub fn game_read<T: Copy>(rva: usize) -> T {
    let p = game_ptr::<T>(rva);
    crate::pointers::check_or_end(
        crate::pointers::game_address(p),
        size_of::<T>(),
        "a game global",
    );
    // Some game tables sit on odd addresses.
    unsafe {
        if p.is_aligned() {
            p.read_volatile()
        } else {
            p.read_unaligned()
        }
    }
}

/// Writes a game global (the tests' setups).
#[cfg(all(test, target_arch = "x86"))]
pub fn game_write<T: Copy>(rva: usize, value: T) {
    let p = game_ptr::<T>(rva);
    crate::pointers::check_or_end(
        crate::pointers::game_address(p),
        size_of::<T>(),
        "a game global",
    );
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

/// The registers of a place in the original (`pushad` order and the flags): what a
/// fatal error's report shows.
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
    pub eflags: u32,
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
