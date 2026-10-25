use crate::errors::Refused;

// What these calls leave in the registers is read by none of their callers (liveness at
// each call site in `race_loop`, `quit_question`, `pause_until_key`, `finish_run` and
// `race::play_movie_loop` / `play_movie`).

/// `sub_4541BF`: the race's frame (drawing and the sound tick), for the callers that do
/// not read the eax it leaves (any value in, as their eax is not read past it either).
pub fn race_frame() -> Result<(), Refused> {
    crate::drive::race_frame::race_frame(0)?;
    Ok(())
}

/// `scene::restart_clock` (its reading into `g_sceneClock`).
pub fn restart_clock() {
    crate::scene::restart_clock::restart_clock(crate::system::tick_count::tick_count());
}

/// `input::update`.
pub fn input_update() {
    crate::input::keys::update_input(&mut crate::input::keys::game_screenshot);
}

/// `input::take_key` (the key taken and dropped).
pub fn take_key() {
    crate::input::take_key::take_key();
}

/// `race::race_keys`.
pub fn race_keys() -> Result<(), Refused> {
    unsafe { crate::race::frame::race_keys() }?;
    Ok(())
}

/// `race::tyre_change`.
pub fn tyre_change() -> Result<(), Refused> {
    unsafe { crate::race::frame::tyre_change_at(&crate::race::frame::tyre_variables()) }?;
    Ok(())
}

/// `race::players_all_out`: the CF.
pub fn players_all_out() -> Result<bool, Refused> {
    unsafe { crate::race::frame::players_all_out() }
}

/// `race::stop_race_sounds` (0x45D28E): `race::reset_race_sounds`, its one call.
pub fn stop_race_sounds() -> Result<(), Refused> {
    crate::race::race_end::reset_race_sounds(&crate::race::race_end::SoundVars::game())
}

/// `race::wake_car_sounds` (its callers keep the registers round it).
pub fn wake_car_sounds() -> Result<(), Refused> {
    crate::race::sounds::wake_car_sounds()?;
    Ok(())
}

/// `logfile::log_newline_l1` and the `pusha`'d `log_write_l1_nl` of a text.
pub fn log_line(text: &std::ffi::CStr) {
    unsafe { crate::logfile::log_write::log_write(text.as_ptr().cast(), true, 1) };
}
