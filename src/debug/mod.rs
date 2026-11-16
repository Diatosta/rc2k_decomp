//! The developers' debug pages, shown as a Dear ImGui window over the game, resized and tabbed,
//! reading the variables the developers' pages read. `overlay`: the keys and the freeze; `ui`: the
//! pages; `display_rows`: the display page's figures named and grouped; `imgui_sdl`: ImGui on
//! SDL; `wear_page`: the display page's surface names and reads.

pub mod check_hash;
#[cfg(debug_ui)]
pub mod imgui_sdl;
/// No debug UI in the 32-bit build (Dear ImGui's crate has no such Windows target).
#[cfg(not(debug_ui))]
pub mod imgui_sdl {
    /// The key does nothing but say so.
    pub fn toggle() -> bool {
        crate::logfile::program_log::log("debug: no debug UI in the 32-bit build");
        false
    }

    /// No event taken.
    pub fn event(_event: &sdl3_sys::everything::SDL_Event) -> bool {
        false
    }

    /// Nothing to note.
    pub fn view_drawn(
        _frame: &crate::race::owner_frame::Frame,
        _scene: &crate::scene::owner::Scene,
    ) {
    }

    /// Never open.
    #[cfg_attr(all(target_os = "linux", target_arch = "aarch64"), allow(dead_code))] // the handheld has neither
    pub fn is_open() -> bool {
        false
    }

    /// Nothing drawn.
    #[cfg_attr(all(target_os = "linux", target_arch = "aarch64"), allow(dead_code))] // the handheld has neither
    pub fn draw(
        _owners: &crate::owners::Owners<'static>,
        _renderer: crate::platform::ffi::sdl::Renderer,
        _window: crate::platform::ffi::sdl::Window,
    ) {
    }
}
#[cfg(debug_ui)]
pub mod dev_front_end;
#[cfg(debug_ui)]
pub mod display_rows;
#[cfg(debug_ui)]
pub mod dyno;
#[cfg(debug_ui)]
pub mod effects_page;
#[cfg(debug_ui)]
pub mod frame_figures;
#[cfg(debug_ui)]
pub mod gpu_cuts;
#[cfg(debug_ui)]
pub mod kstar;
pub mod overlay;
#[cfg(debug_ui)]
pub mod players;
#[cfg(debug_ui)]
pub mod row_cells;
pub mod screens;
#[cfg(debug_ui)]
pub mod switches;
pub mod timing;
#[cfg(debug_ui)]
pub mod ui;
#[cfg(debug_ui)]
pub mod ui_plot;
#[cfg(debug_ui)]
pub mod watch_rows;
#[cfg(debug_ui)]
pub mod wear_page;
