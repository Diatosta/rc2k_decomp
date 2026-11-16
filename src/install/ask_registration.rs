//! Sets the flag that runs the registration program after the game.

pub fn ask_registration(install: &crate::install::owner::Install) {
    install.run.run_registration.set(1);
}
