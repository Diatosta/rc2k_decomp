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
//!    with `call_game` need; any other address is a function that is not ported: the
//!    program ends, logging it and its caller (owner, 2026-10-23; it returned at once
//!    as if it had worked before, which hid it). None of the game's own code runs.
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
use crate::framework::{CALL_REDIRECT, GameHook, LOG_PATH, Regs, game_addr, log, rva};
use crate::install::outside::Game as Outside;
use crate::registry::ALL;
use crate::session::MACHINE;
use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};
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
pub(crate) static IMAGE_DATA: &[u8] =
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
        let mut pointers = Vec::with_capacity(relocations);
        for _ in 0..relocations {
            let place = u32_at(d, at)?;
            at += 4;
            if place + 4 > size {
                return Err("a relocation lies outside the image".into());
            }
            let p = (base + place) as *mut u32;
            p.write_unaligned(p.read_unaligned().wrapping_add(delta));
            pointers.push(place);
        }
        if d.get(at..at + 4) != Some(b"END!") {
            return Err("the image's data does not end where it should".into());
        }
        // The game's pointers to and in the variables that have left the image (phase 5).
        let pointed = crate::moved::point_into_homes(base, &pointers);
        log(&format!(
            "standalone: {pointed} of the game's pointers point at variables' new homes"
        ));
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

pub fn finish(why: &str, code: u32) -> ! {
    log(&format!("standalone: ending: {why}"));
    menu_watch::report();
    // No game code runs here, so a hook's thunk is entered only by a call by address
    // from Rust (`call_game`): these are what keeps each hook (conversion phase 4).
    let mut reached: Vec<(&str, usize)> = ALL
        .iter()
        .map(|h| (h.name(), h.stats().calls.load(Ordering::Relaxed)))
        .filter(|&(_, calls)| calls > 0)
        .collect();
    reached.sort();
    log(&format!(
        "standalone: {} hooks were reached by address:",
        reached.len()
    ));
    for (name, calls) in reached {
        log(&format!("  {name}  {calls} calls"));
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

/// For Rust callers that name by its address a function the fault handler answers
/// without the game (`menu::table_calls`): SDL's (carry clear) and the lone `retn`s, done
/// here directly (conversion phase 3, 2026-10-23). `at` is the address as IDA shows it.
pub fn answers(at: usize, r: &mut Regs) -> bool {
    if SDL_DOES.iter().any(|w| w.0 == at) {
        r.set_carry(false);
        return true;
    }
    LONE_RETN.contains(&at)
}

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
        // A function of the game with no Rust version (owner, 2026-10-23: no longer returned
        // from as if it had worked, which hid it): the program ends, saying which.
        let from = (context.Esp as *const u32).read_unaligned() as usize;
        let from = match ida(from) {
            Some(from) => format!("{from:#x}"),
            None => "Rust code".into(),
        };
        finish(&format!("not ported: {at:#x}, called from {from}"), 1);
    }
}

// ---- `start` (0x401000) ----

unsafe fn call(address: usize, r: &mut Regs) {
    log(&format!(
        "standalone: start calls {}",
        place(game_addr(rva(address)))
    ));
    crate::calls::call(rva(address), r);
}

/// `start`'s call of a function whose hook is gone (conversion phase 4): its Rust version
/// runs; every one of them keeps the registers.
fn rust(name: &str) {
    log(&format!("standalone: start calls {name}"));
}

/// A Rust version's refusal in `start`: a refused input ends the program, a fatal error is
/// raised.
unsafe fn refused(name: &str, e: Refused, r: &Regs) {
    unsafe { crate::fatal::raise::refused_in(name, e, r) };
}

fn stand_in(what: &str, variable: &crate::session::Dword, value: u32) {
    log(&format!(
        "standalone: stand-in: {what} ({:#x} = {value})",
        variable.addr()
    ));
    variable.set(value);
}

