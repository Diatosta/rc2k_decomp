//! The platform under the game, on SDL3.
//!
//! Each function takes the place of a piece of the game's platform (the screen, the window or
//! the message pump), keeps what that piece means to the game's own code (the variables it
//! leaves, the flags it returns with) and does the rest with SDL.
//! Beside this file: `direct3d` (the 3D layer), `sound`, `input` (joysticks and wheels), `events`,
//! `timing`, `film`, `handheld` (the Anbernic), `web` (the browser), `os` (system services), `ffi`
//! (the C libraries) and `sync` (threads and events), `arch` (processor differences).
//!
//! The screen: the game draws into a locked surface in the screen mode's own pixel
//! layout. Here that surface is memory of ours (`Screen::pixels`, rows without a gap),
//! and `show_frame` copies it to the window (the screen is XRGB8888, as the window takes it).
//!
//! - `gfx::choose_screen_modes`: the kept modes are taken as they are, 800 x 600 in 32 bits
//!   when there is none.
//! - `gfx::set_screen_mode`: the mode's size and pixel kind looked up and its variables set; the
//!   surface made; its pixel format described; the texture formats too (`platform::direct3d.rs`).
//!   `gfx::set_menu_screen_mode` is it with the menus' mode.
//! - `gfx::lock_surface`, `gfx::unlock_surface`, `gfx::show_frame`.
//! - `screens::run_frame` (given the frame's function) and `system::pump_messages`: SDL's events
//!   (`platform::events`).
//!
//! Where the time goes: `platform::timing`.

// Not FFI: `PoolBytes::slice` (the frame's pixels, read in place) is a memory primitive.
#![allow(unsafe_code)]

pub mod arch;
pub mod direct3d;
pub mod events;
pub mod ffi;
pub mod film;
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
pub mod handheld;
pub mod icon;
pub mod input;
pub mod os;
pub mod pacing;
pub mod safe_area;
pub mod sound;
pub mod sync;
pub mod timing;
#[cfg(target_os = "emscripten")]
pub mod web;
pub mod window_mode;

use crate::gfx::describe_format::describe_format;
use crate::gfx::set_clip::set_clip;
use crate::logfile::program_log::log;
use crate::platform::ffi::sdl;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};

// ---- game data ----

// ---- the screen ----

