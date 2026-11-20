//! The GL's thread and the game's side of it: starting, stopping, the jobs' channel, and the
//! backend the renderer draws through.

use super::*;
use crate::render::backend::{self, Running};

/// What the GL's thread hands back when it ends: the hidden window its context was on (the
/// game's thread destroys it, as it made it); the panel's context lives on with the program.
#[cfg(not(handheld))]
pub(super) type Closed = crate::platform::ffi::gl::GlWindow;

#[cfg(handheld)]
pub(super) type Closed = ();

/// Why a start failed, and the window its context was on, to be destroyed by the game's thread.
pub(super) type Failed = (String, Option<Closed>);

/// The GL's two threads.
#[cfg_attr(handheld, allow(dead_code))] // only the desktop's swap joins them
pub(super) struct Threads {
    batching: std::thread::JoinHandle<()>,
    gl: std::thread::JoinHandle<Option<Closed>>,
}

/// The game's side of the GL's thread, and what it knows the GPU holds.
pub struct Front {
    jobs: SyncSender<Job<Part>>,
    /// The batching thread and the GL's, joined by `stop`.
    #[cfg_attr(handheld, allow(dead_code))]
    // only the desktop's swap joins them
    threads: Threads,
    /// The textures up to date on the GPU, with their sizes.
    known: crate::render::TextureMap<(usize, usize)>,
    size: (i32, i32),
    /// What the GL's background texture holds, at the screen's size.
    background_held: Option<background::Held>,
    /// The version of the wash (`render::wash`) the GL holds.
    wash_held: u64,
}

/// Jobs waiting for the batching thread at most: a frame's part, its end, a little more; then
/// the game waits. Past them, one being batched and `BATCHED_WAITING` batched wait for the GL:
/// with fewer (one waiting in all, handed over directly) the game waited for the GL three
/// times as long and the handheld drew about 8 % fewer frames.
pub(super) const JOBS_WAITING: usize = 2;

/// Parts batched and waiting for the GL's thread at most.
pub(super) const BATCHED_WAITING: usize = 1;

impl Front {
    fn send(&self, job: Job<Part>) {
        let start = std::time::Instant::now();
        let sent = self.jobs.send(job);
        crate::render::GL_WAIT_MICROS.fetch_add(
            start.elapsed().as_micros() as u64,
            std::sync::atomic::Ordering::Relaxed,
        );
        if sent.is_err() {
            crate::fatal::raise::end(
                "render::gles",
                crate::errors::BadInput("the GL's thread has ended"),
            );
        }
    }

    /// Hands over the commands and the canvas's written rows (as `target` noted them), which it
    /// marks again.
    fn draw(&mut self, target: &Target, canvas: &mut [u8], commands: CommandList) {
        // A frame with only 2D in it (a menu's background and quads) has no `Backend::draw` of
        // its own: the size is the target's here.
        let size = (target.width as i32, target.height as i32);
        if size != self.size {
            // The GL's background texture is remade at the new size.
            self.background_held = None;
            self.size = size;
        }
        crate::render::FANS.fetch_add(commands.len() as u64, std::sync::atomic::Ordering::Relaxed);
        crate::render::FLUSHES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        crate::render::dev::seethru::note_list(&commands, self.size.0 as u32, self.size.1 as u32);
        let changes = crate::render::take_changed_textures();
        for pixels in changes.written.iter().chain(&changes.gone) {
            self.known.remove(pixels);
        }
        if !changes.gone.is_empty() {
            self.send(Job::Forget(changes.gone.into_iter().collect()));
        }
        let (width, _) = self.size;
        let mut uploads = Vec::new();
        // Each texture the fans read once, in the order they first read them.
        for note in commands.textures() {
            if self.known.get(&note.pixels) != Some(&(note.width, note.height)) {
                uploads.push((*note, crate::render::texture_as_bgra(note)));
                self.known.insert(note.pixels, (note.width, note.height));
            }
        }
        let quads = quads::take().fitted_to((width as u32, self.size.1 as u32));
        let background = self.take_background();
        let stride = width as usize * 4;
        let rows = canvas_rows::written_rows(target, canvas);
        let written = canvas_rows::written_runs(&rows)
            .into_iter()
            .map(|(first, last)| {
                let rows = &mut canvas[first * stride..(last + 1) * stride];
                let bytes = rows.to_vec();
                mark_unwritten(rows);
                Written { first, last, bytes }
            })
            .collect();
        self.send(Job::Draw(Part {
            commands,
            uploads,
            written,
            quads,
            background,
            size: self.size,
            scale: crate::render::flush_scale(),
            high: crate::render::menu_high_percent().unwrap_or(0),
        }));
    }

