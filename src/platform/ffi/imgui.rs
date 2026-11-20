//! SDL3 as the debug UI's backend (`sdl3-sys`), for `debug::imgui_sdl`: SDL's events decoded
//! for Dear ImGui, the window's text input, and the window's renderer drawing ImGui's
//! triangles with textures of its own. (Dear ImGui itself is reached through the safe
//! `dear-imgui-rs`: there is no ImGui C glue here, only SDL's.)
//!
//! The window's and renderer's calls are methods on `ffi::sdl`'s handles (the window the
//! platform made, used on its thread while it lives). A `Texture` is made by
//! `Texture::create` and destroyed once by its owner.

#![allow(unsafe_code)]

use super::sdl::Renderer;
use dear_imgui_rs::render::DrawVert;
use sdl3_sys::everything::*;

impl super::sdl::Window {
    /// SDL's text input started for the window.
    pub fn start_text_input(self) {
        // SAFETY: the window SDL made, on its thread.
        unsafe { SDL_StartTextInput(self.0) };
    }

    /// SDL's text input stopped for the window.
    pub fn stop_text_input(self) {
        // SAFETY: as `start_text_input`.
        unsafe { SDL_StopTextInput(self.0) };
    }

    /// The window's size in screen units.
    pub fn size(self) -> (i32, i32) {
        let (mut w, mut h) = (0, 0);
        // SAFETY: the window SDL made, two integers to fill.
        unsafe { SDL_GetWindowSize(self.0, &raw mut w, &raw mut h) };
        (w, h)
    }
}

impl Renderer {
    /// The renderer's output size in pixels.
    pub fn output_size(self) -> (i32, i32) {
        let (mut w, mut h) = (0, 0);
        // SAFETY: the window's renderer, two integers to fill.
        unsafe { SDL_GetRenderOutputSize(self.0, &raw mut w, &raw mut h) };
        (w, h)
    }

    /// The clip rectangle set, or cleared with none.
    pub fn set_clip_rect(self, clip: Option<&SDL_Rect>) {
        // SAFETY: the window's renderer; a rectangle read during the call, or null (clear).
        unsafe { SDL_SetRenderClipRect(self.0, clip.map_or(std::ptr::null(), std::ptr::from_ref)) };
    }

    /// Triangles drawn from `vertices[first..]` (Dear ImGui's vertex records: positions
    /// `xy[first..]` already in pixels, colours `colours[first..]`, texture coordinates from the
    /// records) by `indices` (16-bit, into those vertices; SDL checks them against the count).
    /// `texture_id` is a `Texture::id`.
    pub fn draw_geometry(
        self,
        texture_id: u64,
        xy: &[[f32; 2]],
        colours: &[SDL_FColor],
        vertices: &[DrawVert],
        first: usize,
        indices: &[u16],
    ) {
        let count = vertices.len().saturating_sub(first);
        // The slices must hold what SDL reads: `count` vertices each.
        if first >= vertices.len() || xy.len() < vertices.len() || colours.len() < vertices.len() {
            return;
        }
        let texture = std::ptr::with_exposed_provenance_mut::<SDL_Texture>(texture_id as usize);
        let stride = size_of::<DrawVert>() as i32;
        // SAFETY: the vertices from `first` on (`count` of them, checked above for each of the
        // three arrays), `indices.len()` 16-bit indices that SDL checks against `count`, the
        // texture one `Texture::create` made (or null); the texture coordinates are the
        // records' second pair of floats, at their stride.
        unsafe {
            SDL_RenderGeometryRaw(
                self.0,
                texture,
                xy[first..].as_ptr().cast(),
                8,
                colours[first..].as_ptr(),
                size_of::<SDL_FColor>() as i32,
                vertices[first..].as_ptr().cast::<f32>().add(2),
                stride,
                count as i32,
                indices.as_ptr().cast(),
                indices.len() as i32,
                2,
            );
        }
    }
}

/// An SDL texture of Dear ImGui's, with its width (the bytes a row of its pixels takes).
pub struct Texture {
    raw: *mut SDL_Texture,
    width: i32,
}

