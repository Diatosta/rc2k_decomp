//! The web build has no desktop OpenGL backend (its WebGL2 path is not made yet): the names the
//! platform layer asks of `gles`, answering that no OpenGL window runs, and the mip level
//! arithmetic the GPU backend shares with it.

#[path = "gles/mip_levels.rs"]
pub mod mip_levels;

/// The OpenGL backend is never wanted here.
pub fn wanted() -> bool {
    false
}

/// There is no OpenGL window.
pub fn window() -> Option<crate::platform::ffi::sdl::Window> {
    None
}

/// Nothing to start.
pub fn start_on_window(_size: (usize, usize)) -> bool {
    false
}

/// Nothing to stop.
pub fn stop() -> bool {
    false
}

/// The backend that never runs here: the type the platform layer's code names, held in `GPU`
/// as `None` always.
pub struct Gpu;

/// The running OpenGL backend: never one.
pub static GPU: std::sync::Mutex<Option<Gpu>> = std::sync::Mutex::new(None);

impl super::Backend for Gpu {
    fn draw(&mut self, _target: &super::Target, _commands: super::CommandList) {}
}

impl Gpu {
    pub fn bring_back(&mut self, _target: &super::Target, _canvas: &[u8]) {}
    pub fn set_wash(&mut self, _image: &super::wash::Image) {}
    pub fn finish_and_show(&mut self, _target: &super::Target, _canvas: &[u8], _menu: bool) {}
    pub fn show_picture(&mut self, _frame: &[u8], _width: usize, _height: usize, _menu: bool) {}
}