    /// The noted background (`background`), if there is one, as the part brings it: its picture
    /// when the GL's texture does not hold it already.
    fn take_background(&mut self) -> Option<Background> {
        let pending = background::take()?;
        let bytes = self.size.0 as usize * self.size.1 as usize * 4;
        let mut picture = None;
        let sent = background::bring_up(
            pending,
            &mut self.background_held,
            bytes,
            crate::d3d::device::handles().pictures,
            |rows| {
                picture = Some(rows.to_vec());
                Ok(())
            },
        );
        if let Err(why) = sent {
            crate::fatal::raise::end(
                "render::gles",
                crate::errors::BadInput(Box::leak(why.into_boxed_str())),
            );
        }
        Some(picture.map_or(Background::Held, Background::Upload))
    }

    /// The frame's end: what the canvas has written over the target, and the target shown.
    pub fn finish_and_show(&mut self, target: &Target, canvas: &mut [u8], menu: bool) {
        self.draw(target, canvas, CommandList::new());
        crate::render::dev::seethru::frame_end();
        self.send(Job::ShowTarget { menu });
    }

    /// The menu's wash sent to the GL (which shows it beside the frames of the menus), unless the
    /// GL holds it already.
    pub fn set_wash(&mut self, image: &crate::render::wash::Image) {
        if self.wash_held == image.version {
            return;
        }
        self.wash_held = image.version;
        self.send(Job::Wash {
            pixels: image.wash.pixels.clone(),
            width: image.wash.width as i32,
            height: image.wash.height as i32,
        });
    }

    /// A picture of the game's bytes on the panel.
    pub fn show_picture(&mut self, frame: &[u8], width: usize, height: usize, menu: bool) {
        self.send(Job::ShowPicture {
            frame: frame.to_vec(),
            width: width as i32,
            height: height as i32,
            menu,
        });
    }

    /// The target's picture, with the canvas's written pixels, into the canvas (waits for the
    /// GL: the frame's whole work so far).
    pub fn bring_back(&mut self, target: &Target, canvas: &mut [u8]) {
        self.draw(target, canvas, CommandList::new());
        let (answer, picture) = channel();
        self.send(Job::ReadTarget(answer));
        let Ok(picture) = picture.recv() else {
            crate::fatal::raise::end(
                "render::gles",
                crate::errors::BadInput("the GL's thread has ended"),
            );
        };
        // Red first and bottom row first, as the GL gives them: the game's blue first, top first.
        let stride = self.size.0 as usize * 4;
        for (to, from) in canvas
            .chunks_exact_mut(stride)
            .zip(picture.rchunks_exact(stride))
        {
            for (t, f) in to
                .as_chunks_mut::<4>()
                .0
                .iter_mut()
                .zip(from.as_chunks::<4>().0)
            {
                *t = [f[2], f[1], f[0], 0];
            }
        }
    }
}

impl Front {
    /// The target's picture stretched to `size` (width, height) pixels as 0xAARRGGBB dwords (alpha
    /// 0), each the frame's pixel at its place scaled down in whole numbers, rows top first; the
    /// canvas is not written (waits for the GL as `bring_back` does). `None` when the frame is
    /// not `screen` pixels. The handheld's panel keeps bringing the frame back (not tried there).
    #[cfg(not(handheld))]
    pub fn stretched(
        &mut self,
        target: &Target,
        canvas: &mut [u8],
        size: (usize, usize),
    ) -> Option<Vec<u32>> {
        let (width, height) = (self.size.0 as usize, self.size.1 as usize);
        if (target.width, target.height) != (width, height) || size.0 == 0 || size.1 == 0 {
            return None;
        }
        self.draw(target, canvas, CommandList::new());
        let (answer, picture) = channel();
        self.send(Job::ReadTarget(answer));
        let Ok(picture) = picture.recv() else {
            crate::fatal::raise::end(
                "render::gles",
                crate::errors::BadInput("the GL's thread has ended"),
            );
        };
        let mut out = Vec::with_capacity(size.0 * size.1);
        for y in 0..size.1 {
            // The GL's rows come bottom first.
            let row = height - 1 - y * height / size.1;
            let row = &picture[row * width * 4..(row + 1) * width * 4];
            for x in 0..size.0 {
                let at = x * width / size.0 * 4;
                out.push(
                    u32::from(row[at]) << 16 | u32::from(row[at + 1]) << 8 | u32::from(row[at + 2]),
                );
            }
        }
        Some(out)
    }
}

/// The name `RC2K_FAIL_RENDERER` gives to this backend's start.
#[cfg(handheld)]
pub(super) const FAILURE_STEP: &str = "gles";

#[cfg(not(handheld))]
pub(super) const FAILURE_STEP: &str = "gl";

/// Starts the GL's thread if it is wanted (the handheld: its panel's context).
#[cfg(handheld)]
pub fn start_if_wanted() {
    start_with(|| Gles::open_panel(PANEL.0 as u16, PANEL.1 as u16).map_err(|why| (why, None)));
}

