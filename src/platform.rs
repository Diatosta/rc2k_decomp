//! The platform under the game, on SDL3, for the standalone program (`standalone.rs`).
//!
//! These are not ports: each takes the place of a function of the game that is
//! DirectDraw, the window or the message pump, keeps what that function means to the
//! game's own code (the variables it leaves, the flags it returns with) and does the
//! rest with SDL. There is no original to compare them with, so they have no self-test
//! and are installed by the standalone program only (`HOOKS`).
//!
//! The screen: the game draws into a locked surface in the screen mode's own pixel
//! layout. Here that surface is memory of ours (`Screen::pixels`, rows without a gap),
//! and `show_frame` copies it to the window, 16-bit pixels made 32-bit on the way.
//!
//! - `gfx::choose_screen_modes` (0x404481): the original tries the kept modes and, if
//!   they do not work, a list of others, through DirectDraw. Here the kept modes are
//!   taken as they are, 800 x 600 in 32 bits when there is none.
//! - `gfx::set_screen_mode` (0x404B63, eax = the mode): the mode's size and pixel kind
//!   looked up as the original does and its variables set; the surface made; its pixel
//!   format described; the texture formats too (`platform_3d.rs`).
//!   `gfx::set_menu_screen_mode` (0x404823) is it with the menus' mode.
//! - `gfx::lock_surface` (0x4058AD), `gfx::unlock_surface` (0x405949),
//!   `gfx::show_frame` (0x4057D1).
//! - `screens::run_frame` (0x405A71, eax = the frame's function) and
//!   `system::pump_messages` (0x410535): SDL's events; closing the window sets the
//!   quit switch, and the keys the menus know become the game's pending key.

use crate::framework::{GameHook, RawHook, log, rva};
use crate::gfx::describe_format::describe_format;
use crate::gfx::front::SURFACES;
use crate::gfx::locked::LOCKED;
use crate::gfx::modes::MODES;
use crate::gfx::screen_state::DRAW_SURFACE;
use crate::gfx::screen_state::SCREEN_MODE;
use crate::gfx::set_clip::set_clip;
use crate::input::store::DEVICES;
use crate::session::FLAGS;
use crate::session::SCREEN as GAME_SCREEN;
use crate::system::find_entry::find_entry;
use sdl3_sys::everything::*;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

const DEFAULT_MODE: u32 = 0x32_2802;

// ---- game data ----

// ---- the screen ----

struct Screen {
    /// The pixels a region while the screen lives (the game's code draws on them by
    /// address: `pointers`). (Declared first: fields drop in order, and the region must go before the memory does.)
    _lent: crate::pointers::Lent,
    window: usize,
    width: usize,
    height: usize,
    bits: u32,
    pixels: Vec<u8>,
}

static SCREEN: Mutex<Option<Screen>> = Mutex::new(None);
/// Frames shown, and after how many the program ends itself (0: never): for trying it
/// without anyone at the keyboard. The last frame is written to `rc2k_frame.raw`.
static SHOWN: AtomicU32 = AtomicU32::new(0);
pub static FRAMES_TO_SHOW: AtomicU32 = AtomicU32::new(0);
static LAST_SHOWN: AtomicUsize = AtomicUsize::new(0);

fn quitting() -> bool {
    FLAGS.quitting.get() == 1
}

/// An SDL event the program looks at.
enum Event {
    Quit,
    Focus(bool),
    Key { scancode: i32, down: bool },
    Other,
}

/// SDL's next event, if there is one.
fn next_event() -> Option<Event> {
    // SAFETY: an all-zero `SDL_Event` is valid (a union of plain records); SDL fills it, and
    // its `type` says which member the reads below take.
    unsafe {
        let mut event: SDL_Event = std::mem::zeroed();
        if !SDL_PollEvent(&mut event) {
            return None;
        }
        let kind = event.r#type;
        Some(if kind == SDL_EVENT_QUIT.0 {
            Event::Quit
        } else if kind == SDL_EVENT_WINDOW_FOCUS_GAINED.0 {
            Event::Focus(true)
        } else if kind == SDL_EVENT_WINDOW_FOCUS_LOST.0 {
            Event::Focus(false)
        } else if kind == SDL_EVENT_KEY_DOWN.0 || kind == SDL_EVENT_KEY_UP.0 {
            Event::Key {
                scancode: event.key.scancode.0,
                down: event.key.down,
            }
        } else {
            Event::Other
        })
    }
}

