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
//!   `system::pump_messages` (0x410535): SDL's events (`platform_events`).
//!
//! Where the time goes: `platform_timing`.

use crate::framework::log;
use crate::gfx::describe_format::describe_format;
use crate::gfx::front::SURFACES;
use crate::gfx::locked::LOCKED;
use crate::gfx::modes::MODES;
use crate::gfx::screen_state::DRAW_SURFACE;
use crate::gfx::screen_state::SCREEN_MODE;
use crate::gfx::set_clip::set_clip;
use crate::session::FLAGS;
use crate::session::SCREEN as GAME_SCREEN;
use crate::system::find_entry::find_entry;
use sdl3_sys::everything::*;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

// ---- game data ----

// ---- the screen ----

pub(crate) struct Screen {
    /// The pixels a region while the screen lives (the game's code draws on them by
    /// address: `pointers`). (Declared first: fields drop in order, and the region must go before the memory does.)
    _lent: crate::pointers::Lent,
    pub(crate) window: usize,
    /// The window's renderer (`SDL_Renderer`, 2026-10-30: the frame goes up as a texture,
    /// the debug UI drawn over it) and the frame's texture (XRGB8888, the screen's size).
    renderer: usize,
    texture: usize,
    width: usize,
    height: usize,
    bits: u32,
    pixels: Vec<u8>,
}

pub(crate) static SCREEN: Mutex<Option<Screen>> = Mutex::new(None);
/// Frames shown, and after how many the program ends itself (0: never): for trying it
/// without anyone at the keyboard. The last frame is written to `rc2k_frame.raw`.
pub(crate) static SHOWN: AtomicU32 = AtomicU32::new(0);
pub static FRAMES_TO_SHOW: AtomicU32 = AtomicU32::new(0);
static LAST_SHOWN: AtomicUsize = AtomicUsize::new(0);

fn quitting() -> bool {
    FLAGS.quitting.get() == 1
}

// ---- gfx::choose_screen_modes (0x404481) ----

