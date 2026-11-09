//! libc behind safe functions, for `os/unix.rs` (the Unix build's system facts): file system
//! and memory statistics, the system's name and the host's, anonymous page mapping, ending
//! the process, and the running thread's stack and the program's loaded segments from the
//! loader.
//!
//! Linux and macOS share most of it; where they differ each has its own version: memory
//! (`sysinfo` / `sysctl`), the stack (`pthread_getattr_np` / `pthread_get_stack*_np`), the
//! image (`dl_iterate_phdr` / dyld's Mach-O header), discarded pages (`madvise` zeroes them
//! on Linux only; macOS maps them anew) and the fault's registers (`ucontext_t`).
//!
//! Out-parameters are local to each call and returned as plain Rust values. The raw-pointer
//! items are `map_pages` and `discard_pages`, the page pool's memory primitives (the pool in
//! `pointers/pool.rs` holds the mapping for the program's life). Nothing allocates in
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
#[cfg(target_os = "linux")]
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
#[cfg(target_os = "macos")]
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
#[cfg(target_os = "macos")]
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
#[cfg(target_os = "linux")]
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
#[cfg(target_os = "macos")]
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

/// The process ended at once with `code` (no library's leave-taking).
pub fn exit_now(code: u32) -> ! {
    // SAFETY: ends the process.
    unsafe { libc::_exit(code as i32) }
}

/// The running thread's stack: its lowest byte and the byte past its top.
#[cfg(all(not(miri), target_os = "linux"))]
pub fn thread_stack() -> (usize, usize) {
    // SAFETY: the running thread's attributes, read and destroyed here.
    unsafe {
        let mut attr: libc::pthread_attr_t = std::mem::zeroed();
        if libc::pthread_getattr_np(libc::pthread_self(), &raw mut attr) != 0 {
            return (0, 0);
        }
        let (mut low, mut size) = (std::ptr::null_mut(), 0usize);
        let read = libc::pthread_attr_getstack(&raw const attr, &raw mut low, &raw mut size);
        libc::pthread_attr_destroy(&raw mut attr);
        if read != 0 {
            return (0, 0);
        }
        let low = low.expose_provenance();
        (low, low + size)
    }
}

/// The running thread's stack: its lowest byte and the byte past its top (macOS gives the
/// top and the size).
#[cfg(all(not(miri), target_os = "macos"))]
pub fn thread_stack() -> (usize, usize) {
    // SAFETY: the running thread's own figures; no memory is passed.
    let (top, size) = unsafe {
        let me = libc::pthread_self();
        (
            libc::pthread_get_stackaddr_np(me).expose_provenance(),
            libc::pthread_get_stacksize_np(me),
        )
    };
    (top - size, top)
}

/// Miri cannot ask for the stack; its tests have none to register.
#[cfg(miri)]
pub fn thread_stack() -> (usize, usize) {
    (0, 0)
}

/// Miri cannot ask the loader; its tests have no image to find.
#[cfg(miri)]
pub fn image() -> (usize, Vec<(usize, usize)>) {
    (0, Vec::new())
}

/// The program's own image: its load address, and its loaded segments (first byte, byte
/// past the end), from the program's headers.
#[cfg(all(not(miri), target_os = "linux"))]
pub fn image() -> (usize, Vec<(usize, usize)>) {
    unsafe extern "C" fn each(
        info: *mut libc::dl_phdr_info,
        _size: usize,
        found: *mut libc::c_void,
    ) -> libc::c_int {
        // SAFETY: the loader's record of the program (the first one listed) and its
        // program headers, live during the call; `found` is `image`'s.
        unsafe {
            let info = &*info;
            let found = &mut *found.cast::<(usize, Vec<(usize, usize)>)>();
            let base = info.dlpi_addr as usize;
            found.0 = base;
            for n in 0..info.dlpi_phnum as usize {
                let h = &*info.dlpi_phdr.add(n);
                if h.p_type == libc::PT_LOAD {
                    let start = base + h.p_vaddr as usize;
                    found.1.push((start, start + h.p_memsz as usize));
                }
            }
        }
        1 // the program is listed first: stop there
    }
    let mut found: (usize, Vec<(usize, usize)>) = (0, Vec::new());
    // SAFETY: `each` reads the loader's records as documented; `found` outlives the call.
    unsafe { libc::dl_iterate_phdr(Some(each), (&raw mut found).cast()) };
    found
}

/// The program's own image: its load address (its Mach-O header), and its loaded segments
/// (first byte, byte past the end), from the header's segment commands (dyld lists the
/// program first). `__PAGEZERO`, which maps no access, is left out.
#[cfg(all(not(miri), target_os = "macos"))]
// reason: libc's Mach-O header types are marked for the `mach2` crate; these are their
// plain C layouts, which do not change.
#[allow(deprecated)]
pub fn image() -> (usize, Vec<(usize, usize)>) {
    // SAFETY: dyld's record of the program (image 0), mapped for the program's life: the
    // header, followed by its `ncmds` load commands, each `cmdsize` bytes long.
    unsafe {
        let header = libc::_dyld_get_image_header(0).cast::<libc::mach_header_64>();
        if header.is_null() {
            return (0, Vec::new());
        }
        let slide = libc::_dyld_get_image_vmaddr_slide(0);
        let mut segments = Vec::new();
        let mut command = header.add(1).cast::<u8>();
        for _ in 0..(*header).ncmds {
            let load = command.cast::<libc::load_command>().read_unaligned();
            if load.cmd == libc::LC_SEGMENT_64 {
                let segment = command.cast::<libc::segment_command_64>().read_unaligned();
                if segment.initprot != 0 && segment.vmsize != 0 {
                    let start = (segment.vmaddr as usize).wrapping_add_signed(slide);
                    segments.push((start, start + segment.vmsize as usize));
                }
            }
            command = command.add(load.cmdsize as usize);
        }
        (header.expose_provenance(), segments)
    }
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
        let (ip, write, sp) = fault_registers(context.cast::<libc::ucontext_t>());
        crate::standalone::report_fault(ip, data, write, sp)
    }
}

/// The fault's instruction pointer, whether it was a write (not known here), and the stack
/// pointer, from the signal's context.
///
/// # Safety
/// `context` is the signal handler's, live during the call.
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
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
#[cfg(all(target_os = "linux", not(target_arch = "aarch64")))]
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
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
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
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
unsafe fn fault_registers(context: *const libc::ucontext_t) -> (usize, bool, usize) {
    // SAFETY: the caller's context and the machine context it points to.
    let m = unsafe { &*(*context).uc_mcontext };
    (
        m.__ss.__pc as usize,
        m.__es.__esr & (1 << 6) != 0,
        m.__ss.__sp as usize,
    )
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

/// The program's statics and constants: WebAssembly's code is not in the memory, its data is
/// the memory's first part, from the linker's `__global_base` to `__heap_base` (the main
/// stack's room with it).
#[cfg(target_os = "emscripten")]
pub fn image() -> (usize, Vec<(usize, usize)>) {
    unsafe extern "C" {
        static __global_base: u8;
        static __heap_base: u8;
    }
    // SAFETY: the linker's own symbols; only their addresses are taken.
    let (start, end) = unsafe {
        (
            (&raw const __global_base).expose_provenance(),
            (&raw const __heap_base).expose_provenance(),
        )
    };
    (start, vec![(start, end)])
}

/// No signals for a fault: a trap ends the module, and the browser's console shows it.
#[cfg(target_os = "emscripten")]
pub fn install_fault_reporter() {}
