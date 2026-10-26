//! The standalone's Start menu items and the program's copy (platform code, owner's decision
//! 2026-10-19): what `install::add_start_menu_items` (0x413E42) and
//! `install::remove_start_menu_items` (0x4140C3) ask the Program Manager by DDE is done with
//! shell shortcuts (`IShellLinkW` saved by `IPersistFile`) in the user's Start menu
//! Programs folder, group "Rally Championship" (Wine makes them menu entries):
//! - "Rally": `<installed folder>rc2k.exe` (the original: `ral.exe`);
//! - "Uninstall Rally": `<installed folder>rc2k.exe` with " -u" (the original names the
//!   program it ran from, the CD's, which finds the installation in the registry; the
//!   standalone's record is in the installed folder, so only that copy can remove it;
//!   2026-10-20);
//! - "View Readme": `<installed folder>readme.txt`;
//! - the removal deletes the three and the group's folder (the original's `DeleteGroup`).
//!
//! And the program installed: `rc2k.exe` and its DLLs copied from the running program's
//! folder into the current one (the original copies the program named on its command line
//! to `.\ral.exe` when run from the CD).

use std::ffi::c_void;
use std::path::{Path, PathBuf};
use windows_sys::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, CoCreateInstance, CoInitialize, CoUninitialize,
};
use windows_sys::Win32::UI::Shell::{CSIDL_PROGRAMS, SHGetFolderPathW};
use windows_sys::core::GUID;

/// The group's folder name (the original's `[CreateGroup(Rally Championship)]`).
pub const GROUP: &str = "Rally Championship";
/// The program's file, in place of the original's `ral.exe`.
pub const PROGRAM: &str = "rc2k.exe";
/// The DLLs the program needs next to it (as `deploy_standalone.sh` copies them).
pub const DLLS: [&str; 3] = ["SDL3.dll", "avcodec-61.dll", "avutil-59.dll"];

const CLSID_SHELL_LINK: GUID = GUID::from_u128(0x00021401_0000_0000_c000_000000000046);
const IID_SHELL_LINK_W: GUID = GUID::from_u128(0x000214f9_0000_0000_c000_000000000046);
const IID_PERSIST_FILE: GUID = GUID::from_u128(0x0000010b_0000_0000_c000_000000000046);

// Vtable slots.
const QUERY_INTERFACE: usize = 0;
const RELEASE: usize = 2;
const LINK_SET_DESCRIPTION: usize = 7;
const LINK_SET_WORKING_DIRECTORY: usize = 9;
const LINK_SET_ARGUMENTS: usize = 11;
const LINK_SET_PATH: usize = 20;
const PERSIST_SAVE: usize = 6;

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

fn wide_path(p: &Path) -> Vec<u16> {
    wide(&p.to_string_lossy())
}

unsafe fn slot(object: *mut c_void, n: usize) -> usize {
    unsafe { *(*(object as *const *const usize)).add(n) }
}

unsafe fn release(object: *mut c_void) {
    unsafe {
        let f: unsafe extern "system" fn(*mut c_void) -> u32 =
            std::mem::transmute(slot(object, RELEASE));
        f(object);
    }
}

unsafe fn call_text(object: *mut c_void, n: usize, text: &[u16]) -> i32 {
    unsafe {
        let f: unsafe extern "system" fn(*mut c_void, *const u16) -> i32 =
            std::mem::transmute(slot(object, n));
        f(object, text.as_ptr())
    }
}

/// The user's Start menu Programs folder.
fn programs_folder() -> Option<PathBuf> {
    let mut path = [0u16; 260];
    let hr = unsafe {
        SHGetFolderPathW(
            std::ptr::null_mut(),
            CSIDL_PROGRAMS as i32,
            std::ptr::null_mut(),
            0,
            path.as_mut_ptr(),
        )
    };
    if hr < 0 {
        return None;
    }
    let len = path.iter().position(|&c| c == 0).unwrap_or(path.len());
    Some(PathBuf::from(String::from_utf16_lossy(&path[..len])))
}

fn group_folder() -> Option<PathBuf> {
    programs_folder().map(|p| p.join(GROUP))
}

