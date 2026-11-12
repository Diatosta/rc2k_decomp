//! libc behind safe functions, for `os/unix.rs` (the Unix build's system facts): file system
//! and memory statistics, anonymous page mapping, ending the process, and the running
//! thread's stack and the program's loaded segments from the loader.
//!
//! Out-parameters are local to each call and returned as plain Rust values. The raw-pointer
//! items are `map_pages` and `discard_pages`, the page pool's memory primitives (the pool in
//! `pool.rs` holds the mapping for the program's life). Nothing allocates in
//! `exit_now` (it may run in the fault reporter).

#![allow(unsafe_code)]

use std::ffi::CStr;

/// A file system's counts, in units of `block_size` bytes.
pub struct FileSystemStats {
    pub block_size: u64,
    pub blocks_available: u64,
    pub blocks_total: u64,
}

/// `statvfs` of `path`; None when it fails.
pub fn file_system_stats(path: &CStr) -> Option<FileSystemStats> {
    // SAFETY: all zeros is a valid value of this plain C struct.
    let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
    // SAFETY: a NUL-ended path and an out-parameter of its type.
    if unsafe { libc::statvfs(path.as_ptr(), &raw mut stat) } != 0 {
        return None;
    }
    Some(FileSystemStats {
        block_size: stat.f_frsize as u64,
        blocks_available: stat.f_bavail as u64,
        blocks_total: stat.f_blocks as u64,
    })
}

/// `sysinfo`'s memory figures, in bytes.
pub struct MemoryStats {
    pub free_ram: u64,
    pub total_ram: u64,
    pub free_swap: u64,
    pub total_swap: u64,
}

/// `sysinfo` with its `mem_unit` applied (saturating); None when it fails.
#[cfg(not(target_os = "emscripten"))]
pub fn memory_stats() -> Option<MemoryStats> {
    // SAFETY: all zeros is a valid value of this plain C struct.
    let mut info: libc::sysinfo = unsafe { std::mem::zeroed() };
    // SAFETY: an out-parameter of its type.
    if unsafe { libc::sysinfo(&raw mut info) } != 0 {
        return None;
    }
    let bytes = |n: u64| n.saturating_mul(info.mem_unit as u64);
    Some(MemoryStats {
        free_ram: bytes(info.freeram as u64),
        total_ram: bytes(info.totalram as u64),
        free_swap: bytes(info.freeswap as u64),
        total_swap: bytes(info.totalswap as u64),
    })
}

/// `len` bytes of zeroed, readable and writable pages for the program's life (a pool block):
/// their first byte, or None. Pages not yet touched take no memory.
pub fn map_pages(len: usize) -> Option<*mut u8> {
    // SAFETY: a new anonymous mapping; nothing is passed in.
    let at = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            len,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
            -1,
            0,
        )
    };
    (at != libc::MAP_FAILED).then_some(at.cast())
}

/// The pages of `map_pages`' `len` bytes at `at` handed back to the system, still mapped:
/// their memory is freed and they read as zeros when next touched. Whether the pages came
/// back zeroed (they do here).
///
/// # Safety
/// `at` and `len` are a mapping of `map_pages` (whole pages).
pub unsafe fn discard_pages(at: *mut u8, len: usize) -> bool {
    // SAFETY: the caller's mapping; private anonymous pages read zero after this.
    unsafe { libc::madvise(at.cast(), len, libc::MADV_DONTNEED) == 0 }
}

/// The process ended at once with `code` (no library's leave-taking).
pub fn exit_now(code: u32) -> ! {
    // SAFETY: ends the process.
    unsafe { libc::_exit(code as i32) }
}

// ---- the fault reporter (`standalone`) ----

/// The process is ended by SIGALRM (its default action) after this many seconds. Async-signal
/// safe: it touches no memory of ours.
pub fn alarm(seconds: u32) {
    // SAFETY: `alarm` is async-signal-safe and touches no memory of ours.
    unsafe {
        libc::alarm(seconds);
    }
}

/// The module a host address is in: its path and its load address; None when the loader
/// knows none.
pub fn module_of_address(address: usize) -> Option<(String, usize)> {
    // SAFETY: `address` is only looked up; the record's name is the loader's, live while
    // the module is loaded.
    unsafe {
        let mut found: libc::Dl_info = std::mem::zeroed();
        if libc::dladdr(std::ptr::with_exposed_provenance(address), &raw mut found) == 0
            || found.dli_fname.is_null()
        {
            return None;
        }
        let path = CStr::from_ptr(found.dli_fname)
            .to_string_lossy()
            .into_owned();
        Some((path, found.dli_fbase.expose_provenance()))
    }
}

/// The signal handler (SA_SIGINFO): hands the fault's places to `standalone::report_fault`,
/// which ends the program.
///
/// # Safety
/// The system's signal handler: `info` and `context` are the system's, live during the call.
#[cfg(not(target_os = "emscripten"))]
unsafe extern "C" fn on_signal(
    _signal: libc::c_int,
    info: *mut libc::siginfo_t,
    context: *mut libc::c_void,
) {
    // SAFETY: the records as the system hands them (the function's contract).
    unsafe {
        let data = (*info).si_addr().expose_provenance();
        #[cfg(target_arch = "aarch64")]
        {
            let c = &(*context.cast::<libc::ucontext_t>()).uc_mcontext;
            crate::standalone::report_fault(c.pc as usize, data, false, c.sp as usize)
        }
        #[cfg(not(target_arch = "aarch64"))]
        let regs = &(*context.cast::<libc::ucontext_t>()).uc_mcontext.gregs;
        #[cfg(target_arch = "x86_64")]
        let (ip, sp) = (libc::REG_RIP, libc::REG_RSP);
        #[cfg(target_arch = "x86")]
        let (ip, sp) = (libc::REG_EIP, libc::REG_ESP);
        // The page fault's error code: bit 1 for a write.
        #[cfg(not(target_arch = "aarch64"))]
        let write = regs[libc::REG_ERR as usize] & 2 != 0;
        #[cfg(not(target_arch = "aarch64"))]
        crate::standalone::report_fault(
            regs[ip as usize] as usize,
            data,
            write,
            regs[sp as usize] as usize,
        )
    }
}

/// The fault reporter installed for SIGSEGV and SIGBUS (on the thread's alternate stack,
/// which std gives its threads, so a stack overflow is reported too).
#[cfg(not(target_os = "emscripten"))]
pub fn install_fault_reporter() {
    // SAFETY: a handler of the system's signature, for the program's life.
    unsafe {
        let mut action: libc::sigaction = std::mem::zeroed();
        action.sa_sigaction = on_signal as *const () as usize;
        action.sa_flags = libc::SA_SIGINFO | libc::SA_ONSTACK;
        libc::sigemptyset(&raw mut action.sa_mask);
        for signal in [libc::SIGSEGV, libc::SIGBUS] {
            libc::sigaction(signal, &raw const action, std::ptr::null_mut());
        }
    }
}

// ---- the web build (Emscripten): what a browser has no counterpart for ----

/// The browser says nothing of the machine's memory.
#[cfg(target_os = "emscripten")]
pub fn memory_stats() -> Option<MemoryStats> {
    None
}

/// No signals for a fault: a trap ends the module, and the browser's console shows it.
#[cfg(target_os = "emscripten")]
pub fn install_fault_reporter() {}
