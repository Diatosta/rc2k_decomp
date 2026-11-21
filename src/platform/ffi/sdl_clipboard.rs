//! SDL3's clipboard (`sdl3-sys`), for the save code's copy and paste. The clipboard is text
//! as the system holds it; where there is no clipboard (the handheld's console build) these
//! say so and do nothing.

#![allow(unsafe_code)]

/// Whether this build has a clipboard to use.
pub const AVAILABLE: bool = cfg!(not(handheld));

/// `text` put in the clipboard; false when SDL refuses or there is no clipboard.
#[cfg(not(handheld))]
pub fn set_text(text: &str) -> bool {
    let Ok(text) = std::ffi::CString::new(text) else {
        return false;
    };
    // SAFETY: a NUL-ended text that lives past the call, which copies it.
    unsafe { sdl3_sys::everything::SDL_SetClipboardText(text.as_ptr()) }
}

/// The clipboard's text, if it holds any.
#[cfg(not(handheld))]
pub fn text() -> Option<String> {
    use sdl3_sys::everything::{SDL_GetClipboardText, SDL_free};
    // SAFETY: SDL's NUL-ended copy of the text (null checked), copied at once and freed here.
    unsafe {
        let raw = SDL_GetClipboardText();
        if raw.is_null() {
            return None;
        }
        let text = std::ffi::CStr::from_ptr(raw).to_string_lossy().into_owned();
        SDL_free(raw.cast());
        (!text.is_empty()).then_some(text)
    }
}

#[cfg(handheld)]
pub fn set_text(_: &str) -> bool {
    false
}

#[cfg(handheld)]
pub fn text() -> Option<String> {
    None
}
