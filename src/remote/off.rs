//! The door without the driver (the shipped build): every function does nothing or answers what
//! a game with no test driver has, with the signatures of `driver.rs`.

use crate::owners::Owners;
use crate::race::owner::Race;
use crate::race::owner_course::Course;
use std::borrow::Cow;

#[inline(always)]
pub fn fake_camera() -> bool {
    false
}

#[inline(always)]
pub fn fake_camera_count() -> usize {
    1
}

#[inline(always)]
pub fn fake_camera_frame() -> Option<crate::platform::ffi::sdl_camera::Frame> {
    None
}

#[inline(always)]
pub fn stick_at_start() {}

#[inline(always)]
pub fn note_open(_path: &[u8]) {}

#[inline(always)]
pub fn autodrive() -> bool {
    false
}

#[inline(always)]
pub fn autodrive_keys(_race: &Race, _checkpoint: u32) -> u32 {
    0
}

#[inline(always)]
pub fn poll(_o: &Owners<'static>, _course: &Course) {}

#[inline(always)]
pub fn replay_tick(_clock: u32) {}

#[inline(always)]
pub fn shots_on() -> bool {
    false
}

#[inline(always)]
pub fn shots_loading(
    _race: &Race,
    _effects: &crate::effects::owner::Effects,
    _clock: &crate::clock::tick_count::PictureClock,
) {
}

/// The game's clock runs at the wall clock's speed.
#[inline(always)]
pub fn speed() -> u32 {
    1
}

#[inline(always)]
pub fn wall_micros(milliseconds: u32) -> u64 {
    u64::from(milliseconds) * 1000
}

#[inline(always)]
pub fn scaled_millis(duration: std::time::Duration) -> u128 {
    duration.as_millis()
}

#[inline(always)]
pub fn held_race() -> bool {
    false
}

#[inline(always)]
pub fn race_loading(_race: &Race, _effects: &crate::effects::owner::Effects) {}

#[inline(always)]
pub fn race_loop_begins() {}

#[inline(always)]
pub fn held_clock() -> Option<u32> {
    None
}

#[inline(always)]
pub fn shot_frame_drawn(_race: &Race, _picture_clock: &crate::clock::tick_count::PictureClock) {}

#[inline(always)]
pub fn shot_wanted() -> bool {
    false
}

#[inline(always)]
pub fn window_shot_due() -> bool {
    false
}

#[inline(always)]
pub fn window_taken(_width: u32, _height: u32, _bgra: Vec<u8>) {}

#[inline(always)]
#[cfg_attr(
    handheld,
    allow(dead_code, reason = "the handheld's panel is not an SDL window")
)]
pub fn window_shot_taken(_renderer: crate::platform::ffi::sdl::Renderer) {}

#[inline(always)]
pub fn frame(_pixels: &[u8], _width: u32, _height: u32) {}

/// A race message goes out as it is.
#[inline(always)]
pub fn mangled(data: &[u8]) -> Cow<'_, [u8]> {
    Cow::Borrowed(data)
}

#[inline(always)]
pub fn note_sound_peak(_pcm: &[i16]) {}

#[inline(always)]
pub fn mix_deterministic() -> bool {
    false
}

#[inline(always)]
pub fn note_racing(_racing: bool) {}

#[inline(always)]
pub fn turn_end_taken() {}

#[inline(always)]
pub fn audio_turn_done() {}

#[inline(always)]
pub fn sound_loaded(_voices: u32, _limit: impl std::fmt::Display, _list: &impl std::fmt::Debug) {}

#[inline(always)]
pub fn pace_call(_car: u32, _checkpoint: u32, _group: u32) {}

#[inline(always)]
pub fn count_sound(_racing: bool, _command: u32, _first_word: u32, _is_turn_end: bool) {}

#[inline(always)]
pub fn note_shake(_shake: &rc2k_core::input::feedback::SurfaceEffect, _engine_vibration: bool) {}

#[inline(always)]
pub fn lose_this_frame() -> bool {
    false
}

#[cfg_attr(
    not(steam_support),
    allow(dead_code, reason = "only Steam's start asks")
)]
#[inline(always)]
pub fn steam_off() -> bool {
    false
}

#[inline(always)]
pub fn renderer_step_fails(_step: &str) -> bool {
    false
}

/// The display's own refresh rate.
#[inline(always)]
pub fn refresh_rate(real: Option<f64>) -> Option<f64> {
    real
}

#[inline(always)]
pub fn course_hashes(_race: &Race, _scene: &crate::scene::owner::Scene, _after: &str) {}

#[inline(always)]
pub fn reset_texture_hash() {}

#[inline(always)]
pub fn texture_row(_texture: u32, _level: u32, _y: u32, _row: &[u8]) {}

#[inline(always)]
pub fn mask_hashes(
    _masks: &crate::effects::mask_store::Masks,
    _drops: &crate::effects::windscreen_drop_vars::WindscreenDrops,
    _effects: &crate::effects::owner::Effects,
) {
}

#[inline(always)]
pub fn started_hashes(_race: &Race) {}

#[inline(always)]
pub fn fold_pixel(fold: u64, _pixel: u32) -> u64 {
    fold
}

#[inline(always)]
pub fn hold_loading_frame(_o: Owners<'static>) {}

#[inline(always)]
pub fn sim_hash_step(_race: &Race) {}
