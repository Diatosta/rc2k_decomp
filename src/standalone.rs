//! The ported code run without the game: `RAL.EXE` is only read, as data.
//!
//! What happens (`run`):
//! 1. The file is read and laid out in memory of ours as the loader would lay it out
//!    (headers and sections at their places), wherever there is room: its own
//!    addresses, 0x400000 on, are taken in a program that is not `RAL.EXE`. The file has
//!    its relocations, so every address in it is moved by the difference. `game_addr`
//!    adds the base, so the ported code finds every variable.
//! 2. The hooks are installed on that copy exactly as on the running game, and then the
//!    whole image is made memory that cannot be executed. (Its code cannot simply be
//!    blotted out: the game keeps tables among its code, the weekdays' names at
//!    0x401EA5 for one, which ported code reads.) So every call to an address of the
//!    game faults, and `on_exception` decides: a hook's address goes on in the hook's
//!    thunk, where its jump leads, and so in the Rust version, as the callers written
//!    with `call_game` need; any other address is a function that is not ported: it is
//!    noted, the carry flag cleared (the game's "it worked"), the call made to return
//!    at once, and the list written at the end: what is still the game's on the way
//!    taken. None of the game's own code runs.
//! 3. `start` (0x401000) is gone through as the list of calls it is.
//!
//! `call_game` knows the hooks' thunks here (`CALL_REDIRECT`), so only a call made some
//! other way, or to something that is not ported, costs an exception.
//! A Rust version that refuses its input goes to "the original": here every hook's
//! original is the byte after its jump, which faults and ends the run with the hook's
//! name; `fatal::raise::refused` writes the reason first.
//! Stand-ins, each to be replaced by a piece of the platform: see `stand_in` and
//! `on_exception`.

use crate::framework::{
    CALL_REDIRECT, GameHook, LOG_PATH, Regs, STANDALONE, call_game, game_addr, game_read,
    game_write, log, rva,
};
use crate::registry::ALL;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};
use windows_sys::Win32::Foundation::EXCEPTION_ACCESS_VIOLATION;
use windows_sys::Win32::System::Com::{CoInitialize, CoUninitialize};
use windows_sys::Win32::System::Diagnostics::Debug::{
    AddVectoredExceptionHandler, EXCEPTION_POINTERS,
};
use windows_sys::Win32::System::Memory::{
    MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE, PAGE_READWRITE, VirtualAlloc, VirtualProtect,
};
use windows_sys::Win32::System::Threading::{ExitProcess, GetCurrentProcess, TerminateProcess};

const IMAGE_BASE: usize = 0x400000;

// ---- the game's file, laid out in memory ----

/// Where the game's image is, and its sections of code.
struct Image {
    base: usize,
    size: usize,
    code: Vec<(usize, usize)>,
}

static IMAGE: OnceLock<Image> = OnceLock::new();

fn u32_at(bytes: &[u8], at: usize) -> Result<usize, String> {
    bytes
        .get(at..at + 4)
        .map(|b| u32::from_le_bytes(b.try_into().unwrap()) as usize)
        .ok_or_else(|| format!("the file ends before {at:#x}"))
}

fn u16_at(bytes: &[u8], at: usize) -> Result<usize, String> {
    bytes
        .get(at..at + 2)
        .map(|b| u16::from_le_bytes(b.try_into().unwrap()) as usize)
        .ok_or_else(|| format!("the file ends before {at:#x}"))
}

