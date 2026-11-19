//! libc behind safe functions, for `os/unix.rs` (the Unix build's system facts): file system
//! and memory statistics, the system's name and the host's, anonymous page mapping and
//! ending the process.
//!
//! Linux and macOS share most of it; where they differ each has its own version: memory
//! (`sysinfo` / `sysctl`), discarded pages (`madvise` zeroes them
//! on Linux only; macOS maps them anew) and the fault's registers (`ucontext_t`).
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

/// The machine's memory figures, in bytes.
pub struct MemoryStats {
    pub free_ram: u64,
    pub total_ram: u64,
    pub free_swap: u64,
    pub total_swap: u64,
}

/// `sysinfo` with its `mem_unit` applied (saturating); None when it fails.
#[cfg(any(target_os = "linux", target_os = "android"))]
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

/// The value of `sysctlbyname(name)`, of type `T`; None when it fails or has another size.
#[cfg(apple)]
fn sysctl_value<T: Copy>(name: &CStr) -> Option<T> {
    let mut value = std::mem::MaybeUninit::<T>::uninit();
    let mut size = size_of::<T>();
    // SAFETY: a NUL-ended name and an out-parameter of `size` bytes.
    let read = unsafe {
        libc::sysctlbyname(
            name.as_ptr(),
            value.as_mut_ptr().cast(),
            &raw mut size,
            std::ptr::null_mut(),
            0,
        )
    };
    // SAFETY: the system wrote all `size_of::<T>()` bytes (checked).
    (read == 0 && size == size_of::<T>()).then(|| unsafe { value.assume_init() })
}

/// The memory figures from `sysctl`: the machine's memory (`hw.memsize`), its free pages
/// (`vm.page_free_count`, Activity Monitor's "free") and the swap files' use
/// (`vm.swapusage`); None when one cannot be read.
#[cfg(apple)]
pub fn memory_stats() -> Option<MemoryStats> {
    let total_ram: u64 = sysctl_value(c"hw.memsize")?;
    let free_pages: u32 = sysctl_value(c"vm.page_free_count")?;
    let page_size: u32 = sysctl_value(c"vm.pagesize")?;
    let swap: libc::xsw_usage = sysctl_value(c"vm.swapusage")?;
    Some(MemoryStats {
        free_ram: u64::from(free_pages).saturating_mul(u64::from(page_size)),
        total_ram,
        free_swap: swap.xsu_avail,
        total_swap: swap.xsu_total,
    })
}

/// A C text field of a fixed size, to its first NUL.
fn field_text(field: &[libc::c_char]) -> String {
    let bytes: Vec<u8> = field
        .iter()
        .map(|&c| c.to_ne_bytes()[0])
        .take_while(|&b| b != 0)
        .collect();
    String::from_utf8_lossy(&bytes).into_owned()
}

/// The system's name and its release (`uname`: "Linux", "6.17.1"; "Darwin", "25.0.0").
pub fn system_release() -> Option<(String, String)> {
    // SAFETY: all zeros is a valid value of this plain C struct.
    let mut names: libc::utsname = unsafe { std::mem::zeroed() };
    // SAFETY: an out-parameter of its type.
    if unsafe { libc::uname(&raw mut names) } != 0 {
        return None;
    }
    Some((field_text(&names.sysname), field_text(&names.release)))
}

