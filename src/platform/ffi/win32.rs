//! Win32 behind safe functions (`windows-sys`): the system's own facts for `os` (version,
//! names, language, drives, memory, page mapping, the stack
//! and image bounds, ending the process) and the shell links of the Start menu items
//! (`install::shortcuts`, COM's `IShellLinkW` through its vtables).
//!
//! Every function is plain: values and slices in and out, the out-parameters local to the
//! call. The two raw-pointer items are `map_pages` and `discard_pages`, the page pool's
//! memory primitives (the pool in `pool.rs` holds the mapping for the program's
//! life). Nothing allocates in `terminate` (it may run in the fault handler).

#![allow(unsafe_code)]

use windows_sys::Win32::Globalization::GetSystemDefaultLCID;
use windows_sys::Win32::Storage::FileSystem::{GetDiskFreeSpaceA, GetDriveTypeA};
use windows_sys::Win32::System::Memory::{
    MEM_COMMIT, MEM_RESERVE, MEM_RESET, PAGE_READWRITE, VirtualAlloc,
};
use windows_sys::Win32::System::SystemInformation::{
    GetVersionExA, GlobalMemoryStatus, MEMORYSTATUS, OSVERSIONINFOA,
};
use windows_sys::Win32::System::Threading::{
    ExitProcess, GetCurrentProcess, GetCurrentThreadStackLimits, TerminateProcess,
};
use windows_sys::Win32::System::WindowsProgramming::{GetComputerNameA, GetUserNameA};

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

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

fn wide_path(p: &Path) -> Vec<u16> {
    wide(&p.to_string_lossy())
}

// The interfaces' vtables, slot for slot in the Windows SDK's order (`unknwn.h`,
// `objidl.h`, `shobjidl_core.h`): a method is called by its field, so a wrong slot is a
// wrong name, and each table's size is checked against its count of methods below. The
// methods not called here are `Uncalled`, kept for their places.

/// A method slot this code does not call.
type Uncalled = *const c_void;
/// `HRESULT (this, LPCWSTR)`: the link's text setters.
type SetText = unsafe extern "system" fn(*mut c_void, *const u16) -> i32;

/// `IUnknown`: the first three slots of every COM interface.
#[repr(C)]
struct UnknownMethods {
    query_interface: unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> i32,
    add_ref: Uncalled,
    release: unsafe extern "system" fn(*mut c_void) -> u32,
}

/// `IShellLinkW`'s vtable.
#[repr(C)]
struct ShellLinkMethods {
    unknown: UnknownMethods,
    get_path: Uncalled,
    get_id_list: Uncalled,
    set_id_list: Uncalled,
    get_description: Uncalled,
    set_description: SetText,
    get_working_directory: Uncalled,
    set_working_directory: SetText,
    get_arguments: Uncalled,
    set_arguments: SetText,
    get_hotkey: Uncalled,
    set_hotkey: Uncalled,
    get_show_cmd: Uncalled,
    set_show_cmd: Uncalled,
    get_icon_location: Uncalled,
    set_icon_location: Uncalled,
    set_relative_path: Uncalled,
    resolve: Uncalled,
    set_path: SetText,
}

/// `IPersistFile`'s vtable (`IUnknown`, `IPersist`, then its own).
#[repr(C)]
struct PersistFileMethods {
    unknown: UnknownMethods,
    get_class_id: Uncalled,
    is_dirty: Uncalled,
    load: Uncalled,
    /// `HRESULT (this, LPCOLESTR file, BOOL remember)`.
    save: unsafe extern "system" fn(*mut c_void, *const u16, i32) -> i32,
    save_completed: Uncalled,
    get_cur_file: Uncalled,
}

const SLOT: usize = size_of::<usize>();
const _: () = assert!(size_of::<UnknownMethods>() == 3 * SLOT);
const _: () = assert!(size_of::<ShellLinkMethods>() == 21 * SLOT);
const _: () = assert!(size_of::<PersistFileMethods>() == 9 * SLOT);

/// A COM object: its first word points at its interface's vtable `M`.
#[repr(C)]
struct ComObject<M> {
    methods: *const M,
}

/// A held reference to a COM object of interface `M` (non-null), released once when dropped.
struct Held<M>(*mut ComObject<M>);

impl<M> Held<M> {
    /// The object `object` names, or None for null.
    ///
    /// # Safety
    /// `object` is null or a live COM object of interface `M` whose reference the caller
    /// hands over (to be released once, here).
    unsafe fn new(object: *mut c_void) -> Option<Self> {
        (!object.is_null()).then(|| Held(object.cast()))
    }

    /// The object's vtable.
    fn methods(&self) -> &M {
        // SAFETY: a live COM object of interface `M` (`new`'s contract) while held: its
        // first word points at that interface's vtable, which lives as long as the object.
        unsafe { &*(*self.0).methods }
    }

