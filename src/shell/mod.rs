//! The game's front end on the platform: what a frame's end, the window's events, the films and
//! the touch screen do with the game's owners. `platform` below it knows no game type; this is
//! the one place that joins its windows, devices and clocks to the input, the race and the menus.
//!
//! - `events`: SDL's events pumped into the quit switch, the pending key, the pads and the
//!   fingers.
//! - `android`: the Android build's entry.
//! - `frame`: a frame's end shown, and the window's events pumped for any loop that waits.
//! - `film`: the films' players, their picture and sound.
//! - `touch_menu`: the touch screen's gestures as the keys the menus take.
//! - `window_follow`: a resized window followed by the next race's size.
//! - `timing_report`: the log's periodic line on where the time goes.

#[cfg(target_os = "android")]
pub mod android;
#[cfg(discord_support)]
mod discord;
pub(crate) mod events;
pub(crate) mod film;
pub(crate) mod frame;
mod timing_report;
#[cfg(touch_screen)]
pub(crate) mod touch_menu;
#[cfg(not(handheld))]
mod window_follow;
