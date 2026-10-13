//! Notes the calls into the Direct3D layer made on the current thread while a
//! recording is on, for shadow comparisons and self-tests of the 3D code above it.

use crate::d3d::Vertex;
use std::cell::RefCell;

/// One call, as the code above the layer made it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Call {
    /// `d3d::set_render_state`.
    State { state: u32, value: u32 },
    /// `d3d::set_texture`; `NO_TEXTURE` for none.
    Texture(u32),
    /// `d3d::draw_fan_unclipped`: a triangle fan.
    Fan(Vec<Vertex>),
}

struct Recording {
    swallow: bool,
    calls: Vec<Call>,
}

thread_local! {
    static RECORDING: RefCell<Option<Recording>> = const { RefCell::new(None) };
}

/// Runs `f` and returns what it returned with the calls it made into the layer. With
/// `swallow` those calls go no further: nothing reaches Direct3D. Of the layer's own
/// variables only `g_currentTexture` then follows the calls; the caller puts it back.
pub fn record<R>(swallow: bool, f: impl FnOnce() -> R) -> (R, Vec<Call>) {
    // Inside another recording: that one swallows for this one too, and gets this
    // one's calls as well.
    let outer = RECORDING.with(|r| r.borrow_mut().take());
    let swallow = swallow || outer.as_ref().is_some_and(|outer| outer.swallow);
    RECORDING.with(|r| {
        r.replace(Some(Recording {
            swallow,
            calls: Vec::new(),
        }))
    });
    let result = f();
    let mine = RECORDING
        .with(|r| r.borrow_mut().take())
        .map(|mine| mine.calls)
        .unwrap_or_default();
    RECORDING.with(|r| {
        r.replace(outer.map(|mut outer| {
            outer.calls.extend(mine.iter().cloned());
            outer
        }))
    });
    (result, mine)
}

/// For the layer's hooks: notes the call if a recording is on. True if the call is to
/// be swallowed.
pub fn note(call: impl FnOnce() -> Call) -> bool {
    RECORDING.with(|r| match r.borrow_mut().as_mut() {
        Some(recording) => {
            recording.calls.push(call());
            recording.swallow
        }
        None => false,
    })
}
