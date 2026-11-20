//! How a frame ends where the game has a window of its own (every system but the handheld): the
//! OpenGL backend shows its frames on its own window, the GPU API's colour target goes to the
//! window's swap chain, and the video's frames come back from the card one frame behind.
//! `frame_end_panel.rs` has the same names for the handheld, whose panel the GL ES backend draws
//! straight to.

use super::*;

/// Draws the commands on the OpenGL backend that runs on its own window, if one runs and the
/// frame is 32 bits; the commands come back when it did not take them.
pub(super) fn draw_on_gl(target: &Target, commands: CommandList) -> Option<CommandList> {
    if target.bits == 32
        && let Some(gl) = gles::GPU.lock().unwrap().as_mut()
    {
        if !GPU_HOLDS_FRAME.swap(true, Relaxed) {
            target.note_all_rows_written();
        }
        GPU_DREW_THIS_FRAME.store(true, Relaxed);
        Backend::draw(gl, target, commands);
        return None;
    }
    Some(commands)
}

/// `bring_back`, when the GPU holds the frame.
pub(super) fn bring_back_now(target: &Target) {
    if GPU_HOLDS_FRAME.load(Relaxed)
        && let Some(gl) = gles::GPU.lock().unwrap().as_mut()
    {
        // SAFETY: as below: the game's thread waits for this call and no other slice of the
        // canvas lives.
        gl.bring_back(target, unsafe { target.bytes() });
        target.note_all_rows_written();
        return;
    }
    if GPU_HOLDS_FRAME.load(Relaxed)
        && let Some(gpu) = gpu::GPU.lock().unwrap().as_mut()
    {
        // SAFETY: `Target::bytes`' promise: on the game's thread, which waits for this call, so
        // nothing else reaches the canvas; no other slice of it lives (the frame's showing takes
        // its own after this returns, `platform::with_frame`); the screen keeps its pixels.
        let canvas = unsafe { target.bytes() };
        let began = std::time::Instant::now();
        let rows = canvas_rows::written_rows(target, canvas);
        if let Err(why) = gpu.bring_back(Some(rows), canvas) {
            crate::program_log::log(&format!("render: the frame did not come back: {why}"));
        }
        READ_BACKS.fetch_add(1, Relaxed);
        crate::platform::timing::note_span(
            &crate::platform::timing::READ_BACK_MICROS,
            &crate::platform::timing::READ_BACK_MAX,
            began.elapsed().as_micros() as u64,
        );
        // The picture written in place, unnoted.
        target.note_all_rows_written();
    }
}

/// The frame's picture stretched to `size` (width, height) pixels, each the frame's pixel at its
/// place scaled down in whole numbers, as 0xAARRGGBB dwords, rows top first: what a texture
/// made from the screen reads, without bringing the whole frame back into the canvas. `None`
/// when the GPU does not hold the frame or its backend cannot (the caller then brings the frame
/// back and reads the canvas).
pub fn stretched_frame(target: &Target, size: (u32, u32)) -> Option<Vec<u32>> {
    if !GPU_HOLDS_FRAME.load(Relaxed) {
        return None;
    }
    if let Some(gl) = gles::GPU.lock().unwrap().as_mut() {
        let began = std::time::Instant::now();
        // SAFETY: as below.
        let canvas = unsafe { target.bytes() };
        let picture = gl.stretched(target, canvas, (size.0 as usize, size.1 as usize));
        READ_BACKS.fetch_add(1, Relaxed);
        READ_BACK_MICROS.fetch_add(began.elapsed().as_micros() as u64, Relaxed);
        return picture;
    }
    let mut gpu = gpu::GPU.lock().unwrap();
    let gpu = gpu.as_mut()?;
    let began = std::time::Instant::now();
    // SAFETY: `Target::bytes`' promise: on the game's thread, which waits for this call, so
    // nothing else reaches the canvas; no other slice of it lives.
    let canvas = unsafe { target.bytes() };
    // The canvas's size is `target`'s, as the card's frame (`read_stretched` checks).
    let rows = canvas_rows::written_rows(target, canvas);
    let screen = (target.width as u32, target.height as u32);
    let picture = gpu.read_stretched(Some(rows), canvas, screen, size);
    READ_BACKS.fetch_add(1, Relaxed);
    READ_BACK_MICROS.fetch_add(began.elapsed().as_micros() as u64, Relaxed);
    match picture {
        Ok(Some(bytes)) => Some(
            bytes
                .as_chunks::<4>()
                .0
                .iter()
                .map(|p| u32::from_le_bytes(*p))
                .collect(),
        ),
        Ok(None) => None,
        Err(why) => {
            crate::program_log::log(&format!(
                "render: the stretched frame did not come back: {why}"
            ));
            None
        }
    }
}

/// Collects the frame queued by the last call into `spare` (the buffer is kept in `previous`) and
/// queues the frame now drawn. Only the Vulkan backend queues; any other leaves `started` false.
pub fn video_read_next(target: &Target, spare: Vec<u8>, nv12: bool) -> VideoRead {
    let mut read = VideoRead {
        previous: None,
        started: false,
    };
    let opengl = gles::GPU.lock().unwrap().is_some();
    let mut gpu = gpu::GPU.lock().unwrap();
    let Some(gpu) = gpu.as_mut() else {
        return read;
    };
    // Whatever was queued comes first, so that the frames stay in order.
    let mut buffer = spare;
    match gpu.collect_video_read(&mut buffer) {
        Ok(true) => read.previous = Some(buffer),
        Ok(false) => {}
        Err(why) => crate::program_log::log(&format!(
            "render: the video's last frame did not come back: {why}"
        )),
    }
    if !GPU_HOLDS_FRAME.load(Relaxed) || opengl {
        return read;
    }
    // SAFETY: `Target::bytes`' promise: on the game's thread, which waits for this call, so
    // nothing else reaches the canvas; no other slice of it lives (the frame's showing takes
    // its own after this returns, `platform::with_frame`); the screen keeps its pixels.
    let canvas = unsafe { target.bytes() };
    let rows = canvas_rows::written_rows(target, canvas);
    match gpu.start_video_read(Some(rows), canvas, nv12) {
        Ok(()) => read.started = true,
        Err(why) => crate::program_log::log(&format!(
            "render: the video's frame did not start back: {why}"
        )),
    }
    read
}