unsafe fn map_game(path: &Path) -> Result<Image, String> {
    let file = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let pe = u32_at(&file, 0x3C)?;
    if file.get(pe..pe + 4) != Some(b"PE\0\0") {
        return Err("not a program file".into());
    }
    let sections = u16_at(&file, pe + 6)?;
    let optional = pe + 0x18;
    let optional_len = u16_at(&file, pe + 0x14)?;
    let wanted = u32_at(&file, optional + 0x1C)?;
    let size = u32_at(&file, optional + 0x38)?;
    let headers = u32_at(&file, optional + 0x3C)?;
    let relocations = (
        u32_at(&file, optional + 0x60 + 5 * 8)?,
        u32_at(&file, optional + 0x60 + 5 * 8 + 4)?,
    );
    if wanted != IMAGE_BASE || relocations.1 == 0 {
        return Err("not the game's file (its base or its relocations are not as expected)".into());
    }
    unsafe {
        let base = VirtualAlloc(
            std::ptr::null(),
            size,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_EXECUTE_READWRITE,
        ) as usize;
        if base == 0 {
            return Err("no memory for the game's image".into());
        }
        let place = |from: usize, to: usize, len: usize| -> Result<(), String> {
            let bytes = file
                .get(from..from + len)
                .ok_or("a section lies outside the file")?;
            if to + len > size {
                return Err("a section lies outside the image".into());
            }
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), (base + to) as *mut u8, len);
            Ok(())
        };
        place(0, 0, headers)?;
        let mut code = Vec::new();
        for n in 0..sections {
            let s = optional + optional_len + n * 40;
            let (virtual_len, at, raw_len, raw_at) = (
                u32_at(&file, s + 8)?,
                u32_at(&file, s + 12)?,
                u32_at(&file, s + 16)?,
                u32_at(&file, s + 20)?,
            );
            place(raw_at, at, raw_len.min(file.len().saturating_sub(raw_at)))?;
            if u32_at(&file, s + 36)? & 0x2000_0020 != 0 {
                code.push((base + at, virtual_len));
            }
        }
        // The relocations: blocks of (page, size, entries of kind << 12 | offset).
        let delta = base.wrapping_sub(wanted) as u32;
        let (mut block, end) = (relocations.0, relocations.0 + relocations.1);
        if end > size {
            return Err("the relocations lie outside the image".into());
        }
        let mut moved = 0u32;
        while block + 8 <= end {
            let page = ((base + block) as *const u32).read_unaligned() as usize;
            let len = ((base + block + 4) as *const u32).read_unaligned() as usize;
            if len < 8 || block + len > end {
                break;
            }
            for n in 0..(len - 8) / 2 {
                let entry = ((base + block + 8 + n * 2) as *const u16).read_unaligned() as usize;
                match entry >> 12 {
                    0 => {}
                    3 if page + (entry & 0xFFF) + 4 <= size => {
                        let at = (base + page + (entry & 0xFFF)) as *mut u32;
                        at.write_unaligned(at.read_unaligned().wrapping_add(delta));
                        moved += 1;
                    }
                    kind => return Err(format!("a relocation of kind {kind}")),
                }
            }
            block += len;
        }
        log(&format!(
            "standalone: the game's image is at {base:#x} ({size:#x} bytes, {moved} addresses moved)"
        ));
        Ok(Image { base, size, code })
    }
}

/// The address as IDA shows it, if it lies in the game's image.
fn ida(address: usize) -> Option<usize> {
    let image = IMAGE.get()?;
    (image.base..image.base + image.size)
        .contains(&address)
        .then(|| address - image.base + IMAGE_BASE)
}

fn in_code(address: usize) -> bool {
    IMAGE.get().is_some_and(|image| {
        image
            .code
            .iter()
            .any(|&(at, len)| (at..at + len).contains(&address))
    })
}

// ---- the hooks, and no execution of anything else ----

