//! The platform under the game, on SDL3, for the standalone program (`standalone.rs`).
//!
//! These are not ports: each takes the place of a function of the game that is
//! DirectDraw, the window or the message pump, keeps what that function means to the
//! game's own code (the variables it leaves, the flags it returns with) and does the
//! rest with SDL. There is no original to compare them with.
//!
//! The screen: the game draws into a locked surface in the screen mode's own pixel
//! layout. Here that surface is memory of ours (`Screen::pixels`, rows without a gap),
//! and `show_frame` copies it to the window, 16-bit pixels made 32-bit on the way.
//!
//! - `gfx::choose_screen_modes`: the original tries the kept modes and, if
//!   they do not work, a list of others, through DirectDraw. Here the kept modes are
//!   taken as they are, 800 x 600 in 32 bits when there is none.
//! - `gfx::set_screen_mode`: the mode's size and pixel kind
//!   looked up as the original does and its variables set; the surface made; its pixel
//!   format described; the texture formats too (`platform_3d.rs`).
//!   `gfx::set_menu_screen_mode` is it with the menus' mode.
//! - `gfx::lock_surface`, `gfx::unlock_surface`, `gfx::show_frame`.
//! - `screens::run_frame` (given the frame's function) and `system::pump_messages`: SDL's events (`platform_events`).
//!
//! Where the time goes: `platform_timing`.

// Not FFI: `PoolBytes::slice` (the frame's pixels, read in place) is a memory primitive.
#![allow(unsafe_code)]

use crate::ffi::sdl;
use crate::framework::log;
use crate::gfx::describe_format::describe_format;
use crate::gfx::set_clip::set_clip;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

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
    width: usize,
    height: usize,
    bits: u32,
    /// The pixels the game draws on, named by `locked_pixels::SCREEN` while the screen lives.
    pixels: std::sync::Arc<crate::pool::PoolBytes>,
}

pub(crate) static SCREEN: Mutex<Option<Screen>> = Mutex::new(None);
/// Frames shown, and after how many the program ends itself (0: never): for trying it
/// without anyone at the keyboard. The last frame is written to `rc2k_frame.raw`.
pub(crate) static SHOWN: AtomicU32 = AtomicU32::new(0);
pub static FRAMES_TO_SHOW: AtomicU32 = AtomicU32::new(0);
static LAST_SHOWN: AtomicUsize = AtomicUsize::new(0);

// ---- gfx::choose_screen_modes (0x404481) ----

