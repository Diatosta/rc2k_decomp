//! At the game's end, if the error mode was changed (`system::set_error_mode`), the switch is
//! set to 0 and the mode it had before put back.
pub fn restore_error_mode(system: &crate::system::owner::System) {
    if system.start().error_mode.changed.get() != 1 {
        return;
    }
    system.start().error_mode.changed.set(0);
    crate::platform::os::set_error_mode(system.start().error_mode.mode.get());
}
