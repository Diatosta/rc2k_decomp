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

use crate::framework::{
    GameHook, RawHook, Regs, game_addr, game_ptr, game_read, game_write, hook_asm, log, rva,
};
use crate::gfx::describe_format::describe_format;
use crate::gfx::set_clip::set_clip;
use crate::system::find_entry::find_entry;
use sdl3_sys::everything::*;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

// ---- game data ----

const QUIT_RVA: usize = rva(0x51837C); // dword_51837C
const PENDING_KEY_RVA: usize = rva(0x4F6ADC); // byte_4F6ADC
const FOCUSED_RVA: usize = rva(0x518378); // dword_518378: 1 while the window has the focus
const MENU_MODE_RVA: usize = rva(0x4F4944); // dword_4F4944
const RACE_MODE_RVA: usize = rva(0x4F4948); // dword_4F4948
const MODE_FLAGS_RVA: usize = rva(0x4F4934); // g_screenModeFlags
const MODE_SET_RVA: usize = rva(0x4F4938); // dword_4F4938
const WIDTH_RVA: usize = rva(0x4F494C); // g_screenWidth
const HEIGHT_RVA: usize = rva(0x4F4950); // g_screenHeight
const BITS_RVA: usize = rva(0x4F4954); // g_screenBits
const BYTES_RVA: usize = rva(0x4F4958); // dword_4F4958
const ROW_BYTES_RVA: usize = rva(0x4F495C); // dword_4F495C
const ROW_DWORDS_RVA: usize = rva(0x4F4960); // dword_4F4960
const DEPTH_BITS_RVA: usize = rva(0x4F4964); // dword_4F4964
const KIND_RVA: usize = rva(0x4F4968); // dword_4F4968
const FORMAT_RVA: usize = rva(0x4F496C); // g_surfaceFormat
const PITCH_RVA: usize = rva(0x4F4984); // dword_4F4984
const PIXELS_RVA: usize = rva(0x4F4988); // dword_4F4988
const GAP_RVA: usize = rva(0x4F498C); // dword_4F498C
const FLIPPED_RVA: usize = rva(0x4F4990); // dword_4F4990
const FRAMES_RVA: usize = rva(0x4F4994); // dword_4F4994
const SURFACE_RVA: usize = rva(0x4F4998); // g_surface
const LOCKED_RVA: usize = rva(0x4F3870); // g_surfaceLocked
const FRONT_RVA: usize = rva(0x4F32B4); // dword_4F32B4: the DirectDraw surfaces
const BACK_RVA: usize = rva(0x4F32B8); // dword_4F32B8
const SIZES_RVA: usize = rva(0x4F4E50); // g_screenSizes
const PIXEL_KINDS_RVA: usize = rva(0x4F4E40); // g_pixelKinds
const KINDS_RVA: usize = rva(0x4F4E90); // dword_4F4E90
const FORMAT_SET_RVA: usize = rva(0x5ABBD4); // dword_5ABBD4, dword_5ABBD8, unk_5ABBDC
const DEFAULT_MODE: u32 = 0x32_2802;

// ---- the screen ----

struct Screen {
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
    unsafe { game_read::<u32>(QUIT_RVA) == 1 }
}

