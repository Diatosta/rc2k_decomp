//! The window and renderer where the game has a window of its own (every system but the handheld):
//! the renderer's start in the order the choice names, the GPU's colour target as a texture, the
//! window mode, the window the frames are shown on, and the renderer's swap while the game runs.
//! `window_system_panel.rs` has the same names for the handheld, whose panel has none of it.

use super::*;

/// The number of the targets' set the view was made for (none yet: 0), and the view.
pub(crate) static GPU_VIEW: Mutex<(u64, Option<sdl::Texture>, (u32, u32))> =
    Mutex::new((0, None, (0, 0)));
/// The wash texture of the menu's background (its version, the texture), kept while the
/// background and the window's size stay the same.
pub(crate) static WASH_TEXTURE: Mutex<Option<(u64, sdl::Texture)>> = Mutex::new(None);
/// The GPU device the window's renderer runs on, while there is one (destroyed after the
/// renderer by `swap_renderer`).
pub(super) static WINDOW_DEVICE: Mutex<Option<crate::platform::ffi::sdl_gpu::Device>> =
    Mutex::new(None);
/// The GPU's colour target as a texture of the window's renderer: made once for a set of
/// targets (`targets_made`, their number), in `GPU_VIEW`.
pub(crate) fn make_view(
    renderer: sdl::Renderer,
    (colour, targets_made, (width, height)): (
        crate::platform::ffi::sdl_gpu::Texture,
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
/// The window's renderer and the 3D's backend, started in the order the choice names (the GPU
/// API's renderer on its device, OpenGL beside SDL's own renderer, SDL's own renderer with the
/// software rasteriser); `None` when SDL makes no renderer at all. Logs what runs. `size` is the
/// screen's: the OpenGL backend's window is made at it.
pub(super) fn make_window_renderer(
    window: sdl::Window,
    size: (usize, usize),
) -> Option<sdl::Renderer> {
    let gpu_renderer = start_gpu_renderer(window);
    // Next in the order: OpenGL, which shows its frames on a window of its own in place of
    // the one SDL's renderer has (`render::gles`).
    if gpu_renderer.is_none() {
        crate::render::gles::start_on_window(window, size);
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
pub(super) fn set_window_mode(window: sdl::Window, size: (usize, usize)) {
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
/// The window the frames are shown on, set for the backend that runs: the OpenGL backend's own
/// window takes the game's place (same title, icon, size and spot, the keyboard), SDL's renderer
/// keeping the one it was made on, hidden; with no OpenGL window the game's own is shown.
pub(super) fn place_shown_window(
    window: sdl::Window,
    width: usize,
    height: usize,
    size_changed: bool,
) {
    let Some(gl) = crate::render::gles::window() else {
        window.show_raised();
        return;
    };
    // The browser's one window is the GL's too: it is shown as it is.
    if gl.is_same(window) {
        window.show_raised();
        return;
    }
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
/// The GPU backend's targets at the screen's size; a backend that cannot make them is given up
/// (the software rasteriser draws the 3D from here).
pub(super) fn size_gpu_targets(width: u32, height: u32) {
    crate::render::gpu::size_targets(width, height);
}
/// The window's renderer on a GPU device, with the GPU backend started on the same device.
/// `None` (each failure logged) when the GPU is not wanted or a step fails; the device is
/// then given up and the caller makes SDL's own renderer. A backend that does not start
/// leaves the window's GPU renderer in place, drawing the rasteriser's frames.
pub(super) fn start_gpu_renderer(window: sdl::Window) -> Option<sdl::Renderer> {
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
        sdl::create_gpu_renderer(device, window)
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
    crate::d3d::device::flush();
    let screen_now = o.system.session.screen.get();
    let size = (screen_now.width.get(), screen_now.height.get());
    let target = crate::d3d::device::screen(screen_now);
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
        let target = crate::d3d::device::screen(screen_now);
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
pub(super) fn rebuild_renderer(
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
/// The window put into the kept window mode now (the graphics page's row, Alt+Enter).
pub fn apply_window_mode() {
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
/// The window that has the keyboard and is closed by the player: the OpenGL backend's while it
/// shows the frames, else the game's.
pub(crate) fn input_window(window: sdl::Window) -> sdl::Window {
    if let Some(gl) = crate::render::gles::window() {
        return gl;
    }
    window
}

/// Where the window's frames come from in the log's words: whether the 3D runs on a GPU backend.
pub(super) fn renderer_note() -> &'static str {
    if crate::render::gpu_running() {
        ", the 3D on a GPU backend"
    } else {
        ", the 3D on the software rasteriser"
    }
}

/// The window's renderer, made as `make_window_renderer` has it.
pub(super) fn open_renderer(window: sdl::Window, size: (usize, usize)) -> Option<sdl::Renderer> {
    make_window_renderer(window, size)
}

/// Nothing to do before the video starts: the window is a real one.
pub(super) fn before_video() {}

/// The window shown and put into its mode after the screen's size is set: the shown window's
/// place (`place_shown_window`), then the kept window mode when the window is first shown, or the
/// display's mode when a new resolution comes while in full screen.
pub(super) fn settle_window(
    window: sdl::Window,
    window_size: (usize, usize),
    window_before: Option<(usize, usize)>,
    window_changed: bool,
) {
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

/// Whether the OpenGL backend's own window shows the frames (it never waits for the refresh).
pub(super) fn shows_itself() -> bool {
    crate::render::gles::window().is_some()
}

/// The menus' mode: always the race's kind of pixel at 800 x 600 (what the options keep as the
/// menus' mode no longer changes it).
pub(super) fn menu_mode(race_mode: u32, menu_mode: u32) -> u32 {
    let _ = menu_mode;
    rc2k_core::gfx::screen_modes::menu_mode_for(race_mode)
}

/// Whether the menus' frame has the race's window size (the window stays one size between a menu
/// and a race).
pub(super) const MENU_WINDOW_IS_RACE_SIZED: bool = true;