/// Set by `restore_window` (any thread): the window is restored at the next pump, on the
/// thread that made it.
static RESTORE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// The game's window restored if it is minimised (`IsIconic` / `ShowWindow(SW_RESTORE)`),
/// at the next pump of SDL's events.
pub fn restore_window() {
    RESTORE.store(true, std::sync::atomic::Ordering::Relaxed);
}

/// The window restored if asked and minimised (on the window's thread).
fn restore_if_asked() {
    if !RESTORE.swap(false, std::sync::atomic::Ordering::Relaxed) {
        return;
    }
    let Ok(screen) = SCREEN.lock() else {
        return;
    };
    if let Some(screen) = screen.as_ref() {
        let window = std::ptr::with_exposed_provenance_mut::<SDL_Window>(screen.window);
        // SAFETY: the window SDL made, alive while `SCREEN` holds it (locked here).
        unsafe {
            if SDL_GetWindowFlags(window).0 & SDL_WINDOW_MINIMIZED.0 != 0 {
                SDL_RestoreWindow(window);
            }
        }
    }
}

/// SDL's events: the window's closing, and the keys the menus take.
fn pump() {
    restore_if_asked();
    while let Some(event) = next_event() {
        match event {
            Event::Quit => {
                log("platform: the window was closed: the quit switch is set");
                FLAGS.quitting.set(1);
            }
            // As the window's procedure on `WM_ACTIVATE`: the input thread reads the devices
            // (and does not sleep a second a tick) while it is 1.
            Event::Focus(focused) => FLAGS.window_focused.set(focused as u32),
            Event::Key { scancode, down } => {
                let key = game_key(scancode);
                if key == 0 {
                    continue;
                }
                // The key-down table starts at the pending key and is indexed by the game's
                // key code (0xFF down, as `sar al, 7` leaves it); a key that goes down, or
                // repeats, is the pending key, as the key thread makes it.
                if down {
                    DEVICES.set_key_state(key, 0xFF);
                    crate::input::take_key::pending_key()
                        .store(key, std::sync::atomic::Ordering::SeqCst);
                } else {
                    DEVICES.set_key_state(key, 0);
                }
            }
            Event::Other => {}
        }
    }
}

/// SDL's scan code (USB's numbering) as the game's key code: DirectInput's for
/// 1 .. 0x58, and the key thread's (`sub_407AC2`) own numbers for the extended keys it
/// looks at (0x9C .. 0xDD become 0x60 .. 0x7A). 0: a key the game does not know.
fn game_key(scancode: i32) -> u8 {
    const LETTERS: [u8; 26] = [
        0x1E, 0x30, 0x2E, 0x20, 0x12, 0x21, 0x22, 0x23, 0x17, 0x24, 0x25, 0x26, 0x32, 0x31, 0x18,
        0x19, 0x10, 0x13, 0x1F, 0x14, 0x16, 0x2F, 0x11, 0x2D, 0x15, 0x2C,
    ];
    const KEYPAD: [u8; 10] = [0x4F, 0x50, 0x51, 0x4B, 0x4C, 0x4D, 0x47, 0x48, 0x49, 0x52];
    match scancode {
        4..=29 => LETTERS[(scancode - 4) as usize],
        30..=38 => (scancode - 30 + 2) as u8,    // 1 .. 9
        39 => 0x0B,                              // 0
        40 => 0x1C,                              // return
        41 => 0x01,                              // escape
        42 => 0x0E,                              // backspace
        43 => 0x0F,                              // tab
        44 => 0x39,                              // space
        45 => 0x0C,                              // minus
        46 => 0x0D,                              // equals
        47 => 0x1A,                              // [
        48 => 0x1B,                              // ]
        49 | 50 => 0x2B,                         // backslash
        51 => 0x27,                              // ;
        52 => 0x28,                              // '
        53 => 0x29,                              // `
        54 => 0x33,                              // ,
        55 => 0x34,                              // .
        56 => 0x35,                              // /
        57 => 0x3A,                              // caps lock
        58..=67 => (scancode - 58 + 0x3B) as u8, // F1 .. F10
        68 => 0x57,                              // F11
        69 => 0x58,                              // F12
        71 => 0x46,                              // scroll lock
        73 => 0x6D,                              // insert
        74 => 0x65,                              // home
        75 => 0x67,                              // page up
        76 => 0x6E,                              // delete
        77 => 0x6A,                              // end
        78 => 0x6C,                              // page down
        79 => 0x69,                              // right
        80 => 0x68,                              // left
        81 => 0x6B,                              // down
        82 => 0x66,                              // up
        83 => 0x45,                              // num lock
        84 => 0x62,                              // keypad /
        85 => 0x37,                              // keypad *
        86 => 0x4A,                              // keypad -
        87 => 0x4E,                              // keypad +
        88 => 0x60,                              // keypad enter
        89..=97 => KEYPAD[(scancode - 89) as usize],
        98 => 0x52,  // keypad 0
        99 => 0x53,  // keypad .
        100 => 0x56, // the key beside left shift
        101 => 0x7A, // menu
        224 => 0x1D, // left ctrl
        225 => 0x2A, // left shift
        226 => 0x38, // left alt
        227 => 0x78, // left windows
        228 => 0x61, // right ctrl
        229 => 0x36, // right shift
        230 => 0x64, // right alt (AltGr)
        231 => 0x79, // right windows
        _ => 0,
    }
}

