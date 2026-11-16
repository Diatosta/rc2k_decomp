use crate::errors::Refused;
use crate::race::owner::Race;

pub fn race_frame(race: &Race, o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    crate::drive::race_frame::race_frame(race, o, 0)?;
    Ok(())
}

pub fn restart_clock(race: &Race, system: &crate::system::owner::System) {
    crate::scene::restart_clock::restart_clock(
        &race.shared,
        crate::system::tick_count::tick_count(&system.session.picture_clock),
    );
}

pub fn input_update(o: crate::owners::Owners<'static>) {
    crate::input::keys::update_input(o);
}

pub fn take_key(input: &crate::input::owner::Input) {
    crate::input::take_key::take_key(input);
}

pub fn race_keys(race: &Race, o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    { crate::race::frame::race_keys(race, o) }?;
    Ok(())
}

pub fn tyre_change(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    crate::race::frame::tyre_change(o)?;
    Ok(())
}

pub fn players_all_out(
    race: &Race,
    net_race: &rc2k_core::net::net_race::NetRaceSlot,
) -> Result<bool, Refused> {
    crate::race::frame::players_all_out(race, net_race)
}

pub fn stop_race_sounds(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    crate::race::race_end::reset_race_sounds(&crate::race::race_end::SoundVars::game(o.sound), o)
}

pub fn wake_car_sounds(
    race: &Race,
    sound: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    crate::race::sounds::wake_car_sounds(race, sound, audio, system)?;
    Ok(())
}

/// A text logged at level 1 with a line break after it.
pub fn log_line(text: &std::ffi::CStr) {
    crate::logfile::log_write::log_nul(&text, true, 1);
}