/// Whether the video's frames can come from the card as NV12 (`video_read_next`'s `nv12`): the
/// Vulkan renderer is the one in use and the panel is not the handheld's.
pub fn video_nv12_possible() -> bool {
    gles::GPU.lock().unwrap().is_none() && gpu::GPU.lock().unwrap().is_some()
}

/// The last frame `video_read_next` queued, collected into `spare`; `None` when none is queued.
/// The GPU is only tried, never waited for, so that a program ending in a fault cannot hang here.
pub fn video_read_last(spare: Vec<u8>) -> Option<Vec<u8>> {
    let mut gpu = gpu::GPU.try_lock().ok()?;
    let mut buffer = spare;
    matches!(gpu.as_mut()?.collect_video_read(&mut buffer), Ok(true)).then_some(buffer)
}

/// The frame's end, before it is shown. The GPU's colour target when that is what to show, the
/// canvas's written pixels over it, with the number of its set of targets and its size
/// (`Gpu::colour_texture`).
pub fn finish_frame(
    racing: bool,
    target: &Target,
) -> Option<(*mut sdl3_sys::everything::SDL_GPUTexture, u64, (u32, u32))> {
    if !GPU_HOLDS_FRAME.load(Relaxed) {
        return None;
    }
    // The OpenGL backend's window shows its own frames (`show_on_gl_window`); a frame that
    // comes here is read back into the canvas.
    let stays = frame_stays_on_gpu(racing);
    if !stays || gles::GPU.lock().unwrap().is_some() {
        give_frame_back(target);
        if stays {
            dev::seethru::frame_end();
        }
        return None;
    }
    let mut gpu = gpu::GPU.lock().unwrap();
    let gpu = gpu.as_mut()?;
    // SAFETY: `Target::bytes`' promise: on the game's thread, which waits for this call, so
    // nothing else reaches the canvas; no other slice of it lives (the frame's showing takes
    // its own after this returns, `platform::with_frame`); the screen keeps its pixels.
    let canvas = unsafe { target.bytes() };
    let rows = canvas_rows::written_rows(target, canvas);
    if let Err(why) = gpu.finish_frame(Some(rows), canvas) {
        crate::program_log::log(&format!("render: the frame did not finish: {why}"));
    }
    if let Err(why) = gpu.finish_high() {
        crate::program_log::log(&format!(
            "render: the menu's larger 3D did not finish: {why}"
        ));
    }
    Some(gpu.colour_texture())
}

/// The frame on the OpenGL backend's own window, which shows it without the frame's coming back
/// through the canvas: the target during a race (and in the menus that keep their background on
/// the GPU), else the canvas's picture, which `with_frame` hands over. False: the backend does
/// not show a window (it is not running), and the caller shows the frame.
pub fn show_on_gl_window(
    racing: bool,
    target: &Target,
    with_frame: impl FnOnce(&mut dyn FnMut(&[u8])),
    width: usize,
    height: usize,
    menu: bool,
) -> bool {
    if gles::window().is_none() || gles::GPU.lock().unwrap().is_none() {
        return false;
    }
    if GPU_HOLDS_FRAME.load(Relaxed) && !frame_stays_on_gpu(racing) {
        give_frame_back(target);
    }
    let mut gl = gles::GPU.lock().unwrap();
    let Some(gl) = gl.as_mut() else {
        return false;
    };
    if menu
        && let Some(window) = gles::window().and_then(crate::platform::ffi::sdl::Window::pixel_size)
    {
        // Made and sent only when the background or the window's size has changed.
        if let Some(image) = wash::image_for(window) {
            gl.set_wash(&image);
        }
    }
    if GPU_HOLDS_FRAME.load(Relaxed) {
        // SAFETY: `Target::bytes`' promise: on the game's thread, which waits for this call, so
        // nothing else reaches the canvas; no other slice of it lives (the frame's showing takes
        // its own after this returns, `platform::with_frame`); the screen keeps its pixels.
        gl.finish_and_show(target, unsafe { target.bytes() }, menu);
    } else {
        with_frame(&mut |frame| gl.show_picture(frame, width, height, menu));
    }
    true
}

/// A GPU backend that had started is given up (the screen's targets could not be made): its
/// texture changes are no longer kept.
pub(crate) fn gpu_stopped() {
    GPU_STARTED.store(false, Relaxed);
    crate::gfx::draw2d::quads::gpu_changed();
    background::clear();
    *CHANGED_TEXTURES.lock().unwrap() = None;
}

/// Whether the desktop's OpenGL backend is the one running (the handheld's GL ES is its GPU
/// renderer, `gpu_running`).
pub fn gl_running() -> bool {
    gles::GPU.lock().unwrap().is_some()
}

/// Whether the GPU's frame may be given up: a frame the GPU holds is brought back into the
/// canvas, and the rasteriser draws from here. For the renderer's swap.
pub(crate) fn release_frame(target: Option<&Target>) {
    if let Some(target) = target {
        give_frame_back(target);
    }
    GPU_HOLDS_FRAME.store(false, Relaxed);
    GPU_DREW_THIS_FRAME.store(false, Relaxed);
}
