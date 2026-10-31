//! The system's own facts on Unix (phase 11, the native Linux build): the questions
//! `windows.rs` answers on Windows, answered from `/proc`, the environment and libc.
//! Where Unix has no such thing (a drive's serial, a keyboard's subtype, an error mode, a
//! priority class) the answer is the one a plain Windows machine gives, or none.

/// A record shaped as `OSVERSIONINFOA` (0x94 bytes): the kernel's release as the version
/// (major, minor, build), platform 2 (NT's), and "Linux <release>" as its text. Only the
/// log's head reads it.
pub fn version_record() -> [u8; 0x94] {
    let release = std::fs::read_to_string("/proc/sys/kernel/osrelease").unwrap_or_default();
    let release = release.trim();
    let mut numbers = release
        .split(|c: char| !c.is_ascii_digit())
        .filter_map(|n| n.parse::<u32>().ok());
    let mut record = [0u8; 0x94];
    let dwords = [
        0x94,
        numbers.next().unwrap_or(0),
        numbers.next().unwrap_or(0),
        numbers.next().unwrap_or(0),
        2,
    ];
    for (n, d) in dwords.iter().enumerate() {
        record[4 * n..4 * n + 4].copy_from_slice(&d.to_le_bytes());
    }
    let text = format!("Linux {release}");
    let len = text.len().min(127);
    record[0x14..0x14 + len].copy_from_slice(&text.as_bytes()[..len]);
    record
}

/// No drive serial on Unix: as `GetVolumeInformationA` failing.
pub fn volume() -> Option<(u32, [u8; 12])> {
    None
}

/// A standard keyboard's subtype (0).
pub fn keyboard_subtype() -> u32 {
    0
}

/// A name cut to `room` bytes with its NUL.
fn cut(name: &str, room: u32) -> Option<Vec<u8>> {
    let mut name = crate::files::ansi_bytes(name.trim());
    name.truncate((room as usize).saturating_sub(1));
    (!name.is_empty()).then_some(name)
}

/// The computer's name (the host name), in at most `room` bytes with its NUL.
pub fn computer_name(room: u32) -> Option<Vec<u8>> {
    cut(
        &std::fs::read_to_string("/proc/sys/kernel/hostname").ok()?,
        room,
    )
}

/// The user's name (`$USER`), in at most `room` bytes with its NUL.
pub fn user_name(room: u32) -> Option<Vec<u8>> {
    cut(&std::env::var("USER").ok()?, room)
}

/// The system's language as an LCID, from the locale's language (`LC_ALL`, `LC_MESSAGES`,
/// `LANG`): its primary language with the default sublanguage; English (0x409) for one
/// not known.
pub fn language_id() -> u32 {
    let locale = ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .find_map(|v| std::env::var(v).ok().filter(|l| !l.is_empty()))
        .unwrap_or_default();
    let language = locale.split(['_', '.', '@']).next().unwrap_or("");
    let primary = match language {
        "de" => 0x07,
        "es" => 0x0A,
        "fi" => 0x0B,
        "fr" => 0x0C,
        "it" => 0x10,
        "ja" => 0x11,
        "nl" => 0x13,
        "pl" => 0x15,
        "pt" => 0x16,
        "sv" => 0x1D,
        _ => 0x09,
    };
    0x400 | primary
}

/// DRIVE_FIXED for the current drive; DRIVE_NO_ROOT_DIR for a lettered one (Unix has no
/// drive letters).
pub fn drive_type(root: Option<[u8; 4]>) -> u32 {
    match root {
        None => 3,
        Some(_) => 1,
    }
}

/// The free space of the current folder's file system (a lettered root has none), as
/// `GetDiskFreeSpaceA` gives it: 8 sectors of 512 bytes a cluster, free and all clusters
/// (at most 32 bits' worth).
pub fn disk_free(root: Option<&[u8]>) -> Option<[u32; 4]> {
    if root.is_some_and(|r| r.get(1) == Some(&b':')) {
        return None;
    }
    // SAFETY: all zeros is a valid value of this plain C struct.
    let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
    // SAFETY: a NUL-ended path and an out-parameter of its type.
    if unsafe { libc::statvfs(c".".as_ptr(), &raw mut stat) } != 0 {
        return None;
    }
    let clusters = |blocks: u64| {
        (blocks.saturating_mul(stat.f_frsize as u64) / 4096).min(u32::MAX as u64) as u32
    };
    Some([
        8,
        512,
        clusters(stat.f_bavail as u64),
        clusters(stat.f_blocks as u64),
    ])
}

/// The room (KB, at most 32 bits' worth) left to the user on the file system of folder `at`.
pub fn free_kb_at(at: &std::path::Path) -> Option<u32> {
    use std::os::unix::ffi::OsStrExt;
    let path = std::ffi::CString::new(at.as_os_str().as_bytes()).ok()?;
    // SAFETY: all zeros is a valid value of this plain C struct.
    let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
    // SAFETY: a NUL-ended path and an out-parameter of its type.
    if unsafe { libc::statvfs(path.as_ptr(), &raw mut stat) } != 0 {
        return None;
    }
    let kb = (stat.f_bavail as u64).saturating_mul(stat.f_frsize as u64) / 1024;
    Some(kb.min(u32::MAX as u64) as u32)
}

/// Available and total physical memory, available and total swap (bytes, at most 32
/// bits' worth, as `GlobalMemoryStatus` caps them).
pub fn memory() -> [u32; 4] {
    // SAFETY: all zeros is a valid value of this plain C struct.
    let mut info: libc::sysinfo = unsafe { std::mem::zeroed() };
    // SAFETY: an out-parameter of its type.
    if unsafe { libc::sysinfo(&raw mut info) } != 0 {
        return [0; 4];
    }
    let bytes = |n: u64| n.saturating_mul(info.mem_unit as u64).min(u32::MAX as u64) as u32;
    [
        bytes(info.freeram as u64),
        bytes(info.totalram as u64),
        bytes(info.freeswap as u64),
        bytes(info.totalswap as u64),
    ]
}

/// No error mode on Unix (no dialogs to keep away): the mode before is 0.
pub fn set_error_mode(_mode: u32) -> u32 {
    0
}

/// No priority classes on Unix: the process runs as it was started.
pub fn set_priority_class(_class: u32) {}

/// A range of `len` addresses reserved with no access, for the program's life.
pub fn reserve(len: usize) -> Option<usize> {
    // SAFETY: a new anonymous mapping with no access; nothing is passed in.
    let at = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            len,
            libc::PROT_NONE,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | libc::MAP_NORESERVE,
            -1,
            0,
        )
    };
    (at != libc::MAP_FAILED).then(|| at.expose_provenance())
}

/// The process ended at once with `code` (no library's leave-taking).
pub fn terminate(code: u32) -> ! {
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
