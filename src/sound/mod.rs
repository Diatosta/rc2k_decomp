//! The game's side of the sound: the commands it sends to the audio thread.

use crate::framework::GameHook;

pub mod apply_settings;
pub mod hold_sounds_back;
pub mod let_sounds_play;
pub mod load_race_sounds;
pub mod play_car_stream;
pub mod play_menu_sound;
pub mod play_music;
pub mod play_placed_sound;
pub mod play_sound;
pub mod play_tune_2;
pub mod play_tune_3;
pub mod play_tune_4;
pub mod play_tune_5;
pub mod play_tune_6;
pub mod play_tune_7;
pub mod play_tune_8;
pub mod play_tune_9;
pub mod queue;
pub mod race_clock;
pub mod send_command;
pub mod senders;
pub mod set_music_volume;
pub mod set_up_race_sounds;
pub mod start_race_clock;
pub mod start_stream;
pub mod step_car_stream;
pub mod step_race_clock;
pub mod stop_music;
pub mod stop_sound;
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
    &play_sound::HOOK,
    &play_menu_sound::HOOK,
    &load_race_sounds::HOOK,
    &set_up_race_sounds::HOOK,
    &play_placed_sound::HOOK,
    &stop_sound::HOOK,
    &start_race_clock::HOOK,
    &step_race_clock::HOOK,
    &play_car_stream::HOOK,
    &step_car_stream::HOOK,
    &hold_sounds_back::HOOK,
    &let_sounds_play::HOOK,
];
