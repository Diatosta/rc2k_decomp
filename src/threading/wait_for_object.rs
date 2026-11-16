//! `WaitForSingleObject(handle, milliseconds)`. The network's receiving thread reads only
//! whether it was signalled (a failure counts as the time running out).

pub use rc2k_core::threading::Waited;

pub fn wait_for_object(handle: u32, milliseconds: u32) -> Waited {
    Waited::of(crate::platform::sync::wait(handle, milliseconds))
}