/// Every hook's place and name.
static HOOKED: OnceLock<Vec<(usize, &'static str)>> = OnceLock::new();
const JUMP_LEN: usize = 5;

unsafe fn install_and_lock(image: &Image) -> Result<(), String> {
    let installed = crate::install_all(image.base);
    let mut hooks: Vec<&'static dyn GameHook> = ALL
        .iter()
        .zip(&installed)
        .filter(|&(_, &ok)| ok)
        .map(|(hook, _)| *hook)
        .collect();
    // The platform's functions take the places of the game's.
    for hook in crate::platform::HOOKS
        .iter()
        .chain(crate::platform_3d::HOOKS)
    {
        match unsafe { hook.install(image.base + hook.rva()) } {
            Ok(()) => hooks.push(*hook),
            Err(e) => return Err(format!("the platform's {}: {e}", hook.name())),
        }
    }
    // `call_game` goes straight to a hook's thunk, and a hook's "original" is an
    // address inside the hooked function, which faults and names the hook.
    let mut thunks = std::collections::HashMap::new();
    let mut hooked = Vec::new();
    for hook in &hooks {
        let address = image.base + hook.rva();
        match unsafe { thunk_of(address) } {
            Some(thunk) => thunks.insert(address, thunk),
            None => return Err(format!("the hook {} has no jump", hook.name())),
        };
        hook.set_original(address + 1);
        hooked.push((address, hook.name()));
    }
    let _ = CALL_REDIRECT.set(thunks);
    STANDALONE.store(true, Ordering::Relaxed);
    let mut old = 0;
    if unsafe { VirtualProtect(image.base as *const _, image.size, PAGE_READWRITE, &mut old) } == 0
    {
        return Err("the game's image cannot be kept from running".into());
    }
    log(&format!(
        "standalone: {} hooks installed, {} of them the platform's; the game's image is data only",
        hooked.len(),
        crate::platform::HOOKS.len() + crate::platform_3d::HOOKS.len()
    ));
    let _ = HOOKED.set(hooked);
    Ok(())
}

/// Where a hook's jump leads: its thunk.
unsafe fn thunk_of(hook: usize) -> Option<usize> {
    unsafe {
        ((hook as *const u8).read() == 0xE9).then(|| {
            let distance = ((hook + 1) as *const i32).read_unaligned();
            (hook + JUMP_LEN).wrapping_add(distance as usize)
        })
    }
}

// ---- what the faults catch ----

/// The game's functions that were called and are not ported: address as IDA shows it,
/// how often, and where the first call came from.
static UNPORTED: Mutex<Vec<(usize, u32, String)>> = Mutex::new(Vec::new());
/// Calls of the game's functions that are not ported (each a fault), for the timing line.
pub static FAULTS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
/// `start`'s way out (0x4010EE after the main loop, 0x401116 the short one), where
/// `fatal::report` and the `-v` switch jump to.
const EXITS: [usize; 2] = [0x4010EE, 0x401116];
static FILTER: AtomicUsize = AtomicUsize::new(0);

/// The module an address is in, and the address relative to its base ("rc2k.exe+0x1234").
fn in_module(address: usize) -> Option<String> {
    use windows_sys::Win32::System::LibraryLoader::{
        GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
        GetModuleFileNameW, GetModuleHandleExW,
    };
    unsafe {
        let mut module = std::ptr::null_mut();
        let flags =
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT;
        if GetModuleHandleExW(flags, address as *const u16, &mut module) == 0 {
            return None;
        }
        let mut name = [0u16; 260];
        let n = GetModuleFileNameW(module, name.as_mut_ptr(), name.len() as u32) as usize;
        let path = String::from_utf16_lossy(&name[..n]);
        let file = path.rsplit(['\\', '/']).next().unwrap_or(&path).to_string();
        Some(format!("{file}+{:#x}", address - module as usize))
    }
}

/// The return addresses into this program found on the stack from `esp` (at most 24), for
/// `llvm-symbolizer` with the program's PDB.
fn stack_returns(esp: usize) -> String {
    let own = in_module(stack_returns as *const () as usize)
        .map(|m| m.split('+').next().unwrap_or("").to_string());
    let mut out = Vec::new();
    // 2 KB of stack: well inside it at any fault the game code can reach.
    for k in 0..512usize {
        let at = esp + 4 * k;
        let value = unsafe { (at as *const u32).read_unaligned() } as usize;
        if let (Some(m), Some(own)) = (in_module(value), own.as_ref())
            && m.starts_with(own.as_str())
        {
            out.push(m);
            if out.len() == 24 {
                break;
            }
        }
    }
    out.join(" ")
}

/// The game closed during a race (`g_quitting`): what the original's way out does
/// (0x4154B5 back into the main loop, which asks for the shutdown, logs "Leaving main
/// loop..." and returns to `start`), the program ended as `start` ends it.
pub fn leave_from_race() -> ! {
    if let Err(e) = unsafe { crate::threading::request_quit::request_quit() } {
        log(&format!(
            "standalone: the shutdown thread did not start ({:#x})",
            e.code
        ));
    }
    log("system::main_loop: leaving (the game was closed during a race)");
    finish("the game was closed during a race", 0)
}

fn finish(why: &str, code: u32) -> ! {
    log(&format!("standalone: ending: {why}"));
    // Another thread may be in the handler with the list; not for long.
    let unported = (0..200)
        .find_map(|_| {
            let list = UNPORTED.try_lock().ok().map(|list| list.clone());
            if list.is_none() {
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            list
        })
        .unwrap_or_default();
    log(&format!(
        "standalone: {} functions of the game were called that are not ported, in the order first met:",
        unported.len()
    ));
    for (address, calls, from) in unported {
        log(&format!("  {address:#x}  {calls} calls, first from {from}"));
    }
    // Not `ExitProcess`: this may be an exception's handler, where the libraries'
    // leave-taking can hang.
    unsafe {
        TerminateProcess(GetCurrentProcess(), code);
        ExitProcess(code)
    }
}

fn place(address: usize) -> String {
    match ida(address) {
        Some(at) => {
            let hook = HOOKED.get().and_then(|hooked| {
                hooked.iter().find(|&&(hook, _)| {
                    IMAGE
                        .get()
                        .is_some_and(|image| hook - image.base + IMAGE_BASE == at)
                })
            });
            match hook {
                Some((_, name)) => format!("{at:#x} ({name})"),
                None => format!("{at:#x}"),
            }
        }
        None => "Rust code".into(),
    }
}

/// The platform's functions `start` calls that SDL stands for: (the function, what it is).
const SDL_DOES: [(usize, &str); 5] = [
    (0x4103DE, "system::open_window"),
    (0x404981, "gfx::probe_exclusive_mode"),
    (0x404CD6, "gfx::release_screens"),
    (0x40FA8B, "sound::close_cd_audio"),
    (0x410509, "system::drain_messages"),
];

/// The log's wrappers: (the wrapper, its edx, the hooked function it jumps to), as IDA
/// shows them (`logfile::log_write_l1` .. `logfile::log_hex4_l1`).
const LOG_WRAPPERS: [(usize, u32, usize); 16] = [
    (0x401427, 0x000, 0x4013CB), // log_write_l1
    (0x40142B, 0x001, 0x4013CB), // log_write_l1_nl
    (0x401432, 0x100, 0x4013CB), // log_write_l2
    (0x401439, 0x101, 0x4013CB), // log_write_l2_nl
    (0x401472, 0x000, 0x401440), // log_int_l1
    (0x401476, 0x001, 0x401440), // log_int_l1_nl
    (0x40147D, 0x100, 0x401440), // log_int_l2
    (0x401484, 0x101, 0x401440), // log_int_l2_nl
    (0x4014BF, 0x000, 0x40148B), // log_int_fmt_l1
    (0x4015CF, 0x000, 0x4014D8), // log_float_l1
    (0x4015D6, 0x001, 0x4014D8), // log_float_l1_nl
    (0x401622, 0x000, 0x4015F4), // log_hex8_l1
    (0x401626, 0x001, 0x4015F4), // log_hex8_l1_nl
    (0x40162D, 0x100, 0x4015F4), // log_hex8_l2
    (0x401634, 0x101, 0x4015F4), // log_hex8_l2_nl
    (0x4016B0, 0x000, 0x401682), // log_hex4_l1
];

unsafe extern "system" fn on_exception(info: *mut EXCEPTION_POINTERS) -> i32 {
    const CONTINUE_EXECUTION: i32 = -1;
    const CONTINUE_SEARCH: i32 = 0;
    unsafe {
        let record = &*(*info).ExceptionRecord;
        let context = &mut *(*info).ContextRecord;
        let address = record.ExceptionAddress as usize;
        if record.ExceptionCode != EXCEPTION_ACCESS_VIOLATION {
            return CONTINUE_SEARCH;
        }
        // A call or jump into the game's image faults at the address it went to.
        let ran_into_game = ida(address).is_some() && context.Eip as usize == address;
        if !ran_into_game {
            log(&format!(
                "standalone: the fault is in {}; return addresses on the stack: {}",
                in_module(address).unwrap_or_else(|| "no module".into()),
                stack_returns(context.Esp as usize)
            ));
            finish(
                &format!(
                    "access violation at {address:#x} ({}), {} address {:#x}",
                    place(address),
                    if record.ExceptionInformation[0] == 0 {
                        "reading"
                    } else {
                        "writing"
                    },
                    record.ExceptionInformation[1]
                ),
                1,
            );
        }
        let at = ida(address).unwrap_or(0);
        let hooked = HOOKED.get().map(Vec::as_slice).unwrap_or_default();
        if hooked.iter().any(|&(hook, _)| hook == address) {
            let Some(thunk) = thunk_of(address) else {
                finish(&format!("the hook at {at:#x} has no jump"), 1);
            };
            context.Eip = thunk as u32;
            return CONTINUE_EXECUTION;
        }
        // The log's few-byte wrappers (edx the newline and level, then a jump into a
        // hooked function): edx set and on at that function's thunk.
        if let Some(&(_, edx, target)) = LOG_WRAPPERS.iter().find(|w| w.0 == at)
            && let Some(thunk) = thunk_of(game_addr(rva(target)))
        {
            context.Edx = edx;
            context.Eip = thunk as u32;
            return CONTINUE_EXECUTION;
        }
        // `logfile::log_newline_l1` (0x4013DE): an empty text with a line break, at level
        // 1, every register kept (`pusha` .. `popa`).
        if at == 0x4013DE {
            crate::logfile::log_write::log_write(game_addr(rva(0x4E00B7)) as *const u8, true, 1);
            let from = (context.Esp as *const u32).read_unaligned();
            context.Eip = from;
            context.Esp += 4;
            return CONTINUE_EXECUTION;
        }
        // The platform's own at the start and the end, which SDL does here: nothing.
        if let Some(&(_, what)) = SDL_DOES.iter().find(|w| w.0 == at) {
            log(&format!(
                "standalone: {at:#x} ({what}): SDL does it, nothing to do"
            ));
            let from = (context.Esp as *const u32).read_unaligned();
            context.Eip = from;
            context.Esp += 4;
            context.EFlags &= !1;
            return CONTINUE_EXECUTION;
        }
        // An empty function of the game's (a lone `retn`, `nullsub_6` 0x44347E): it does
        // nothing, so nothing is skipped by not running it.
        if in_code(address) && (address as *const u8).read() == 0xC3 {
            let from = (context.Esp as *const u32).read_unaligned();
            context.Eip = from;
            context.Esp += 4;
            return CONTINUE_EXECUTION;
        }
        if EXITS.contains(&at) {
            finish(&format!("the game's way out {at:#x} was reached"), 0);
        }
        if !in_code(address) {
            finish(&format!("a jump into the game's data, {at:#x}"), 1);
        }
        // A hook's "original" is the byte after its jump (`install_and_lock`): its
        // Rust version refused and handed the call on. Only that byte: a function of
        // its own can start a few bytes after a hooked one (0x47CEB4 after 0x47CEA8).
        let original = hooked.iter().find(|&&(hook, _)| address == hook + 1);
        if let Some((_, name)) = original {
            finish(
                &format!("the original of {name} was called (its Rust version refused, {at:#x})"),
                1,
            );
        }
        let from = (context.Esp as *const u32).read_unaligned() as usize;
        FAULTS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if let Ok(mut unported) = UNPORTED.lock() {
            match unported.iter_mut().find(|entry| entry.0 == at) {
                Some(entry) => entry.1 += 1,
                None => {
                    let from = match ida(from) {
                        Some(from) => format!("{from:#x}"),
                        None => "Rust code".into(),
                    };
                    log(&format!(
                        "standalone: not ported: {at:#x}, called from {from}"
                    ));
                    unported.push((at, 1, from));
                }
            }
        }
        // As if the function had returned at once, saying that it worked.
        context.Eip = from as u32;
        context.Esp += 4;
        context.EFlags &= !1;
        CONTINUE_EXECUTION
    }
}

// ---- `start` (0x401000) ----

unsafe fn call(address: usize, r: &mut Regs) {
    log(&format!(
        "standalone: start calls {}",
        place(game_addr(rva(address)))
    ));
    unsafe { call_game(r, game_addr(rva(address))) };
}

unsafe fn stand_in(what: &str, address: usize, value: u32) {
    log(&format!(
        "standalone: stand-in: {what} ({address:#x} = {value})"
    ));
    unsafe { game_write(rva(address), value) };
}

/// The game's `start`, call for call. The checks it makes after calls of the platform
/// (already running, no screen mode) cannot fail here: a call that is not ported comes
/// back with the carry clear.
unsafe fn start() {
    unsafe {
        let mut r = Regs::default();
        for address in [
            0x410F29, 0x401864, 0x403131, 0x40188B, 0x410D4C, 0x410883, 0x410AA3, 0x410AC5,
        ] {
            call(address, &mut r);
        }
        CoInitialize(std::ptr::null());
        for address in [0x40298A, 0x402B7C, 0x4114A6, 0x4114BB, 0x401135] {
            call(address, &mut r);
        }
        // The folders the game saves into but never makes (`notes`: `sub_413A97` makes
        // them only from the installer's script): made here when missing, in the game's
        // folder, as the path templates at 0x5296FB .. 0x52974B name them.
        for folder in [
            "savedata\\savegame",
            "savedata\\carsetup",
            "savedata\\snapshot",
            "savedata\\replay",
        ] {
            if !Path::new(folder).is_dir() {
                match std::fs::create_dir_all(folder) {
                    Ok(()) => log(&format!("standalone: made the folder {folder}")),
                    Err(e) => log(&format!(
                        "standalone: the folder {folder} cannot be made: {e}"
                    )),
                }
            }
        }
        // `system::test_processor` (0x410883, called above) has set it from `cpuid`.
        if game_read::<u32>(rva(0x5183EC)) != 1 {
            finish("fatal error 0x900010FF, FPU is required", 1);
        }
        for address in [
            0x410684, 0x416653, 0x416477, 0x416292, 0x413908, 0x4110C9, 0x40331F, 0x411239,
            0x4103BE,
        ] {
            call(address, &mut r);
        }
        if r.carry() {
            finish("the game is running already (0x4103BE)", 0);
        }
        for address in [0x4103DE, 0x404981, 0x404270, 0x4043DA, 0x404401, 0x41848E] {
            call(address, &mut r);
        }
        r.eax = 0x80;
        call(0x410B68, &mut r);
        call(0x410B84, &mut r);
        call(0x409C1E, &mut r);
        r.eax = 1000;
        call(0x41100C, &mut r);
        call(0x404481, &mut r);
        if r.carry() {
            finish("fatal error 0x90201101, no screen mode (0x404481)", 1);
        }
        for address in [0x40608F, 0x405DB4, 0x405580, 0x407062, 0x40F9DE] {
            call(address, &mut r);
        }
        // What the window's procedure and the key thread tell the main loop (the films
        // play since 2026-10-19: `film.rs`).
        stand_in("the window is the active one", 0x518374, 1);
        stand_in(
            "the window has the focus (until SDL says otherwise)",
            0x518378,
            1,
        );
        stand_in("the key thread runs", 0x518998, 1);
        call(0x415383, &mut r);
        for address in [0x4184D6, 0x404CD6, 0x40FA8B, 0x4060B7, 0x410509] {
            call(address, &mut r);
        }
        r.eax = 0x20;
        call(0x410B68, &mut r);
        for address in [0x412FA2, 0x402B95, 0x4029DB] {
            call(address, &mut r);
        }
        CoUninitialize();
        call(0x407855, &mut r);
    }
}

/// Maps the game's file, installs the hooks and goes through `start`. The current
/// directory has to be the game's. Does not come back.
/// `frames`, if not 0: the program ends itself after that many frames were shown.
pub fn run(game: &Path, log_file: &Path, frames: u32) -> ! {
    crate::platform::FRAMES_TO_SHOW.store(frames, Ordering::SeqCst);
    crate::platform::start_threads();
    crate::platform_input::SdlBackend::start();
    let _ = std::fs::File::create(log_file);
    let _ = LOG_PATH.set(log_file.to_path_buf());
    let image = match unsafe { map_game(game) } {
        Ok(image) => image,
        Err(why) => {
            log(&format!("standalone: {why}"));
            unsafe { ExitProcess(2) }
        }
    };
    let image = IMAGE.get_or_init(|| image);
    unsafe {
        if let Err(why) = install_and_lock(image) {
            finish(&why, 2);
        }
        FILTER.store(
            AddVectoredExceptionHandler(1, Some(on_exception)) as usize,
            Ordering::SeqCst,
        );
        // The audio thread runs (its Rust version), on DirectSound over SDL
        // (`platform_sound`).
        start();
    }
    finish("start came to its end", 0)
}
