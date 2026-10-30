//! Dear ImGui on SDL3, ours (2026-10-30; `dear-imgui-sdl3` pulls in an SDL built from its
//! source, which the Windows builds' `SDL3.dll` would clash with): SDL's events into ImGui's
//! input, and ImGui's frame drawn with the window's renderer (`SDL_RenderGeometryRaw`), its
//! textures (the font atlas) as SDL textures, through the crate's managed-texture requests.
//! Everything here runs on the window's thread (`platform`'s pump and frame).

use dear_imgui_rs::render::{DrawCmd, PendingFrame, SnapshotTextureId, TextureOp};
use dear_imgui_rs::texture::TextureFormat;
use dear_imgui_rs::{BackendFlags, ConfigFlags, Context, Key, MouseButton, TextureId};
use sdl3_sys::everything::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

/// The debug UI shown (`debug::overlay`'s key).
static OPEN: AtomicBool = AtomicBool::new(false);

/// Whether the debug UI is shown.
#[inline]
pub fn is_open() -> bool {
    OPEN.load(Ordering::Relaxed)
}

/// SDL's text input is on (it goes off with the UI, once).
static TEXT_INPUT: AtomicBool = AtomicBool::new(false);

/// The debug UI shown or hidden.
pub fn toggle() -> bool {
    !OPEN.fetch_xor(true, Ordering::Relaxed)
}

/// The first view's scene has been drawn.
pub fn view_drawn() {
    if OPEN.load(Ordering::Relaxed) {
        crate::debug::ui::note_view();
    }
}

/// An SDL texture of ImGui's.
struct Texture {
    sdl: *mut SDL_Texture,
    format: TextureFormat,
}

/// ImGui and its renderer's state.
struct Gui {
    context: Context,
    consumer: dear_imgui_rs::render::SynchronousRendererConsumer,
    textures: HashMap<SnapshotTextureId, Texture>,
    last: std::time::Instant,
    /// The style scaled for this height.
    scaled_for: i32,
}

thread_local! {
    static GUI: RefCell<Option<Gui>> = const { RefCell::new(None) };
}

/// ImGui made at the first use.
fn with_gui<T>(f: impl FnOnce(&mut Gui) -> T) -> Option<T> {
    GUI.with(|gui| {
        let mut gui = gui.borrow_mut();
        if gui.is_none() {
            let mut context = Context::create();
            // In the game's `var` folder, beside its other saved files (owner, 2026-10-30).
            let ini = crate::platform::beside_log("var").join("imgui.ini");
            let _ = context.set_ini_filename(Some(ini));
            let io = context.io_mut();
            io.set_config_flags(io.config_flags() | ConfigFlags::DOCKING_ENABLE);
            // A drag's click without a move types its value (owner, 2026-10-30).
            io.set_config_drag_click_to_input_text(true);
            let flags = io.backend_flags()
                | BackendFlags::RENDERER_HAS_TEXTURES
                | BackendFlags::RENDERER_HAS_VTX_OFFSET;
            io.set_backend_flags(flags);
            let consumer = match context.create_synchronous_renderer_consumer() {
                Ok(consumer) => consumer,
                Err(why) => {
                    crate::framework::log(&format!("debug: ImGui has no renderer: {why}"));
                    return None;
                }
            };
            *gui = Some(Gui {
                context,
                consumer,
                textures: HashMap::new(),
                last: std::time::Instant::now(),
                scaled_for: 0,
            });
        }
        gui.as_mut().map(f)
    })
}