// ---- gfx::choose_screen_modes (0x404481) ----

static CHOOSE: RawHook = RawHook::new("gfx::choose_screen_modes", rva(0x404481));

/// `gfx::choose_screen_modes`: the menus' and the race's modes (the default where none is
/// kept) and the graphics options' choices. It cannot fail here (the original's carry).
pub fn choose_screen_modes() {
    CHOOSE.count_call();
    {
        for mode in [&SCREEN_MODE.menu_mode, &SCREEN_MODE.race_mode] {
            if mode.get() == 0 {
                mode.set(DEFAULT_MODE);
            }
        }
        log(&format!(
            "platform: screen modes {:#x} for the menus, {:#x} for a race",
            SCREEN_MODE.menu_mode.get(),
            SCREEN_MODE.race_mode.get()
        ));
    }
    // The graphics options' choices made, as the original ends (`loc_4045CA`): the list
    // the options page shows its resolution from (not made yet at the start, so made
    // whatever eax holds).
    let made = crate::gfx::screen_mode_choices::screen_mode_choices(false);
    if let Err(e) = made {
        log(&format!(
            "platform: the screen mode choices are not made: {e:?}"
        ));
    }
}

// ---- gfx::set_screen_mode (0x404B63) ----

static SET_MODE: RawHook = RawHook::new("gfx::set_screen_mode", rva(0x404B63));

/// The value of a table's entry for a key.
fn looked_up(key: u32, table: usize, entries: u32) -> Option<u32> {
    let entry = find_entry(key, table as u32, entries).ok()??;
    Some(crate::pointers::read_or_end(
        entry + 4,
        "a screen mode table",
    ))
}

