//! Notes the calls into the Direct3D layer made on the current thread while a
//! recording is on.

use crate::d3d::Vertex;
use std::cell::RefCell;

/// One call, as the code above the layer made it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Call {
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

/// Notes the call if a recording is on. True if the call is to
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