/// `gfx::choose_screen_modes`: the menus' and the race's modes (the kept ones, else the
/// original's choices in order) and the graphics options' choices. Fails (the error the caller makes fatal error 0x90201101) when no mode works.
pub fn choose_screen_modes(
    o: crate::owners::Owners<'static>,
) -> Result<(), crate::errors::FatalError> {
    let gfx = o.gfx;
    let gfx_display = &gfx.display;
    {
        // Converted: `rc2k_core::gfx::screen_modes::choose`. A mode works here when the
        // mode table lists it: the window takes any listed mode, and every listed mode
        // passes the 3D check (`gfx::mark_3d_modes`).
        use rc2k_core::gfx::screen_modes::{Lookup, choose, look_up};
        let display = &gfx_display;
        let modes = &display.modes;
        let table = core::mem::offset_of!(crate::gfx::modes::Modes, table);
        let works = |mode: u32| {
            let entries = modes.entries(table, modes.count.get());
            matches!(look_up(entries, mode), Lookup::Found(_))
        };
        let listed = |bits: u32| modes.lists_bits(bits);
        let (menu, race) = (display.mode.menu_mode.get(), display.mode.race_mode.get());
        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        let race = crate::handheld::race_mode(race);
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
    // The graphics options' choices made, as the original ends (`loc_4045CA`): the list
    // the options page shows its resolution from (not made yet at the start, so made
    // whatever the caller holds).
    let made = crate::gfx::screen_mode_choices::screen_mode_choices(gfx, o.d3d, o.system, false);
    if let Err(e) = made {
        log(&format!(
            "platform: the screen mode choices are not made: {e:?}"
        ));
    }
    Ok(())
}

// ---- gfx::set_screen_mode (0x404B63) ----

fn set_screen_mode(
    mode: u32,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(), String> {
    gfx.display.mode.mode_flags.set(mode);
    use crate::gfx::modes::Modes;
    use core::mem::offset_of;
    let modes = &gfx.display.modes;
    let size = modes
        .value_of(mode & 0xFF, offset_of!(Modes, sizes), 8)
        .ok_or("a screen size that is not known")?;
    let (width, height) = (size & 0xFFFF, size >> 16);
    let bits = modes
        .value_of(mode & 0xF00, offset_of!(Modes, pixel_kinds), 2)
        .ok_or("a kind of pixel that is not known")?;
    let kind = modes
        .value_of(mode & 0xF_0000, offset_of!(Modes, kinds), 3)
        .ok_or("a kind that is not known")?;
    let bytes = bits >> 3;
    // As the original: every texture given back before the screen changes;
    // the game loads again what it needs (a trial's 0x40 more found the pool full).
    crate::platform_3d::release_all_textures(gfx);
    system.session.screen.width.set(width);
    system.session.screen.height.set(height);
    system.session.screen.bits.set(bits);
    system.session.screen.bytes.set(bytes);
    system.session.screen.row_bytes.set(bytes * width);
    system.session.screen.row_dwords.set((bytes * width) >> 2);
    system.session.screen.kind.set(kind);
    // The z-buffer's depth, which the original picks by the 3D device: none here.
    system.session.screen.depth_bits.set(0x10u32);

    // The surface's pixel format, as DirectDraw would report it: +4 blue, +8 green,
    // +0xC red, +0x10 no alpha.
    let masks: [u32; 3] = if bits == 32 {
        [0xFF, 0xFF00, 0xFF_0000]
    } else {
        [0x001F, 0x07E0, 0xF800]
    };
    let format = &system.session.screen.format;
    for (n, mask) in masks.into_iter().enumerate() {
        format[1 + n].set(mask);
    }
    format[4].set(0);
    describe_format(&gfx.pixel.formats, format);
    // The screen's own texture description (`TEXTURE_TABLES.descriptions[4]`).
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
    let (window, renderer) = match screen.as_ref() {
        Some(old) => (old.window, old.renderer),
        None => {
            // The handheld's frames go to the framebuffer (`handheld`).
            #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
            sdl::use_dummy_video_driver();
            if !sdl::init_video() {
                return Err("SDL does not start".into());
            }
            let Some(window) =
                sdl::create_window(c"Rally Championship", width as usize, height as usize)
            else {
                return Err("SDL opens no window".into());
            };
            // `RC2K_RENDERER=gpu`: the window's renderer shares the 3D's device.
            #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
            let gpu_device = if crate::render::gpu::wanted() {
                crate::render::gpu::make_device()
            } else {
                None
            };
            #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
            let device = gpu_device.map(crate::ffi::sdl_gpu::Device::as_raw);
            // The handheld's starts with its first frame (`render::show_on_panel`).
            #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
            let device: Option<*mut sdl3_sys::everything::SDL_GPUDevice> = None;
            let renderer = match device {
                Some(device) => sdl::create_gpu_renderer(device, window),
                None => sdl::create_renderer(window),
            };
            #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
            if let Some(device) = gpu_device
                && renderer.is_some()
            {
                crate::render::gpu::start(device);
            }
            let Some(renderer) = renderer else {
                return Err("SDL makes no renderer for the window".into());
            };
            (window, renderer)
        }
    };
    window.set_size(width as usize, height as usize);
    let Some(texture) = renderer.create_frame_texture(width as usize, height as usize) else {
        return Err("SDL makes no texture for the frame".into());
    };
    // Black at once, as a mode change leaves the screen: until the first frame is
    // shown the resized window would show what the window system puts there (white
    // under Wine, a frame's flash before the loading picture).
    renderer.clear_black();
    renderer.present();
    #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
    if let Some(gpu) = crate::render::gpu::GPU.lock().unwrap().as_mut()
        && let Err(why) = gpu.set_size(width, height)
    {
        // The texture made above is dropped (destroyed) with `texture`.
        return Err(why);
    }
    let pixels = std::sync::Arc::new(crate::pool::PoolBytes::from_vec(vec![
        0u8;
        (bytes * width * height)
            as usize
    ]));
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
        bits,
        pixels,
    });
    gfx.display.mode.mode_set.set(mode);
    crate::platform_3d::describe_texture_formats(bits, gfx);
    log(&format!(
        "platform: screen mode {mode:#x}: {width} x {height}, {bits} bits"
    ));
    Ok(())
}