pub(crate) struct Screen {
    /// The game's window.
    pub(crate) window: sdl::Window,
    /// The window's renderer (`SDL_Renderer`: the frame goes up as a texture, the
    /// debug UI drawn over it) and the frame's texture (XRGB8888, the screen's size).
    renderer: sdl::Renderer,
    #[cfg_attr(all(target_os = "linux", target_arch = "aarch64"), allow(dead_code))]
    // the handheld shows frames on its framebuffer
    texture: sdl::Texture,
    /// The frame's size: what the game draws on (800 x 600 in the menus, the race's mode in a race).
    width: usize,
    height: usize,
    /// The window's size: the race's mode, from the start to the end; a frame of another size is
    /// shown scaled in it (`rc2k_core::gfx::screen_fit`).
    window_size: (usize, usize),
    /// The frame is a menu's: the window's side space beside it continues its edge colours.
    #[cfg_attr(all(target_os = "linux", target_arch = "aarch64"), allow(dead_code))]
    // the handheld's panel is the frame's own size
    menu: bool,
    bits: u32,
    /// The pixels the game draws on, named by `locked_pixels::SCREEN` while the screen lives.
    pixels: std::sync::Arc<crate::memory::pool::PoolBytes>,
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
    gfx: &'static crate::gfx::owner::Gfx,
    d3d: &crate::d3d::owner::D3d,
    system: &crate::system::owner::System,
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
        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
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
fn mode_size(gfx: &crate::gfx::owner::Gfx, mode: u32) -> Option<(u32, u32)> {
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
    #[cfg(any(test, all(target_os = "linux", target_arch = "aarch64")))]
    return (Vec::new(), None);
    #[cfg(not(any(test, all(target_os = "linux", target_arch = "aarch64"))))]
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

fn set_screen_mode(
    mode: u32,
    purpose: Purpose,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(), String> {
    // The screen has 32 bits a pixel: a 16-bit mode asked for is the same size at 32.
    let mode = rc2k_core::gfx::screen_modes::at_32_bits(mode);
    gfx.display.mode.mode_flags.set(mode);
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
    crate::platform::direct3d::release_all_textures(gfx);
    system.session.screen.get().width.set(width);
    system.session.screen.get().height.set(height);
    system.session.screen.get().bits.set(bits);
    if let Some(scratch) = crate::platform::direct3d::game_scratch() {
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

    let mut screen = SCREEN.lock().map_err(|_| "the screen's lock")?;
    // The old frame's texture is dropped only once the new screen is whole: on a failure
    // the old one stays as it was.
    // The window is the race's size for the menus' frame too, so that it does not change between
    // a menu and a race.
    let window_size = match purpose {
        Purpose::Menu if cfg!(not(all(target_os = "linux", target_arch = "aarch64"))) => {
            mode_size(gfx, gfx.display.mode.race_mode.get())
                .map_or((width as usize, height as usize), |(w, h)| {
                    (w as usize, h as usize)
                })
        }
        _ => (width as usize, height as usize),
    };
    let window_before = screen.as_ref().map(|old| old.window_size);
    let (window, renderer) = match screen.as_ref() {
        Some(old) => (old.window, old.renderer),
        None => {
            // The handheld's frames go to the framebuffer (`handheld`).
            #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
            sdl::use_dummy_video_driver();
            // The menu entry the installer makes (`install::shortcuts`): the desktop matches
            // the window to it, and to its icon.
            sdl::set_app_identifier(c"rally-championship-rally");
            if !sdl::init_video() {
                return Err("SDL does not start".into());
            }
            let Some(window) =
                sdl::create_window(c"Rally Championship", window_size.0, window_size.1)
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
            #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
            let renderer = make_window_renderer(window, window_size);
            #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
            let renderer = sdl::create_renderer(window);
            let Some(renderer) = renderer else {
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
                if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
                    ""
                } else if crate::render::gpu_running() {
                    ", the 3D on a GPU backend"
                } else {
                    ", the 3D on the software rasteriser"
                }
            ));
            (window, renderer)
        }
    };
    // Only a change of the window's size touches the window: the player's own resizing stays
    // through the menus and the races.
    let window_changed = window_before != Some(window_size);
    if window_changed && !input_window(window).is_full_screen() {
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
    #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
    {
        place_shown_window(window, window_size.0, window_size.1, window_changed);
        // The kept window mode, when the window is first shown (a change later is made at
        // once by `apply_window_mode`).
        if window_before.is_none() && window_mode::get() != window_mode::WindowMode::Window {
            set_window_mode(window, window_size);
        } else if window_changed
            && window_mode::get() == window_mode::WindowMode::FullScreen
            && input_window(window).is_full_screen()
        {
            // A new resolution while in full screen: the display is set to the new size, as
            // leaving full screen and entering it again would (a window's size cannot be
            // changed in full screen, the display's mode is what has the size).
            set_window_mode(window, window_size);
        }
    }
    // The OpenGL backend's window, where it shows the frames, never waits for the refresh.
    #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
    let shows_itself = crate::render::gles::window().is_some();
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    let shows_itself = false;
    crate::platform::pacing::set_display(
        window.display_refresh(),
        renderer.present_waits() && !shows_itself,
    );
    #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
    size_gpu_targets(width, height);
    // What was noted for the old screen is drawn on it before it goes.
    crate::gfx::draw2d::quads::before_canvas_write();
    // Its written rows noted: what the GPU's path uploads of the canvas (`render::gles`). A
    // write to it first draws the glyphs noted as quads (`gfx::draw2d::quads`).
    let pixels = std::sync::Arc::new(
        crate::memory::pool::PoolBytes::noting_rows(
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
    crate::platform::direct3d::describe_texture_formats(gfx);
    log(&format!(
        "platform: screen mode {mode:#x}: {width} x {height}, {bits} bits"
    ));
    Ok(())
}

/// The size the race starts at on an iPhone or iPad that has no choice of its own: the screen's
/// own pixels cut down to what the processor-drawn menus
/// can keep up with (`rc2k_core::gfx::resolutions::native_default`).
#[cfg(target_os = "ios")]
pub fn phone_default_size() -> Option<(u32, u32)> {
    let (_, usable) = monitor_modes();
    usable.map(rc2k_core::gfx::resolutions::native_default)
}

/// The window's renderer and the 3D's backend, started in the order the choice names (the GPU
/// API's renderer on its device, OpenGL beside SDL's own renderer, SDL's own renderer with the
/// software rasteriser); `None` when SDL makes no renderer at all. Logs what runs. `size` is the
/// screen's: the OpenGL backend's window is made at it.
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
fn make_window_renderer(window: sdl::Window, size: (usize, usize)) -> Option<sdl::Renderer> {
    let gpu_renderer = start_gpu_renderer(window);
    // Next in the order: OpenGL, which shows its frames on a window of its own in place of
    // the one SDL's renderer has (`render::gles`).
    if gpu_renderer.is_none() {
        crate::render::gles::start_on_window(size);
    }
    let renderer = gpu_renderer.or_else(|| sdl::create_renderer(window))?;
    log(&format!(
        "platform: the window's renderer is SDL's `{}`{}",
        renderer.name(),
        if crate::render::gpu_running() {
            ", the 3D on a GPU backend"
        } else {
            ", the 3D on the software rasteriser"
        }
    ));
    Some(renderer)
}

/// The shown window put into the kept window mode (`window_mode`); logs what SDL says.
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
fn set_window_mode(window: sdl::Window, size: (usize, usize)) {
    // A phone's window always fills the screen.
    if cfg!(target_os = "ios") {
        return;
    }
    use sdl::FullScreen;
    use window_mode::WindowMode;
    let shown = input_window(window);
    let mode = window_mode::get();
    let how = match mode {
        WindowMode::Window => FullScreen::Off,
        WindowMode::FullScreen => FullScreen::Exclusive(size.0 as u32, size.1 as u32),
        WindowMode::Borderless => FullScreen::Desktop,
    };
    let done = shown.set_full_screen(how);
    log(&format!(
        "platform: the window mode {} ({how:?}): {}",
        mode.name(),
        if done { "set" } else { "SDL refused it" }
    ));
    if !done && mode != WindowMode::Window {
        // The window stays as it is; the setting goes back so the page shows what is.
        window_mode::set(WindowMode::Window);
    }
}

/// The window put into the kept window mode now (the graphics page's row, Alt+Enter).
pub fn apply_window_mode() {
    #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
    {
        let found = SCREEN
            .lock()
            .ok()
            .and_then(|screen| screen.as_ref().map(|s| (s.window, s.window_size)));
        if let Some((window, size)) = found {
            set_window_mode(window, size);
        }
    }
}

/// The window the frames are shown on, set for the backend that runs: the OpenGL backend's own
/// window takes the game's place (same title, icon, size and spot, the keyboard), SDL's renderer
/// keeping the one it was made on, hidden; with no OpenGL window the game's own is shown.
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
fn place_shown_window(window: sdl::Window, width: usize, height: usize, size_changed: bool) {
    let Some(gl) = crate::render::gles::window() else {
        window.show_raised();
        return;
    };
    match icon::pixels(4) {
        Some(p) if sdl::set_window_icon(gl, p.width, p.height, &p.rgb) => {}
        _ => log("platform: the window's icon was not set"),
    }
    if size_changed || gl.is_hidden() {
        gl.set_size(width, height);
    }
    if gl.is_hidden() {
        // First shown: where the game's window was; after that the player's own place for it.
        gl.set_position(window.position());
        gl.show_raised();
        window.hide();
    }
}

/// The window that has the keyboard and is closed by the player: the OpenGL backend's while it
/// shows the frames, else the game's.
pub(crate) fn input_window(window: sdl::Window) -> sdl::Window {
    #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
    if let Some(gl) = crate::render::gles::window() {
        return gl;
    }
    window
}

/// The GPU backend's targets at the screen's size; a backend that cannot make them is given up
/// (the software rasteriser draws the 3D from here).
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
fn size_gpu_targets(width: u32, height: u32) {
    let mut held = crate::render::gpu::GPU.lock().unwrap();
    let sized = match held.as_mut() {
        Some(_) if crate::render::choice::forced_failure("size") => Err("made to fail".to_string()),
        Some(gpu) => gpu.set_size(width, height),
        None => Ok(()),
    };
    if let Err(why) = sized {
        // The frame texture's size does not matter to the window: the 3D goes on the
        // software rasteriser from here, and the GPU's targets are not kept.
        log(&format!(
            "render: the GPU backend cannot make its targets ({why}): the software \
             rasteriser draws the 3D"
        ));
        *held = None;
        crate::render::gpu_stopped();
    }
}

/// The window's renderer on a GPU device, with the GPU backend started on the same device.
/// `None` (each failure logged) when the GPU is not wanted or a step fails; the device is
/// then given up and the caller makes SDL's own renderer. A backend that does not start
/// leaves the window's GPU renderer in place, drawing the rasteriser's frames.
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
fn start_gpu_renderer(window: sdl::Window) -> Option<sdl::Renderer> {
    if !crate::render::gpu::wanted() {
        if !crate::render::gles::wanted() {
            log("render: the software rasteriser draws the 3D (--renderer or RC2K_RENDERER)");
        }
        return None;
    }
    let device = if crate::render::choice::forced_failure("device") {
        None
    } else {
        crate::render::gpu::make_device()
    }?;
    let renderer = if crate::render::choice::forced_failure("gpu_renderer") {
        None
    } else {
        sdl::create_gpu_renderer(device.as_raw(), window)
    };
    let Some(renderer) = renderer else {
        log(&format!(
            "render: SDL makes no renderer on the GPU device: {}",
            crate::platform::ffi::sdl_gpu::error()
        ));
        device.destroy();
        return None;
    };
    *WINDOW_DEVICE.lock().unwrap() = Some(device);
    crate::render::gpu::start(device);
    Some(renderer)
}

/// The GPU device the window's renderer runs on, while there is one (destroyed after the
/// renderer by `swap_renderer`).
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
static WINDOW_DEVICE: Mutex<Option<crate::platform::ffi::sdl_gpu::Device>> = Mutex::new(None);

/// The 3D's renderer swapped for `wish` while the game runs: the frame is brought back into the
/// canvas, the running backend and the window's renderer are stopped and made again as the
/// order for `wish` has them (each start that fails falls to the next as at the program's
/// start), the frame texture and the GPU's targets are made again. The textures and the canvas
/// go up to the new backend as it first draws them.
///
/// `Ok` when what runs is what was asked for. Otherwise the renderer that ran before is made
/// again, and the error says what the asked one did; if that cannot be made either, whatever
/// the order gives runs (the software rasteriser at the last).
///
/// On the game's thread, outside a frame's showing.
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
pub fn swap_renderer(
    o: &crate::owners::Owners<'static>,
    wish: crate::render::choice::Wish,
) -> Result<(), String> {
    let before = crate::render::running_wish();
    if before == wish && crate::render::choice::wish() == wish {
        return Ok(());
    }
    log(&format!(
        "render: swapping the renderer {before:?} -> {wish:?}"
    ));
    // Whatever was noted is drawn first.
    crate::platform::direct3d::flush();
    let screen_now = o.system.session.screen.get();
    let size = (screen_now.width.get(), screen_now.height.get());
    let target = crate::platform::direct3d::screen(screen_now);
    rebuild_renderer(wish, target.as_ref(), size)?;
    let running = crate::render::running_wish();
    if running == wish {
        log(&format!("render: swapped, now {running:?}"));
        return Ok(());
    }
    log(&format!(
        "render: the swap did not give {wish:?}: {running:?} runs"
    ));
    if running != before {
        // The renderer that ran before, again.
        let target = crate::platform::direct3d::screen(screen_now);
        match rebuild_renderer(before, target.as_ref(), size) {
            Ok(()) => log(&format!(
                "render: the old renderer is back, now {:?}",
                crate::render::running_wish()
            )),
            Err(e) => log(&format!(
                "render: the old renderer cannot be made again: {e}"
            )),
        }
    }
    Err(format!("{wish:?} does not start here"))
}

/// Stops what draws the 3D and shows the frame, and starts the renderers the choice `wish`
/// names (`make_window_renderer`).
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
fn rebuild_renderer(
    wish: crate::render::choice::Wish,
    target: Option<&crate::render::Target>,
    (width, height): (u32, u32),
) -> Result<(), String> {
    // The GPU's frame back into the canvas: the picture the new renderer starts from.
    crate::render::release_frame(target);
    let mut guard = SCREEN.lock().map_err(|_| "the screen's lock")?;
    let Some(old) = guard.take() else {
        return Err("there is no screen".into());
    };
    let Screen {
        window,
        renderer,
        texture,
        width: screen_width,
        height: screen_height,
        window_size,
        menu,
        bits,
        pixels,
    } = old;
    // Everything made on the window's renderer goes before it does.
    drop(texture);
    *GPU_VIEW.lock().unwrap() = (0, None, (0, 0));
    *WASH_TEXTURE.lock().unwrap() = None;
    crate::debug::imgui_sdl::renderer_going();
    // Where the player left the OpenGL backend's window, if it was the one shown.
    let left_at = crate::render::gles::window().map(sdl::Window::position);
    crate::render::gles::stop();
    crate::render::gpu::stop();
    sdl::destroy_renderer(renderer);
    if let Some(device) = WINDOW_DEVICE.lock().unwrap().take() {
        device.destroy();
    }
    crate::render::choice::set_swapped_to(wish);
    // Some renderer is made whatever the choice gives: the window needs one.
    let Some(renderer) = make_window_renderer(window, window_size) else {
        crate::fatal::raise::end(
            "platform::swap_renderer",
            crate::errors::BadInput("SDL makes no renderer for the window"),
        );
    };
    let Some(texture) = renderer.create_frame_texture(screen_width, screen_height) else {
        crate::fatal::raise::end(
            "platform::swap_renderer",
            crate::errors::BadInput("SDL makes no texture for the frame"),
        );
    };
    renderer.clear_black();
    renderer.present();
    if let Some(at) = left_at
        && crate::render::gles::window().is_none()
    {
        window.set_position(at);
    }
    place_shown_window(window, window_size.0, window_size.1, true);
    crate::platform::pacing::set_display(
        window.display_refresh(),
        renderer.present_waits() && crate::render::gles::window().is_none(),
    );
    size_gpu_targets(width, height);
    *guard = Some(Screen {
        window,
        renderer,
        texture,
        width: screen_width,
        height: screen_height,
        window_size,
        menu,
        bits,
        pixels,
    });
    Ok(())
}

/// `gfx::set_screen_mode`: the screen in `mode`; false (and logged) if it cannot be.
pub fn set_mode_now(
    mode: u32,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> bool {
    // The 3D drawing noted so far, before the screen is shown, written or changed.
    crate::platform::direct3d::flush();
    let done = set_screen_mode(mode, Purpose::Race, gfx, system);
    if let Err(why) = &done {
        log(&format!("platform: screen mode {mode:#x}: {why}"));
    }
    done.is_ok()
}

// ---- gfx::set_menu_screen_mode ----

/// `gfx::set_menu_screen_mode`: `gfx::set_screen_mode` with the menus' mode; false (and
/// logged) if it cannot be.
pub fn set_menu_mode_now(
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> bool {
    // The 3D drawing noted so far, before the screen is shown, written or changed.
    crate::platform::direct3d::flush();
    // The menus' frame is always 800 x 600 (the kind of pixel the race's): what the options
    // keep as the menus' mode no longer changes it.
    #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
    let mode = rc2k_core::gfx::screen_modes::menu_mode_for(gfx.display.mode.race_mode.get());
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    let mode = gfx.display.mode.menu_mode.get();
    let done = set_screen_mode(mode, Purpose::Menu, gfx, system);
    if let Err(why) = &done {
        log(&format!("platform: screen mode {mode:#x}: {why}"));
    }
    done.is_ok()
}

// ---- gfx::lock_surface, gfx::unlock_surface ----

/// `gfx::lock_surface`: the surface locked for drawing (unless quitting or there is no
/// screen); the locked flag is 1 after it.
pub fn lock_surface(gfx: &crate::gfx::owner::Gfx, system: &crate::system::owner::System) -> u32 {
    // The 3D drawing noted so far, before the screen is shown, written or changed.
    crate::platform::direct3d::flush();
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
pub fn unlock_surface(gfx: &crate::gfx::owner::Gfx) -> u32 {
    let gfx_locked = &gfx.locked;
    let locked: u32 = gfx_locked.surface.get();
    if locked == 1 {
        gfx_locked.surface.set(0);
    }
    locked
}

// ---- gfx::show_frame ----

/// The GPU's colour target as a texture of the window's renderer: made once for a set of
/// targets (`targets_made`, their number), in `GPU_VIEW`.
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
fn make_view(
    renderer: sdl::Renderer,
    (colour, targets_made, (width, height)): (
        *mut sdl3_sys::everything::SDL_GPUTexture,
        u64,
        (u32, u32),
    ),
) {
    let mut view = GPU_VIEW.lock().unwrap();
    if view.0 != targets_made {
        // The old view is destroyed (dropped) when the new one is made.
        let Some(made) = renderer.create_gpu_view(colour, width as usize, height as usize) else {
            crate::fatal::raise::end(
                "platform::view_of",
                crate::errors::BadInput("SDL makes no texture of the GPU's picture"),
            );
        };
        *view = (targets_made, Some(made), (width, height));
    }
}

/// The number of the targets' set the view was made for (none yet: 0), and the view.
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
static GPU_VIEW: Mutex<(u64, Option<sdl::Texture>, (u32, u32))> = Mutex::new((0, None, (0, 0)));

/// `show` on the frame, XRGB8888.
fn with_frame<R>(screen: &Screen, show: impl FnOnce(&[u8]) -> R) -> R {
    // SAFETY: `PoolBytes::slice`'s promise: the screen's pixels are reached from the game's
    // thread only (the backends' calls and the frame's showing run on it; the drawing threads
    // they lend the rows to are joined before `Backend::draw` returns), and nothing on it writes
    // them while `show` runs: the GPU's canvas writes (`render::finish_frame`, `bring_back`,
    // `show_on_panel`) come before the frame is taken, and `show` only reads it. The slice
    // borrows `screen`, which keeps the pixels.
    let pixels = unsafe { screen.pixels.slice() };
    show(pixels)
}

fn show_frame(o: &crate::owners::Owners<'static>, screen: &Screen) -> Result<(), &'static str> {
    let race = o.race;
    // The frame's length (`with_frame`), checked before anything is shown.
    if screen.pixels.len() < screen.width * screen.height * 4 {
        return Err("the frame is smaller than the screen");
    }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        // The framebuffer's mode first, its memory then the GPU's or ours.
        crate::platform::handheld::open_screen()?;
        if let Some(target) = crate::platform::direct3d::screen(o.system.session.screen.get())
            && crate::render::show_on_panel(
                &race.course,
                &target,
                |show| with_frame(screen, show),
                screen.width,
                screen.height,
            )
        {
            return Ok(());
        }
        with_frame(screen, |frame| {
            crate::platform::handheld::present(frame, screen.width, screen.height)
        })
    }
    #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
    {
        // The OpenGL backend's own window shows its frames itself (the debug UI, drawn through
        // SDL's renderer, is not over them).
        if let Some(target) = crate::platform::direct3d::screen(o.system.session.screen.get())
            && crate::render::show_on_gl_window(
                &race.course,
                &target,
                |show| with_frame(screen, show),
                screen.width,
                screen.height,
                screen.menu,
            )
        {
            return Ok(());
        }
        let renderer = screen.renderer;
        // A race on the GPU: its picture, the canvas's written pixels over it.
        let gpu_view = crate::platform::direct3d::screen(o.system.session.screen.get())
            .and_then(|target| crate::render::finish_frame(&race.course, &target))
            .map(|colour| make_view(renderer, colour))
            .is_some();
        if !gpu_view
            && !with_frame(screen, |frame| {
                screen.texture.upload(frame, screen.width * 4)
            })
        {
            return Err("SDL cannot take the frame into its texture");
        }
        // `make_view` has just made the GPU's view.
        let view = GPU_VIEW.lock().unwrap();
        let texture = view
            .1
            .as_ref()
            .filter(|_| gpu_view)
            .unwrap_or(&screen.texture);
        let shown = if gpu_view {
            view.2
        } else {
            (screen.width as u32, screen.height as u32)
        };
        show_fitted(renderer, texture, shown, screen);
        drop(view);
        // The debug UI over the frame.
        crate::debug::imgui_sdl::draw(o, renderer, screen.window);
        #[cfg(feature = "remote")]
        if crate::remote::window_shot_due()
            && let Some((width, height, pixels)) = renderer.read_window()
        {
            crate::remote::window_taken(width, height, pixels);
        }
        renderer.present();
        Ok(())
    }
}

/// The frame's texture drawn on the window: the frame keeps its shape and is centred, magnified
/// with a linear filter (none for a whole factor, which stays sharp). The window's rest is black
/// in a race; beside a menu's frame it shows the menu's background picture, covering the
/// window, blurred and darkened (`render::wash`), black while no background has been noted.
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
fn show_fitted(
    renderer: sdl::Renderer,
    texture: &sdl::Texture,
    shown: (u32, u32),
    screen: &Screen,
) {
    let (frame_width, frame_height) = (screen.width as u32, screen.height as u32);
    let window = renderer
        .window_pixels()
        .unwrap_or((screen.window_size.0 as u32, screen.window_size.1 as u32));
    // A phone's race frame covers the whole screen and its menu page is centred across the part
    // clear of the notch, the rounded corners and the home indicator (`safe_area`).
    #[cfg(target_os = "ios")]
    if let Some(safe) = screen.window.safe_area_pixels() {
        safe_area::set(safe, window);
    }
    let fit = safe_area::frame_fit((frame_width, frame_height), window, screen.menu);
    // A frame drawn larger than the frame (a menu's 3D at the window's size) is shown with the
    // filter unless it is the very size it is shown at.
    texture.set_sharp(if shown == (frame_width, frame_height) {
        fit.sharp
    } else {
        shown == (fit.width, fit.height)
    });
    // The menus' 3D of the next frames is drawn at the size this frame's place has in the window,
    // times the 3D's share.
    crate::render::set_menu_high(if screen.menu {
        let enlarged = u64::from(fit.width) * 100 / u64::from(frame_width.max(1));
        let share = u64::from(crate::render::render_scale());
        (enlarged * share / 100).min(400) as u32
    } else {
        0
    });
    renderer.clear_black();
    if fit.is_whole(window) {
        renderer.draw_whole(texture);
        return;
    }
    if screen.menu {
        draw_wash(renderer, window);
    }
    renderer.draw_part(texture, None, [fit.x, fit.y, fit.width, fit.height]);
}

/// The wash texture of the menu's background (its version, the texture), kept while the
/// background and the window's size stay the same.
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
static WASH_TEXTURE: Mutex<Option<(u64, sdl::Texture)>> = Mutex::new(None);

/// The wash drawn over the whole window, magnified with a linear filter.
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
fn draw_wash(renderer: sdl::Renderer, window: (u32, u32)) {
    let Some(image) = crate::render::wash::image_for(window) else {
        return;
    };
    let mut kept = WASH_TEXTURE.lock().unwrap();
    if kept
        .as_ref()
        .is_none_or(|(version, _)| *version != image.version)
    {
        let wash = &image.wash;
        let made = renderer
            .create_frame_texture(wash.width as usize, wash.height as usize)
            .filter(|texture| texture.upload(&wash.pixels, wash.width as usize * 4));
        *kept = made.map(|texture| {
            texture.set_sharp(false);
            (image.version, texture)
        });
    }
    if let Some((_, texture)) = kept.as_ref() {
        renderer.draw_whole(texture);
    }
}

/// The frames' running counters, for the debug UI's profiler: microseconds in the
/// rasteriser, noting the triangles and showing the frame; triangle fans; pixels drawn.
#[cfg(debug_ui)]
pub fn frame_counters() -> [u64; 5] {
    use std::sync::atomic::Ordering::Relaxed;
    [
        crate::render::RASTER_MICROS.load(Relaxed),
        crate::platform::direct3d::NOTE_MICROS.load(Relaxed),
        crate::platform::timing::SHOW_MICROS.load(Relaxed),
        crate::render::FANS.load(Relaxed),
        crate::render::PIXELS.load(Relaxed),
    ]
}

/// The screen's width and height.
#[cfg(debug_ui)]
pub fn screen_size(system: &crate::system::owner::System) -> (u32, u32) {
    (
        system.session.screen.get().width.get(),
        system.session.screen.get().height.get(),
    )
}

/// The screen's pixels (its width times its height).
#[cfg(debug_ui)]
pub fn screen_pixels(system: &crate::system::owner::System) -> u64 {
    u64::from(system.session.screen.get().width.get())
        * u64::from(system.session.screen.get().height.get())
}

/// `gfx::show_frame`: the frame drawn shown, at the display's pace (`pacing`); false while
/// quitting (nothing shown).
pub fn show_frame_now(o: crate::owners::Owners<'static>) -> bool {
    crate::input::osk_draw::draw(o);
    let drawing = &o.gfx.drawing;
    // The 3D drawing noted so far, before the screen is shown, written or changed.
    crate::platform::direct3d::flush();
    crate::gfx::draw2d::check::end_frame(o.gfx, o.menus.screens.loop_flags.menu.get().ok());
    {
        if o.system.session.flags.quitting.is_set() {
            return false;
        }
        #[cfg(feature = "remote")]
        let shot_wanted = crate::remote::shot_wanted();
        #[cfg(not(feature = "remote"))]
        let shot_wanted = false;
        let video_wanted = crate::video::frame_wanted();
        if (shot_wanted || video_wanted)
            && let Some(target) = crate::platform::direct3d::screen(o.system.session.screen.get())
        {
            crate::render::bring_back(&target);
        }
        if let Ok(screen) = SCREEN.lock()
            && let Some(screen) = screen.as_ref()
        {
            // Before it is shown: on the GPU's path showing clears the canvas.
            #[cfg(feature = "remote")]
            let shot = crate::remote::shot_wanted().then(|| screen.pixels.to_vec());
            let video_frame = video_wanted.then(|| screen.pixels.to_vec());
            let showing = std::time::Instant::now();
            let shown_ok = show_frame(&o, screen);
            crate::platform::timing::SHOW_MICROS.fetch_add(
                showing.elapsed().as_micros() as u64,
                std::sync::atomic::Ordering::Relaxed,
            );
            if let Err(why) = shown_ok {
                log(&format!("platform: no frame shown: {why}"));
            }
            drawing
                .draw_surface
                .flipped
                .set(drawing.draw_surface.flipped.get() ^ 1);
            drawing
                .draw_surface
                .frames
                .set(drawing.draw_surface.frames.get().wrapping_add(1));
            #[cfg(feature = "remote")]
            if let Some(pixels) = shot {
                crate::remote::frame(&pixels, screen.width as u32, screen.height as u32);
            }
            if let Some(pixels) = video_frame {
                crate::video::frame(&pixels, screen.width as u32, screen.height as u32);
                crate::video::sound_turn(o.audio);
                crate::video::note_progress(crate::race::render_row::fraction(o.race));
            }
            crate::platform::timing::note_frame_shown();
            let shown = SHOWN.fetch_add(1, Ordering::SeqCst) + 1;
            let wanted = FRAMES_TO_SHOW.load(Ordering::SeqCst);
            if wanted != 0 && shown == wanted {
                if let Some(target) =
                    crate::platform::direct3d::screen(o.system.session.screen.get())
                {
                    crate::render::bring_back(&target);
                }
                let _ = std::fs::write("rc2k_frame.raw", screen.pixels.to_vec());
                log(&format!(
                    "platform: {shown} frames shown; the last is in rc2k_frame.raw ({} x {}, {} bits); quitting",
                    screen.width, screen.height, screen.bits
                ));
                o.system.session.flags.quitting.set(true);
            }
        }
        crate::platform::timing::start_sampler();
        crate::platform::timing::report_timing(o.system);
        // The next frame waits for its turn on the display.
        crate::platform::pacing::frame_shown();
    }
    true
}

// ---- system::pump_messages, screens::run_frame ----

/// `system::pump_messages`: the window's events (SDL's).
pub fn pump_messages_now(o: crate::owners::Owners<'static>) {
    crate::platform::events::pump(o);
}

/// `screens::run_frame` for Rust callers with a frame of their own (the screen loops): the
/// messages, then the frame unless quitting (None then).
pub fn run_frame_now<T>(o: crate::owners::Owners<'static>, frame: impl FnOnce() -> T) -> Option<T> {
    crate::platform::events::pump(o);
    if o.system.session.flags.quitting.is_set() {
        None
    } else {
        Some(frame())
    }
}

/// A file of ours next to the log (the current folder can change: the installer).
#[cfg_attr(
    any(all(target_os = "linux", target_arch = "aarch64"), target_os = "ios"),
    allow(dead_code, reason = "used by the debug UI and the profiler")
)]
pub fn beside_log(name: &str) -> std::path::PathBuf {
    match crate::logfile::program_log::LOG_PATH
        .get()
        .and_then(|p| p.parent())
    {
        Some(folder) => folder.join(name),
        None => name.into(),
    }
}
