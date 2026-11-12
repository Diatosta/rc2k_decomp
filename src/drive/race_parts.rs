use crate::errors::Refused;
use crate::race::owner::Race;

pub fn race_frame(race: &Race, o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    crate::drive::race_frame::race_frame(race, o, 0)?;
    Ok(())
}

/// `scene::restart_clock` (its reading into `g_sceneClock`).
pub fn restart_clock(race: &Race) {
    crate::scene::restart_clock::restart_clock(
        &race.shared,
        crate::system::tick_count::tick_count(),
    );
}

/// `input::update`.
pub fn input_update(o: crate::owners::Owners<'static>) {
    crate::write_watch::frame_begun();
    let _step = crate::write_watch::step("input");
    crate::input::keys::update_input(o);
}

/// `input::take_key` (the key taken and dropped).
pub fn take_key() {
    crate::input::take_key::take_key();
}

/// `race::race_keys`.
pub fn race_keys(race: &Race, o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    { crate::race::frame::race_keys(race, o) }?;
    Ok(())
}

/// `race::tyre_change`.
pub fn tyre_change(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    crate::race::frame::tyre_change(o)?;
    Ok(())
}

/// `race::players_all_out`: the CF.
pub fn players_all_out(race: &Race, net: &crate::net::owner::Net) -> Result<bool, Refused> {
    crate::race::frame::players_all_out(race, net)
}

/// `race::stop_race_sounds` (0x45D28E): `race::reset_race_sounds`, its one call.
pub fn stop_race_sounds(
    sound: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    crate::race::race_end::reset_race_sounds(
        &crate::race::race_end::SoundVars::game(sound),
        sound,
        audio,
        system,
    )
}

/// `race::wake_car_sounds` (its callers keep the registers round it).
pub fn wake_car_sounds(
    race: &Race,
    sound: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
) -> Result<(), Refused> {
    crate::race::sounds::wake_car_sounds(race, sound, audio)?;
    Ok(())
}

/// `logfile::log_newline_l1` and the `pusha`'d `log_write_l1_nl` of a text.
pub fn log_line(text: &std::ffi::CStr) {
    crate::logfile::log_write::log_nul(&text, true, 1);
}