fn set_screen_mode(mode: u32) -> Result<(), String> {
    SCREEN_MODE.mode_flags.set(mode);
    let size =
        looked_up(mode & 0xFF, MODES.sizes_addr(), 8).ok_or("a screen size that is not known")?;
    let (width, height) = (size & 0xFFFF, size >> 16);
    let bits = looked_up(mode & 0xF00, MODES.pixel_kinds_addr(), 2)
        .ok_or("a kind of pixel that is not known")?;
    let kind =
        looked_up(mode & 0xF_0000, MODES.kinds_addr(), 3).ok_or("a kind that is not known")?;
    let bytes = bits >> 3;
    // As the original (0x404C55): every texture given back before the screen changes;
    // the game loads again what it needs (a trial's 0x40 more found the pool full).
    crate::platform_3d::release_all_textures();
    GAME_SCREEN.width.set(width);
    GAME_SCREEN.height.set(height);
    GAME_SCREEN.bits.set(bits);
    GAME_SCREEN.bytes.set(bytes);
    GAME_SCREEN.row_bytes.set(bytes * width);
    GAME_SCREEN.row_dwords.set((bytes * width) >> 2);
    GAME_SCREEN.kind.set(kind);
    // The z-buffer's depth, which the original picks by the 3D device: none here.
    GAME_SCREEN.depth_bits.set(0x10u32);

    // The surface's pixel format, as DirectDraw would report it: +4 blue, +8 green,
    // +0xC red, +0x10 no alpha.
    let masks: [u32; 3] = if bits == 32 {
        [0xFF, 0xFF00, 0xFF_0000]
    } else {
        [0x001F, 0x07E0, 0xF800]
    };
    let format = &GAME_SCREEN.format;
    for (n, mask) in masks.into_iter().enumerate() {
        format[1 + n].set(mask);
    }
    format[4].set(0);
    describe_format(format);
    // The screen's own texture description (`TEXTURE_TABLES.descriptions[4]`).
    crate::gfx::texture_upload::TEXTURE_TABLES.descriptions[4].update(|d| {
        d.found = 1;
        d.bytes_per_pixel = bytes;
        let six: [u32; 6] = std::array::from_fn(|n| format[n].get());
        d.format = rc2k_core::gfx::pixels::PixelFormat {
            layout: six[0],
            masks: [six[1], six[2], six[3], six[4]],
        };
        d.number = six[5];
    });

    let mut screen = SCREEN.lock().map_err(|_| "the screen's lock")?;
    // SAFETY: SDL's calls on the window SDL made (kept in `SCREEN` while it lives), on
    // the thread that opened it.
    let window = unsafe {
        let window = match screen.as_ref() {
            Some(old) => std::ptr::with_exposed_provenance_mut::<SDL_Window>(old.window),
            None => {
                if !SDL_Init(SDL_INIT_VIDEO) {
                    return Err("SDL does not start".into());
                }
                SDL_CreateWindow(
                    c"Rally Championship".as_ptr(),
                    width as i32,
                    height as i32,
                    SDL_WindowFlags(0),
                )
            }
        };
        if window.is_null() {
            return Err("SDL opens no window".into());
        }
        SDL_SetWindowSize(window, width as i32, height as i32);
        // Black at once, as a mode change leaves the screen: until the first frame is
        // shown the resized window would show what the window system puts there (white
        // under Wine, a frame's flash before the loading picture: owner, 2026-10-19).
        let surface = SDL_GetWindowSurface(window);
        if !surface.is_null() {
            SDL_FillSurfaceRect(surface, std::ptr::null(), 0);
            SDL_UpdateWindowSurface(window);
        }
        window
    };
    let pixels = vec![0u8; (bytes * width * height) as usize];
    let lent = crate::pointers::Lent::new(&pixels);
    DRAW_SURFACE.present_pixels.set(lent.addr());
    DRAW_SURFACE.present_pitch.set(bytes * width);
    DRAW_SURFACE.present_gap.set(0u32);
    // "There are surfaces": what the game's own code asks.
    SURFACES.front.set(1);
    SURFACES.back.set(1);
    *screen = Some(Screen {
        window: window.expose_provenance(),
        width: width as usize,
        height: height as usize,
        bits,
        _lent: lent,
        pixels,
    });
    SCREEN_MODE.mode_set.set(mode);
    crate::platform_3d::describe_texture_formats(bits);
    log(&format!(
        "platform: screen mode {mode:#x}: {width} x {height}, {bits} bits"
    ));
    Ok(())
}

/// `gfx::set_screen_mode`: the screen in `mode`; false (and logged) if it cannot be.
pub fn set_mode_now(mode: u32) -> bool {
    // The 3D drawing noted so far, before the screen is shown, written or changed.
    crate::platform_3d::flush();
    SET_MODE.count_call();
    let done = set_screen_mode(mode);
    if let Err(why) = &done {
        log(&format!("platform: screen mode {mode:#x}: {why}"));
    }
    done.is_ok()
}

// ---- gfx::set_menu_screen_mode (0x404823) ----

static MENU_MODE: RawHook = RawHook::new("gfx::set_menu_screen_mode", rva(0x404823));

/// `gfx::set_menu_screen_mode`: `gfx::set_screen_mode` with the menus' mode; false (and
/// logged) if it cannot be.
pub fn set_menu_mode_now() -> bool {
    // The 3D drawing noted so far, before the screen is shown, written or changed.
    crate::platform_3d::flush();
    MENU_MODE.count_call();
    let mode = SCREEN_MODE.menu_mode.get();
    let done = set_screen_mode(mode);
    if let Err(why) = &done {
        log(&format!("platform: screen mode {mode:#x}: {why}"));
    }
    done.is_ok()
}

// ---- gfx::lock_surface (0x4058AD), gfx::unlock_surface (0x405949) ----

static LOCK: RawHook = RawHook::new("gfx::lock_surface", rva(0x4058AD));

