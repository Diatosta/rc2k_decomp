//! The system's own facts on Windows (phase 11): what has no std or SDL form, each a plain
//! call with a typed answer. The one file of the game's code that calls Win32 for them; a
//! Linux build answers the same questions in its own file.

use windows_sys::Win32::Globalization::GetSystemDefaultLCID;
use windows_sys::Win32::Storage::FileSystem::{
    GetDiskFreeSpaceA, GetDriveTypeA, GetVolumeInformationA,
};
use windows_sys::Win32::System::Diagnostics::Debug::{SetErrorMode, THREAD_ERROR_MODE};
use windows_sys::Win32::System::Memory::{MEM_RESERVE, PAGE_NOACCESS, VirtualAlloc};
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
        GetVersionExA(&mut info);
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
            &mut serial,
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
        GlobalMemoryStatus(&mut m);
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

/// A range of `len` addresses reserved with no access, for the program's life: its first
/// (a host address), or None.
pub fn reserve(len: usize) -> Option<usize> {
    // SAFETY: a reservation only; nothing is passed in.
    let at = unsafe { VirtualAlloc(std::ptr::null(), len, MEM_RESERVE, PAGE_NOACCESS) };
    (!at.is_null()).then(|| at.expose_provenance())
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
    unsafe { GetCurrentThreadStackLimits(&mut low, &mut high) };
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