/// ImGui's key of SDL's.
fn key_of(code: SDL_Keycode) -> Option<Key> {
    Some(match code {
        SDLK_TAB => Key::Tab,
        SDLK_LEFT => Key::LeftArrow,
        SDLK_RIGHT => Key::RightArrow,
        SDLK_UP => Key::UpArrow,
        SDLK_DOWN => Key::DownArrow,
        SDLK_PAGEUP => Key::PageUp,
        SDLK_PAGEDOWN => Key::PageDown,
        SDLK_HOME => Key::Home,
        SDLK_END => Key::End,
        SDLK_INSERT => Key::Insert,
        SDLK_DELETE => Key::Delete,
        SDLK_BACKSPACE => Key::Backspace,
        SDLK_SPACE => Key::Space,
        SDLK_RETURN => Key::Enter,
        SDLK_KP_ENTER => Key::KeypadEnter,
        SDLK_ESCAPE => Key::Escape,
        SDLK_A => Key::A,
        SDLK_C => Key::C,
        SDLK_V => Key::V,
        SDLK_X => Key::X,
        SDLK_Y => Key::Y,
        SDLK_Z => Key::Z,
        _ => return None,
    })
}

/// SDL's `event` given to ImGui while the debug UI is shown; true when ImGui takes it (the
/// game then does not see it).
pub fn event(event: &SDL_Event) -> bool {
    if !OPEN.load(Ordering::Relaxed) {
        return false;
    }
    with_gui(|gui| {
        let io = gui.context.io_mut();
        // SAFETY: `type` says which member of the event SDL filled.
        unsafe {
            let kind = SDL_EventType(event.r#type);
            match kind {
                SDL_EVENT_MOUSE_MOTION => {
                    io.add_mouse_pos_event([event.motion.x, event.motion.y]);
                    io.want_capture_mouse()
                }
                SDL_EVENT_MOUSE_BUTTON_DOWN | SDL_EVENT_MOUSE_BUTTON_UP => {
                    let button = match i32::from(event.button.button) {
                        1 => MouseButton::Left,
                        2 => MouseButton::Middle,
                        3 => MouseButton::Right,
                        _ => return false,
                    };
                    io.add_mouse_button_event(button, kind == SDL_EVENT_MOUSE_BUTTON_DOWN);
                    io.want_capture_mouse()
                }
                SDL_EVENT_MOUSE_WHEEL => {
                    io.add_mouse_wheel_event([event.wheel.x, event.wheel.y]);
                    io.want_capture_mouse()
                }
                SDL_EVENT_TEXT_INPUT => {
                    if !event.text.text.is_null() {
                        let text = std::ffi::CStr::from_ptr(event.text.text);
                        io.add_input_characters_utf8(text.to_string_lossy());
                    }
                    io.want_capture_keyboard()
                }
                SDL_EVENT_KEY_DOWN | SDL_EVENT_KEY_UP => {
                    let down = event.key.down;
                    let m = event.key.r#mod;
                    io.add_key_event(Key::ModCtrl, m.0 & SDL_KMOD_CTRL.0 != 0);
                    io.add_key_event(Key::ModShift, m.0 & SDL_KMOD_SHIFT.0 != 0);
                    io.add_key_event(Key::ModAlt, m.0 & SDL_KMOD_ALT.0 != 0);
                    if let Some(key) = key_of(event.key.key) {
                        io.add_key_event(key, down);
                    }
                    io.want_capture_keyboard()
                }
                _ => false,
            }
        }
    })
    .unwrap_or(false)
}

/// The debug UI drawn on `renderer` (after the game's frame, before the present).
pub fn draw(renderer: *mut SDL_Renderer, window: *mut SDL_Window) {
    let open = OPEN.load(Ordering::Relaxed);
    // ImGui's text fields need SDL's text input, started while the UI is shown.
    if open != TEXT_INPUT.swap(open, Ordering::Relaxed) {
        // SAFETY: the window SDL made, on its thread.
        unsafe {
            if open {
                SDL_StartTextInput(window);
            } else {
                SDL_StopTextInput(window);
            }
        }
    }
    // Closed: nothing else, and ImGui not even made until it is first opened.
    if !open {
        return;
    }
    with_gui(|gui| {
        let (mut w, mut h, mut pw, mut ph) = (0, 0, 0, 0);
        // SAFETY: as above; the renderer is the window's.
        unsafe {
            SDL_GetWindowSize(window, &mut w, &mut h);
            SDL_GetRenderOutputSize(renderer, &mut pw, &mut ph);
        }
        if w <= 0 || h <= 0 {
            return;
        }
        if gui.scaled_for != h {
            // Sizes with the screen, so that the menus' 480 lines are not filled.
            let scale = (h as f32 / 720.0).max(0.7);
            let style = gui.context.style_mut();
            style.set_font_scale_main(scale);
            gui.scaled_for = h;
        }
        let now = std::time::Instant::now();
        let delta = now.duration_since(gui.last).as_secs_f32().max(1e-4);
        gui.last = now;
        let io = gui.context.io_mut();
        io.set_display_size([w as f32, h as f32]);
        io.set_display_framebuffer_scale([pw as f32 / w as f32, ph as f32 / h as f32]);
        io.set_delta_time(delta);
        let frame = gui.context.begin_frame();
        crate::debug::ui::build(frame.ui());
        let pending = frame.render(&gui.consumer);
        if let Err(why) = render(&mut gui.textures, renderer, pending) {
            crate::framework::log(&format!("debug: the UI not drawn: {why}"));
        }
    });
}

/// RGBA bytes of a texture's rows (an alpha texture's as white with its alpha).
fn rgba(format: TextureFormat, rows: &[u8]) -> std::borrow::Cow<'_, [u8]> {
    match format {
        TextureFormat::RGBA32 => std::borrow::Cow::Borrowed(rows),
        TextureFormat::Alpha8 => rows.iter().flat_map(|&a| [255, 255, 255, a]).collect(),
    }
}