/// `gfx::set_screen_mode`: the screen in `mode`; false (and logged) if it cannot be.
pub fn set_mode_now(
    mode: u32,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> bool {
    // The 3D drawing noted so far, before the screen is shown, written or changed.
    crate::platform_3d::flush();
    let done = set_screen_mode(mode, gfx, system);
    if let Err(why) = &done {
        log(&format!("platform: screen mode {mode:#x}: {why}"));
    }
    done.is_ok()
}

// ---- gfx::set_menu_screen_mode (0x404823) ----

/// `gfx::set_menu_screen_mode`: `gfx::set_screen_mode` with the menus' mode; false (and
/// logged) if it cannot be.
pub fn set_menu_mode_now(
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> bool {
    // The 3D drawing noted so far, before the screen is shown, written or changed.
    crate::platform_3d::flush();
    let mode = gfx.display.mode.menu_mode.get();
    let done = set_screen_mode(mode, gfx, system);
    if let Err(why) = &done {
        log(&format!("platform: screen mode {mode:#x}: {why}"));
    }
    done.is_ok()
}

// ---- gfx::lock_surface (0x4058AD), gfx::unlock_surface (0x405949) ----

/// `gfx::lock_surface`: the surface locked for drawing (unless quitting or there is no
/// screen); the locked flag is 1 after it.
pub fn lock_surface(gfx: &crate::gfx::owner::Gfx, system: &crate::system::owner::System) -> u32 {
    // The 3D drawing noted so far, before the screen is shown, written or changed.
    crate::platform_3d::flush();
    let have = SCREEN.lock().is_ok_and(|screen| screen.is_some());
    if !system.session.flags.quitting.is_set() && have {
        gfx.locked.surface.set(1);
        // The surface the blitters draw on, and the clip rectangle.
        let s = &gfx.drawing.draw_surface;
        let (row_bytes, pitch): (u32, u32) = (
            system.session.screen.row_bytes.get(),
            gfx.drawing.draw_surface.present_pitch.get(),
        );
        s.width.set(system.session.screen.width.get());
        s.height.set(system.session.screen.height.get());
        s.bits_per_pixel.set(system.session.screen.bits.get());
        s.bytes_per_pixel.set(system.session.screen.bytes.get());
        s.row_bytes.set(row_bytes);
        s.row_dwords.set(system.session.screen.row_dwords.get());
        s.format.set(crate::gfx::pixel_format::SCREEN_FORMAT);
        s.pitch.set(pitch);
        s.pixels.set(gfx.drawing.draw_surface.present_pixels.get());
        crate::gfx::surface::on_address();
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

// ---- gfx::show_frame (0x4057D1) ----

/// The GPU's colour target as a texture of the window's renderer: made once for a set of
/// targets (`targets_made`, their number), in `GPU_VIEW`.
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
fn make_view(
    renderer: sdl::Renderer,
    (colour, targets_made): (*mut sdl3_sys::everything::SDL_GPUTexture, u64),
    width: usize,
    height: usize,
) {
    let mut view = GPU_VIEW.lock().unwrap();
    if view.0 != targets_made {
        // The old view is destroyed (dropped) when the new one is made.
        let Some(made) = renderer.create_gpu_view(colour, width, height) else {
            crate::fatal::raise::end(
                "platform::view_of",
                crate::errors::BadInput("SDL makes no texture of the GPU's picture"),
            );
        };
        *view = (targets_made, Some(made));
    }
}

/// The number of the targets' set the view was made for (none yet: 0), and the view.
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
static GPU_VIEW: Mutex<(u64, Option<sdl::Texture>)> = Mutex::new((0, None));

fn show_frame(o: &crate::owners::Owners<'static>, screen: &Screen) -> Result<(), &'static str> {
    let race = o.race;
    // The frame as XRGB8888: itself at 32 bits, else its 16-bit pixels widened.
    let bytes = (screen.bits / 8) as usize;
    let converted;
    // SAFETY: the frame is drawn on this thread, which is here to show it.
    let pixels = unsafe { screen.pixels.slice() };
    let frame: &[u8] = if bytes == 4 {
        pixels
    } else {
        converted = pixels
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
        if let Some(target) = crate::platform_3d::screen(crate::workers::game_scratch())
            && crate::render::show_on_panel(
                &race.course,
                &target,
                frame,
                screen.width,
                screen.height,
            )
        {
            return Ok(());
        }
        crate::handheld::present(frame, screen.width, screen.height)
    }
    #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
    {
        let renderer = screen.renderer;
        // A race on the GPU: its picture, the canvas's written pixels over it.
        let gpu_view = crate::platform_3d::screen(crate::workers::game_scratch())
            .and_then(|target| crate::render::finish_frame(&race.course, &target))
            .map(|colour| make_view(renderer, colour, screen.width, screen.height))
            .is_some();
        if !gpu_view && !screen.texture.upload(frame, screen.width * 4) {
            return Err("SDL cannot take the frame into its texture");
        }
        renderer.clear_black();
        if gpu_view {
            let view = GPU_VIEW.lock().unwrap();
            // `make_view` has just made it.
            if let Some(view) = view.1.as_ref() {
                renderer.draw_whole(view);
            }
        } else {
            renderer.draw_whole(&screen.texture);
        }
        // Not the original's: the debug UI over the frame.
        crate::debug::imgui_sdl::draw(o, renderer, screen.window);
        renderer.present();
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
    let system = crate::system::owner::system();
    (
        system.session.screen.width.get(),
        system.session.screen.height.get(),
    )
}

/// The screen's pixels (its width times its height).
#[cfg(debug_ui)]
pub fn screen_pixels() -> u64 {
    let system = crate::system::owner::system();
    u64::from(system.session.screen.width.get()) * u64::from(system.session.screen.height.get())
}

/// `gfx::show_frame`: the frame drawn shown (at most some sixty a second); false while
/// quitting (nothing shown).
pub fn show_frame_now() -> bool {
    let o = crate::owners::boundary();
    let drawing = &o.gfx.drawing;
    // The 3D drawing noted so far, before the screen is shown, written or changed.
    crate::platform_3d::flush();
    {
        if o.system.session.flags.quitting.is_set() {
            return false;
        }
        #[cfg(feature = "remote")]
        if crate::remote::shot_wanted()
            && let Some(target) = crate::platform_3d::screen(crate::workers::game_scratch())
        {
            crate::render::bring_back(&target);
        }
        if let Ok(screen) = SCREEN.lock()
            && let Some(screen) = screen.as_ref()
        {
            // Before it is shown: on the GPU's path showing clears the canvas.
            #[cfg(feature = "remote")]
            let shot = crate::remote::shot_wanted().then(|| screen.pixels.to_vec());
            let showing = std::time::Instant::now();
            let shown_ok = show_frame(&o, screen);
            crate::platform_timing::SHOW_MICROS.fetch_add(
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
            crate::platform_timing::note_frame_shown();
            let shown = SHOWN.fetch_add(1, Ordering::SeqCst) + 1;
            let wanted = FRAMES_TO_SHOW.load(Ordering::SeqCst);
            if wanted != 0 && shown == wanted {
                if let Some(target) = crate::platform_3d::screen(crate::workers::game_scratch()) {
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
        crate::platform_timing::start_sampler();
        crate::platform_timing::report_timing(o.system);
        // A flip waits for the screen: some sixty frames a second at most. The next frame's
        // gap is counted from the end of this wait (counted from before it, every other
        // frame skipped the wait: about 95 a second with 5 ms of work a frame).
        let now = sdl::ticks() as usize;
        let since = now.wrapping_sub(LAST_SHOWN.load(Ordering::SeqCst));
        if since < 16 {
            sdl::delay((16 - since) as u32);
        }
        LAST_SHOWN.store(sdl::ticks() as usize, Ordering::SeqCst);
    }
    true
}

// ---- system::pump_messages (0x410535), screens::run_frame (0x405A71) ----

/// `system::pump_messages`: the window's events (SDL's).
pub fn pump_messages_now() {
    crate::platform_events::pump();
}

/// `screens::run_frame` for Rust callers with a frame of their own (the screen loops): the messages, then the frame unless quitting (None then).
pub fn run_frame_now<T>(
    system: &crate::system::owner::System,
    frame: impl FnOnce() -> T,
) -> Option<T> {
    crate::platform_events::pump();
    if system.session.flags.quitting.is_set() {
        None
    } else {
        Some(frame())
    }
}

/// A file of ours next to the log (the current folder can change: the installer).
#[cfg_attr(all(target_os = "linux", target_arch = "aarch64"), allow(dead_code))] // used by the debug UI, and on Windows
pub fn beside_log(name: &str) -> std::path::PathBuf {
    match crate::framework::LOG_PATH.get().and_then(|p| p.parent()) {
        Some(folder) => folder.join(name),
        None => name.into(),
    }
}
