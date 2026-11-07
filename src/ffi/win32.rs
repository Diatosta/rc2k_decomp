//! Win32 behind safe functions (`windows-sys`): the system's own facts for `os` (version,
//! volume, names, language, drives, memory, error mode, priority, page mapping, the stack
//! and image bounds, ending the process) and the shell links of the Start menu items
//! (`install::shortcuts`, COM's `IShellLinkW` through its vtables).
//!
//! Every function is plain: values and slices in and out, the out-parameters local to the
//! call. The two raw-pointer items are `map_pages` and `discard_pages`, the page pool's
//! memory primitives (the pool in `pointers/pool.rs` holds the mapping for the program's
//! life). Nothing allocates in `terminate` (it may run in the fault handler).

#![allow(unsafe_code)]

use windows_sys::Win32::Globalization::GetSystemDefaultLCID;
use windows_sys::Win32::Storage::FileSystem::{
    GetDiskFreeSpaceA, GetDriveTypeA, GetVolumeInformationA,
};
use windows_sys::Win32::System::Diagnostics::Debug::{SetErrorMode, THREAD_ERROR_MODE};
use windows_sys::Win32::System::Memory::{
    MEM_COMMIT, MEM_RESERVE, MEM_RESET, PAGE_READWRITE, VirtualAlloc,
};
use windows_sys::Win32::System::SystemInformation::{
    GetVersionExA, GlobalMemoryStatus, MEMORYSTATUS, OSVERSIONINFOA,
};
use windows_sys::Win32::System::Threading::{
    ExitProcess, GetCurrentProcess, GetCurrentThreadStackLimits, SetPriorityClass, TerminateProcess,
};
use windows_sys::Win32::System::WindowsProgramming::{GetComputerNameA, GetUserNameA};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetKeyboardType;

/// `OSVERSIONINFOA` (0x94 bytes) as `GetVersionExA` fills it.
pub fn version_record() -> [u8; 0x94] {
    // SAFETY: a zeroed record of its own size, its size set as the call asks.
    unsafe {
        let mut info: OSVERSIONINFOA = std::mem::zeroed();
        info.dwOSVersionInfoSize = size_of::<OSVERSIONINFOA>() as u32;
        GetVersionExA(&raw mut info);
        let mut record = [0u8; 0x94];
        let dwords = [
            info.dwOSVersionInfoSize,
            info.dwMajorVersion,
            info.dwMinorVersion,
            info.dwBuildNumber,
            info.dwPlatformId,
        ];
        for (n, d) in dwords.iter().enumerate() {
            record[4 * n..4 * n + 4].copy_from_slice(&d.to_le_bytes());
        }
        for (to, &c) in record[0x14..].iter_mut().zip(&info.szCSDVersion) {
            *to = c as u8;
        }
        record
    }
}

/// The current drive's serial number and the first 12 bytes of its name, if they can be had.
pub fn volume() -> Option<(u32, [u8; 12])> {
    let (mut serial, mut name) = (0u32, [0u8; 12]);
    // SAFETY: out-parameters of their sizes; no root (the current drive).
    let worked = unsafe {
        GetVolumeInformationA(
            std::ptr::null(),
            name.as_mut_ptr(),
            name.len() as u32,
            &raw mut serial,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0,
        )
    } != 0;
    worked.then_some((serial, name))
}

/// `GetKeyboardType(1)`: the keyboard's subtype.
pub fn keyboard_subtype() -> u32 {
    // SAFETY: a plain value.
    unsafe { GetKeyboardType(1) as u32 }
}

/// A name the system writes into `room` bytes (its NUL among them): the name, if any.
fn name(room: u32, call: impl FnOnce(*mut u8, &mut u32) -> i32) -> Option<Vec<u8>> {
    let mut place = vec![0u8; room as usize];
    let mut len = room;
    if call(place.as_mut_ptr(), &mut len) == 0 {
        return None;
    }
    let end = place.iter().position(|&b| b == 0).unwrap_or(place.len());
    place.truncate(end);
    Some(place)
}

/// The computer's name, in at most `room` bytes with its NUL.
pub fn computer_name(room: u32) -> Option<Vec<u8>> {
    // SAFETY: the place and its length, as the call takes them.
    name(room, |at, len| unsafe { GetComputerNameA(at, len) })
}

