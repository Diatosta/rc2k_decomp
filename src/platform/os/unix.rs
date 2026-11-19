//! The system's own facts on Unix (the native Linux build, and macOS): the questions
//! `windows.rs` answers on Windows, answered from `/proc`, the environment and libc
//! (`ffi::unix`, which has each system's own calls where they differ).
//! Where Unix has no such thing the answer is the one a plain Windows machine gives, or none.

/// A record shaped as `OSVERSIONINFOA` (0x94 bytes): the kernel's release as the version
/// (major, minor, build), platform 2 (NT's), and "<system> <release>" as its text ("Linux
/// 6.17.1", "Darwin 25.0.0"). Only the log's head reads it.
pub fn version_record() -> [u8; 0x94] {
    let (system, release) = crate::platform::ffi::unix::system_release().unwrap_or_default();
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
    let text = format!("{system} {release}");
    let len = text.len().min(127);
    record[0x14..0x14 + len].copy_from_slice(&text.as_bytes()[..len]);
    record
}

/// A name cut to `room` bytes with its NUL.
fn cut(name: &str, room: u32) -> Option<Vec<u8>> {
    let mut name = crate::files::ansi_bytes(name.trim());
    name.truncate((room as usize).saturating_sub(1));
    (!name.is_empty()).then_some(name)
}

/// The computer's name (the host name), in at most `room` bytes with its NUL.
pub fn computer_name(room: u32) -> Option<Vec<u8>> {
    cut(&crate::platform::ffi::unix::host_name()?, room)
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
    let stats = crate::platform::ffi::unix::file_system_stats(c".")?;
    let clusters =
        |blocks: u64| (blocks.saturating_mul(stats.block_size) / 4096).min(u32::MAX as u64) as u32;
    Some([
        8,
        512,
        clusters(stats.blocks_available),
        clusters(stats.blocks_total),
    ])
}

/// The room (KB, at most 32 bits' worth) left to the user on the file system of folder `at`.
pub fn free_kb_at(at: &std::path::Path) -> Option<u32> {
    use std::os::unix::ffi::OsStrExt;
    let path = std::ffi::CString::new(at.as_os_str().as_bytes()).ok()?;
    let stats = crate::platform::ffi::unix::file_system_stats(&path)?;
    let kb = stats.blocks_available.saturating_mul(stats.block_size) / 1024;
    Some(kb.min(u32::MAX as u64) as u32)
}

/// Available and total physical memory, available and total swap (bytes, at most 32
/// bits' worth, as `GlobalMemoryStatus` caps them).
pub fn memory() -> [u32; 4] {
    let Some(info) = crate::platform::ffi::unix::memory_stats() else {
        return [0; 4];
    };
    let capped = |n: u64| n.min(u32::MAX as u64) as u32;
    [
        capped(info.free_ram),
        capped(info.total_ram),
        capped(info.free_swap),
        capped(info.total_swap),
    ]
}

pub use crate::platform::ffi::unix::{discard_pages, map_pages, own_stack_word};

/// On iOS the program's folder is the app's Documents folder (`$HOME/Documents`, made if it is
/// missing), the one the Files app shows and file sharing reaches: the game's folder is copied
/// there, and `var` and `savedata` are written there. Made the current folder.
#[cfg(target_os = "ios")]
pub fn enter_documents() -> std::io::Result<std::path::PathBuf> {
    let home = std::env::var_os("HOME")
        .filter(|h| !h.is_empty())
        .ok_or_else(|| std::io::Error::other("no HOME folder"))?;
    let documents = std::path::PathBuf::from(home).join("Documents");
    std::fs::create_dir_all(&documents)?;
    std::env::set_current_dir(&documents)?;
    Ok(documents)
}

/// On macOS a program started from its `.app` bundle (from Finder, or `open`) runs in `/`.
/// - An installed bundle (it holds `Contents/Resources/files`) reads the game's files from
///   itself and runs in the user's Application Support folder, where `var` and `savedata` are
///   written; the bundle is not written to.
/// - Any other bundle (`<folder>/<name>.app/Contents/MacOS/rc2k`) makes the folder that holds
///   it current, as `linux/rc2k` is started in the game's folder.
///
/// The folder made current; None for a program outside a bundle (started where it is to run);
/// the error when the folder cannot be made current.
#[cfg(target_os = "macos")]
pub fn enter_bundle_folder() -> std::io::Result<Option<std::path::PathBuf>> {
    use crate::install::mac_bundle::{bundle_of, files_in, support_folder};
    let exe = std::env::current_exe()?;
    let Some(bundle) = bundle_of(&exe) else {
        return Ok(None);
    };
    let files = files_in(&bundle);
    if files.is_dir()
        && let Some(support) = support_folder()
    {
        std::fs::create_dir_all(&support)?;
        std::env::set_current_dir(&support)?;
        crate::files::redirect::add("files", &files.to_string_lossy());
        return Ok(Some(support));
    }
    let Some(folder) = bundle.parent() else {
        return Ok(None);
    };
    std::env::set_current_dir(folder)?;
    Ok(Some(folder.to_path_buf()))
}

/// The process ended at once with `code` (no library's leave-taking).
pub fn terminate(code: u32) -> ! {
    crate::platform::ffi::unix::exit_now(code)
}