/// `gfx::choose_screen_modes`: the menus' and the race's modes (the kept ones, else the
/// original's choices in order) and the graphics options' choices. Fails (the original's
/// carry, which its caller makes fatal error 0x90201101) when no mode works.
pub fn choose_screen_modes() -> Result<(), crate::errors::FatalError> {
    {
        // Converted: `rc2k_core::gfx::screen_modes::choose`. A mode works here when the
        // mode table lists it: the window takes any listed mode, and every listed mode
        // passes the 3D check (`gfx::mark_3d_modes`).
        use crate::gfx::modes::MODES;
        use rc2k_core::gfx::screen_modes::{Lookup, choose, look_up};
        let (table, count) = (MODES.table_addr() as u32, MODES.count.get());
        let works = |mode: u32| {
            let entries = crate::system::find_entry::entries(table, count);
            matches!(look_up(entries, mode), Lookup::Found(_))
        };
        let listed = |bits: u32| {
            crate::system::find_entry_with_bits::find_entry_with_bits(bits, table, count).0
        };
        let (menu, race) = (SCREEN_MODE.menu_mode.get(), SCREEN_MODE.race_mode.get());
        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        let race = crate::handheld::race_mode(race);
        match choose(menu, race, works, listed) {
            Some((menu, race)) => {
                SCREEN_MODE.race_mode.set(race);
                SCREEN_MODE.menu_mode.set(menu);
                log(&format!(
                    "platform: screen modes {menu:#x} for the menus, {race:#x} for a race"
                ));
            }
            None => {
                return Err(crate::errors::FatalError {
                    code: 0x9020_1101,
                    message: "",
                });
            }
        }
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
    Ok(())
}

// ---- gfx::set_screen_mode (0x404B63) ----

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
    // the thread that opened it; its renderer and textures the same.
    let (window, renderer, texture) = unsafe {
        let (window, renderer) = match screen.as_ref() {
            Some(old) => {
                SDL_DestroyTexture(std::ptr::with_exposed_provenance_mut::<SDL_Texture>(
                    old.texture,
                ));
                (
                    std::ptr::with_exposed_provenance_mut::<SDL_Window>(old.window),
                    std::ptr::with_exposed_provenance_mut::<SDL_Renderer>(old.renderer),
                )
            }
            None => {
                // The handheld's frames go to the framebuffer (`handheld`).
                #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
                SDL_SetHint(SDL_HINT_VIDEO_DRIVER, c"dummy".as_ptr());
                if !SDL_Init(SDL_INIT_VIDEO) {
                    return Err("SDL does not start".into());
                }
                let window = SDL_CreateWindow(
                    c"Rally Championship".as_ptr(),
                    width as i32,
                    height as i32,
                    SDL_WindowFlags(0),
                );
                if window.is_null() {
                    return Err("SDL opens no window".into());
                }
                // `RC2K_RENDERER=gpu`: the window's renderer shares the 3D's device.
                #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
                let device = if crate::render::gpu::wanted() {
                    crate::render::gpu::make_device()
                } else {
                    None
                };
                // The handheld's starts with its first frame (`render::show_on_panel`).
                #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
                let device: Option<*mut SDL_GPUDevice> = None;
                let renderer = match device {
                    Some(device) => SDL_CreateGPURenderer(device, window),
                    None => SDL_CreateRenderer(window, std::ptr::null()),
                };
                #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
                if let Some(device) = device
                    && !renderer.is_null()
                {
                    crate::render::gpu::start(device);
                }
                if renderer.is_null() {
                    return Err("SDL makes no renderer for the window".into());
                }
                (window, renderer)
            }
        };
        SDL_SetWindowSize(window, width as i32, height as i32);
        let texture = SDL_CreateTexture(
            renderer,
            SDL_PIXELFORMAT_XRGB8888,
            SDL_TEXTUREACCESS_STREAMING,
            width as i32,
            height as i32,
        );
        if texture.is_null() {
            return Err("SDL makes no texture for the frame".into());
        }
        // Black at once, as a mode change leaves the screen: until the first frame is
        // shown the resized window would show what the window system puts there (white
        // under Wine, a frame's flash before the loading picture: owner, 2026-10-19).
        SDL_SetRenderDrawColor(renderer, 0, 0, 0, 255);
        SDL_RenderClear(renderer);
        SDL_RenderPresent(renderer);
        (window, renderer, texture)
    };
    #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
    if let Some(gpu) = crate::render::gpu::GPU.lock().unwrap().as_mut() {
        gpu.set_size(width, height)?;
    }
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
        renderer: renderer.expose_provenance(),
        texture: texture.expose_provenance(),
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
    let done = set_screen_mode(mode);
    if let Err(why) = &done {
        log(&format!("platform: screen mode {mode:#x}: {why}"));
    }
    done.is_ok()
}

// ---- gfx::set_menu_screen_mode (0x404823) ----

/// `gfx::set_menu_screen_mode`: `gfx::set_screen_mode` with the menus' mode; false (and
/// logged) if it cannot be.
pub fn set_menu_mode_now() -> bool {
    // The 3D drawing noted so far, before the screen is shown, written or changed.
    crate::platform_3d::flush();
    let mode = SCREEN_MODE.menu_mode.get();
    let done = set_screen_mode(mode);
    if let Err(why) = &done {
        log(&format!("platform: screen mode {mode:#x}: {why}"));
    }
    done.is_ok()
}

// ---- gfx::lock_surface (0x4058AD), gfx::unlock_surface (0x405949) ----

