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

use crate::errors::Refused;
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
    MEM_COMMIT, MEM_RESERVE, PAGE_NOACCESS, PAGE_READONLY, PAGE_READWRITE, VirtualAlloc,
    VirtualProtect,
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
        .ok_or_else(|| format!("the image's data ends before {at:#x}"))
}

/// What the program keeps of RAL.EXE (`tools/extract_image`, decoupling step B): its data
/// sections, packed, and their relocations; its code sections only as places.
static IMAGE_DATA: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/ral_image.bin"));

/// The game's image built from `IMAGE_DATA` at the places RAL.EXE would have them: the data
/// unpacked and its addresses moved to where the image is; the code sections reserved but
/// left without bytes (no access: a read of them faults and is reported).
unsafe fn build_image() -> Result<Image, String> {
    let d = IMAGE_DATA;
    if d.get(..8) != Some(b"RC2KIMG1") {
        return Err("the program's image data is not what it should be".into());
    }
    let (size, wanted, sections) = (u32_at(d, 8)?, u32_at(d, 12)?, u32_at(d, 16)?);
    if wanted != IMAGE_BASE {
        return Err("the program's image data is not the game's".into());
    }
    unsafe {
        let base = VirtualAlloc(
            std::ptr::null(),
            size,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        ) as usize;
        if base == 0 {
            return Err("no memory for the game's image".into());
        }
        let mut at = 20;
        let mut code = Vec::new();
        let mut pieces: Vec<(usize, usize)> = Vec::new();
        for _ in 0..sections {
            let (rva, len, flags) = (u32_at(d, at)?, u32_at(d, at + 4)?, u32_at(d, at + 8)?);
            at += 12;
            if flags & 1 != 0 {
                code.push((base + rva, len));
            }
            // the data among the code (IDA's: the log's month names, jump tables)
            if flags & 4 != 0 {
                let count = u32_at(d, at)?;
                at += 4;
                for _ in 0..count {
                    let (place, n) = (u32_at(d, at)?, u32_at(d, at + 4)?);
                    at += 8;
                    let bytes = d.get(at..at + n).ok_or("the image's data is cut short")?;
                    if place + n > size {
                        return Err("a piece lies outside the image".into());
                    }
                    std::ptr::copy_nonoverlapping(bytes.as_ptr(), (base + place) as *mut u8, n);
                    pieces.push((base + place, n));
                    at += n;
                }
            }
            if flags & 2 == 0 {
                continue;
            }
            let raw = u32_at(d, at)?;
            at += 4;
            let mut done = 0;
            while done < raw {
                let (zeros, literal) = (u32_at(d, at)?, u32_at(d, at + 4)?);
                at += 8;
                let bytes = d
                    .get(at..at + literal)
                    .ok_or("the image's data is cut short")?;
                let to = rva + done + zeros;
                if to + literal > size {
                    return Err("a section lies outside the image".into());
                }
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), (base + to) as *mut u8, literal);
                at += literal;
                done += zeros + literal;
            }
        }
        let delta = base.wrapping_sub(wanted) as u32;
        let relocations = u32_at(d, at)?;
        at += 4;
        for _ in 0..relocations {
            let place = u32_at(d, at)?;
            at += 4;
            if place + 4 > size {
                return Err("a relocation lies outside the image".into());
            }
            let p = (base + place) as *mut u32;
            p.write_unaligned(p.read_unaligned().wrapping_add(delta));
        }
        if d.get(at..at + 4) != Some(b"END!") {
            return Err("the image's data does not end where it should".into());
        }
        // The code: no access, but the pages that hold data among it read only (the rest of
        // such a page reads as 0).
        for &(at, len) in &code {
            let mut old = 0;
            VirtualProtect(at as *const _, len, PAGE_NOACCESS, &mut old);
        }
        for &(at, len) in &pieces {
            let first = at & !0xFFF;
            let mut old = 0;
            VirtualProtect(first as *const _, at + len - first, PAGE_READONLY, &mut old);
        }
        log(&format!(
            "standalone: the game's image is at {base:#x} ({size:#x} bytes, {relocations} addresses moved; no code of RAL.EXE)"
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

/// The game's empty functions (a lone `retn`; from IDA, 2026-10-19; and 0x48558A, `pusha;
/// popa; retn` in a draw list): a call to one is done.
const LONE_RETN: [usize; 119] = [
    0x40AC76, 0x40CB09, 0x40CF32, 0x40D27E, 0x41E2F5, 0x428D74, 0x428DB2, 0x429823, 0x429B17,
    0x429B18, 0x436832, 0x436B97, 0x436B98, 0x439329, 0x439593, 0x43DFB0, 0x44326C, 0x443471,
    0x44347E, 0x443DB9, 0x44403F, 0x444F54, 0x445AF0, 0x4467C6, 0x446993, 0x44DEA9, 0x44E1C0,
    0x44E76A, 0x44E776, 0x44E777, 0x44EB65, 0x44FF1A, 0x450222, 0x450A4A, 0x454F61, 0x454FEF,
    0x4554C9, 0x456091, 0x457F2F, 0x457F82, 0x45836E, 0x45840D, 0x45A854, 0x45BBFA, 0x45BECC,
    0x45C550, 0x45CCD3, 0x45D254, 0x465A3D, 0x465A3E, 0x465AB5, 0x465AB6, 0x465D78, 0x467BD5,
    0x46D0F7, 0x47019D, 0x47019E, 0x471391, 0x4713DD, 0x471C80, 0x471C81, 0x475C0B, 0x4907B8,
    0x4907B9, 0x4A94E9, 0x4A94EA, 0x4A94EB, 0x4A9678, 0x4B695B, 0x4B7ED9, 0x4B9789, 0x4BA849,
    0x4BA84A, 0x4BA84B, 0x4BAC86, 0x4BD018, 0x4BD121, 0x4BEF98, 0x4BEFD1, 0x4C072A, 0x4C07DA,
    0x4C08B9, 0x4C08BA, 0x4C1162, 0x4C16F8, 0x4C21EB, 0x4C25F5, 0x4C2DDA, 0x4C3901, 0x4C39D4,
    0x4C4E14, 0x4C4E15, 0x4C4E16, 0x4C4E17, 0x4C4E18, 0x4C4E19, 0x4C4E1A, 0x4C4E1B, 0x4C4E2E,
    0x4C4EA0, 0x4C4EA1, 0x4C4FFA, 0x4C65BF, 0x4C65C0, 0x4C65C1, 0x4C74D9, 0x4C8167, 0x4C9699,
    0x4C9AF5, 0x4CAE5B, 0x4CB795, 0x4D0BA7, 0x4D0BA8, 0x4D0EB3, 0x4D1710, 0x4D2C9A, 0x4D5C40,
    0x4D6AE4, 0x48558A,
];

/// Every hook's place and name.
static HOOKED: OnceLock<Vec<(usize, &'static str)>> = OnceLock::new();

unsafe fn install_and_lock(image: &Image) -> Result<(), String> {
    // No code to patch: every hooked address goes to its hook's thunk, the platform's
    // functions taking the places of the game's.
    let installed = crate::mark_all_installed(image.base);
    let hooks: Vec<&'static dyn GameHook> = ALL
        .iter()
        .zip(&installed)
        .filter(|&(_, &ok)| ok)
        .map(|(hook, _)| *hook)
        .chain(crate::platform::HOOKS.iter().copied())
        .chain(crate::platform_3d::HOOKS.iter().copied())
        .collect();
    // `call_game` goes straight to a hook's thunk, and a hook's "original" is an
    // address inside the hooked function, which faults and names the hook.
    let mut thunks = std::collections::HashMap::new();
    let mut hooked = Vec::new();
    for hook in &hooks {
        let address = image.base + hook.rva();
        thunks.insert(address, hook.thunk());
        hook.set_original(address + 1);
        hooked.push((address, hook.name()));
    }
    let _ = CALL_REDIRECT.set(thunks);
    STANDALONE.store(true, Ordering::Relaxed);
    log(&format!(
        "standalone: {} hooks in place, {} of them the platform's; the game's image is data only",
        hooked.len(),
        crate::platform::HOOKS.len() + crate::platform_3d::HOOKS.len()
    ));
    let _ = HOOKED.set(hooked);
    Ok(())
}

/// A hooked address's thunk.
fn thunk_of(hook: usize) -> Option<usize> {
    CALL_REDIRECT.get().and_then(|t| t.get(&hook).copied())
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
            // Rust reading the game's code, which the standalone does not have (step B's audit)
            let read = record.ExceptionInformation[1];
            if record.ExceptionInformation[0] == 0 && in_code(read) {
                log(&format!(
                    "standalone: Rust read RAL.EXE's code bytes at {:#x}, which the program does not have; return addresses on the stack: {}",
                    ida(read).unwrap_or(0),
                    stack_returns(context.Esp as usize)
                ));
            }
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
        if LONE_RETN.contains(&at) {
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
        // folder, as the path templates at 0x5296FB .. 0x52974B name them; not in the
        // download folder, where the program is the installer.
        let folders: &[&str] = if crate::install::portable::is_source_folder() {
            &[]
        } else {
            &[
                "savedata\\savegame",
                "savedata\\carsetup",
                "savedata\\snapshot",
                "savedata\\replay",
            ]
        };
        for &folder in folders {
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
        for address in [0x410684, 0x416653, 0x416477, 0x416292, 0x413908, 0x4110C9] {
            call(address, &mut r);
        }
        // `strings::load_language` (0x40331F): Rust, its hook gone (it keeps every register).
        log("standalone: start calls strings::load_language");
        let compile = &mut || crate::strings::compile_lexicon::compile_lexicon();
        match crate::strings::load_language::load_language(compile) {
            Ok(()) => {}
            Err(Refused::Fatal(error)) => crate::fatal::raise::raise(error, &r),
            Err(Refused::BadInput(why)) => {
                finish(&format!("strings::load_language refused: {}", why.0), 1)
            }
        }
        for address in [0x411239, 0x4103BE] {
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
pub fn run(log_file: &Path, frames: u32) -> ! {
    crate::platform::FRAMES_TO_SHOW.store(frames, Ordering::SeqCst);
    crate::platform::start_threads();
    crate::platform_input::SdlBackend::start();
    // Absolute: the installer changes the current folder (`install::make_folders`).
    let log_file = &std::path::absolute(log_file).unwrap_or_else(|_| log_file.to_path_buf());
    let _ = std::fs::File::create(log_file);
    let _ = LOG_PATH.set(log_file.to_path_buf());
    // A panic in a hook's dispatch ends the program at once (it cannot unwind out of an
    // `extern "C"` function): say where first.
    std::panic::set_hook(Box::new(|info| {
        log(&format!("standalone: panic: {info}"));
    }));
    let image = match unsafe { build_image() } {
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
