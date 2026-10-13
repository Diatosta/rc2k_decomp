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
//!   format described. The 3D device, the z-buffer and the textures are not there.
//!   `gfx::set_menu_screen_mode` (0x404823) is it with the menus' mode.
//! - `gfx::lock_surface` (0x4058AD), `gfx::unlock_surface` (0x405949),
//!   `gfx::show_frame` (0x4057D1).
//! - `screens::run_frame` (0x405A71, eax = the frame's function) and
//!   `system::pump_messages` (0x410535): SDL's events; closing the window sets the
//!   quit switch, and the keys the menus know become the game's pending key.

use crate::framework::{
    GameHook, RawHook, Regs, call_game, game_addr, game_ptr, game_read, game_write, hook_asm, log,
    rva,
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
            } else if event.r#type == SDL_EVENT_KEY_DOWN.0 {
                let key = match event.key.scancode {
                    SDL_SCANCODE_ESCAPE => 0x01u8,
                    SDL_SCANCODE_RETURN | SDL_SCANCODE_KP_ENTER => 0x1C,
                    SDL_SCANCODE_UP => 0x66,
                    SDL_SCANCODE_DOWN => 0x6B,
                    SDL_SCANCODE_LEFT => 0x68,
                    SDL_SCANCODE_RIGHT => 0x69,
                    _ => 0,
                };
                if key != 0 {
                    game_write(PENDING_KEY_RVA, key);
                }
            }
        }
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
hook_asm!(
    choose_thunk,
    choose_original,
    dispatch = choose,
    original = CHOOSE_ORIGINAL
);

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
hook_asm!(
    set_mode_thunk,
    set_mode_original,
    dispatch = set_mode,
    original = SET_MODE_ORIGINAL
);

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
        log(&format!(
            "platform: screen mode {mode:#x}: {width} x {height}, {bits} bits"
        ));
        Ok(())
    }
}

extern "C" fn set_mode(r: &mut Regs) {
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
hook_asm!(
    menu_mode_thunk,
    menu_mode_original,
    dispatch = menu_mode,
    original = MENU_MODE_ORIGINAL
);

/// `gfx::set_screen_mode` with the menus' mode; every register kept, the flags its.
extern "C" fn menu_mode(r: &mut Regs) {
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
hook_asm!(
    lock_thunk,
    lock_original,
    dispatch = lock,
    original = LOCK_ORIGINAL
);

/// The flags of `cmp g_surfaceLocked, 1`.
fn compared_with_one(r: &mut Regs, locked: u32) {
    r.set_carry(locked < 1);
    r.set_zero(locked == 1);
}

extern "C" fn lock(r: &mut Regs) {
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
hook_asm!(
    unlock_thunk,
    unlock_original,
    dispatch = unlock,
    original = UNLOCK_ORIGINAL
);

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
hook_asm!(
    show_thunk,
    show_original,
    dispatch = show,
    original = SHOW_ORIGINAL
);

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
    SHOW.count_call();
    unsafe {
        if quitting() {
            r.set_carry(true);
            return;
        }
        if let Ok(screen) = SCREEN.lock()
            && let Some(screen) = screen.as_ref()
        {
            if let Err(why) = show_frame(screen) {
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
hook_asm!(
    pump_thunk,
    pump_original,
    dispatch = pump_messages,
    original = PUMP_ORIGINAL
);

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
hook_asm!(
    run_frame_thunk,
    run_frame_original,
    dispatch = run_frame,
    original = RUN_FRAME_ORIGINAL,
    ungated
);

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
    unsafe { call_game(r, frame) };
    if !r.carry() {
        r.set_zero(r.eax == 0);
    }
}

// None of these has an original to go back to.
const _: [unsafe extern "C" fn(*mut Regs); 8] = [
    menu_mode_original,
    choose_original,
    set_mode_original,
    lock_original,
    unlock_original,
    show_original,
    pump_original,
    run_frame_original,
];

pub static HOOKS: &[&dyn GameHook] = &[
    &CHOOSE, &SET_MODE, &MENU_MODE, &LOCK, &UNLOCK, &SHOW, &PUMP, &RUN_FRAME,
];
