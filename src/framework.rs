#![allow(unsafe_code)]
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

/// RVA of an address as IDA shows it (the tests).
#[cfg(test)]
pub const fn rva(addr: usize) -> usize {
    addr - IMAGE_BASE
}

#[cfg(test)]
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

/// Pointer to game memory at `rva` (the x86 reference tests).
#[cfg(all(test, target_arch = "x86"))]
pub fn game_ptr<T>(rva: usize) -> *mut T {
    crate::pointers::host(game_addr(rva)).cast::<T>()
}

/// Reads a game global (the x86 reference tests).
#[cfg(all(test, target_arch = "x86"))]
pub fn game_read<T: Copy>(rva: usize) -> T {
    let p = game_ptr::<T>(rva);
    crate::pointers::check_or_end(
        crate::pointers::game_address(p),
        size_of::<T>(),
        "a game global",
    );
    // SAFETY: inside one live region (checked above). Some game tables sit on odd addresses.
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
    // SAFETY: as the read's.
    unsafe {
        if p.is_aligned() {
            p.write_volatile(value);
        } else {
            p.write_unaligned(value);
        }
    }
}

pub fn log(msg: &str) {
    if let Some(path) = LOG_PATH.get()
        && let Ok(mut f) = OpenOptions::new().create(true).append(true).open(path)
    {
        let _ = writeln!(f, "{msg}");
    }
    // The web build: the browser's console shows the log as it goes (the file is in the
    // origin's storage, out of sight).
    #[cfg(target_os = "emscripten")]
    eprintln!("{msg}");
}

/// The carry and zero flags a function leaves, for Rust callers that still read them
/// (conversion phase 3: a function whose callers hold registers returns these, and they
/// go into the callers' registers with `apply`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Flags {
    pub carry: bool,
    pub zero: bool,
}

impl Flags {}