/// `gfx::lock_surface`: the surface locked for drawing (unless quitting or there is no
/// screen); `g_surfaceLocked` after it (1: locked).
pub fn lock_surface() -> u32 {
    // The 3D drawing noted so far, before the screen is shown, written or changed.
    crate::platform_3d::flush();
    LOCK.count_call();
    let have = SCREEN.lock().is_ok_and(|screen| screen.is_some());
    if !quitting() && have {
        LOCKED.surface.set(1);
        // `sub_405726`: the surface the blitters draw on, and the clip rectangle.
        let s = &DRAW_SURFACE;
        let (row_bytes, pitch): (u32, u32) = (
            GAME_SCREEN.row_bytes.get(),
            DRAW_SURFACE.present_pitch.get(),
        );
        s.width.set(GAME_SCREEN.width.get());
        s.height.set(GAME_SCREEN.height.get());
        s.bits_per_pixel.set(GAME_SCREEN.bits.get());
        s.bytes_per_pixel.set(GAME_SCREEN.bytes.get());
        s.row_bytes.set(row_bytes);
        s.row_dwords.set(GAME_SCREEN.row_dwords.get());
        s.format.set(GAME_SCREEN.format_addr());
        s.pitch.set(pitch);
        s.pixels.set(DRAW_SURFACE.present_pixels.get());
        crate::gfx::surface::on_address();
        s.row_gap.set(pitch.wrapping_sub(row_bytes));
        set_clip([s.x.get(), s.y.get(), s.width.get(), s.height.get()]);
    }
    LOCKED.surface.get()
}

static UNLOCK: RawHook = RawHook::new("gfx::unlock_surface", rva(0x405949));

/// `gfx::unlock_surface`: the surface unlocked if it was locked; `g_surfaceLocked` before
/// it (1: it was).
pub fn unlock_surface() -> u32 {
    UNLOCK.count_call();
    let locked: u32 = LOCKED.surface.get();
    if locked == 1 {
        LOCKED.surface.set(0);
    }
    locked
}

// ---- gfx::show_frame (0x4057D1) ----

static SHOW: RawHook = RawHook::new("gfx::show_frame", rva(0x4057D1));

// ---- where the time goes: the main thread and the rasteriser's sampled every millisecond ----

/// Where the sampled threads were, by address, since the last report.
#[cfg(windows)]
static SAMPLES: std::sync::Mutex<Option<std::collections::HashMap<usize, u32>>> =
    std::sync::Mutex::new(None);

/// The time putting frames on the screen (`show_frame`), in microseconds.
static SHOW_MICROS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// The threads the sampler looks at: the main one and the rasteriser's.
#[cfg(windows)]
static SAMPLED: std::sync::Mutex<Vec<u32>> = std::sync::Mutex::new(Vec::new());

/// The rasteriser's threads (rayon's pool), each noted for the sampler when it starts.
pub fn start_threads() {
    let _ = rayon::ThreadPoolBuilder::new()
        .start_handler(|_| {
            // The sampler's ids are the system's own (it opens the threads by them).
            #[cfg(windows)]
            SAMPLED
                .lock()
                .unwrap()
                // SAFETY: no arguments; the calling thread's id.
                .push(unsafe { windows_sys::Win32::System::Threading::GetCurrentThreadId() });
        })
        .build_global();
}

/// No sampler off Windows (it stops threads through Win32).
#[cfg(not(windows))]
fn start_sampler() {}

/// No samples off Windows.
#[cfg(not(windows))]
fn hot_places() -> String {
    String::new()
}

/// Starts the sampler, once (from the main thread): a thread of its own stops each sampled
/// thread every millisecond, notes its instruction pointer and lets it go on.
#[cfg(windows)]
fn start_sampler() {
    use std::sync::Once;
    use windows_sys::Win32::System::Diagnostics::Debug::{CONTEXT, GetThreadContext};
    use windows_sys::Win32::System::Threading::{
        GetCurrentThreadId, OpenThread, ResumeThread, SuspendThread, THREAD_GET_CONTEXT,
        THREAD_SUSPEND_RESUME,
    };
    static STARTED: Once = Once::new();
    STARTED.call_once(|| {
        SAMPLED
            .lock()
            .unwrap()
            // SAFETY: no arguments.
            .insert(0, unsafe { GetCurrentThreadId() });
        *SAMPLES.lock().unwrap() = Some(Default::default());
        std::thread::spawn(move || {
            let mut handles: Vec<(u32, usize)> = Vec::new();
            loop {
                std::thread::sleep(std::time::Duration::from_millis(1));
                for &id in SAMPLED.lock().unwrap().iter() {
                    if !handles.iter().any(|h| h.0 == id) {
                        // SAFETY: a thread id; a null handle on failure, checked.
                        let h = unsafe {
                            OpenThread(THREAD_SUSPEND_RESUME | THREAD_GET_CONTEXT, 0, id)
                        };
                        if !h.is_null() {
                            handles.push((id, h.expose_provenance()));
                        }
                    }
                }
                for &(_, h) in &handles {
                    let thread: windows_sys::Win32::Foundation::HANDLE =
                        std::ptr::with_exposed_provenance_mut(h);
                    // SAFETY: a handle opened above with suspend and context rights, kept
                    // open for the program's life; the thread is resumed before `SAMPLES`
                    // is locked (it may have been stopped holding that lock); `CONTEXT` is
                    // plain data, zeroed as the API expects.
                    unsafe {
                        if SuspendThread(thread) == u32::MAX {
                            continue;
                        }
                        let mut context: CONTEXT = std::mem::zeroed();
                        context.ContextFlags = crate::arch::CONTEXT_CONTROL;
                        let got = GetThreadContext(thread, &mut context) != 0;
                        ResumeThread(thread);
                        if got
                            && let Ok(mut samples) = SAMPLES.lock()
                            && let Some(samples) = samples.as_mut()
                        {
                            *samples.entry(crate::arch::ip(&context)).or_insert(0) += 1;
                        }
                    }
                }
            }
        });
    });
}

