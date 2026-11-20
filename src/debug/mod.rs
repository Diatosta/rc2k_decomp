//! The developers' debug pages, shown as a Dear ImGui window over the game, resized and tabbed,
//! reading the variables the developers' pages read. `overlay`: the keys and the freeze; `ui`: the
//! pages; `display_rows`: the display page's figures named and grouped; `imgui_sdl`: ImGui on
//! SDL; `wear_page`: the display page's surface names and reads.

#[cfg(debug_ui)]
pub(crate) mod imgui_sdl;
/// No debug UI in the 32-bit build (Dear ImGui's crate has no such Windows target).
#[cfg(not(debug_ui))]
pub mod imgui_sdl {
    /// The key does nothing but say so.
    pub fn toggle() -> bool {
        crate::program_log::log("debug: no debug UI in the 32-bit build");
        false
    }

    /// Nothing to let go.
    #[cfg_attr(handheld, allow(dead_code))] // the handheld swaps nothing
    pub fn renderer_going() {}

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
    #[cfg_attr(handheld, allow(dead_code))] // the handheld has neither
    pub fn is_open() -> bool {
        false
    }

    /// Nothing drawn.
    #[cfg_attr(handheld, allow(dead_code))] // the handheld has neither
    pub fn draw(
        _owners: &crate::owners::Owners<'static>,
        _renderer: crate::platform::ffi::sdl::Renderer,
        _window: crate::platform::ffi::sdl::Window,
    ) {
    }
}
#[cfg(debug_ui)]
mod dev_front_end;
#[cfg(debug_ui)]
mod display_rows;
#[cfg(debug_ui)]
mod dyno;
#[cfg(debug_ui)]
mod effects_page;
#[cfg(debug_ui)]
mod frame_figures;
#[cfg(debug_ui)]
mod gpu_cuts;
#[cfg(debug_ui)]
mod kstar;
pub(crate) mod overlay;
#[cfg(debug_ui)]
mod players;
#[cfg(debug_ui)]
mod row_cells;
pub(crate) mod screens;
#[cfg(debug_ui)]
mod switches;
pub(crate) mod timing;
#[cfg(debug_ui)]
mod ui;
#[cfg(debug_ui)]
mod ui_plot;
#[cfg(debug_ui)]
mod watch_rows;
#[cfg(debug_ui)]
mod wear_page;