impl Texture {
    /// A static ABGR8888 texture (RGBA bytes on a little-endian machine) of `width` x `height`
    /// pixels holding `rows` (`pitch` bytes a row), blended and linearly scaled. SDL's error
    /// text when it makes none; a short `rows` makes none.
    pub fn create(
        renderer: Renderer,
        width: i32,
        height: i32,
        rows: &[u8],
        pitch: i32,
    ) -> Result<Self, String> {
        // SAFETY: the window's renderer.
        let sdl = unsafe {
            SDL_CreateTexture(
                renderer.0,
                SDL_PIXELFORMAT_ABGR8888,
                SDL_TEXTUREACCESS_STATIC,
                width,
                height,
            )
        };
        if sdl.is_null() {
            return Err("SDL makes no texture".into());
        }
        let texture = Self { raw: sdl, width };
        if !texture.update(None, height, rows, pitch) {
            texture.destroy();
            return Err("SDL's texture is given too few pixels".into());
        }
        // SAFETY: the texture made above.
        unsafe {
            SDL_SetTextureBlendMode(sdl, SDL_BLENDMODE_BLEND);
            SDL_SetTextureScaleMode(sdl, SDL_SCALEMODE_LINEAR);
        }
        Ok(texture)
    }

    /// The pixels of `rect` (the whole texture with none) from `rows`: `height` rows of `pitch`
    /// bytes. False, and nothing done, when `rows` is too short for that.
    pub fn update(&self, rect: Option<&SDL_Rect>, height: i32, rows: &[u8], pitch: i32) -> bool {
        // The call reads the rectangle's rows (the texture's, with none): as wide as the
        // rectangle, as many as it is high, `pitch` bytes apart.
        let (width, rows_read) = rect.map_or((self.width, height), |r| (r.w, r.h));
        if pitch < 0
            || height < 0
            || width < 0
            || rows_read > height
            || !super::sdl::rows_fit(
                rows.len(),
                pitch as usize,
                (width as usize).saturating_mul(4),
                height as usize,
            )
        {
            return false;
        }
        // SAFETY: a texture `create` made and not yet destroyed; `rows` holds `height` rows
        // of `pitch` bytes, each at least as wide as what the call reads (checked above), the
        // rectangle read during the call.
        unsafe {
            SDL_UpdateTexture(
                self.raw,
                rect.map_or(std::ptr::null(), std::ptr::from_ref),
                rows.as_ptr().cast(),
                pitch,
            );
        }
        true
    }

    /// The number Dear ImGui carries for this texture (`Renderer::draw_geometry` takes it).
    pub fn id(&self) -> u64 {
        self.raw.expose_provenance() as u64
    }

    /// Destroyed; no id of it is drawn after.
    pub fn destroy(self) {
        // SAFETY: a texture `create` made, destroyed once.
        unsafe { SDL_DestroyTexture(self.raw) };
    }
}

/// What Dear ImGui takes from one of SDL's events.
pub enum UiEvent {
    MouseMotion {
        x: f32,
        y: f32,
    },
    /// A mouse button's number (1 left, 2 middle, 3 right) and whether it went down.
    MouseButton {
        button: i32,
        down: bool,
    },
    MouseWheel {
        x: f32,
        y: f32,
    },
    /// Typed text (none when SDL gave a null one).
    Text(Option<String>),
    Key {
        down: bool,
        modifiers: SDL_Keymod,
        key: SDL_Keycode,
    },
    /// Anything else.
    Other,
}

/// SDL's `event` as Dear ImGui's kinds.
pub fn decode_event(event: &SDL_Event) -> UiEvent {
    // SAFETY: `type` says which member of the event SDL filled.
    unsafe {
        match SDL_EventType(event.r#type) {
            SDL_EVENT_MOUSE_MOTION => UiEvent::MouseMotion {
                x: event.motion.x,
                y: event.motion.y,
            },
            kind @ (SDL_EVENT_MOUSE_BUTTON_DOWN | SDL_EVENT_MOUSE_BUTTON_UP) => {
                UiEvent::MouseButton {
                    button: i32::from(event.button.button),
                    down: kind == SDL_EVENT_MOUSE_BUTTON_DOWN,
                }
            }
            SDL_EVENT_MOUSE_WHEEL => UiEvent::MouseWheel {
                x: event.wheel.x,
                y: event.wheel.y,
            },
            SDL_EVENT_TEXT_INPUT => UiEvent::Text(if event.text.text.is_null() {
                None
            } else {
                Some(
                    std::ffi::CStr::from_ptr(event.text.text)
                        .to_string_lossy()
                        .into_owned(),
                )
            }),
            SDL_EVENT_KEY_DOWN | SDL_EVENT_KEY_UP => UiEvent::Key {
                down: event.key.down,
                modifiers: event.key.r#mod,
                key: event.key.key,
            },
            _ => UiEvent::Other,
        }
    }
}