/// Starts the GL's thread on a hidden window made for OpenGL at the screen's `size`, if it is
/// wanted; whether it runs. The window is made here (on the game's thread) and the context on the GL's.
#[cfg(not(handheld))]
pub fn start_on_window(
    game_window: crate::platform::ffi::sdl::Window,
    size: (usize, usize),
) -> bool {
    if backend::gl_runs() {
        return true;
    }
    if TRIED.load(std::sync::atomic::Ordering::Relaxed) || !wanted() {
        return false;
    }
    // macOS makes an OpenGL context only on the main thread: SDL hands the GL thread's request
    // to it while the game's thread (the main one) waits for the GL thread's answer, and both
    // wait for ever. A Mac without Metal goes on to SDL's renderer and the software rasteriser.
    if cfg!(apple) {
        TRIED.store(true, std::sync::atomic::Ordering::Relaxed);
        log("render: the OpenGL backend is not used on macOS");
        return false;
    }
    // Android has one window, which the window's renderer (or the GPU API) uses: the OpenGL
    // backend's second window cannot be made. A GL ES context on the game's own window, its
    // frames shown through the window's renderer, is not built yet (the notes of the droidperf
    // lane in the wiki's Android page).
    if cfg!(target_os = "android") {
        TRIED.store(true, std::sync::atomic::Ordering::Relaxed);
        log("render: the OpenGL backend is not used on Android");
        return false;
    }
    // The browser has one canvas for the game's window: the GL draws on a second one over it.
    #[cfg(web)]
    let made = {
        let _ = size;
        Ok::<_, String>(crate::platform::ffi::gl::GlWindow::new(game_window))
    };
    #[cfg(not(web))]
    let made = {
        let _ = game_window;
        crate::platform::ffi::gl::GlWindow::new(size)
    };
    let window = match made {
        Ok(window) => window,
        Err(why) => {
            TRIED.store(true, std::sync::atomic::Ordering::Relaxed);
            log(&format!(
                "render: the OpenGL backend does not start ({why})"
            ));
            return false;
        }
    };
    let handle = window.handle();
    start_with(move || Gles::open_window(window).map_err(|(why, window)| (why, Some(window))));
    let runs = backend::gl_runs();
    if runs {
        *window_lock() = Some(handle);
    }
    runs
}

/// The window the OpenGL backend shows its frames on while it runs: the game's window in all but
/// name (`platform` shows it in place of the one SDL's renderer has, and gives the keyboard to
/// it).
#[cfg(not(handheld))]
pub(super) static WINDOW: std::sync::Mutex<Option<crate::platform::ffi::sdl::Window>> =
    std::sync::Mutex::new(None);

/// `WINDOW` locked (a panic elsewhere does not stop the game's thread here).
#[cfg(not(handheld))]
fn window_lock() -> std::sync::MutexGuard<'static, Option<crate::platform::ffi::sdl::Window>> {
    WINDOW
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// The window the OpenGL backend shows its frames on, while it runs.
#[cfg(not(handheld))]
pub fn window() -> Option<crate::platform::ffi::sdl::Window> {
    *window_lock()
}

/// The OpenGL backend stopped, if it runs: the jobs' end is waited for, both threads are
/// joined, the context and its window are destroyed. The next `start_on_window` asks again.
#[cfg(not(handheld))]
pub fn stop() -> bool {
    let front = {
        let mut running = backend::lock();
        match running.take() {
            Some(Running::Gl(front)) => front,
            other => {
                *running = other;
                return false;
            }
        }
    };
    *window_lock() = None;
    crate::render::gpu_stopped();
    let Front { jobs, threads, .. } = front;
    // The batching thread ends with the game's side, the GL's with the batching thread.
    drop(jobs);
    let batching = threads.batching.join();
    let closed = threads.gl.join();
    if batching.is_err() || closed.is_err() {
        log("render: the OpenGL backend's threads ended in a panic");
    }
    if let Ok(Some(window)) = closed {
        window.destroy();
    }
    TRIED.store(false, std::sync::atomic::Ordering::Relaxed);
    true
}