/// The machine's host name (`gethostname`).
pub fn host_name() -> Option<String> {
    let mut name = [0 as libc::c_char; 256];
    // SAFETY: a buffer of `name.len()` bytes, of which the call writes at most one less: the
    // last stays NUL.
    if unsafe { libc::gethostname(name.as_mut_ptr(), name.len() - 1) } != 0 {
        return None;
    }
    Some(field_text(&name))
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
            libc::MAP_PRIVATE | libc::MAP_ANON,
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
#[cfg(any(target_os = "linux", target_os = "android"))]
pub unsafe fn discard_pages(at: *mut u8, len: usize) -> bool {
    // SAFETY: the caller's mapping; private anonymous pages read zero after this.
    unsafe { libc::madvise(at.cast(), len, libc::MADV_DONTNEED) == 0 }
}

/// As on Linux, but macOS's `madvise` frees pages without zeroing them: new anonymous pages
/// are mapped over the old ones instead (`MAP_FIXED`: the same addresses, zeroed, taking no
/// memory until touched).
///
/// # Safety
/// `at` and `len` are a mapping of `map_pages` (whole pages).
#[cfg(apple)]
pub unsafe fn discard_pages(at: *mut u8, len: usize) -> bool {
    // SAFETY: the caller's mapping, replaced in place by one of the same kind.
    let mapped = unsafe {
        libc::mmap(
            at.cast(),
            len,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANON | libc::MAP_FIXED,
            -1,
            0,
        )
    };
    mapped == at.cast()
}

/// Elsewhere (the web build) the pages stay as they are: nothing is handed back, and the
/// pool clears a block itself before it is used again.
///
/// # Safety
/// `at` and `len` are a mapping of `map_pages` (whole pages).
#[cfg(not(any(target_os = "linux", target_os = "android", apple)))]
pub unsafe fn discard_pages(_at: *mut u8, _len: usize) -> bool {
    false
}

/// The process ended at once with `code` (no library's leave-taking).
pub fn exit_now(code: u32) -> ! {
    // The browser's page is told (its `onExit`): `_exit` would leave it with the last frame.
    #[cfg(web)]
    crate::platform::web::end_program(code);
    #[cfg(not(web))]
    // SAFETY: ends the process.
    unsafe {
        libc::_exit(code as i32)
    }
}

// ---- the fault reporter ----

/// The process is ended by SIGALRM (its default action) after this many seconds. Async-signal
/// safe: it touches no memory of ours.
pub fn alarm(seconds: u32) {
    // SAFETY: `alarm` is async-signal-safe and touches no memory of ours.
    unsafe {
        libc::alarm(seconds);
    }
}

/// `f` given the module a host address is in: its path's bytes (the loader's, lent for the
/// call) and its load address; None when the loader knows none. Allocates nothing, for the
/// fault reporter (`dladdr` itself may take the loader's lock: the reporter's time limit
/// covers a fault that holds it).
pub fn with_module_of_address<T>(address: usize, f: impl FnOnce(&[u8], usize) -> T) -> Option<T> {
    // SAFETY: `address` is only looked up; the record's name is the loader's, NUL-ended and
    // live while the module is loaded, which it is over the call.
    unsafe {
        let mut found: libc::Dl_info = std::mem::zeroed();
        if libc::dladdr(std::ptr::with_exposed_provenance(address), &raw mut found) == 0
            || found.dli_fname.is_null()
        {
            return None;
        }
        let path = CStr::from_ptr(found.dli_fname).to_bytes();
        Some(f(path, found.dli_fbase.expose_provenance()))
    }
}

/// The signal handler (SA_SIGINFO): hands the fault's places to `standalone::report_fault`,
/// which ends the program. Reads the system's records only; what the report calls is listed
/// at `report_fault`.
///
/// # Safety
/// The system's signal handler: `info` and `context` are the system's, live during the call.
#[cfg(not(web))]
unsafe extern "C" fn on_signal(
    _signal: libc::c_int,
    info: *mut libc::siginfo_t,
    context: *mut libc::c_void,
) {
    // SAFETY: the records as the system hands them (the function's contract).
    unsafe {
        let data = (*info).si_addr().expose_provenance();
        let (ip, write, sp) = fault_registers(context.cast::<libc::ucontext_t>());
        crate::standalone::report_fault(ip, data, write, sp)
    }
}

/// The fault's instruction pointer, whether it was a write (not known here), and the stack
/// pointer, from the signal's context.
///
/// # Safety
/// `context` is the signal handler's, live during the call.
#[cfg(all(
    any(target_os = "linux", target_os = "android"),
    target_arch = "aarch64"
))]
unsafe fn fault_registers(context: *const libc::ucontext_t) -> (usize, bool, usize) {
    // SAFETY: the caller's context.
    let c = unsafe { &(*context).uc_mcontext };
    (c.pc as usize, false, c.sp as usize)
}

/// The fault's instruction pointer, whether it was a write (the page fault's error code has
/// bit 1 set), and the stack pointer, from the signal's context.
///
/// # Safety
/// `context` is the signal handler's, live during the call.
#[cfg(all(
    any(target_os = "linux", target_os = "android"),
    not(target_arch = "aarch64")
))]
unsafe fn fault_registers(context: *const libc::ucontext_t) -> (usize, bool, usize) {
    // SAFETY: the caller's context.
    let regs = unsafe { &(*context).uc_mcontext.gregs };
    #[cfg(target_arch = "x86_64")]
    let (ip, sp) = (libc::REG_RIP, libc::REG_RSP);
    #[cfg(target_arch = "x86")]
    let (ip, sp) = (libc::REG_EIP, libc::REG_ESP);
    let write = regs[libc::REG_ERR as usize] & 2 != 0;
    (
        regs[ip as usize] as usize,
        write,
        regs[sp as usize] as usize,
    )
}

/// The fault's instruction pointer, whether it was a write (the page fault's error code has
/// bit 1 set), and the stack pointer, from macOS's machine context (`uc_mcontext` points to
/// it).
///
/// # Safety
/// `context` is the signal handler's, live during the call.
#[cfg(all(apple, target_arch = "x86_64"))]
unsafe fn fault_registers(context: *const libc::ucontext_t) -> (usize, bool, usize) {
    // SAFETY: the caller's context and the machine context it points to.
    let m = unsafe { &*(*context).uc_mcontext };
    (
        m.__ss.__rip as usize,
        m.__es.__err & 2 != 0,
        m.__ss.__rsp as usize,
    )
}