/// The samples since the last report, as a line of the hottest places: the program's
/// (`rc2k+offset`, for `llvm-symbolizer`), the game's (its IDA address) or a DLL's.
#[cfg(windows)]
fn hot_places() -> String {
    use windows_sys::Win32::System::LibraryLoader::{
        GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
        GetModuleFileNameA, GetModuleHandleA, GetModuleHandleExA,
    };
    let Some(samples) = SAMPLES
        .lock()
        .ok()
        .and_then(|mut s| s.as_mut().map(std::mem::take))
    else {
        return String::new();
    };
    let total: u32 = samples.values().sum();
    if total == 0 {
        return String::new();
    }
    // Host addresses throughout (sampled instruction pointers, modules); the game's
    // reserved image by its host address.
    // SAFETY: a null name asks for the program's own module.
    let program = unsafe { GetModuleHandleA(std::ptr::null()) }.expose_provenance();
    let game = crate::pointers::host(crate::framework::game_addr(0)).expose_provenance();
    // by function-sized buckets would need symbols: by address, the top 24
    let mut places: Vec<(usize, u32)> = samples.into_iter().collect();
    places.sort_by_key(|p| std::cmp::Reverse(p.1));
    let name = |at: usize| -> String {
        // SAFETY: `at` is only looked up (UNCHANGED_REFCOUNT: nothing held); the module
        // handle is live while the program runs; `file` is the 260 bytes said.
        unsafe {
            let mut module = std::ptr::null_mut();
            if GetModuleHandleExA(
                GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS
                    | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
                std::ptr::with_exposed_provenance::<u8>(at),
                &mut module,
            ) == 0
            {
                return format!("{at:#x}");
            }
            let base = module.expose_provenance();
            if base == program {
                return format!("rc2k+{:#x}", at - base);
            }
            if base == game.wrapping_add(0x40_0000) || base == game {
                return format!("game {:#x}", at.wrapping_sub(game));
            }
            let mut file = [0u8; 260];
            let n = GetModuleFileNameA(module, file.as_mut_ptr(), 260) as usize;
            let file = String::from_utf8_lossy(&file[..n]);
            let file = file.rsplit(['\\', '/']).next().unwrap_or("?").to_string();
            format!("{file}+{:#x}", at - base)
        }
    };
    // Every place, for adding up by function offline (`llvm-symbolizer` on rc2k.exe).
    let all: String = places
        .iter()
        .map(|&(at, n)| format!("{} {n}\n", name(at)))
        .collect();
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(beside_log("rc2k_profile.txt"))
        .and_then(|mut f| {
            use std::io::Write;
            write!(f, "# {total} samples\n{all}")
        });
    let mut line = format!("platform: {total} samples, the hottest:");
    for &(at, n) in places.iter().take(24) {
        line += &format!(" {} {:.1}%,", name(at), n as f64 * 100.0 / total as f64);
    }
    line
}

