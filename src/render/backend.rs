//! The 3D's GPU backend that runs, if one does: the Vulkan-style GPU API's (`gpu`) or the OpenGL
//! one's (`gles`), never both, behind one lock. The handheld has the OpenGL one only, which is
//! its GPU renderer.

use std::sync::{Mutex, MutexGuard};

use super::gles::Front;
#[cfg(not(handheld))]
use super::gpu::Gpu;

/// A backend that runs.
pub(crate) enum Running {
    /// The GPU API's (boxed: it is far larger than the other).
    #[cfg(not(handheld))]
    Vulkan(Box<Gpu>),
    /// The OpenGL one's, on its own thread.
    Gl(Front),
}

static RUNNING: Mutex<Option<Running>> = Mutex::new(None);

/// The running backend, if any, held until the guard goes.
pub(crate) fn lock() -> MutexGuard<'static, Option<Running>> {
    RUNNING.lock().unwrap()
}

/// As `lock`, or `None` when another thread holds the lock (a program ending in a fault must
/// not wait for it).
#[cfg(not(handheld))]
pub(crate) fn try_lock() -> Option<MutexGuard<'static, Option<Running>>> {
    RUNNING.try_lock().ok()
}

/// Whether the OpenGL backend runs (the handheld's GL ES is its GPU renderer, not asked here).
#[cfg(not(handheld))]
pub(crate) fn gl_runs() -> bool {
    matches!(*lock(), Some(Running::Gl(_)))
}

/// Whether the GPU API's backend runs.
#[cfg(not(handheld))]
pub(crate) fn vulkan_runs() -> bool {
    matches!(*lock(), Some(Running::Vulkan(_)))
}

impl Running {
    /// The OpenGL backend, if that is the one.
    pub(crate) fn gl(&mut self) -> Option<&mut Front> {
        match self {
            Running::Gl(front) => Some(front),
            #[cfg(not(handheld))]
            Running::Vulkan(_) => None,
        }
    }

    /// The GPU API's backend, if that is the one.
    #[cfg(not(handheld))]
    pub(crate) fn vulkan(&mut self) -> Option<&mut Gpu> {
        match self {
            Running::Vulkan(gpu) => Some(&mut **gpu),
            Running::Gl(_) => None,
        }
    }

    /// The backend that draws the frame's commands where the game has no window of the
    /// OpenGL backend's own: the GPU API's, or on the handheld its GL ES.
    pub(crate) fn drawing(&mut self) -> Option<&mut dyn super::Backend> {
        #[cfg(not(handheld))]
        return self.vulkan().map(|gpu| gpu as &mut dyn super::Backend);
        #[cfg(handheld)]
        return self.gl().map(|gl| gl as &mut dyn super::Backend);
    }
}