/// The fault's instruction pointer, whether it was a write (a data abort's syndrome has bit
/// 6, WnR, set), and the stack pointer, from macOS's machine context (`uc_mcontext` points
/// to it).
///
/// # Safety
/// `context` is the signal handler's, live during the call.
#[cfg(all(apple, target_arch = "aarch64"))]
unsafe fn fault_registers(context: *const libc::ucontext_t) -> (usize, bool, usize) {
    // SAFETY: the caller's context and the machine context it points to.
    let m = unsafe { &*(*context).uc_mcontext };
    (
        m.__ss.__pc as usize,
        m.__es.__esr & (1 << 6) != 0,
        m.__ss.__sp as usize,
    )
}

/// The word at `at` on this thread's stack, for the fault report (`standalone::stack_returns`):
/// None where no memory is mapped, so a walk up the stack stops at the stack's end instead of
/// faulting again in the handler.
///
/// The kernel reads it (`process_vm_readv` on the process itself), which answers `EFAULT`
/// for an address with nothing mapped: one system call, safe in a signal handler. The
/// thread's stack range cannot be asked there (`pthread_getattr_np` allocates, and for the
/// main thread reads `/proc/self/maps`), and the threads SDL starts have no start of ours to
/// note it at; the mapping's end is where the stack's end faulted.
#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn own_stack_word(at: usize) -> Option<usize> {
    let mut word = 0usize;
    let local = libc::iovec {
        iov_base: (&raw mut word).cast(),
        iov_len: size_of::<usize>(),
    };
    let remote = libc::iovec {
        iov_base: std::ptr::without_provenance_mut(at),
        iov_len: size_of::<usize>(),
    };
    // SAFETY: the kernel writes at most `local`'s one word, `word`, a local of this frame; the
    // remote address is only read by the kernel, which checks it (an unmapped one fails with
    // `EFAULT`, no fault). `getpid` and `process_vm_readv` take no lock and allocate nothing.
    let read = unsafe {
        libc::process_vm_readv(libc::getpid(), &raw const local, 1, &raw const remote, 1, 0)
    };
    (read == size_of::<usize>() as isize).then_some(word)
}

/// The word at `at` on this thread's stack, for the fault report (`standalone::stack_returns`):
/// None outside the thread's stack (`pthread_get_stackaddr_np`, its top, and
/// `pthread_get_stacksize_np` below it), so a walk up the stack stops at its end.
#[cfg(apple)]
pub fn own_stack_word(at: usize) -> Option<usize> {
    // SAFETY: plain reads of the running thread's own record (no lock, no allocation in
    // Apple's libpthread; they are not on POSIX's async-signal-safe list, being Apple's own).
    let (top, size) = unsafe {
        let thread = libc::pthread_self();
        (
            libc::pthread_get_stackaddr_np(thread) as usize,
            libc::pthread_get_stacksize_np(thread),
        )
    };
    let low = top.saturating_sub(size);
    let inside = at >= low
        && at
            .checked_add(size_of::<usize>())
            .is_some_and(|end| end <= top);
    // SAFETY: a word inside the thread's stack, `low .. top`, which is mapped whole (a pthread's
    // stack is one mapping, its guard below `low`; the main thread's stack region is reserved
    // whole and reads as zeros where untouched). The handler ends the process, so the frames
    // read are never run again; the read needs no alignment.
    inside.then(|| unsafe { std::ptr::with_exposed_provenance::<usize>(at).read_unaligned() })
}

/// No fault report in the web build (`install_fault_reporter` installs nothing).
#[cfg(web)]
pub fn own_stack_word(_at: usize) -> Option<usize> {
    None
}

/// The fault reporter installed for SIGSEGV and SIGBUS (on the thread's alternate stack,
/// which std gives its threads, so a stack overflow is reported too).
#[cfg(not(web))]
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
#[cfg(web)]
pub fn memory_stats() -> Option<MemoryStats> {
    None
}

/// No signals for a fault: a trap ends the module, and the browser's console shows it.
#[cfg(web)]
pub fn install_fault_reporter() {}

// ---- iOS: the main thread's run loop ----

#[cfg(target_os = "ios")]
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    static kCFRunLoopDefaultMode: *const std::ffi::c_void;
    fn CFRunLoopRunInMode(
        mode: *const std::ffi::c_void,
        seconds: f64,
        return_after_source_handled: u8,
    ) -> i32;
}

/// Waits `wait` inside the calling thread's run loop (its default mode), so that the system's
/// work queued for that thread (touches, display updates) is done while the program waits.
#[cfg(target_os = "ios")]
pub fn run_loop_for(wait: std::time::Duration) {
    // SAFETY: `kCFRunLoopDefaultMode` is a constant string CoreFoundation defines for the
    // program's lifetime; running the current thread's run loop for a time has no other input.
    unsafe {
        CFRunLoopRunInMode(kCFRunLoopDefaultMode, wait.as_secs_f64(), 0);
    }
}

/// Whether the calling thread is the process's main thread.
#[cfg(target_os = "ios")]
pub fn on_main_thread() -> bool {
    // SAFETY: `pthread_main_np` takes nothing and only reads the calling thread's identity.
    unsafe { libc::pthread_main_np() != 0 }
}