/// Starts the GL's thread, which makes its context with `open`.
pub(super) fn start_with(open: impl FnOnce() -> Result<Gles, Failed> + Send + 'static) {
    let mut held = backend::lock();
    if held.is_some() || TRIED.swap(true, std::sync::atomic::Ordering::Relaxed) {
        return;
    }
    if !wanted() {
        log("render: the software rasteriser draws the 3D (--renderer or RC2K_RENDERER)");
        return;
    }
    let (jobs, to_batch) = sync_channel(JOBS_WAITING);
    let (batched, to_draw) = sync_channel(BATCHED_WAITING);
    let (answer, started) = channel();
    let thread = std::thread::Builder::new()
        .name("GL".into())
        .spawn(move || match Gpu::new(open) {
            Ok(gpu) => {
                let _ = answer.send(Ok(()));
                gpu.run(to_draw)
            }
            Err((why, window)) => {
                let _ = answer.send(Err(why));
                window
            }
        });
    let outcome = match thread {
        Ok(handle) => match started.recv() {
            Ok(Ok(())) => Ok(handle),
            Ok(Err(why)) => {
                // The thread has ended; its window is destroyed here.
                #[cfg(not(handheld))]
                if let Ok(Some(window)) = handle.join() {
                    window.destroy();
                }
                Err(why)
            }
            Err(_) => Err("the GL's thread ended".into()),
        },
        Err(why) => Err(why.to_string()),
    };
    // The GL's thread, without its batches' sender, ends when this does not start.
    let outcome = outcome.and_then(|gl| {
        let spawned = std::thread::Builder::new()
            .name("GL batches".into())
            .spawn(move || {
                crate::render::gl_batching::run(to_batch, batched, same_array, bench::enabled());
            });
        match spawned {
            Ok(batching) => Ok(Threads { batching, gl }),
            Err(why) => {
                // The closure, with the batches' sender, is gone, so the GL's thread ends; its
                // window is destroyed here as in the failed start above.
                #[cfg(not(handheld))]
                if let Ok(Some(window)) = gl.join() {
                    window.destroy();
                }
                #[cfg(handheld)]
                let _ = gl.join();
                Err(why.to_string())
            }
        }
    });
    match outcome {
        Ok(threads) => {
            log(if cfg!(handheld) {
                "render: the GPU backend on libmali's OpenGL ES, on its own thread"
            } else if cfg!(web) {
                "render: the OpenGL backend (WebGL 2), on its own thread"
            } else {
                "render: the OpenGL backend (3.3 core), on its own thread"
            });
            *held = Some(Running::Gl(Front {
                jobs,
                threads,
                known: Default::default(),
                size: (0, 0),
                background_held: None,
                wash_held: 0,
            }));
            crate::render::gpu_started();
        }
        Err(why) => log(&format!(
            "render: the OpenGL backend does not start ({why}): the next renderer in the order draws the 3D"
        )),
    }
}

/// It is asked for once: failing, the rasteriser goes on.
pub(super) static TRIED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

impl crate::render::Backend for Front {
    fn draw(&mut self, target: &Target, commands: CommandList) {
        target.with_canvas(|canvas| Front::draw(self, target, canvas, commands));
    }
}

impl Gpu {
    /// Draws the jobs until the batching thread's end, then closes the context (on this
    /// thread, which made it).
    pub(super) fn run(mut self, jobs: Receiver<Job<Batched>>) -> Option<Closed> {
        let mut sampler = crate::render::dev::gpu_time::Sampler::from_env();
        for job in jobs {
            let start = std::time::Instant::now();
            match job {
                Job::Forget(gone) => self.forget(&gone),
                Job::Draw(part) => {
                    if let Some(sampler) = &mut sampler {
                        sampler.part_begins();
                    }
                    self.draw(part);
                }
                Job::ShowTarget { menu } => {
                    let wait = sampler
                        .as_mut()
                        .is_some_and(crate::render::dev::gpu_time::Sampler::frame_ends);
                    let waited = self.show_target(wait, menu);
                    // The next frame starts with a fresh depth buffer: the game clears the depth
                    // only where it needs the clear, and what the last frame wrote would fail
                    // the next one's depth test in patches.
                    self.depth_is_new = true;
                    if let Some(small) = self.small.as_mut() {
                        small.depth_is_new = true;
                    }
                    if let Some(high) = self.high.as_mut() {
                        high.depth_is_new = true;
                    }
                    if let Some(sampler) = &mut sampler {
                        match waited {
                            Some(waited) => sampler.counted(waited),
                            None => sampler.skipped(),
                        }
                    }
                    crate::render::GL_SHOW_MICROS.fetch_add(
                        start.elapsed().as_micros() as u64,
                        std::sync::atomic::Ordering::Relaxed,
                    );
                }
                Job::ShowPicture {
                    frame,
                    width,
                    height,
                    menu,
                } => self.show_picture(&frame, width, height, menu),
                Job::Wash {
                    pixels,
                    width,
                    height,
                } => self.set_wash(&pixels, (width, height)),
                Job::ReadTarget(answer) => {
                    let _ = answer.send(self.read_target());
                }
            }
            crate::render::GL_BUSY_MICROS.fetch_add(
                start.elapsed().as_micros() as u64,
                std::sync::atomic::Ordering::Relaxed,
            );
        }
        #[cfg(not(handheld))]
        return Some(self.gl.close());
        #[cfg(handheld)]
        None
    }
}