/// SDL's events: the window's closing, and the keys the menus take.
fn pump() {
    unsafe {
        let mut event: SDL_Event = std::mem::zeroed();
        while SDL_PollEvent(&mut event) {
            if event.r#type == SDL_EVENT_QUIT.0 {
                log("platform: the window was closed: the quit switch is set");
                game_write(QUIT_RVA, 1u32);
            } else if event.r#type == SDL_EVENT_WINDOW_FOCUS_GAINED.0
                || event.r#type == SDL_EVENT_WINDOW_FOCUS_LOST.0
            {
                // As the window's procedure on `WM_ACTIVATE`: the input thread reads the
                // devices (and does not sleep a second a tick) while it is 1.
                let focused = event.r#type == SDL_EVENT_WINDOW_FOCUS_GAINED.0;
                game_write(FOCUSED_RVA, focused as u32);
            } else if event.r#type == SDL_EVENT_KEY_DOWN.0 || event.r#type == SDL_EVENT_KEY_UP.0 {
                let key = game_key(event.key.scancode.0);
                if key == 0 {
                    continue;
                }
                // The key-down table starts at the pending key and is indexed by the
                // game's key code (0xFF down, as `sar al, 7` leaves it); a key that goes
                // down, or repeats, is the pending key, as the key thread makes it.
                let state = (game_addr(PENDING_KEY_RVA) as *mut u8).add(key as usize);
                if event.key.down {
                    state.write_volatile(0xFF);
                    game_write(PENDING_KEY_RVA, key);
                } else {
                    state.write_volatile(0);
                }
            }
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

static CHOOSE_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static CHOOSE: RawHook = RawHook::new(
    "gfx::choose_screen_modes",
    rva(0x404481),
    choose_thunk,
    &CHOOSE_ORIGINAL,
);
hook_asm!(choose_thunk, dispatch = choose);

extern "C" fn choose(r: &mut Regs) {
    CHOOSE.count_call();
    unsafe {
        for mode in [MENU_MODE_RVA, RACE_MODE_RVA] {
            if game_read::<u32>(mode) == 0 {
                game_write(mode, DEFAULT_MODE);
            }
        }
        log(&format!(
            "platform: screen modes {:#x} for the menus, {:#x} for a race",
            game_read::<u32>(MENU_MODE_RVA),
            game_read::<u32>(RACE_MODE_RVA)
        ));
    }
    // The graphics options' choices made, as the original ends (`loc_4045CA`): the list
    // the options page shows its resolution from.
    let made = crate::gfx::screen_mode_choices::screen_mode_choices(
        r,
        &mut crate::gfx::screen_mode_choices::game_part,
    );
    if let Err(e) = made {
        log(&format!(
            "platform: the screen mode choices are not made: {e:?}"
        ));
    }
    r.set_carry(false);
}

// ---- gfx::set_screen_mode (0x404B63) ----

static SET_MODE_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static SET_MODE: RawHook = RawHook::new(
    "gfx::set_screen_mode",
    rva(0x404B63),
    set_mode_thunk,
    &SET_MODE_ORIGINAL,
);
hook_asm!(set_mode_thunk, dispatch = set_mode);

/// The value of a table's entry for a key.
unsafe fn looked_up(key: u32, table: usize, entries: u32) -> Option<u32> {
    unsafe {
        let entry = find_entry(key, game_addr(table) as u32, entries).ok()??;
        Some(((entry + 4) as *const u32).read_unaligned())
    }
}

unsafe fn set_screen_mode(mode: u32) -> Result<(), String> {
    unsafe {
        game_write(MODE_FLAGS_RVA, mode);
        let size = looked_up(mode & 0xFF, SIZES_RVA, 8).ok_or("a screen size that is not known")?;
        let (width, height) = (size & 0xFFFF, size >> 16);
        let bits = looked_up(mode & 0xF00, PIXEL_KINDS_RVA, 2)
            .ok_or("a kind of pixel that is not known")?;
        let kind = looked_up(mode & 0xF_0000, KINDS_RVA, 3).ok_or("a kind that is not known")?;
        let bytes = bits >> 3;
        // As the original (0x404C55): every texture given back before the screen changes;
        // the game loads again what it needs (a trial's 0x40 more found the pool full).
        crate::platform_3d::release_all_textures();
        game_write(WIDTH_RVA, width);
        game_write(HEIGHT_RVA, height);
        game_write(BITS_RVA, bits);
        game_write(BYTES_RVA, bytes);
        game_write(ROW_BYTES_RVA, bytes * width);
        game_write(ROW_DWORDS_RVA, (bytes * width) >> 2);
        game_write(KIND_RVA, kind);
        // The z-buffer's depth, which the original picks by the 3D device: none here.
        game_write(DEPTH_BITS_RVA, 0x10u32);

        // The surface's pixel format, as DirectDraw would report it: +4 blue, +8 green,
        // +0xC red, +0x10 no alpha.
        let masks: [u32; 3] = if bits == 32 {
            [0xFF, 0xFF00, 0xFF_0000]
        } else {
            [0x001F, 0x07E0, 0xF800]
        };
        let format = game_ptr::<u32>(FORMAT_RVA);
        for (n, mask) in masks.into_iter().enumerate() {
            format.add(1 + n).write_unaligned(mask);
        }
        format.add(4).write_unaligned(0);
        describe_format(format).map_err(|e| e.0)?;
        game_write(FORMAT_SET_RVA, 1u32);
        game_write(FORMAT_SET_RVA + 4, bytes);
        game_write(
            FORMAT_SET_RVA + 8,
            (format as *const [u32; 6]).read_unaligned(),
        );

        let mut screen = SCREEN.lock().map_err(|_| "the screen's lock")?;
        let window = match screen.as_ref() {
            Some(old) => old.window as *mut SDL_Window,
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
        // Black at once, as a mode change leaves the screen: until the first frame is shown
        // the resized window would show what the window system puts there (white under
        // Wine, a frame's flash before the loading picture: owner, 2026-10-19).
        let surface = SDL_GetWindowSurface(window);
        if !surface.is_null() {
            SDL_FillSurfaceRect(surface, std::ptr::null(), 0);
            SDL_UpdateWindowSurface(window);
        }
        let pixels = vec![0u8; (bytes * width * height) as usize];
        game_write(PIXELS_RVA, pixels.as_ptr() as u32);
        game_write(PITCH_RVA, bytes * width);
        game_write(GAP_RVA, 0u32);
        // "There are surfaces": what the game's own code asks.
        game_write(FRONT_RVA, 1u32);
        game_write(BACK_RVA, 1u32);
        *screen = Some(Screen {
            window: window as usize,
            width: width as usize,
            height: height as usize,
            bits,
            pixels,
        });
        game_write(MODE_SET_RVA, mode);
        crate::platform_3d::describe_texture_formats(bits);
        log(&format!(
            "platform: screen mode {mode:#x}: {width} x {height}, {bits} bits"
        ));
        Ok(())
    }
}

extern "C" fn set_mode(r: &mut Regs) {
    // The 3D drawing noted so far, before the screen is shown, written or changed.
    crate::platform_3d::flush();
    SET_MODE.count_call();
    let done = unsafe { set_screen_mode(r.eax) };
    if let Err(why) = &done {
        log(&format!("platform: screen mode {:#x}: {why}", r.eax));
    }
    r.set_carry(done.is_err());
}

// ---- gfx::set_menu_screen_mode (0x404823) ----

static MENU_MODE_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static MENU_MODE: RawHook = RawHook::new(
    "gfx::set_menu_screen_mode",
    rva(0x404823),
    menu_mode_thunk,
    &MENU_MODE_ORIGINAL,
);
hook_asm!(menu_mode_thunk, dispatch = menu_mode);

/// `gfx::set_screen_mode` with the menus' mode; every register kept, the flags its.
extern "C" fn menu_mode(r: &mut Regs) {
    // The 3D drawing noted so far, before the screen is shown, written or changed.
    crate::platform_3d::flush();
    MENU_MODE.count_call();
    let mode = unsafe { game_read(MENU_MODE_RVA) };
    let done = unsafe { set_screen_mode(mode) };
    if let Err(why) = &done {
        log(&format!("platform: screen mode {mode:#x}: {why}"));
    }
    r.set_carry(done.is_err());
}

// ---- gfx::lock_surface (0x4058AD), gfx::unlock_surface (0x405949) ----

static LOCK_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static LOCK: RawHook = RawHook::new(
    "gfx::lock_surface",
    rva(0x4058AD),
    lock_thunk,
    &LOCK_ORIGINAL,
);
hook_asm!(lock_thunk, dispatch = lock);

/// The flags of `cmp g_surfaceLocked, 1`.
fn compared_with_one(r: &mut Regs, locked: u32) {
    r.set_carry(locked < 1);
    r.set_zero(locked == 1);
}

extern "C" fn lock(r: &mut Regs) {
    // The 3D drawing noted so far, before the screen is shown, written or changed.
    crate::platform_3d::flush();
    LOCK.count_call();
    unsafe {
        let have = SCREEN.lock().is_ok_and(|screen| screen.is_some());
        if !quitting() && have {
            game_write(LOCKED_RVA, 1u32);
            // `sub_405726`: the surface the blitters draw on, and the clip rectangle.
            let surface = game_ptr::<u32>(SURFACE_RVA);
            let (row_bytes, pitch): (u32, u32) = (game_read(ROW_BYTES_RVA), game_read(PITCH_RVA));
            let fields = [
                (2, game_read(WIDTH_RVA)),
                (3, game_read(HEIGHT_RVA)),
                (4, game_read(BITS_RVA)),
                (5, game_read(BYTES_RVA)),
                (6, row_bytes),
                (7, game_read(ROW_DWORDS_RVA)),
                (8, game_addr(FORMAT_RVA) as u32),
                (9, pitch),
                (10, game_read(PIXELS_RVA)),
                (11, pitch.wrapping_sub(row_bytes)),
            ];
            for (n, value) in fields {
                surface.add(n).write_unaligned(value);
            }
            let _ = set_clip(surface as *const [u32; 4]);
        }
        compared_with_one(r, game_read(LOCKED_RVA));
    }
}

static UNLOCK_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static UNLOCK: RawHook = RawHook::new(
    "gfx::unlock_surface",
    rva(0x405949),
    unlock_thunk,
    &UNLOCK_ORIGINAL,
);
hook_asm!(unlock_thunk, dispatch = unlock);

extern "C" fn unlock(r: &mut Regs) {
    UNLOCK.count_call();
    unsafe {
        let locked: u32 = game_read(LOCKED_RVA);
        if locked == 1 {
            game_write(LOCKED_RVA, 0u32);
            r.set_carry(false);
            r.set_zero(true);
        } else {
            compared_with_one(r, locked);
        }
    }
}

// ---- gfx::show_frame (0x4057D1) ----

static SHOW_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static SHOW: RawHook = RawHook::new("gfx::show_frame", rva(0x4057D1), show_thunk, &SHOW_ORIGINAL);
hook_asm!(show_thunk, dispatch = show);

// ---- where the time goes: the main thread and the rasteriser's sampled every millisecond ----

/// Where the sampled threads were, by address, since the last report.
static SAMPLES: std::sync::Mutex<Option<std::collections::HashMap<u32, u32>>> =
    std::sync::Mutex::new(None);

/// The time putting frames on the screen (`show_frame`), in microseconds.
static SHOW_MICROS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// The threads the sampler looks at: the main one and the rasteriser's.
static SAMPLED: std::sync::Mutex<Vec<u32>> = std::sync::Mutex::new(Vec::new());

/// The rasteriser's threads (rayon's pool), each noted for the sampler when it starts.
pub fn start_threads() {
    let _ = rayon::ThreadPoolBuilder::new()
        .start_handler(|_| {
            let id = unsafe { windows_sys::Win32::System::Threading::GetCurrentThreadId() };
            SAMPLED.lock().unwrap().push(id);
        })
        .build_global();
}

/// Starts the sampler, once (from the main thread): a thread of its own stops each sampled
/// thread every millisecond, notes its instruction pointer and lets it go on.
fn start_sampler() {
    use std::sync::Once;
    use windows_sys::Win32::System::Diagnostics::Debug::{
        CONTEXT, CONTEXT_CONTROL_X86, GetThreadContext,
    };
    use windows_sys::Win32::System::Threading::{
        GetCurrentThreadId, OpenThread, ResumeThread, SuspendThread, THREAD_GET_CONTEXT,
        THREAD_SUSPEND_RESUME,
    };
    static STARTED: Once = Once::new();
    STARTED.call_once(|| {
        SAMPLED
            .lock()
            .unwrap()
            .insert(0, unsafe { GetCurrentThreadId() });
        *SAMPLES.lock().unwrap() = Some(Default::default());
        std::thread::spawn(move || {
            let mut handles: Vec<(u32, usize)> = Vec::new();
            loop {
                std::thread::sleep(std::time::Duration::from_millis(1));
                for &id in SAMPLED.lock().unwrap().iter() {
                    if !handles.iter().any(|h| h.0 == id) {
                        let h = unsafe {
                            OpenThread(THREAD_SUSPEND_RESUME | THREAD_GET_CONTEXT, 0, id)
                        };
                        if !h.is_null() {
                            handles.push((id, h as usize));
                        }
                    }
                }
                for &(_, h) in &handles {
                    let thread = h as windows_sys::Win32::Foundation::HANDLE;
                    unsafe {
                        if SuspendThread(thread) == u32::MAX {
                            continue;
                        }
                        let mut context: CONTEXT = std::mem::zeroed();
                        context.ContextFlags = CONTEXT_CONTROL_X86;
                        let got = GetThreadContext(thread, &mut context) != 0;
                        ResumeThread(thread);
                        if got
                            && let Ok(mut samples) = SAMPLES.lock()
                            && let Some(samples) = samples.as_mut()
                        {
                            *samples.entry(context.Eip).or_insert(0) += 1;
                        }
                    }
                }
            }
        });
    });
}

/// The samples since the last report, as a line of the hottest places: the program's
/// (`rc2k+offset`, for `llvm-symbolizer`), the game's (its IDA address) or a DLL's.
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
    let program = unsafe { GetModuleHandleA(std::ptr::null()) } as u32;
    let game = crate::framework::game_addr(0) as u32;
    // by function-sized buckets would need symbols: by address, the top 24
    let mut places: Vec<(u32, u32)> = samples.into_iter().collect();
    places.sort_by_key(|p| std::cmp::Reverse(p.1));
    let name = |at: u32| -> String {
        unsafe {
            let mut module = std::ptr::null_mut();
            if GetModuleHandleExA(
                GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS
                    | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
                at as usize as *const u8,
                &mut module,
            ) == 0
            {
                return format!("{at:#x}");
            }
            let base = module as u32;
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
    let screen = unsafe {
        (game_read::<u32>(rva(0x4F494C)).max(1) * game_read::<u32>(rva(0x4F4950)).max(1)) as f64
    };
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

unsafe fn show_frame(screen: &Screen) -> Result<(), &'static str> {
    unsafe {
        let window = screen.window as *mut SDL_Window;
        let surface = SDL_GetWindowSurface(window);
        if surface.is_null() {
            return Err("the window has no surface");
        }
        let s = &*surface;
        if s.format != SDL_PIXELFORMAT_XRGB8888 || s.pixels.is_null() {
            return Err("the window's surface is not 32 bits of red, green, blue");
        }
        let rows = screen.height.min(s.h as usize);
        let columns = screen.width.min(s.w as usize);
        for y in 0..rows {
            let to = (s.pixels as *mut u8).add(y * s.pitch as usize) as *mut u32;
            if screen.bits == 32 {
                let from = screen.pixels.as_ptr().add(y * screen.width * 4) as *const u32;
                std::ptr::copy_nonoverlapping(from, to, columns);
            } else {
                let from = screen.pixels.as_ptr().add(y * screen.width * 2) as *const u16;
                for x in 0..columns {
                    let p = from.add(x).read_unaligned() as u32;
                    let (red, green, blue) = (p >> 11 & 0x1F, p >> 5 & 0x3F, p & 0x1F);
                    to.add(x).write_unaligned(
                        (red << 3 | red >> 2) << 16
                            | (green << 2 | green >> 4) << 8
                            | blue << 3
                            | blue >> 2,
                    );
                }
            }
        }
        SDL_UpdateWindowSurface(window);
        Ok(())
    }
}

extern "C" fn show(r: &mut Regs) {
    // The 3D drawing noted so far, before the screen is shown, written or changed.
    crate::platform_3d::flush();
    SHOW.count_call();
    unsafe {
        if quitting() {
            r.set_carry(true);
            return;
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
            game_write(FLIPPED_RVA, game_read::<u32>(FLIPPED_RVA) ^ 1);
            game_write(FRAMES_RVA, game_read::<u32>(FRAMES_RVA).wrapping_add(1));
            let shown = SHOWN.fetch_add(1, Ordering::SeqCst) + 1;
            let wanted = FRAMES_TO_SHOW.load(Ordering::SeqCst);
            if wanted != 0 && shown == wanted {
                let _ = std::fs::write("rc2k_frame.raw", &screen.pixels);
                log(&format!(
                    "platform: {shown} frames shown; the last is in rc2k_frame.raw ({} x {}, {} bits); quitting",
                    screen.width, screen.height, screen.bits
                ));
                game_write(QUIT_RVA, 1u32);
            }
        }
        start_sampler();
        report_timing();
        // A flip waits for the screen: some sixty frames a second at most.
        let now = SDL_GetTicks() as usize;
        let since = now.wrapping_sub(LAST_SHOWN.swap(now, Ordering::SeqCst));
        if since < 16 {
            SDL_Delay((16 - since) as u32);
        }
        r.set_carry(false);
    }
}

// ---- system::pump_messages (0x410535), screens::run_frame (0x405A71) ----

static PUMP_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static PUMP: RawHook = RawHook::new(
    "system::pump_messages",
    rva(0x410535),
    pump_thunk,
    &PUMP_ORIGINAL,
);
hook_asm!(pump_thunk, dispatch = pump_messages);

extern "C" fn pump_messages(r: &mut Regs) {
    PUMP.count_call();
    pump();
    r.set_carry(false);
}

static RUN_FRAME_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static RUN_FRAME: RawHook = RawHook::new(
    "screens::run_frame",
    rva(0x405A71),
    run_frame_thunk,
    &RUN_FRAME_ORIGINAL,
);
hook_asm!(run_frame_thunk, dispatch = run_frame);

/// The messages, then the frame's function in eax, whose registers and flags are what
/// the caller gets. (The original also waits for the window to be the active one and
/// gets a lost surface back: neither can happen here.)
extern "C" fn run_frame(r: &mut Regs) {
    RUN_FRAME.count_call();
    pump();
    if quitting() {
        r.set_carry(false);
        r.set_zero(true);
        return;
    }
    if r.eax == 0 {
        r.set_carry(false);
        r.set_zero(true);
        return;
    }
    let frame = r.eax as usize;
    crate::menu::table_calls::call_any(frame as u32, r);
    if !r.carry() {
        r.set_zero(r.eax == 0);
    }
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
pub fn beside_log(name: &str) -> std::path::PathBuf {
    match crate::framework::LOG_PATH.get().and_then(|p| p.parent()) {
        Some(folder) => folder.join(name),
        None => name.into(),
    }
}
