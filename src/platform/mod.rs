//! The platform under the game, on SDL3.
//!
//! Each function takes the place of a piece of the game's platform (the screen, the window or
//! the message pump), keeps what that piece means to the game's own code (the variables it
//! leaves, the flags it returns with) and does the rest with SDL.
//! Beside this file: `sound`, `input` (joysticks and wheels), `event` (SDL's events decoded),
//! `pacing`, `timing`, `handheld` (the Anbernic), `os` (system services), `ffi` (the C
//! libraries, with `web`, `arch` and `process` among them) and `sync` (threads and events).
//!
//! The screen: the game draws into a locked surface in the screen mode's own pixel
//! layout. Here that surface is memory of ours (`Screen::pixels`, rows without a gap),
//! and `shell::frame` shows it on the window (the screen is XRGB8888, as the window takes it).
//!
//! - `gfx::choose_screen_modes`: the kept modes are taken as they are, 800 x 600 in 32 bits
//!   when there is none.
//! - `gfx::set_screen_mode`: the mode's size and pixel kind looked up and its variables set; the
//!   surface made; its pixel format described; the texture formats too (`d3d::device.rs`).
//!   `gfx::set_menu_screen_mode` is it with the menus' mode.
//! - `gfx::lock_surface`, `gfx::unlock_surface`; the frame's end and the window's events are
//!   `shell::frame` and `shell::events`.
//!
//! Where the time goes: `platform::timing`.

// Not FFI: `PoolBytes::slice` (the frame's pixels, read in place) is a memory primitive.
#![allow(unsafe_code)]

#[cfg(target_os = "android")]
pub(crate) mod android;
pub(crate) mod caps;
pub(crate) mod event;
pub(crate) mod ffi;
#[cfg(handheld)]
pub(crate) mod handheld;
pub(crate) mod icon;
pub(crate) mod input;
pub(crate) mod os;
pub(crate) mod pacing;
pub(crate) mod pointer;
pub(crate) mod pool;
pub(crate) mod safe_area;
pub(crate) mod sound;
pub(crate) mod sync;
pub(crate) mod timing;
#[cfg(phone)]
pub(crate) mod touch;
#[cfg(web)]
pub(crate) mod web_pads;
pub(crate) mod window_mode;
#[cfg_attr(handheld, path = "window_system_panel.rs")]
mod window_system;

pub use self::window_system::*;

use crate::gfx::describe_format::describe_format;
use crate::gfx::owner::Gfx;
use crate::gfx::set_clip::set_clip;
use crate::platform::ffi::sdl;
use crate::program_log::log;
use crate::system::owner::System;
use std::sync::Mutex;
use std::sync::atomic::AtomicU32;

// ---- game data ----

// ---- the screen ----

pub(crate) struct Screen {
    /// The game's window.
    pub(crate) window: sdl::Window,
    /// The window's renderer (`SDL_Renderer`: the frame goes up as a texture, the
    /// debug UI drawn over it) and the frame's texture (XRGB8888, the screen's size).
    pub(crate) renderer: sdl::Renderer,
    #[cfg_attr(handheld, allow(dead_code))]
    // the handheld shows frames on its framebuffer
    pub(crate) texture: sdl::Texture,
    /// The frame's size: what the game draws on (800 x 600 in the menus, the race's mode in a race).
    pub(crate) width: usize,
    pub(crate) height: usize,
    /// The window's size: the race's mode, from the start to the end; a frame of another size is
    /// shown scaled in it (`rc2k_core::gfx::screen_fit`).
    pub(crate) window_size: (usize, usize),
    /// The frame is a menu's: the window's side space beside it continues its edge colours.
    #[cfg_attr(handheld, allow(dead_code))]
    // the handheld's panel is the frame's own size
    pub(crate) menu: bool,
    pub(crate) bits: u32,
    /// The pixels the game draws on, named by `locked_pixels::SCREEN` while the screen lives.
    pub(crate) pixels: std::sync::Arc<crate::platform::pool::PoolBytes>,
}