/// The user's name, in at most `room` bytes with its NUL.
pub fn user_name(room: u32) -> Option<Vec<u8>> {
    // SAFETY: as `computer_name`.
    name(room, |at, len| unsafe { GetUserNameA(at, len) })
}

/// The system's language (the default LCID).
pub fn language_id() -> u32 {
    // SAFETY: no arguments.
    unsafe { GetSystemDefaultLCID() }
}

/// `GetDriveTypeA` of a root (`x:\` with its NUL), or of the current drive.
pub fn drive_type(root: Option<[u8; 4]>) -> u32 {
    // SAFETY: a NUL-ended root, or none.
    unsafe {
        match root {
            Some(root) => GetDriveTypeA(root.as_ptr()),
            None => GetDriveTypeA(std::ptr::null()),
        }
    }
}

/// `GetDiskFreeSpaceA` of a root (a path, no NUL) or the current drive: sectors per
/// cluster, bytes per sector, free clusters, all clusters.
pub fn disk_free(root: Option<&[u8]>) -> Option<[u32; 4]> {
    let mut out = [0u32; 4];
    let root = match root {
        Some(r) => Some(std::ffi::CString::new(r.to_vec()).ok()?),
        None => None,
    };
    let [a, b, c, d] = &mut out;
    // SAFETY: four out-parameters; a NUL-ended root or none.
    let worked = unsafe {
        GetDiskFreeSpaceA(
            root.as_ref()
                .map_or(std::ptr::null(), |r| r.as_ptr().cast()),
            a,
            b,
            c,
            d,
        )
    } != 0;
    worked.then_some(out)
}

/// `GlobalMemoryStatus`: available and total physical memory, available and total page
/// file (bytes, as 32 bits).
pub fn memory() -> [u32; 4] {
    // SAFETY: a zeroed record with its length set.
    let m = unsafe {
        let mut m: MEMORYSTATUS = std::mem::zeroed();
        m.dwLength = size_of::<MEMORYSTATUS>() as u32;
        GlobalMemoryStatus(&raw mut m);
        m
    };
    [
        m.dwAvailPhys as u32,
        m.dwTotalPhys as u32,
        m.dwAvailPageFile as u32,
        m.dwTotalPageFile as u32,
    ]
}

/// `SetErrorMode`: the mode set, the one before returned.
pub fn set_error_mode(mode: u32) -> u32 {
    // SAFETY: a plain value.
    unsafe { SetErrorMode(mode as THREAD_ERROR_MODE) }
}

/// `SetPriorityClass` of the process.
pub fn set_priority_class(class: u32) {
    // SAFETY: the process's own pseudo-handle and a plain value.
    unsafe { SetPriorityClass(GetCurrentProcess(), class) };
}

/// `len` bytes of zeroed, readable and writable pages for the program's life (a pool block):
/// their first byte, or None. Pages not yet touched take no memory.
pub fn map_pages(len: usize) -> Option<*mut u8> {
    // SAFETY: a new committed range; nothing is passed in.
    let at = unsafe {
        VirtualAlloc(
            std::ptr::null(),
            len,
            MEM_RESERVE | MEM_COMMIT,
            PAGE_READWRITE,
        )
    };
    (!at.is_null()).then_some(at.cast())
}

/// The pages of `map_pages`' `len` bytes at `at` handed back to the system, still committed
/// (`MEM_RESET`: their contents need not be kept). Whether the pages came back zeroed (they
/// do not here: the caller zeroes them when it lends them again).
///
/// # Safety
/// `at` and `len` are a range of `map_pages` (whole pages).
pub unsafe fn discard_pages(at: *mut u8, len: usize) -> bool {
    // SAFETY: the caller's committed range; the pages stay readable and writable.
    unsafe { VirtualAlloc(at.cast(), len, MEM_RESET, PAGE_READWRITE) };
    false
}

/// The process ended at once with `code`: no library's leave-taking (it can hang when this
/// is a fault's handler).
pub fn terminate(code: u32) -> ! {
    // SAFETY: the process's own pseudo-handle; neither call returns.
    unsafe {
        TerminateProcess(GetCurrentProcess(), code);
        ExitProcess(code)
    }
}

/// The running thread's stack: its lowest byte and the byte past its top (host addresses).
pub fn thread_stack() -> (usize, usize) {
    let (mut low, mut high) = (0usize, 0usize);
    // SAFETY: two out-parameters of the documented call.
    unsafe { GetCurrentThreadStackLimits(&raw mut low, &raw mut high) };
    (low, high)
}

