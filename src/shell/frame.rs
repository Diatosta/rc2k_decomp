//! The end of a frame: the game's drawing handed to the renderer, the video's and the screenshots'
//! copies, the touch controls and the debug window over it, and the display's pace. The window,
//! the renderers and the frame's pixels are `platform`'s; what is shown and when is decided here,
//! where the owners are known.
//!
//! - `show_frame_now`: the frame drawn shown; false while quitting.
//! - `pump_messages_now`, `run_frame_now`: the window's events, then the frame's function.
//! - `frame_counters`, `screen_size`, `screen_pixels`: the debug window's figures.

use super::{events, timing_report};
#[cfg(not(handheld))]
use crate::platform::ffi::sdl;
use crate::platform::{FRAMES_TO_SHOW, SCREEN, SHOWN, Screen, pacing, timing};
#[cfg(not(handheld))]
use crate::platform::{GPU_VIEW, WASH_TEXTURE, safe_area};
#[cfg(phone)]
use crate::platform::{input_window, os, touch};
use crate::program_log::log;
use std::sync::atomic::Ordering;

fn show_frame(o: &crate::owners::Owners<'static>, screen: &Screen) -> Result<(), &'static str> {
    let race = o.race;
    let racing = race.course.numbers.get().in_race.is_set();
    // The frame's length (`with_frame`), checked before anything is shown.
    if screen.pixels.len() < screen.width * screen.height * 4 {
        return Err("the frame is smaller than the screen");
    }
    #[cfg(handheld)]
    {
        // The framebuffer's mode first, its memory then the GPU's or ours.
        crate::platform::handheld::open_screen()?;
        if let Some(target) = crate::d3d::device::screen(o.system.session.screen.get())
            && crate::render::show_on_panel(
                racing,
                &target,
                |show| screen.with_frame(show),
                screen.width,
                screen.height,
                screen.menu,
            )
        {
            return Ok(());
        }
        screen.with_frame(|frame| {
            crate::platform::handheld::present(frame, screen.width, screen.height)
        })
    }
    #[cfg(not(handheld))]
    {
        // The OpenGL backend's own window shows its frames itself (the debug UI, drawn through
        // SDL's renderer, is not over them).
        if let Some(target) = crate::d3d::device::screen(o.system.session.screen.get())
            && crate::render::show_on_gl_window(
                racing,
                &target,
                |show| screen.with_frame(show),
                screen.width,
                screen.height,
                screen.menu,
            )
        {
            return Ok(());
        }
        let renderer = screen.renderer;
        // A race on the GPU: its picture, the canvas's written pixels over it.
        let finishing = std::time::Instant::now();
        let gpu_view = crate::d3d::device::screen(o.system.session.screen.get())
            .and_then(|target| crate::render::finish_frame(racing, &target))
            .map(|colour| crate::platform::make_view(renderer, colour))
            .is_some();
        timing::note_span(
            &timing::FINISH_MICROS,
            &timing::FINISH_MAX,
            finishing.elapsed().as_micros() as u64,
        );
        if !gpu_view && !screen.with_frame(|frame| screen.texture.upload(frame, screen.width * 4)) {
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
        // The race's touch controls.
        #[cfg(touch_screen)]
        if events::touch_controls_live(o.race) {
            #[cfg(phone)]
            touch::draw(
                renderer,
                input_window(screen.window),
                u64::from(os::ticks()),
            );
        }
        // The pause's and the question's answer buttons.
        #[cfg(phone)]
        {
            let asked = events::touch_asked(o.race);
            let mut kept = ANSWER_LABELS.take();
            match asked {
                // The words are made once for a question, not for every frame it is up.
                Some(asked) => {
                    if kept.as_ref().is_none_or(|(was, _)| *was != asked) {
                        kept = Some((asked, answer_labels(*o, asked)));
                    }
                    if let Some((_, labels)) = &kept {
                        touch::draw_answers(renderer, input_window(screen.window), asked, labels);
                    }
                }
                None => kept = None,
            }
            ANSWER_LABELS.set(kept);
        }
        // The debug UI over the frame.
        crate::debug::imgui_sdl::draw(o, renderer, screen.window);
        crate::remote::window_shot_taken(renderer);
        let presenting = std::time::Instant::now();
        renderer.present();
        timing::note_span(
            &timing::PRESENT_MICROS,
            &timing::PRESENT_MAX,
            presenting.elapsed().as_micros() as u64,
        );
        Ok(())
    }
}

/// The frame's texture drawn on the window: the frame keeps its shape and is centred, magnified
/// with a linear filter (none for a whole factor, which stays sharp). The window's rest is black
/// in a race; beside a menu's frame it shows the menu's background picture, covering the
/// window, blurred and darkened (`render::wash`), black while no background has been noted.
#[cfg(not(handheld))]
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

/// The wash drawn over the whole window, magnified with a linear filter.
#[cfg(not(handheld))]
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
        crate::d3d::device::NOTE_MICROS.load(Relaxed),
        timing::SHOW_MICROS.load(Relaxed),
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

/// the frame drawn shown, at the display's pace (`pacing`); false while
/// quitting (nothing shown).
pub fn show_frame_now(o: crate::owners::Owners<'static>) -> bool {
    crate::input::osk_draw::draw(o);
    let drawing = &o.gfx.drawing;
    // The 3D drawing noted so far, before the screen is shown, written or changed.
    crate::d3d::device::flush();
    crate::gfx::draw2d::check::end_frame(o.gfx, o.menus.screens.loop_flags.menu.get().ok());
    {
        if o.system.session.flags.quitting.is_set() {
            return false;
        }
        let shot_wanted = crate::remote::shot_wanted();
        let video_wanted = crate::video::frame_wanted();
        // The video's frames come back from the GPU a frame late, while the next one is drawn
        // (a screenshot wants the picture at once, and any other renderer brings it back now).
        let mut queued = crate::render::VideoRead {
            previous: None,
            started: false,
        };
        let screen_target = (shot_wanted || video_wanted)
            .then(|| crate::d3d::device::screen(o.system.session.screen.get()))
            .flatten();
        if let Some(target) = &screen_target {
            let began = std::time::Instant::now();
            if video_wanted && !shot_wanted {
                queued = crate::render::video_read_next(
                    target,
                    crate::video::spare_buffer(),
                    crate::video::nv12(),
                );
            }
            if !queued.started {
                crate::render::bring_back(target);
            }
            if video_wanted {
                crate::video::note_stage(crate::video::Stage::ReadBack, began.elapsed());
            }
        }
        if let Ok(screen) = SCREEN.lock()
            && let Some(screen) = screen.as_ref()
        {
            // Before it is shown: on the GPU's path showing clears the canvas.
            let shot = crate::remote::shot_wanted().then(|| screen.pixels.to_vec());
            let video_frame = (video_wanted && !queued.started).then(|| {
                let began = std::time::Instant::now();
                let pixels = screen.with_frame(crate::video::copy_frame);
                crate::video::note_stage(crate::video::Stage::Copy, began.elapsed());
                pixels
            });
            let showing = std::time::Instant::now();
            let shown_ok = show_frame(&o, screen);
            let shown_micros = showing.elapsed().as_micros() as u64;
            timing::SHOW_MICROS.fetch_add(shown_micros, std::sync::atomic::Ordering::Relaxed);
            timing::SHOW_MAX.fetch_max(shown_micros, std::sync::atomic::Ordering::Relaxed);
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
            if let Some(pixels) = shot {
                crate::remote::frame(&pixels, screen.width as u32, screen.height as u32);
            }
            if video_wanted {
                crate::video::frame_taken(screen.width as u32, screen.height as u32);
                // In the order the frames were drawn: the one queued before, then this one.
                for pixels in queued.previous.into_iter().chain(video_frame) {
                    crate::video::send(pixels);
                }
                let began = std::time::Instant::now();
                crate::video::sound_turn(o.audio);
                crate::video::note_stage(crate::video::Stage::Sound, began.elapsed());
                crate::video::note_progress(crate::race::render_row::fraction(o.race));
            }
            timing::note_frame_shown();
            let shown = SHOWN.fetch_add(1, Ordering::SeqCst) + 1;
            let wanted = FRAMES_TO_SHOW.load(Ordering::SeqCst);
            if wanted != 0 && shown == wanted {
                if let Some(target) = crate::d3d::device::screen(o.system.session.screen.get()) {
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
        timing::start_sampler();
        timing_report::report_timing(o.system);
        // The next frame waits for its turn on the display.
        pacing::frame_shown(crate::video::active());
    }
    true
}

/// The window's events (SDL's).
pub fn pump_messages_now(o: crate::owners::Owners<'static>) {
    events::pump(o);
}

/// For callers with a frame of their own (the screen loops): the messages, then the frame
/// unless quitting (None then).
pub fn run_frame_now<T>(o: crate::owners::Owners<'static>, frame: impl FnOnce() -> T) -> Option<T> {
    events::pump(o);
    if o.system.session.flags.quitting.is_set() {
        None
    } else {
        Some(frame())
    }
}

#[cfg(phone)]
thread_local! {
    /// The answer buttons' words made for the question now up (none while no question is).
    static ANSWER_LABELS: std::cell::Cell<Option<(rc2k_core::input::touch::Asked, Vec<touch::Label>)>> =
        const { std::cell::Cell::new(None) };
}

/// The words on the answer buttons in the game's language (its own yes and no, "continue" for
/// the plain pause), in capitals, drawn in the game's notice font as pixel runs; the plain text
/// where the font cannot draw them.
#[cfg(phone)]
fn answer_labels(
    o: crate::owners::Owners<'static>,
    asked: rc2k_core::input::touch::Asked,
) -> Vec<touch::Label> {
    use rc2k_core::strings::own::id;
    let word = |id| {
        // Windows-1252 text: the Latin-1 small letters (but the division sign) have capitals 32 less.
        let mut bytes = crate::strings::append_string_by_id::own_text(o.strings, id);
        for b in &mut bytes {
            *b = match *b {
                b'a'..=b'z' | 0xE0..=0xF6 | 0xF8..=0xFE => *b - 32,
                other => other,
            };
        }
        let plain = bytes.iter().map(|&b| char::from(b)).collect::<String>();
        touch::Label {
            runs: crate::gfx::notice_text::runs(o, &bytes),
            plain,
        }
    };
    match asked {
        rc2k_core::input::touch::Asked::Resume => vec![word(id::DIALOG_CONTINUE)],
        rc2k_core::input::touch::Asked::YesNo => vec![word(id::HINTS), word(id::HINTS + 1)],
    }
}