/// `gfx::lock_surface`: the surface locked for drawing (unless quitting or there is no
/// screen); `g_surfaceLocked` after it (1: locked).
pub fn lock_surface() -> u32 {
    // The 3D drawing noted so far, before the screen is shown, written or changed.
    crate::platform_3d::flush();
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

/// `gfx::unlock_surface`: the surface unlocked if it was locked; `g_surfaceLocked` before
/// it (1: it was).
pub fn unlock_surface() -> u32 {
    let locked: u32 = LOCKED.surface.get();
    if locked == 1 {
        LOCKED.surface.set(0);
    }
    locked
}

// ---- gfx::show_frame (0x4057D1) ----

/// The GPU's colour target as a texture of the window's renderer: made once for a target.
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
fn view_of(
    renderer: *mut SDL_Renderer,
    colour: *mut SDL_GPUTexture,
    width: usize,
    height: usize,
) -> *mut SDL_Texture {
    /// The target's handle and its view, as numbers.
    static VIEW: Mutex<(usize, usize)> = Mutex::new((0, 0));
    let mut view = VIEW.lock().unwrap();
    if view.0 != colour.expose_provenance() {
        // SAFETY: SDL's properties, set and read in this block; the renderer and the target
        // are alive (`SCREEN`, `GPU`); the old view is this function's.
        unsafe {
            if view.1 != 0 {
                SDL_DestroyTexture(std::ptr::with_exposed_provenance_mut(view.1));
            }
            let properties = SDL_CreateProperties();
            SDL_SetPointerProperty(
                properties,
                SDL_PROP_TEXTURE_CREATE_GPU_TEXTURE_POINTER,
                colour.cast(),
            );
            SDL_SetNumberProperty(
                properties,
                SDL_PROP_TEXTURE_CREATE_FORMAT_NUMBER,
                SDL_PIXELFORMAT_ARGB8888.0 as i64,
            );
            SDL_SetNumberProperty(
                properties,
                SDL_PROP_TEXTURE_CREATE_WIDTH_NUMBER,
                width as i64,
            );
            SDL_SetNumberProperty(
                properties,
                SDL_PROP_TEXTURE_CREATE_HEIGHT_NUMBER,
                height as i64,
            );
            let made = SDL_CreateTextureWithProperties(renderer, properties);
            SDL_DestroyProperties(properties);
            if made.is_null() {
                crate::fatal::raise::end(
                    "platform::view_of",
                    crate::errors::BadInput("SDL makes no texture of the GPU's picture"),
                );
            }
            SDL_SetTextureBlendMode(made, SDL_BLENDMODE_NONE);
            *view = (colour.expose_provenance(), made.expose_provenance());
        }
    }
    std::ptr::with_exposed_provenance_mut(view.1)
}

fn show_frame(screen: &Screen) -> Result<(), &'static str> {
    // The frame as XRGB8888: itself at 32 bits, else its 16-bit pixels widened.
    let bytes = (screen.bits / 8) as usize;
    let converted;
    let frame: &[u8] = if bytes == 4 {
        &screen.pixels
    } else {
        converted = screen
            .pixels
            .as_chunks::<2>()
            .0
            .iter()
            .flat_map(|from| {
                let p = u16::from_le_bytes(*from) as u32;
                let (red, green, blue) = (p >> 11 & 0x1F, p >> 5 & 0x3F, p & 0x1F);
                let p = (red << 3 | red >> 2) << 16
                    | (green << 2 | green >> 4) << 8
                    | blue << 3
                    | blue >> 2;
                p.to_le_bytes()
            })
            .collect::<Vec<u8>>();
        &converted
    };
    if frame.len() < screen.width * screen.height * 4 {
        return Err("the frame is smaller than the screen");
    }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        // The framebuffer's mode first, its memory then the GPU's or ours.
        crate::handheld::open_screen()?;
        if let Some(target) = crate::platform_3d::screen()
            && crate::render::show_on_panel(&target, frame, screen.width, screen.height)
        {
            return Ok(());
        }
        crate::handheld::present(frame, screen.width, screen.height)
    }
    #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
    {
        let window = std::ptr::with_exposed_provenance_mut::<SDL_Window>(screen.window);
        let renderer = std::ptr::with_exposed_provenance_mut::<SDL_Renderer>(screen.renderer);
        let texture = std::ptr::with_exposed_provenance_mut::<SDL_Texture>(screen.texture);
        // A race on the GPU: its picture, the canvas's written pixels over it.
        let gpu_view = crate::platform_3d::screen()
            .and_then(|target| crate::render::finish_frame(&target))
            .map(|colour| view_of(renderer, colour, screen.width, screen.height));
        // SAFETY: the window's renderer and the frame's texture (`set_screen_mode`), alive while
        // `SCREEN` holds them; `frame` holds the screen's rows of `width` XRGB8888 pixels.
        unsafe {
            if gpu_view.is_none()
                && !SDL_UpdateTexture(
                    texture,
                    std::ptr::null(),
                    frame.as_ptr().cast(),
                    (screen.width * 4) as i32,
                )
            {
                return Err("SDL cannot take the frame into its texture");
            }
            SDL_SetRenderDrawColor(renderer, 0, 0, 0, 255);
            SDL_RenderClear(renderer);
            let shown = match gpu_view {
                Some(view) => view,
                None => texture,
            };
            SDL_RenderTexture(renderer, shown, std::ptr::null(), std::ptr::null());
        }
        // Not the original's: the debug UI over the frame.
        crate::debug::imgui_sdl::draw(renderer, window);
        // SAFETY: as above.
        unsafe { SDL_RenderPresent(renderer) };
        Ok(())
    }
}