/// The game's `start`, call for call. The checks it makes after calls of the platform
/// (already running, no screen mode) cannot fail here: a call that is not ported comes
/// back with the carry clear.
unsafe fn start() {
    unsafe {
        let mut r = Regs::default();
        // The input module's two device-record pointers name the records' new places.
        crate::input::store::point_devices_at_records();
        crate::sound::store::point_sound_objects();
        // `g_surface.format` (0x4F49B8): the image points it at the format record's old
        // place; the record is in `session::SCREEN` now (the platform sets it again at
        // every lock).
        crate::framework::game_write(
            crate::framework::rva(0x4F49B8),
            crate::session::SCREEN.format_addr() as u32,
        );
        rust("system::keep_start_time");
        crate::system::keep_start_time::keep_start_time();
        rust("math::seed_random");
        crate::math::seed_random::seed_random();
        rust("system::read_volume_serial");
        crate::system::read_volume_serial::read_volume_serial();
        // eax: what `system::read_volume_serial` leaves there, `start`'s esi at entry (its
        // `pop eax`; the serial is lost: `notes/bugs.md`), 0 here.
        rust("math::stir_random");
        crate::math::stir_random::stir_random(0);
        // `system::set_exception_filter` (0x410D4C) sets nothing in the standalone.
        rust("system::test_processor");
        if let Err(e) =
            crate::system::test_processor::test_processor(&crate::system::test_processor::Real)
        {
            refused("system::test_processor", e, &r);
        }
        rust("system::read_keyboard_type");
        crate::system::read_keyboard_type::read_keyboard_type();
        rust("system::read_windows_version");
        crate::system::read_windows_version::read_windows_version();
        CoInitialize(std::ptr::null());
        rust("memory::open_own_heap");
        if let Err(e) = crate::memory::open_own_heap::open_own_heap() {
            refused("memory::open_own_heap", e, &r);
        }
        rust("system::set_error_mode");
        crate::system::set_error_mode::set_error_mode();
        rust("settings::read_settings");
        if let Err(why) = crate::settings::read_settings::read_settings() {
            finish(&format!("settings::read_settings refused: {}", why.0), 1);
        }
        rust("settings::apply_feature_switches");
        crate::settings::apply_feature_switches::apply_feature_switches();
        rust("logfile::start_log");
        if let Err(why) = crate::logfile::start_log::start_log() {
            refused("logfile::start_log", why.into(), &r);
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
        if MACHINE.has_fpu.get() != 1 {
            finish("fatal error 0x900010FF, FPU is required", 1);
        }
        rust("system::read_command_line");
        if let Err(why) = crate::system::read_command_line::read_own_command_line() {
            refused("system::read_command_line", why.into(), &r);
        }
        rust("install::check_clio_disc");
        if let Err(e) = crate::install::check_clio_disc::check_clio_disc(&mut Outside) {
            refused("install::check_clio_disc", e, &r);
        }
        rust("install::check_at_start");
        let no_audio = crate::store::game_ptr(crate::install::check_at_start::NO_AUDIO_RVA);
        if let Err(e) = crate::install::check_at_start::check_at_start(&mut Outside, no_audio) {
            refused("install::check_at_start", e, &r);
        }
        rust("system::check_developer_machine");
        crate::system::check_developer_machine::check_developer_machine();
        rust("install::read_languages");
        if let Err(e) = crate::install::read_languages::read_languages(game_addr(
            crate::install::read_languages::PATH_RVA,
        ) as *const u8)
        .map_err(Refused::from)
        {
            refused("install::read_languages", e, &r);
        }
        rust("install::read_record");
        if let Err(e) = crate::install::read_record::read_record(game_addr(
            crate::install::read_record::PATH_RVA,
        ) as *const u8)
        .map_err(Refused::from)
        {
            refused("install::read_record", e, &r);
        }
        // `strings::load_language` (0x40331F): Rust, its hook gone (it keeps every register).
        rust("strings::load_language");
        let compile = &mut || crate::strings::compile_lexicon::compile_lexicon();
        if let Err(e) = crate::strings::load_language::load_language(compile) {
            refused("strings::load_language", e, &r);
        }
        rust("system::load_game_data");
        if let Err(e) = crate::system::load_game_data::load_in_game() {
            refused("system::load_game_data", e, &r);
        }
        rust("system::find_running_game");
        if crate::system::find_running_game::find_running_game() {
            finish("the game is running already (0x4103BE)", 0);
        }
        // `system::open_window` (0x4103DE) and `gfx::probe_exclusive_mode` (0x404981):
        // SDL's (`SDL_DOES`).
        rust("gfx::find_3d_cards");
        crate::gfx::find_3d_cards::find_3d_cards();
        rust("gfx::list_primary_modes");
        if let Err(e) = crate::gfx::list_primary_modes::list_primary_modes() {
            refused("gfx::list_primary_modes", e, &r);
        }
        rust("gfx::list_secondary_modes");
        if let Err(e) = crate::gfx::list_secondary_modes::list_secondary_modes() {
            refused("gfx::list_secondary_modes", e, &r);
        }
        rust("net::set_up");
        if let Err(e) = crate::net::set_up::set_up() {
            refused("net::set_up", e, &r);
        }
        rust("system::set_priority_class");
        crate::system::set_priority_class::set_priority_class(0x80);
        rust("threading::start_threads");
        if let Err(e) = crate::threading::start_threads::start_threads_in_game() {
            refused("threading::start_threads", e, &r);
        }
        rust("sound::wait_for_commands");
        crate::sound::wait_for_commands::wait_in_game();
        rust("system::sleep");
        crate::system::sleep::sleep(1000);
        call(0x404481, &mut r);
        if r.carry() {
            finish("fatal error 0x90201101, no screen mode (0x404481)", 1);
        }
        rust("gfx::allocate_picture_buffer");
        if let Err(e) = crate::gfx::allocate_picture_buffer::allocate_picture_buffer() {
            refused("gfx::allocate_picture_buffer", e, &r);
        }
        rust("gfx::set_component_format");
        if let Err(e) = crate::gfx::set_component_format::set_component_format() {
            refused("gfx::set_component_format", e, &r);
        }
        rust("gfx::fill_thirds_table");
        crate::gfx::fill_thirds_table::fill_thirds_table();
        rust("gfx::install_converters");
        crate::gfx::install_converters::install_converters();
        // `sound::open_cd_audio` (0x40F9DE): cut out of this build (`pusha; jmp` to its
        // end), so not called.
        // What the window's procedure and the key thread tell the main loop (the films
        // play since 2026-10-19: `film.rs`).
        stand_in(
            "the window is the active one",
            &crate::session::FLAGS.window_active,
            1,
        );
        stand_in(
            "the window has the focus (until SDL says otherwise)",
            &crate::session::FLAGS.window_focused,
            1,
        );
        stand_in("the key thread runs", &crate::session::CLOCKS.key_ticks, 1);
        rust("system::main_loop");
        if let Err(e) = crate::system::main_loop::main_loop_in_game() {
            refused("system::main_loop", e, &r);
        }
        rust("net::shut_down");
        if let Err(e) = crate::net::shut_down::shut_down() {
            refused("net::shut_down", e, &r);
        }
        // `gfx::release_screens` (0x404CD6), `sound::close_cd_audio` (0x40FA8B): SDL's
        // (`SDL_DOES`).
        rust("gfx::free_picture_buffer");
        if let Err(why) = crate::gfx::free_picture_buffer::free_picture_buffer() {
            finish(&format!("gfx::free_picture_buffer refused: {}", why.0), 1);
        }
        // `system::drain_messages` (0x410509): SDL's (`SDL_DOES`).
        rust("system::set_priority_class");
        crate::system::set_priority_class::set_priority_class(0x20);
        rust("install::run_after_exit");
        if let Err(e) = crate::install::run_after_exit::run_after_exit() {
            refused("install::run_after_exit", e, &r);
        }
        rust("system::restore_error_mode");
        crate::system::restore_error_mode::restore_error_mode();
        rust("memory::free_own_heap");
        crate::memory::free_own_heap::free_own_heap();
        CoUninitialize();
        rust("fatal::show_report_at_exit");
        crate::fatal::show_report_at_exit::show_report_at_exit();
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
    menu_watch::snapshot(image.base);
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

/// The menus' and draw lists' memory as the image starts and as it ends (typed tables,
/// 2026-10-22): which of their fields the game changes in play.
mod menu_watch {
    use super::log;
    use std::sync::OnceLock;

    /// RAL.EXE's menus (0x7482B8 ..) and its screens' draw lists (0x758B70 ..).
    const RANGES: [(usize, usize); 2] = [(0x7482B8, 0x755130), (0x758B70, 0x7594B0)];
    const IMAGE_BASE: usize = 0x400000;

    static START: OnceLock<(usize, Vec<Vec<u8>>)> = OnceLock::new();

    fn bytes(base: usize, (from, to): (usize, usize)) -> Vec<u8> {
        let at = base + from - IMAGE_BASE;
        // The image is ours, mapped read-write for its whole size.
        unsafe { std::slice::from_raw_parts(at as *const u8, to - from) }.to_vec()
    }

    pub fn snapshot(base: usize) {
        let _ = START.set((base, RANGES.iter().map(|&r| bytes(base, r)).collect()));
    }

    pub fn report() {
        let Some((base, start)) = START.get() else {
            return;
        };
        let mut changed = Vec::new();
        for (&range, before) in RANGES.iter().zip(start) {
            let now = bytes(*base, range);
            for (n, (a, b)) in before.chunks(4).zip(now.chunks(4)).enumerate() {
                if a != b {
                    changed.push(format!(
                        "  {:#x}: {:02x?} -> {:02x?}",
                        range.0 + 4 * n,
                        a,
                        b
                    ));
                }
            }
        }
        log(&format!(
            "standalone: {} dwords of the menus and draw lists changed in play:",
            changed.len()
        ));
        for line in changed {
            log(&line);
        }
    }
}