/// Bytes a pixel.
fn wide(format: TextureFormat) -> usize {
    match format {
        TextureFormat::RGBA32 => 4,
        TextureFormat::Alpha8 => 1,
    }
}

/// ImGui's texture requests met and its triangles drawn.
fn render(
    textures: &mut HashMap<SnapshotTextureId, Texture>,
    renderer: *mut SDL_Renderer,
    frame: PendingFrame<'_>,
) -> Result<(), String> {
    let mut feedback = Vec::with_capacity(frame.texture_requests().len());
    for request in frame.texture_requests() {
        match request.operation() {
            TextureOp::Create {
                format,
                width,
                height,
                row_pitch,
                pixels,
            } => {
                if let Some(old) = textures.remove(&request.texture()) {
                    // SAFETY: a texture made below, no longer used.
                    unsafe { SDL_DestroyTexture(old.sdl) };
                }
                // SAFETY: the window's renderer; RGBA bytes are ABGR8888 on a little-endian
                // machine.
                let sdl = unsafe {
                    SDL_CreateTexture(
                        renderer,
                        SDL_PIXELFORMAT_ABGR8888,
                        SDL_TEXTUREACCESS_STATIC,
                        *width as i32,
                        *height as i32,
                    )
                };
                if sdl.is_null() {
                    return Err("SDL makes no texture".into());
                }
                let rows = rgba(*format, pixels);
                let pitch = row_pitch * 4 / wide(*format);
                // SAFETY: `rows` holds `height` rows of `pitch` bytes.
                unsafe {
                    SDL_UpdateTexture(sdl, std::ptr::null(), rows.as_ptr().cast(), pitch as i32);
                    SDL_SetTextureBlendMode(sdl, SDL_BLENDMODE_BLEND);
                    SDL_SetTextureScaleMode(sdl, SDL_SCALEMODE_LINEAR);
                }
                textures.insert(
                    request.texture(),
                    Texture {
                        sdl,
                        format: *format,
                    },
                );
                let id = TextureId::new(sdl.expose_provenance() as u64);
                feedback.push(request.uploaded(id).map_err(|e| e.to_string())?);
            }
            TextureOp::Update { rects, .. } => {
                let Some(texture) = textures.get(&request.texture()) else {
                    feedback.push(request.retry());
                    continue;
                };
                for update in rects {
                    let r = update.rect;
                    let rows = rgba(texture.format, &update.data);
                    let pitch = update.row_pitch * 4 / wide(texture.format);
                    let rect = SDL_Rect {
                        x: i32::from(r.x),
                        y: i32::from(r.y),
                        w: i32::from(r.w),
                        h: i32::from(r.h),
                    };
                    // SAFETY: `rows` holds the rectangle's rows of `pitch` bytes.
                    unsafe {
                        SDL_UpdateTexture(texture.sdl, &rect, rows.as_ptr().cast(), pitch as i32)
                    };
                }
                let id = TextureId::new(texture.sdl.expose_provenance() as u64);
                feedback.push(request.uploaded(id).map_err(|e| e.to_string())?);
            }
            TextureOp::Destroy => {
                if let Some(old) = textures.remove(&request.texture()) {
                    // SAFETY: a texture made above, no longer drawn.
                    unsafe { SDL_DestroyTexture(old.sdl) };
                }
                feedback.push(request.destroyed().map_err(|e| e.to_string())?);
            }
        }
    }
    let frame = frame
        .reconcile_texture_feedback(feedback)
        .map_err(|e| e.to_string())?;
    let data = frame.draw_data();
    let origin = data.display_pos();
    let scale = data.framebuffer_scale();
    for list in data.draw_lists() {
        let vertices = list.vtx_buffer();
        let indices = list.idx_buffer();
        let xy: Vec<[f32; 2]> = vertices
            .iter()
            .map(|v| {
                [
                    (v.pos[0] - origin[0]) * scale[0],
                    (v.pos[1] - origin[1]) * scale[1],
                ]
            })
            .collect();
        let colours: Vec<SDL_FColor> = vertices
            .iter()
            .map(|v| {
                let [r, g, b, a] = v.rgba();
                SDL_FColor {
                    r: f32::from(r) / 255.0,
                    g: f32::from(g) / 255.0,
                    b: f32::from(b) / 255.0,
                    a: f32::from(a) / 255.0,
                }
            })
            .collect();
        for command in list.commands() {
            let DrawCmd::Elements { count, cmd_params } = command else {
                continue;
            };
            let [l, t, r, b] = cmd_params.clip_rect;
            let clip = SDL_Rect {
                x: ((l - origin[0]) * scale[0]) as i32,
                y: ((t - origin[1]) * scale[1]) as i32,
                w: ((r - l) * scale[0]) as i32,
                h: ((b - t) * scale[1]) as i32,
            };
            if clip.w <= 0 || clip.h <= 0 {
                continue;
            }
            let first = cmd_params.vtx_offset;
            let Some(index) = indices.get(cmd_params.idx_offset..cmd_params.idx_offset + count)
            else {
                return Err("a draw command past its indices".into());
            };
            if first >= vertices.len() {
                return Err("a draw command past its vertices".into());
            }
            let texture = std::ptr::with_exposed_provenance_mut::<SDL_Texture>(
                cmd_params.texture_id.id() as usize,
            );
            let stride = core::mem::size_of_val(&vertices[0]) as i32;
            // SAFETY: the vertices from `first` on and `count` indices of them (each below
            // the vertex count, ImGui's), the texture one made above.
            unsafe {
                SDL_SetRenderClipRect(renderer, &clip);
                SDL_RenderGeometryRaw(
                    renderer,
                    texture,
                    xy[first..].as_ptr().cast(),
                    8,
                    colours[first..].as_ptr(),
                    core::mem::size_of::<SDL_FColor>() as i32,
                    vertices[first..].as_ptr().cast::<f32>().add(2),
                    stride,
                    (vertices.len() - first) as i32,
                    index.as_ptr().cast(),
                    count as i32,
                    2,
                );
            }
        }
    }
    // SAFETY: the window's renderer.
    unsafe { SDL_SetRenderClipRect(renderer, std::ptr::null()) };
    Ok(())
}
