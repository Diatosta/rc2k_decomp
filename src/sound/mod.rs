//! The game's side of the sound: the commands it sends to the audio thread.

use crate::framework::GameHook;

pub mod apply_settings;
pub mod play_music;
pub mod play_tune_2;
pub mod play_tune_3;
pub mod play_tune_4;
pub mod play_tune_5;
pub mod play_tune_6;
pub mod play_tune_7;
pub mod play_tune_8;
pub mod play_tune_9;
pub mod queue;
pub mod send_command;
pub mod senders;
pub mod set_music_volume;
pub mod start_stream;
pub mod stop_music;
pub mod stop_stream;
pub mod wait_for_commands;

pub static HOOKS: &[&dyn GameHook] = &[
    &send_command::HOOK,
    &wait_for_commands::HOOK,
    &stop_music::HOOK,
    &play_music::HOOK,
    &set_music_volume::HOOK,
    &stop_stream::HOOK,
    &start_stream::HOOK,
    &play_tune_2::HOOK,
    &play_tune_3::HOOK,
    &play_tune_4::HOOK,
    &play_tune_9::HOOK,
    &play_tune_5::HOOK,
    &play_tune_6::HOOK,
    &play_tune_7::HOOK,
    &play_tune_8::HOOK,
    &apply_settings::HOOK,
];