/// Every five seconds a line on where the time went: the frames shown, the fans drawn and
/// the time in the rasteriser, the calls of functions that are not ported (each a fault).
fn report_timing() {
    use std::sync::Mutex;
    use std::sync::atomic::Ordering::Relaxed;
    use std::time::Instant;
    /// When, and the counts then.
    type Mark = (Instant, u64, u64, u64, u64, u64);
    static LAST: Mutex<Option<Mark>> = Mutex::new(None);
    let now = Instant::now();
    let (frames, fans, micros, flushes, pixels) = (
        SHOWN.load(Ordering::SeqCst) as u64,
        crate::platform_3d::FANS.load(Relaxed),
        crate::platform_3d::RASTER_MICROS.load(Relaxed),
        crate::platform_3d::FLUSHES.load(Relaxed),
        crate::platform_3d::PIXELS.load(Relaxed),
    );
    // The screen's places, for the pixels a frame in screens.
    let screen = (GAME_SCREEN.width.get().max(1) * GAME_SCREEN.height.get().max(1)) as f64;
    let Ok(mut last) = LAST.lock() else {
        return;
    };
    match *last {
        Some((at, f0, n0, m0, l0, p0)) if now.duration_since(at).as_secs() >= 5 => {
            let seconds = now.duration_since(at).as_secs_f64();
            log(&format!(
                "platform: the last {seconds:.1} s: {} frames ({:.1} a second), {} fans in {} flushes, {:.1} screens of pixels a frame ({:.0} ms in the rasteriser, {:.0}%)",
                frames - f0,
                (frames - f0) as f64 / seconds,
                fans - n0,
                flushes - l0,
                (pixels - p0) as f64 / (frames - f0).max(1) as f64 / screen,
                (micros - m0) as f64 / 1000.0,
                (micros - m0) as f64 / 10000.0 / seconds
            ));
            *last = Some((now, frames, fans, micros, flushes, pixels));
            {
                use crate::platform_3d::{PARALLEL_MICROS, TILE_MICROS};
                static LAST_P: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
                static LAST_T: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
                static LAST_S: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
                let (p, t) = (PARALLEL_MICROS.load(Relaxed), TILE_MICROS.load(Relaxed));
                let (dp, dt) = (p - LAST_P.swap(p, Relaxed), t - LAST_T.swap(t, Relaxed));
                static LAST_N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
                let sh = SHOW_MICROS.load(Relaxed);
                let ds = sh - LAST_S.swap(sh, Relaxed);
                let no = crate::platform_3d::NOTE_MICROS.load(Relaxed);
                let dn = no - LAST_N.swap(no, Relaxed);
                log(&format!(
                    "platform: the rasteriser's parallel part {:.0} ms, the threads in its tiles {:.0} ms ({:.1} threads busy on average); showing the frames {:.0} ms; noting the fans {:.0} ms",
                    dp as f64 / 1000.0,
                    dt as f64 / 1000.0,
                    dt as f64 / dp.max(1) as f64,
                    ds as f64 / 1000.0,
                    dn as f64 / 1000.0
                ));
            }
            let hot = hot_places();
            if !hot.is_empty() {
                log(&hot);
            }
        }
        Some(_) => {}
        None => *last = Some((now, frames, fans, micros, flushes, pixels)),
    }
}

fn show_frame(screen: &Screen) -> Result<(), &'static str> {
    let window = std::ptr::with_exposed_provenance_mut::<SDL_Window>(screen.window);
    // SAFETY: the window SDL made (`set_screen_mode`), alive while `SCREEN` holds it.
    let surface = unsafe { SDL_GetWindowSurface(window) };
    if surface.is_null() {
        return Err("the window has no surface");
    }
    // SAFETY: SDL's surface of a live window, until the next call on the window.
    let s = unsafe { &*surface };
    if s.pixels.is_null() || s.pitch <= 0 {
        return Err("the window's surface has no pixels");
    }
    let rows = screen.height.min(s.h as usize);
    let columns = screen.width.min(s.w as usize);
    // The frame as XRGB8888: into the surface when it is that (Windows'), else into a
    // buffer SDL converts into the surface's format (Wayland's, X11's: the same colours).
    let direct = s.format == SDL_PIXELFORMAT_XRGB8888;
    let mut buffer = Vec::new();
    let (to_all, pitch) = if direct {
        // SAFETY: the surface's `h` rows of `pitch` bytes (SDL's surface), written only here.
        let all = unsafe {
            std::slice::from_raw_parts_mut(s.pixels as *mut u8, s.pitch as usize * s.h as usize)
        };
        (all, s.pitch as usize)
    } else {
        buffer.resize(columns * 4 * rows, 0u8);
        (&mut buffer[..], columns * 4)
    };
    let bytes = (screen.bits / 8) as usize;
    for y in 0..rows {
        let to = &mut to_all[y * pitch..][..columns * 4];
        let from = &screen.pixels[y * screen.width * bytes..][..columns * bytes];
        if bytes == 4 {
            to.copy_from_slice(from);
            continue;
        }
        for (to, from) in to
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .zip(from.as_chunks::<2>().0)
        {
            let p = u16::from_le_bytes(*from) as u32;
            let (red, green, blue) = (p >> 11 & 0x1F, p >> 5 & 0x3F, p & 0x1F);
            let p = (red << 3 | red >> 2) << 16
                | (green << 2 | green >> 4) << 8
                | blue << 3
                | blue >> 2;
            to.copy_from_slice(&p.to_le_bytes());
        }
    }
    // SAFETY: `buffer` holds `rows` rows of `pitch` bytes; the surface's pixels are its
    // `h` rows of `pitch` bytes, at least `rows` of `columns` pixels.
    if !direct
        && !unsafe {
            SDL_ConvertPixels(
                columns as i32,
                rows as i32,
                SDL_PIXELFORMAT_XRGB8888,
                buffer.as_ptr().cast(),
                pitch as i32,
                s.format,
                s.pixels,
                s.pitch,
            )
        }
    {
        return Err("SDL cannot put the frame in the window's surface's format");
    }
    // SAFETY: as above.
    unsafe { SDL_UpdateWindowSurface(window) };
    Ok(())
}

