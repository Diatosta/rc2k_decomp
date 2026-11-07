//! libc behind safe functions, for `os/unix.rs` (the Unix build's system facts): file system
//! and memory statistics, anonymous page mapping, ending the process, and the running
//! thread's stack and the program's loaded segments from the loader.
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

/// `sysinfo`'s memory figures, in bytes.
pub struct MemoryStats {
    pub free_ram: u64,
    pub total_ram: u64,
    pub free_swap: u64,
    pub total_swap: u64,
}

/// `sysinfo` with its `mem_unit` applied (saturating); None when it fails.
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

/// The running thread's stack: its lowest byte and the byte past its top.
#[cfg(not(miri))]
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
#[cfg(not(miri))]
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