/// The frames' running counters, for the debug UI's profiler: microseconds in the
/// rasteriser, noting the triangles and showing the frame; triangle fans; pixels drawn.
#[cfg(debug_ui)]
pub fn frame_counters() -> [u64; 5] {
    use std::sync::atomic::Ordering::Relaxed;
    [
        crate::render::RASTER_MICROS.load(Relaxed),
        crate::platform_3d::NOTE_MICROS.load(Relaxed),
        crate::platform_timing::SHOW_MICROS.load(Relaxed),
        crate::render::FANS.load(Relaxed),
        crate::render::PIXELS.load(Relaxed),
    ]
}

/// The screen's width and height.
#[cfg(debug_ui)]
pub fn screen_size() -> (u32, u32) {
    (GAME_SCREEN.width.get(), GAME_SCREEN.height.get())
}

/// The screen's pixels (its width times its height).
#[cfg(debug_ui)]
pub fn screen_pixels() -> u64 {
    u64::from(GAME_SCREEN.width.get()) * u64::from(GAME_SCREEN.height.get())
}

/// `gfx::show_frame`: the frame drawn shown (at most some sixty a second); false while
/// quitting (nothing shown).
pub fn show_frame_now() -> bool {
    // The 3D drawing noted so far, before the screen is shown, written or changed.
    crate::platform_3d::flush();
    {
        if quitting() {
            return false;
        }
        if let Ok(screen) = SCREEN.lock()
            && let Some(screen) = screen.as_ref()
        {
            let showing = std::time::Instant::now();
            let shown_ok = show_frame(screen);
            crate::platform_timing::SHOW_MICROS.fetch_add(
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
            #[cfg(feature = "remote")]
            crate::remote::frame(&screen.pixels, screen.width as u32, screen.height as u32);
            let shown = SHOWN.fetch_add(1, Ordering::SeqCst) + 1;
            let wanted = FRAMES_TO_SHOW.load(Ordering::SeqCst);
            if wanted != 0 && shown == wanted {
                if let Some(target) = crate::platform_3d::screen() {
                    crate::render::bring_back(&target);
                }
                let _ = std::fs::write("rc2k_frame.raw", &screen.pixels);
                log(&format!(
                    "platform: {shown} frames shown; the last is in rc2k_frame.raw ({} x {}, {} bits); quitting",
                    screen.width, screen.height, screen.bits
                ));
                FLAGS.quitting.set(1);
            }
        }
        crate::platform_timing::start_sampler();
        crate::platform_timing::report_timing();
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

/// `system::pump_messages`: the window's events (SDL's).
pub fn pump_messages_now() {
    crate::platform_events::pump();
}

/// `screens::run_frame` for Rust callers with a frame of their own (the screen loops,
/// 2026-10-24): the messages, then the frame unless quitting (None then).
pub fn run_frame_now<T>(frame: impl FnOnce() -> T) -> Option<T> {
    crate::platform_events::pump();
    if quitting() { None } else { Some(frame()) }
}

/// A file of ours next to the log (the current folder can change: the installer).
#[cfg_attr(all(target_os = "linux", target_arch = "aarch64"), allow(dead_code))] // used by the debug UI, and on Windows
pub fn beside_log(name: &str) -> std::path::PathBuf {
    match crate::framework::LOG_PATH.get().and_then(|p| p.parent()) {
        Some(folder) => folder.join(name),
        None => name.into(),
    }
}