impl Screen {
    /// Runs `show` on the frame, XRGB8888.
    pub(crate) fn with_frame<R>(&self, show: impl FnOnce(&[u8]) -> R) -> R {
        // SAFETY: `PoolBytes::slice`'s promise: the screen's pixels are reached from the game's
        // thread only (the backends' calls and the frame's showing run on it; the drawing threads
        // they lend the rows to are joined before `Backend::draw` returns), and nothing on it writes
        // them while `show` runs: the GPU's canvas writes (`render::finish_frame`, `bring_back`,
        // `show_on_panel`) come before the frame is taken, and `show` only reads it. The slice
        // borrows `self`, which keeps the pixels.
        let pixels = unsafe { self.pixels.slice() };
        show(pixels)
    }
}

pub(crate) static SCREEN: Mutex<Option<Screen>> = Mutex::new(None);
/// Frames shown, and after how many the program ends itself (0: never): for trying it
/// without anyone at the keyboard. The last frame is written to `rc2k_frame.raw`.
pub(crate) static SHOWN: AtomicU32 = AtomicU32::new(0);
pub static FRAMES_TO_SHOW: AtomicU32 = AtomicU32::new(0);

// ---- gfx::choose_screen_modes ----

/// `gfx::choose_screen_modes`: the menus' and the race's modes (the kept ones, else the defaults)
/// and the graphics options' choices. Fails (the error the caller makes fatal error 0x90201101)
/// when no mode works.
pub fn choose_screen_modes(
    gfx: &'static Gfx,
    d3d: &crate::d3d::owner::D3d,
    system: &System,
) -> Result<(), crate::errors::FatalError> {
    let gfx_display = &gfx.display;
    {
        // The logic is `rc2k_core::gfx::screen_modes::choose`. A mode works here when the mode
        // table lists it: the window takes any listed mode, and every listed mode passes the 3D
        // check (`gfx::mark_3d_modes`).
        use rc2k_core::gfx::screen_modes::{Lookup, at_32_bits, choose, look_up};
        let display = &gfx_display;
        let modes = &display.modes;
        let table = crate::gfx::modes::Modes::TABLE_AT;
        let works = |mode: u32| {
            let entries = modes.entries(table, modes.count.get());
            matches!(look_up(entries, mode), Lookup::Found(_))
        };
        let listed = |bits: u32| modes.lists_bits(bits);
        // The screen has 32 bits a pixel: a kept 16-bit mode is the same size at 32.
        let (menu, race) = (
            at_32_bits(display.mode.menu_mode.get()),
            at_32_bits(display.mode.race_mode.get()),
        );
        #[cfg(handheld)]
        let race = crate::platform::handheld::race_mode(race);
        match choose(menu, race, works, listed) {
            Some((menu, race)) => {
                display.mode.race_mode.set(race);
                display.mode.menu_mode.set(menu);
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
    // A phone finds out now whether it has a GPU device: the race's size depends on it.
    #[cfg(target_os = "android")]
    {
        sdl::set_app_identifier(c"rally-championship-rally");
        if sdl::init_video() {
            crate::render::gpu::probe_early();
        }
    }
    // The graphics options' choices made: the list the options page shows its resolution from (not
    // made yet at the start, so made whatever the caller holds).
    let made = crate::gfx::screen_mode_choices::screen_mode_choices(gfx, d3d, system, false);
    if let Err(e) = made {
        log(&format!(
            "platform: the screen mode choices are not made: {e:?}"
        ));
    }
    Ok(())
}

// ---- gfx::set_screen_mode ----

/// What a screen mode is for: the window's size follows from it.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Purpose {
    /// A race's (or a trial of one's) mode: the window has the frame's size.
    Race,
    /// The menus' frame: the window stays at the race's size, the frame is shown scaled in it
    /// (the handheld's panel shows the frame itself).
    Menu,
}

/// The screen's size for `mode`: a size beyond the table by its number, else the mode table's.
fn mode_size(gfx: &Gfx, mode: u32) -> Option<(u32, u32)> {
    use crate::gfx::modes::Modes;
    if let Some(size) = crate::gfx::monitor_sizes::size_of_mode(mode) {
        return Some(size);
    }
    let size = gfx.display.modes.value_of(
        rc2k_core::gfx::screen_modes::at_32_bits(mode) & 0xFF,
        Modes::SIZES_AT,
        8,
    )?;
    Some((size & 0xFFFF, size >> 16))
}

/// The display's modes as (width, height, refresh) and its desktop's size; empty where the
/// program has no window system to ask (the handheld, the unit tests). `RC2K_MONITOR_SIZES`
/// (`WxH` words, comma separated) and `RC2K_DESKTOP_SIZE` (`WxH`) stand in for them in trials
/// on a machine without the display.
pub fn monitor_modes() -> sdl::DisplayModes {
    let parse = |text: &str| -> Option<(u32, u32)> {
        let (w, h) = text.trim().split_once(['x', 'X'])?;
        Some((w.trim().parse().ok()?, h.trim().parse().ok()?))
    };
    let modes = std::env::var("RC2K_MONITOR_SIZES").ok().map(|list| {
        list.split(',')
            .filter_map(parse)
            .map(|(w, h)| (w, h, 60.0))
            .collect::<Vec<_>>()
    });
    let desktop = std::env::var("RC2K_DESKTOP_SIZE")
        .ok()
        .and_then(|text| parse(&text));
    if modes.is_some() || desktop.is_some() {
        return (modes.unwrap_or_default(), desktop);
    }
    // The browser: its window's size (the picture fills it) and its screen's (the full screen).
    #[cfg(web)]
    if let Some(sizes) = ffi::web::sizes() {
        let modes = vec![
            (sizes.window.0, sizes.window.1, 60.0),
            (sizes.screen.0, sizes.screen.1, 60.0),
        ];
        return (modes, Some(sizes.screen));
    }
    #[cfg(any(test, handheld))]
    return (Vec::new(), None);
    #[cfg(not(any(test, handheld)))]
    {
        let window = SCREEN
            .lock()
            .ok()
            .and_then(|screen| screen.as_ref().map(|s| s.window));
        #[cfg(target_os = "ios")]
        {
            // A phone's one mode in pixels (the whole screen, landscape), and its fractions: what
            // a frame is made at. The frame covers the screen edge to edge; the HUD keeps to the
            // part clear of the notch and home indicator (`safe_area`).
            let (_, desktop) = sdl::display_modes(window);
            let usable = window
                .and_then(sdl::Window::pixel_size)
                .or(desktop)
                .map(|(w, h)| (w.max(h), w.min(h)));
            let modes = usable
                .map(rc2k_core::gfx::resolutions::native_fractions)
                .unwrap_or_default()
                .into_iter()
                .map(|(w, h)| (w, h, 60.0))
                .collect();
            (modes, usable)
        }
        #[cfg(not(target_os = "ios"))]
        sdl::display_modes(window)
    }
}

/// What a screen mode comes to: the frame's size and its pixels' width.
struct ModeSizes {
    width: u32,
    height: u32,
    bits: u32,
    bytes: u32,
}

fn set_screen_mode(mode: u32, purpose: Purpose, gfx: &Gfx, system: &System) -> Result<(), String> {
    // The screen has 32 bits a pixel: a 16-bit mode asked for is the same size at 32.
    let mode = rc2k_core::gfx::screen_modes::at_32_bits(mode);
    gfx.display.mode.mode_flags.set(mode);
    let ModeSizes {
        width,
        height,
        bits,
        bytes,
    } = set_mode_variables(mode, gfx, system)?;

    let mut screen = SCREEN.lock().map_err(|_| "the screen's lock")?;
    // The old frame's texture is dropped only once the new screen is whole: on a failure
    // the old one stays as it was.
    let window_size = window_size_for(purpose, gfx, (width as usize, height as usize));
    let window_before = screen.as_ref().map(|old| old.window_size);
    let (window, renderer) = match screen.as_ref() {
        Some(old) => (old.window, old.renderer),
        None => open_window(window_size)?,
    };
    // Only a change of the window's size touches the window: the player's own resizing stays
    // through the menus and the races. The browser's full screen is the page's, not the
    // window's: the canvases take the new size in it too, or the frame would sit small in the
    // old size's canvas.
    let window_changed = window_before != Some(window_size);
    if window_changed && (cfg!(web) || !input_window(window).is_full_screen()) {
        window.set_size(window_size.0, window_size.1);
    }
    let Some(texture) = renderer.create_frame_texture(width as usize, height as usize) else {
        return Err("SDL makes no texture for the frame".into());
    };
    // Black at once, as a mode change leaves the screen: until the first frame is
    // shown the resized window would show what the window system puts there (white
    // under Wine, a frame's flash before the loading picture).
    renderer.clear_black();
    renderer.present();
    window_system::settle_window(window, window_size, window_before, window_changed);
    // The OpenGL backend's window, where it shows the frames, never waits for the refresh.
    crate::platform::pacing::set_display(
        window.display_refresh(),
        renderer.present_waits() && !window_system::shows_itself(),
    );
    window_system::size_gpu_targets(width, height);
    let pixels = make_screen_pixels(gfx, width, height, bytes);
    *screen = Some(Screen {
        window,
        renderer,
        texture,
        width: width as usize,
        height: height as usize,
        window_size,
        menu: purpose == Purpose::Menu,
        bits,
        pixels,
    });
    gfx.display.mode.mode_set.set(mode);
    crate::d3d::device::describe_texture_formats(gfx);
    log(&format!(
        "platform: screen mode {mode:#x}: {width} x {height}, {bits} bits"
    ));
    Ok(())
}

/// The mode's size and kind looked up, the textures given back, and the game's screen variables,
/// the surface's pixel format and the screen's texture description set for it.
fn set_mode_variables(mode: u32, gfx: &Gfx, system: &System) -> Result<ModeSizes, String> {
    use crate::gfx::modes::Modes;
    let modes = &gfx.display.modes;
    let (width, height) = mode_size(gfx, mode).ok_or("a screen size that is not known")?;
    let bits = modes
        .value_of(mode & 0xF00, Modes::PIXEL_KINDS_AT, 2)
        .ok_or("a kind of pixel that is not known")?;
    let kind = modes
        .value_of(mode & 0xF_0000, Modes::KINDS_AT, 3)
        .ok_or("a kind that is not known")?;
    let bytes = bits >> 3;
    // Every texture is given back before the screen changes; the game loads again what it needs (a
    // trial's 0x40 more found the pool full).
    crate::d3d::device::release_all_textures(gfx);
    system.session.screen.get().width.set(width);
    system.session.screen.get().height.set(height);
    system.session.screen.get().bits.set(bits);
    if let Some(scratch) = crate::d3d::device::game_scratch() {
        crate::scene::workers::set_game_part_screen(scratch, width, height, bits);
    }
    system.session.screen.get().bytes.set(bytes);
    system.session.screen.get().row_bytes.set(bytes * width);
    system
        .session
        .screen
        .get()
        .row_dwords
        .set((bytes * width) >> 2);
    system.session.screen.get().kind.set(kind);
    // The z-buffer's depth: none here (there is no 3D device to pick it).
    system.session.screen.get().depth_bits.set(0x10u32);

    // The surface's pixel format, as DirectDraw would report it: +4 blue, +8 green,
    // +0xC red, +0x10 no alpha.
    if bits != 32 {
        return Err("a screen mode that is not 32 bits a pixel".into());
    }
    let masks: [u32; 3] = [0xFF, 0xFF00, 0xFF_0000];
    let format = &system.session.screen.get().format;
    for (n, mask) in masks.into_iter().enumerate() {
        format[1 + n].set(mask);
    }
    format[4].set(0);
    describe_format(&gfx.pixel.formats, format);
    // The screen's own texture description (`texture_tables.descriptions[4]`).
    gfx.card.texture_tables.descriptions[4].update(|d| {
        d.found = 1;
        d.bytes_per_pixel = bytes;
        let six: [u32; 6] = std::array::from_fn(|n| format[n].get());
        d.format = rc2k_core::gfx::pixels::PixelFormat {
            layout: six[0],
            masks: [six[1], six[2], six[3], six[4]],
        };
        d.number = six[5];
    });
    Ok(ModeSizes {
        width,
        height,
        bits,
        bytes,
    })
}

/// The window's size for a frame of `frame` pixels: the race's size for the menus' frame too
/// (where the system has a window of its own), so that it does not change between a menu and a
/// race.
fn window_size_for(purpose: Purpose, gfx: &Gfx, frame: (usize, usize)) -> (usize, usize) {
    match purpose {
        Purpose::Menu if window_system::MENU_WINDOW_IS_RACE_SIZED => {
            mode_size(gfx, gfx.display.mode.race_mode.get())
                .map_or(frame, |(w, h)| (w as usize, h as usize))
        }
        _ => frame,
    }
}

/// The video started, the window made and its renderer, for the screen's first mode.
fn open_window(window_size: (usize, usize)) -> Result<(sdl::Window, sdl::Renderer), String> {
    window_system::before_video();
    // The menu entry the installer makes (`install::shortcuts`): the desktop matches
    // the window to it, and to its icon.
    sdl::set_app_identifier(c"rally-championship-rally");
    if !sdl::init_video() {
        return Err("SDL does not start".into());
    }
    let Some(window) = sdl::create_window(c"Rally Championship", window_size.0, window_size.1)
    else {
        return Err("SDL opens no window".into());
    };
    // The window's own icon (128 x 128), where the window system shows one.
    match icon::pixels(4) {
        Some(p) if sdl::set_window_icon(window, p.width, p.height, &p.rgb) => {}
        _ => log("platform: the window's icon was not set"),
    }
    // The GPU first (`render::choice`), each step that fails logged and the next tried
    // (`make_window_renderer`). The handheld's GPU starts with its first frame
    // (`render::show_on_panel`).
    let Some(renderer) = window_system::open_renderer(window, window_size) else {
        return Err("SDL makes no renderer for the window".into());
    };
    #[cfg(target_os = "ios")]
    log(&format!(
        "platform: the window is {:?} points, {:?} pixels, its safe area {:?} (x, y, width, height, pixels)",
        window.point_size(),
        window.pixel_size(),
        window.safe_area_pixels()
    ));
    log(&format!(
        "platform: the window's renderer is SDL's `{}`{}",
        renderer.name(),
        window_system::renderer_note()
    ));
    Ok((window, renderer))
}

/// The screen's memory for a `width` x `height` frame of `bytes` a pixel, registered as the
/// surface the game draws on; what was noted for the old screen is drawn on it before it goes.
fn make_screen_pixels(
    gfx: &Gfx,
    width: u32,
    height: u32,
    bytes: u32,
) -> std::sync::Arc<crate::platform::pool::PoolBytes> {
    // What was noted for the old screen is drawn on it before it goes.
    crate::gfx::draw2d::quads::before_canvas_write();
    // Its written rows noted: what the GPU's path uploads of the canvas (`render::gles`). A
    // write to it first draws the glyphs noted as quads (`gfx::draw2d::quads`).
    let pixels = std::sync::Arc::new(
        crate::platform::pool::PoolBytes::noting_rows(
            vec![0u8; (bytes * width * height) as usize],
            (bytes * width) as usize,
        )
        .with_write_hook(crate::gfx::draw2d::quads::before_canvas_write),
    );
    crate::gfx::locked_pixels::register(crate::gfx::locked_pixels::SCREEN, &pixels);
    gfx.drawing
        .draw_surface
        .present_pixels
        .set(crate::gfx::locked_pixels::SCREEN);
    gfx.drawing.draw_surface.present_pitch.set(bytes * width);
    gfx.drawing.draw_surface.present_gap.set(0u32);
    // "There are surfaces": what the game's own code asks.
    gfx.drawing.surfaces.front.set(1);
    gfx.drawing.surfaces.back.set(1);
    pixels
}

/// The size the race starts at on an iPhone or iPad that has no choice of its own: the screen's
/// own pixels cut down to what the processor-drawn menus
/// can keep up with (`rc2k_core::gfx::resolutions::native_default`).
#[cfg(target_os = "ios")]
pub fn phone_default_size() -> Option<(u32, u32)> {
    let (_, usable) = monitor_modes();
    usable.map(rc2k_core::gfx::resolutions::native_default)
}

/// `gfx::set_screen_mode`: the screen in `mode`; false (and logged) if it cannot be.
pub fn set_mode_now(mode: u32, gfx: &Gfx, system: &System) -> bool {
    // The 3D drawing noted so far, before the screen is shown, written or changed.
    crate::d3d::device::flush();
    let done = set_screen_mode(mode, Purpose::Race, gfx, system);
    if let Err(why) = &done {
        log(&format!("platform: screen mode {mode:#x}: {why}"));
    }
    done.is_ok()
}

// ---- gfx::set_menu_screen_mode ----

/// `gfx::set_menu_screen_mode`: `gfx::set_screen_mode` with the menus' mode; false (and
/// logged) if it cannot be.
pub fn set_menu_mode_now(gfx: &Gfx, system: &System) -> bool {
    // The 3D drawing noted so far, before the screen is shown, written or changed.
    crate::d3d::device::flush();
    // The menus' frame is always 800 x 600 (the kind of pixel the race's): what the options
    // keep as the menus' mode no longer changes it.
    let mode = window_system::menu_mode(
        gfx.display.mode.race_mode.get(),
        gfx.display.mode.menu_mode.get(),
    );
    let done = set_screen_mode(mode, Purpose::Menu, gfx, system);
    if let Err(why) = &done {
        log(&format!("platform: screen mode {mode:#x}: {why}"));
    }
    done.is_ok()
}

// ---- gfx::lock_surface, gfx::unlock_surface ----

/// `gfx::lock_surface`: the surface locked for drawing (unless quitting or there is no
/// screen); the locked flag is 1 after it.
pub fn lock_surface(gfx: &Gfx, system: &System) -> u32 {
    // The 3D drawing noted so far, before the screen is shown, written or changed.
    crate::d3d::device::flush();
    let have = SCREEN.lock().is_ok_and(|screen| screen.is_some());
    if !system.session.flags.quitting.is_set() && have {
        gfx.locked.surface.set(1);
        // The surface the blitters draw on, and the clip rectangle.
        let s = &gfx.drawing.draw_surface;
        let (row_bytes, pitch): (u32, u32) = (
            system.session.screen.get().row_bytes.get(),
            gfx.drawing.draw_surface.present_pitch.get(),
        );
        s.width.set(system.session.screen.get().width.get());
        s.height.set(system.session.screen.get().height.get());
        s.bits_per_pixel.set(system.session.screen.get().bits.get());
        s.bytes_per_pixel
            .set(system.session.screen.get().bytes.get());
        s.row_bytes.set(row_bytes);
        s.row_dwords
            .set(system.session.screen.get().row_dwords.get());
        s.format.set(crate::gfx::pixel_format::SCREEN_FORMAT);
        s.pitch.set(pitch);
        s.pixels.set(gfx.drawing.draw_surface.present_pixels.get());
        crate::gfx::surface::on_address(gfx);
        s.row_gap.set(pitch.wrapping_sub(row_bytes));
        set_clip([s.x.get(), s.y.get(), s.width.get(), s.height.get()], gfx);
    }
    gfx.locked.surface.get()
}

/// `gfx::unlock_surface`: the surface unlocked if it was locked; the locked flag before it
/// (1: it was) is returned.
pub fn unlock_surface(gfx: &Gfx) -> u32 {
    let gfx_locked = &gfx.locked;
    let locked: u32 = gfx_locked.surface.get();
    if locked == 1 {
        gfx_locked.surface.set(0);
    }
    locked
}

/// A file of ours next to the log (the current folder can change: the installer).
#[cfg_attr(
    not(debug_ui),
    allow(dead_code, reason = "the debug UI is the only user")
)]
pub fn beside_log(name: &str) -> std::path::PathBuf {
    match crate::program_log::LOG_PATH.get().and_then(|p| p.parent()) {
        Some(folder) => folder.join(name),
        None => name.into(),
    }
}

/// Opens Android's share sheet with the program's log (the advanced page's share row). Other
/// systems have no such row; the call only logs there.
pub fn share_log() {
    #[cfg(target_os = "android")]
    {
        let Some(path) = crate::program_log::LOG_PATH.get() else {
            log("platform: share log: the log has no file");
            return;
        };
        let Some(text) = path.to_str().and_then(|t| std::ffi::CString::new(t).ok()) else {
            log("platform: share log: the log's path is not usable");
            return;
        };
        match crate::platform::ffi::android_java::call_with_path(
            c"org.rc2k.rally.Share",
            c"log",
            &text,
        ) {
            Some(true) => log("platform: share log: the share sheet is open"),
            Some(false) => log("platform: share log: Android refused the share"),
            None => log("platform: share log: the Java side could not be called"),
        }
    }
    #[cfg(not(target_os = "android"))]
    log("platform: share log: only on Android");
}