/// `gfx::show_frame`: the frame drawn shown (at most some sixty a second); false while
/// quitting (nothing shown).
pub fn show_frame_now() -> bool {
    // The 3D drawing noted so far, before the screen is shown, written or changed.
    crate::platform_3d::flush();
    SHOW.count_call();
    {
        if quitting() {
            return false;
        }
        if let Ok(screen) = SCREEN.lock()
            && let Some(screen) = screen.as_ref()
        {
            let showing = std::time::Instant::now();
            let shown_ok = show_frame(screen);
            SHOW_MICROS.fetch_add(
                showing.elapsed().as_micros() as u64,
                std::sync::atomic::Ordering::Relaxed,
            );
            if let Err(why) = shown_ok {
                log(&format!("platform: no frame shown: {why}"));
            }
            DRAW_SURFACE.flipped.set(DRAW_SURFACE.flipped.get() ^ 1);
            DRAW_SURFACE
                .frames
                .set(DRAW_SURFACE.frames.get().wrapping_add(1));
            let shown = SHOWN.fetch_add(1, Ordering::SeqCst) + 1;
            let wanted = FRAMES_TO_SHOW.load(Ordering::SeqCst);
            if wanted != 0 && shown == wanted {
                let _ = std::fs::write("rc2k_frame.raw", &screen.pixels);
                log(&format!(
                    "platform: {shown} frames shown; the last is in rc2k_frame.raw ({} x {}, {} bits); quitting",
                    screen.width, screen.height, screen.bits
                ));
                FLAGS.quitting.set(1);
            }
        }
        start_sampler();
        report_timing();
        // A flip waits for the screen: some sixty frames a second at most.
        // SAFETY: SDL's clock and wait, no arguments that point anywhere.
        let now = unsafe { SDL_GetTicks() } as usize;
        let since = now.wrapping_sub(LAST_SHOWN.swap(now, Ordering::SeqCst));
        if since < 16 {
            // SAFETY: as above.
            unsafe { SDL_Delay((16 - since) as u32) };
        }
    }
    true
}

// ---- system::pump_messages (0x410535), screens::run_frame (0x405A71) ----

static PUMP: RawHook = RawHook::new("system::pump_messages", rva(0x410535));

/// `system::pump_messages`: the window's events (SDL's).
pub fn pump_messages_now() {
    PUMP.count_call();
    pump();
}

static RUN_FRAME: RawHook = RawHook::new("screens::run_frame", rva(0x405A71));

/// `screens::run_frame` for Rust callers with a frame of their own (the screen loops,
/// 2026-10-24): the messages, then the frame unless quitting (None then).
pub fn run_frame_now<T>(frame: impl FnOnce() -> T) -> Option<T> {
    RUN_FRAME.count_call();
    pump();
    if quitting() { None } else { Some(frame()) }
}

pub static HOOKS: &[&dyn GameHook] = &[
    &CHOOSE,
    &SET_MODE,
    &MENU_MODE,
    &LOCK,
    &UNLOCK,
    &SHOW,
    &PUMP,
    &RUN_FRAME,
    &crate::film::HOOK,
];

/// A file of ours next to the log (the current folder can change: the installer).
#[cfg(windows)]
pub fn beside_log(name: &str) -> std::path::PathBuf {
    match crate::framework::LOG_PATH.get().and_then(|p| p.parent()) {
        Some(folder) => folder.join(name),
        None => name.into(),
    }
}
