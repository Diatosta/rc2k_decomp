//! How a frame ends on the handheld, whose panel the GL ES backend draws straight to (no window
//! of the game's own, no second backend, no pipelined read-back). `frame_end.rs` has the
//! same names for the other systems.

use super::*;
use backend::Running;

/// The commands come back: only the GPU backend draws on the panel (`draw`'s first backend).
pub(super) fn draw_on_gl(_target: &Target, commands: CommandList) -> Option<CommandList> {
    Some(commands)
}

/// `bring_back`, when the GPU holds the frame.
pub(super) fn bring_back_now(target: &Target) {
    if GPU_HOLDS_FRAME.load(Relaxed)
        && let Some(gpu) = backend::lock().as_mut().and_then(Running::gl)
    {
        // SAFETY: `Target::bytes`' promise: on the game's thread, which waits for this call, so
        // nothing else reaches the canvas; no other slice of it lives (the frame's showing takes
        // its own after this returns, `platform::with_frame`); the screen keeps its pixels.
        let canvas = unsafe { target.bytes() };
        let began = std::time::Instant::now();
        gpu.bring_back(target, canvas);
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
    let _ = (target, size);
    None
}

/// The handheld's panel has no pipelined read-back: every frame is brought back at once.
pub fn video_read_next(_target: &Target, _spare: Vec<u8>, _nv12: bool) -> VideoRead {
    VideoRead {
        previous: None,
        started: false,
    }
}

/// The handheld's panel queues no frame: its frames stay 32 bits a pixel.
pub fn video_nv12_possible() -> bool {
    false
}

/// The handheld's panel queues no frame (`video_read_next`): nothing is left to collect.
pub fn video_read_last(_spare: Vec<u8>) -> Option<Vec<u8>> {
    None
}

/// The frame on the panel, by the GPU when it is running: the target during a race, else
/// the canvas's picture, which `with_frame` hands over (taken only after the GPU's writes to the
/// canvas here, so that no slice of it is held across them). False: it is not running.
pub fn show_on_panel(
    racing: bool,
    target: &Target,
    with_frame: impl FnOnce(&mut dyn FnMut(&[u8])),
    width: usize,
    height: usize,
) -> bool {
    gpu::start_if_wanted();
    if GPU_HOLDS_FRAME.load(Relaxed) && !frame_stays_on_gpu(racing) {
        give_frame_back(target);
    }
    let mut running = backend::lock();
    let Some(gpu) = running.as_mut().and_then(Running::gl) else {
        return false;
    };
    if GPU_HOLDS_FRAME.load(Relaxed) {
        // SAFETY: `Target::bytes`' promise: on the game's thread, which waits for this call, so
        // nothing else reaches the canvas; no other slice of it lives (the frame's showing takes
        // its own after this returns, `platform::with_frame`); the screen keeps its pixels.
        // The handheld's panel is the frame's own size: no space beside it to fill.
        gpu.finish_and_show(target, unsafe { target.bytes() }, false);
    } else {
        with_frame(&mut |frame| gpu.show_picture(frame, width, height, false));
    }
    true
}

/// Never: the handheld's GL ES is its GPU renderer (`gpu_running`).
pub(super) fn gl_running() -> bool {
    false
}