    /// The object as the `this` its methods take.
    fn this(&self) -> *mut c_void {
        self.0.cast()
    }
}

impl<M> Drop for Held<M> {
    fn drop(&mut self) {
        // Every interface's vtable starts with `IUnknown`'s: `M`'s first field.
        let unknown = self.0.cast::<ComObject<UnknownMethods>>();
        // SAFETY: the reference `new` was handed, released once; `Release` is slot 2 of every
        // interface, as `UnknownMethods` lays it out.
        unsafe { ((*(*unknown).methods).release)(self.this()) };
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
    let mut object: *mut c_void = std::ptr::null_mut();
    // SAFETY: COM started on this thread by the caller (`with_com`); the out-parameter is
    // local.
    let created = unsafe {
        CoCreateInstance(
            &CLSID_SHELL_LINK,
            std::ptr::null_mut(),
            CLSCTX_INPROC_SERVER,
            &IID_SHELL_LINK_W,
            &raw mut object,
        )
    };
    if created < 0 {
        return false;
    }
    // SAFETY: `CoCreateInstance` succeeded asking for `IID_IShellLinkW`: `object` is null or
    // that interface, whose reference is ours.
    let Some(link) = (unsafe { Held::<ShellLinkMethods>::new(object) }) else {
        return false;
    };
    let methods = link.methods();
    let folder = target.parent().map(wide_path).unwrap_or_else(|| wide(""));
    // SAFETY: `IShellLinkW`'s setters on the live link, each with a NUL-ended wide text that
    // lives over the call.
    let set = unsafe {
        (methods.set_path)(link.this(), wide_path(target).as_ptr()) >= 0
            && (methods.set_working_directory)(link.this(), folder.as_ptr()) >= 0
            && (methods.set_arguments)(link.this(), wide(arguments).as_ptr()) >= 0
            && (methods.set_description)(link.this(), wide(name).as_ptr()) >= 0
    };
    if !set {
        return false;
    }
    let mut object: *mut c_void = std::ptr::null_mut();
    // SAFETY: `QueryInterface` on the live link, with a GUID and a local out-parameter.
    let queried = unsafe {
        (methods.unknown.query_interface)(link.this(), &IID_PERSIST_FILE, &raw mut object)
    };
    if queried < 0 {
        return false;
    }
    // SAFETY: `QueryInterface` succeeded asking for `IID_IPersistFile`: `object` is null or
    // that interface, whose (added) reference is ours.
    let Some(file) = (unsafe { Held::<PersistFileMethods>::new(object) }) else {
        return false;
    };
    // SAFETY: `IPersistFile::Save` on the live object with a NUL-ended wide path that lives
    // over the call, remembering it (TRUE).
    unsafe { (file.methods().save)(file.this(), wide_path(at).as_ptr(), 1) >= 0 }
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

// ---- the fault reporter and the profiler's sampler (`platform::timing`) ----

/// A manual-reset event, made by `create_event` and never closed: valid for the program's
/// life, so a copy may be signalled or waited on from any thread.
#[derive(Clone, Copy)]
pub struct Event(usize);

impl Event {
    /// Signals the event.
    pub fn set(self) {
        // SAFETY: the event made by `create_event`, open for the program's life.
        unsafe {
            windows_sys::Win32::System::Threading::SetEvent(std::ptr::with_exposed_provenance_mut(
                self.0,
            ));
        }
    }

    /// Waits, without limit, until the event is signalled.
    pub fn wait_forever(self) {
        use windows_sys::Win32::System::Threading::{INFINITE, WaitForSingleObject};
        // SAFETY: the event made by `create_event`, never closed.
        unsafe {
            WaitForSingleObject(std::ptr::with_exposed_provenance_mut(self.0), INFINITE);
        }
    }
}

/// An unnamed manual-reset event, not signalled; None if the system gives none.
pub fn create_event() -> Option<Event> {
    use windows_sys::Win32::System::Threading::CreateEventW;
    // SAFETY: an unnamed manual-reset event with default security, kept for the
    // program's life.
    let event = unsafe { CreateEventW(std::ptr::null(), 1, 0, std::ptr::null()) };
    if event.is_null() {
        None
    } else {
        Some(Event(event.expose_provenance()))
    }
}

/// `f` given the module a host address is in: its full path's UTF-16 units (as
/// `GetModuleFileNameW` gives it, in a buffer on the stack) and its base address; None when
/// the address is in none. Allocates nothing, for the fault reporter (the loader's lock it
/// may take is covered by the reporter's time limit).
pub fn with_module_of_address_wide<T>(
    address: usize,
    f: impl FnOnce(&[u16], usize) -> T,
) -> Option<T> {
    use windows_sys::Win32::System::LibraryLoader::{
        GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
        GetModuleFileNameW, GetModuleHandleExW,
    };
    // SAFETY: `address` is only looked up (UNCHANGED_REFCOUNT: nothing held); the module
    // handle is live while the program runs; `name` is the length said.
    unsafe {
        let mut module = std::ptr::null_mut();
        let flags =
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT;
        if GetModuleHandleExW(
            flags,
            std::ptr::with_exposed_provenance::<u16>(address),
            &raw mut module,
        ) == 0
        {
            return None;
        }
        let mut name = [0u16; 260];
        let n = GetModuleFileNameW(module, name.as_mut_ptr(), name.len() as u32) as usize;
        Some(f(&name[..n], module.expose_provenance()))
    }
}

/// The module a host address is in, as `with_module_of_address_wide`, with the ANSI calls (the
/// profiler's): the path lossy from UTF-8.
pub fn module_of_address_ansi(address: usize) -> Option<(String, usize)> {
    use windows_sys::Win32::System::LibraryLoader::{
        GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
        GetModuleFileNameA, GetModuleHandleExA,
    };
    // SAFETY: `address` is only looked up (UNCHANGED_REFCOUNT: nothing held); the module
    // handle is live while the program runs; `file` is the 260 bytes said.
    unsafe {
        let mut module = std::ptr::null_mut();
        if GetModuleHandleExA(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
            std::ptr::with_exposed_provenance::<u8>(address),
            &raw mut module,
        ) == 0
        {
            return None;
        }
        let mut file = [0u8; 260];
        let n = GetModuleFileNameA(module, file.as_mut_ptr(), 260) as usize;
        Some((
            String::from_utf8_lossy(&file[..n]).into_owned(),
            module.expose_provenance(),
        ))
    }
}

/// The base address of the program's own module.
pub fn program_base() -> usize {
    // SAFETY: a null name asks for the program's own module.
    unsafe { windows_sys::Win32::System::LibraryLoader::GetModuleHandleA(std::ptr::null()) }
        .expose_provenance()
}

/// The running thread's system id.
pub fn current_thread_id() -> u32 {
    // SAFETY: no arguments; the calling thread's id.
    unsafe { windows_sys::Win32::System::Threading::GetCurrentThreadId() }
}

/// An open thread the profiler samples (suspend and context rights), kept open for the
/// program's life.
pub struct SampledThread(usize);

impl SampledThread {
    /// Opens the thread with this system id; None if the system refuses.
    pub fn open(id: u32) -> Option<SampledThread> {
        use windows_sys::Win32::System::Threading::{
            OpenThread, THREAD_GET_CONTEXT, THREAD_SUSPEND_RESUME,
        };
        // SAFETY: a thread id; a null handle on failure, checked.
        let handle = unsafe { OpenThread(THREAD_SUSPEND_RESUME | THREAD_GET_CONTEXT, 0, id) };
        if handle.is_null() {
            None
        } else {
            Some(SampledThread(handle.expose_provenance()))
        }
    }

    /// Stops the thread, reads its instruction pointer and lets it go on; None if it could
    /// not be stopped or its context not read. The thread is running again on return (it
    /// may have been stopped holding a lock the caller takes next).
    pub fn instruction_pointer(&self) -> Option<usize> {
        use windows_sys::Win32::System::Diagnostics::Debug::{CONTEXT, GetThreadContext};
        use windows_sys::Win32::System::Threading::{ResumeThread, SuspendThread};
        let thread: windows_sys::Win32::Foundation::HANDLE =
            std::ptr::with_exposed_provenance_mut(self.0);
        // SAFETY: a handle opened by `open` with suspend and context rights, kept open for
        // the program's life; `CONTEXT` is plain data, zeroed as the API expects.
        unsafe {
            if SuspendThread(thread) == u32::MAX {
                return None;
            }
            let mut context: CONTEXT = std::mem::zeroed();
            context.ContextFlags = crate::platform::arch::CONTEXT_CONTROL;
            let got = GetThreadContext(thread, &raw mut context) != 0;
            ResumeThread(thread);
            got.then(|| crate::platform::arch::ip(&context))
        }
    }
}

/// The unhandled-exception filter: reports an access violation (`standalone::report_fault`,
/// which ends the program) and passes every other exception on. Reads the system's records
/// only; what the report calls is listed at `report_fault`.
///
/// # Safety
/// The system's filter: `info` and the records it points at are the system's, live during
/// the call.
unsafe extern "system" fn on_exception(
    info: *const windows_sys::Win32::System::Diagnostics::Debug::EXCEPTION_POINTERS,
) -> i32 {
    const CONTINUE_SEARCH: i32 = 0;
    // SAFETY: `info`'s records as the system hands them (the function's contract).
    unsafe {
        let record = &*(*info).ExceptionRecord;
        let context = &*(*info).ContextRecord;
        if record.ExceptionCode != windows_sys::Win32::Foundation::EXCEPTION_ACCESS_VIOLATION {
            return CONTINUE_SEARCH;
        }
        crate::standalone::report_fault(
            record.ExceptionAddress.expose_provenance(),
            record.ExceptionInformation[1],
            record.ExceptionInformation[0] != 0,
            crate::platform::arch::sp(context),
        )
    }
}

/// The word at `at` on this thread's stack, for the fault report (`standalone::stack_returns`):
/// None outside the part of the stack from this call's frame up to the stack's base
/// (`GetCurrentThreadStackLimits`' high limit), so a walk up the stack stops at its end. The
/// filter runs on the faulting thread, on its stack below the faulting frame, so the walk's
/// words, from the fault's stack pointer up, are all inside that part.
pub fn own_stack_word(at: usize) -> Option<usize> {
    let (mut low, mut high) = (0usize, 0usize);
    // SAFETY: writes the two limits only; it reads the thread's own record (no lock, no
    // allocation), so it is callable in the filter.
    unsafe { GetCurrentThreadStackLimits(&raw mut low, &raw mut high) };
    let here = (&raw const low) as usize;
    let inside = low <= here
        && at >= here
        && at
            .checked_add(size_of::<usize>())
            .is_some_and(|end| end <= high);
    // SAFETY: a word between this frame and the stack's base: a thread's stack is committed
    // downwards from its base without gaps, and this frame is committed (it runs), so every
    // byte up to the base is. The filter ends the process, so the frames read are never run
    // again; the read needs no alignment.
    inside.then(|| unsafe { std::ptr::with_exposed_provenance::<usize>(at).read_unaligned() })
}

/// Installs the access-violation reporter as the process's unhandled-exception filter (it
/// runs only for exceptions no handler took, so system code that catches its own faults is
/// left alone).
pub fn install_fault_reporter() {
    // SAFETY: `on_exception` is a filter of the system's signature, for the program's life.
    unsafe {
        windows_sys::Win32::System::Diagnostics::Debug::SetUnhandledExceptionFilter(Some(
            on_exception,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The UTF-16 units of `text`, as the link file holds its texts.
    fn units(text: &str) -> Vec<u8> {
        text.encode_utf16().flat_map(u16::to_le_bytes).collect()
    }

    /// Where `text` is in `file` as a `StringData` entry: its count of characters (Wine's
    /// writer counts a NUL after them, Windows' does not), then its UTF-16 units.
    fn string_data_at(file: &[u8], text: &str) -> Option<usize> {
        let count = text.encode_utf16().count() as u16;
        [count, count + 1].into_iter().find_map(|count| {
            let entry: Vec<u8> = count.to_le_bytes().into_iter().chain(units(text)).collect();
            file.windows(entry.len()).position(|w| w == entry)
        })
    }

    /// A link written through the typed vtables: a shell link file (its header and class)
    /// with a target, and the description, working folder and arguments given, in the
    /// `StringData` order (`NAME_STRING`, `WORKING_DIR`, `COMMAND_LINE_ARGUMENTS`): each
    /// setter reached its own slot.
    #[test]
    fn write_link_writes_a_shell_link() {
        let folder = std::env::temp_dir().join(format!("rc2k_link_{}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();
        let at = folder.join("Check.lnk");
        let target = std::env::current_exe().unwrap();
        let (arguments, name) = ("--rc2k-arguments-check", "rc2k description check");
        assert!(with_com(|| write_link(&at, &target, arguments, name)));
        let file = std::fs::read(&at).unwrap();
        let _ = std::fs::remove_dir_all(&folder);
        // `ShellLinkHeader`: its size, 0x4C, then CLSID_ShellLink, then the link's flags.
        assert_eq!(file[..4], [0x4C, 0, 0, 0]);
        assert_eq!(file[4..20], CLSID_SHELL_LINK_BYTES);
        let flags = u32::from_le_bytes(file[0x14..0x18].try_into().unwrap());
        // HasLinkTargetIDList or HasLinkInfo: `SetPath` gave it a target.
        assert_ne!(flags & 0x3, 0, "no target in the link (flags {flags:#x})");
        let working = target.parent().unwrap().to_string_lossy().into_owned();
        let places = [name, &working, arguments].map(|text| string_data_at(&file, text));
        assert!(
            matches!(places, [Some(n), Some(w), Some(a)] if n < w && w < a),
            "the link's texts are not description, working folder, arguments: {places:?}"
        );
    }

    /// `CLSID_ShellLink` as the file holds it (the GUID's first three fields little-endian).
    const CLSID_SHELL_LINK_BYTES: [u8; 16] = [
        0x01, 0x14, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x46,
    ];
}