/// The program's own image: its base, and its sections (first byte, byte past the end),
/// host addresses, from its PE headers.
pub fn image() -> (usize, Vec<(usize, usize)>) {
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    // SAFETY: the running module's PE headers are mapped and read-only; the offsets are
    // the PE format's (e_lfanew at 0x3C; the file header after the 4-byte signature; the
    // 40-byte section headers after the optional header).
    unsafe {
        let base = GetModuleHandleW(std::ptr::null()).expose_provenance();
        if base == 0 {
            return (0, Vec::new());
        }
        let read16 =
            |at: usize| (std::ptr::with_exposed_provenance::<u16>(at)).read_unaligned() as usize;
        let read32 =
            |at: usize| (std::ptr::with_exposed_provenance::<u32>(at)).read_unaligned() as usize;
        let nt = base + read32(base + 0x3C);
        let count = read16(nt + 4 + 2);
        let optional = read16(nt + 4 + 16);
        let first = nt + 4 + 20 + optional;
        let sections = (0..count)
            .map(|n| {
                let header = first + 40 * n;
                let start = base + read32(header + 12);
                (start, start + read32(header + 8))
            })
            .collect();
        (base, sections)
    }
}

// ---- shell links ----

use std::ffi::c_void;
use std::path::{Path, PathBuf};
use windows_sys::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, CoCreateInstance, CoInitialize, CoUninitialize,
};
use windows_sys::Win32::UI::Shell::{CSIDL_PROGRAMS, SHGetFolderPathW};
use windows_sys::core::GUID;

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

/// Vtable slot `n` of a COM `object`.
///
/// # Safety
/// `object` is a live COM object with at least `n + 1` slots.
unsafe fn slot(object: *mut c_void, n: usize) -> usize {
    // SAFETY: the caller's COM object: its first word points at its vtable.
    unsafe { *(*(object as *const *const usize)).add(n) }
}

/// `IUnknown::Release`.
///
/// # Safety
/// `object` is a live COM object the caller holds a reference to.
unsafe fn release(object: *mut c_void) {
    // SAFETY: slot `RELEASE` is `IUnknown::Release`, as the caller's object has.
    unsafe {
        let f: unsafe extern "system" fn(*mut c_void) -> u32 =
            std::mem::transmute(slot(object, RELEASE));
        f(object);
    }
}

/// A COM method of slot `n` taking one wide text (NUL-ended).
///
/// # Safety
/// `object` is a live COM object whose slot `n` takes a `*const u16` and returns an HRESULT.
unsafe fn call_text(object: *mut c_void, n: usize, text: &[u16]) -> i32 {
    // SAFETY: the caller's slot `n`, a method of that shape.
    unsafe {
        let f: unsafe extern "system" fn(*mut c_void, *const u16) -> i32 =
            std::mem::transmute(slot(object, n));
        f(object, text.as_ptr())
    }
}

/// The user's Start menu Programs folder.
pub fn programs_folder() -> Option<PathBuf> {
    let mut path = [0u16; 260];
    // SAFETY: a buffer of the 260 wide characters `MAX_PATH` asks for; no window or token.
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

/// One shortcut written; false when COM or the shell refused.
pub fn write_link(at: &Path, target: &Path, arguments: &str, name: &str) -> bool {
    // SAFETY: COM started on this thread by the caller (`with_com`). The shell link object
    // comes from `CoCreateInstance` (checked), is called only through its documented
    // `IShellLink` / `IUnknown` / `IPersistFile` slots with NUL-ended wide texts that live
    // over each call, and is released once (the file interface too).
    unsafe {
        let mut link: *mut c_void = std::ptr::null_mut();
        if CoCreateInstance(
            &CLSID_SHELL_LINK,
            std::ptr::null_mut(),
            CLSCTX_INPROC_SERVER,
            &IID_SHELL_LINK_W,
            &raw mut link,
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
            ok = query(link, &IID_PERSIST_FILE, &raw mut file) >= 0 && !file.is_null();
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
pub fn with_com<T>(f: impl FnOnce() -> T) -> T {
    // SAFETY: no arguments to speak of; the matching `CoUninitialize` follows below only
    // when this call succeeded.
    let hr = unsafe { CoInitialize(std::ptr::null()) };
    let value = f();
    if hr >= 0 {
        // SAFETY: balances the successful `CoInitialize` above.
        unsafe { CoUninitialize() };
    }
    value
}