/// One shortcut written; false when COM or the shell refused.
unsafe fn write_link(at: &Path, target: &Path, arguments: &str, name: &str) -> bool {
    unsafe {
        let mut link: *mut c_void = std::ptr::null_mut();
        if CoCreateInstance(
            &CLSID_SHELL_LINK,
            std::ptr::null_mut(),
            CLSCTX_INPROC_SERVER,
            &IID_SHELL_LINK_W,
            &mut link,
        ) < 0
            || link.is_null()
        {
            return false;
        }
        let folder = target.parent().map(wide_path).unwrap_or_else(|| wide(""));
        let mut ok = call_text(link, LINK_SET_PATH, &wide_path(target)) >= 0
            && call_text(link, LINK_SET_WORKING_DIRECTORY, &folder) >= 0
            && call_text(link, LINK_SET_ARGUMENTS, &wide(arguments)) >= 0
            && call_text(link, LINK_SET_DESCRIPTION, &wide(name)) >= 0;
        if ok {
            let query: unsafe extern "system" fn(
                *mut c_void,
                *const GUID,
                *mut *mut c_void,
            ) -> i32 = std::mem::transmute(slot(link, QUERY_INTERFACE));
            let mut file: *mut c_void = std::ptr::null_mut();
            ok = query(link, &IID_PERSIST_FILE, &mut file) >= 0 && !file.is_null();
            if ok {
                let save: unsafe extern "system" fn(*mut c_void, *const u16, i32) -> i32 =
                    std::mem::transmute(slot(file, PERSIST_SAVE));
                ok = save(file, wide_path(at).as_ptr(), 1) >= 0;
                release(file);
            }
        }
        release(link);
        ok
    }
}

/// COM for the calls in `f` (the thread may have it already).
fn with_com<T>(f: impl FnOnce() -> T) -> T {
    let hr = unsafe { CoInitialize(std::ptr::null()) };
    let value = f();
    if hr >= 0 {
        unsafe { CoUninitialize() };
    }
    value
}

/// A C text at `at` as a path (the game's folders end in `\`).
pub unsafe fn c_path(at: *const u8) -> PathBuf {
    let text = unsafe { std::ffi::CStr::from_ptr(at as *const std::ffi::c_char) };
    PathBuf::from(text.to_string_lossy().into_owned())
}

/// The three items, all in `installed`, the installed folder.
pub fn add_start_menu_items(installed: &Path) {
    let Some(group) = group_folder() else {
        crate::framework::log("install: no Start menu Programs folder; no items added");
        return;
    };
    if let Err(e) = std::fs::create_dir_all(&group) {
        crate::framework::log(&format!("install: {}: {e}", group.display()));
        return;
    }
    let items = [
        ("Rally", installed.join(PROGRAM), ""),
        ("Uninstall Rally", installed.join(PROGRAM), "-u"),
        ("View Readme", installed.join("readme.txt"), ""),
    ];
    with_com(|| {
        for (name, target, arguments) in items {
            let at = group.join(format!("{name}.lnk"));
            let done = unsafe { write_link(&at, &target, arguments, name) };
            crate::framework::log(&format!(
                "install: Start menu item {} -> {} {arguments}: {}",
                at.display(),
                target.display(),
                if done { "written" } else { "failed" }
            ));
        }
    });
}

/// The group and its items removed.
pub fn remove_start_menu_items() {
    let Some(group) = group_folder() else {
        return;
    };
    let gone = std::fs::remove_dir_all(&group);
    crate::framework::log(&format!(
        "install: Start menu group {} removed: {gone:?}",
        group.display()
    ));
}

/// The program and its DLLs copied from the running program's folder into the current one,
/// unless it is the same folder. False if a copy failed.
pub fn copy_program() -> bool {
    let Ok(exe) = std::env::current_exe() else {
        return false;
    };
    let (Some(from), Ok(to)) = (exe.parent(), std::env::current_dir()) else {
        return false;
    };
    let same = match (from.canonicalize(), to.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    };
    if same {
        return true;
    }
    let mut files = vec![(exe.clone(), to.join(PROGRAM))];
    files.extend(DLLS.iter().map(|d| (from.join(d), to.join(d))));
    let mut all = true;
    for (a, b) in files {
        let done = std::fs::copy(&a, &b);
        crate::framework::log(&format!(
            "install: {} -> {}: {}",
            a.display(),
            b.display(),
            match &done {
                Ok(_) => "copied".to_string(),
                Err(e) => e.to_string(),
            }
        ));
        all &= done.is_ok();
    }
    all
}

/// A program started, not waited for (the original's `system::run_program` with ebx 1).
pub fn run_program(command: &Path) -> bool {
    let started = std::process::Command::new(command).spawn();
    crate::framework::log(&format!(
        "install: run {}: {}",
        command.display(),
        if started.is_ok() { "started" } else { "failed" }
    ));
    started.is_ok()
}
