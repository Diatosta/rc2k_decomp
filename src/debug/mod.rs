//! The developers' debug pages: in RAL.EXE (0x41FF44 .. 0x435F2F and its helpers) but
//! reached from nowhere; shown here as a Dear ImGui window over the game (owner,
//! 2026-10-30: "show them in dearImgui, ... resized, ... tabbed"), reading the variables the
//! original pages read. `overlay`: the keys and the freeze; `ui`: the pages; `display_rows`: the display page's figures named and grouped; `imgui_sdl`:
//! ImGui on SDL; `wear_page`: the display page's surface names and reads.

#[cfg(target_pointer_width = "64")]
pub mod imgui_sdl;
/// No debug UI in the 32-bit build (Dear ImGui's crate has no such Windows target).
#[cfg(not(target_pointer_width = "64"))]
pub mod imgui_sdl {
    /// The key does nothing but say so.
    pub fn toggle() -> bool {
        crate::framework::log("debug: no debug UI in the 32-bit build");
        false
    }

    /// No event taken.
    pub fn event(_event: &sdl3_sys::everything::SDL_Event) -> bool {
        false
    }

    /// Nothing to note.
    pub fn view_drawn() {}

    /// Never open.
    pub fn is_open() -> bool {
        false
    }

    /// Nothing drawn.
    pub fn draw(
        _renderer: *mut sdl3_sys::everything::SDL_Renderer,
        _window: *mut sdl3_sys::everything::SDL_Window,
    ) {
    }
}
#[cfg(target_pointer_width = "64")]
pub mod dev_front_end;
#[cfg(target_pointer_width = "64")]
pub mod display_rows;
#[cfg(target_pointer_width = "64")]
pub mod dyno;
#[cfg(target_pointer_width = "64")]
pub mod effects_page;
#[cfg(target_pointer_width = "64")]
pub mod kstar;
pub mod overlay;
#[cfg(target_pointer_width = "64")]
pub mod players;
pub mod screens;
#[cfg(target_pointer_width = "64")]
pub mod switches;
pub mod timing;
#[cfg(target_pointer_width = "64")]
pub mod ui;
#[cfg(target_pointer_width = "64")]
pub mod ui_plot;
#[cfg(target_pointer_width = "64")]
pub mod watch_rows;
#[cfg(target_pointer_width = "64")]
pub mod wear_page;
