//! The game's side of the sound: the commands it sends to the audio thread.

use crate::framework::GameHook;

pub mod apply_settings;
pub mod car_cue_sound;
pub mod car_engine_state;
pub mod car_gear_sounds;
pub mod car_hit_sound;
pub mod car_slot_sounds;
pub mod car_sounds;
pub mod car_sounds_all;
pub mod channels_in_use;
pub mod commands_pending;
pub mod drop_cue;
pub mod exclusive_spot_voices;
pub mod free_spot_voices;
pub mod hold_sounds_back;
pub mod let_sounds_play;
pub mod load_race_sounds;
pub mod mark_car_voices;
pub mod match_spot_voices;
pub mod merge_spot_voices;
pub mod merge_spot_voices_all;
pub mod open_cd_audio;
pub mod plain_sender;
pub mod play_car_stream;
pub mod play_cue_queue;
pub mod play_cue_queues;
pub mod play_menu_sound;
pub mod play_music;
pub mod play_placed_sound;
pub mod play_sound;
pub mod play_spot_voices;
pub mod play_timed_slot;
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
pub mod race_sound_tick;
pub mod race_turn;
pub mod race_turn_if;
pub mod repeat_spot_voices;
pub mod revs_sound;
pub mod sample_duration;
pub mod sample_settings;
pub mod send_command;
pub mod send_command_2;
pub mod send_command_7;
pub mod send_command_8;
pub mod send_end_of_turn;
pub mod send_music_command;
pub mod senders;
pub mod set_music_volume;
pub mod set_up_race_sounds;
pub mod speed_sound;
pub mod spot_sounds;
pub mod spot_voice_distances;
pub mod spot_voices;
pub mod stage_sounds;
pub mod start_race_clock;
pub mod start_stream;
pub mod step_car_stream;
pub mod step_race_clock;
pub mod stop_cd_audio;
pub mod stop_cd_track;
pub mod stop_music;
pub mod stop_sound;
pub mod stop_stream;
pub mod stream_playing;
pub mod surface_sound;
pub mod surface_sounds;
pub mod thin_spot_voices;
pub mod thin_spot_voices_all;
pub mod throttle_sound;
pub mod turn_parts;
pub mod wait_for_commands;

pub static HOOKS: &[&dyn GameHook] = &[
    &send_command::HOOK,
    &stream_playing::HOOK,
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
    &sample_settings::HOOK,
    &sample_duration::HOOK,
    &play_timed_slot::HOOK,
    &mark_car_voices::HOOK,
    &car_sounds_all::HOOK,
    &car_sounds::HOOK,
    &car_engine_state::HOOK,
    &revs_sound::HOOK,
    &throttle_sound::HOOK,
    &speed_sound::HOOK,
    &surface_sounds::HOOK,
    &surface_sound::HOOK,
    &car_slot_sounds::HOOK,
    &car_hit_sound::HOOK,
    &car_cue_sound::HOOK,
    &car_gear_sounds::HOOK,
    &drop_cue::HOOK,
    &play_cue_queue::HOOK,
    &play_cue_queues::HOOK,
    &stage_sounds::HOOK,
    &match_spot_voices::HOOK,
    &free_spot_voices::HOOK,
    &spot_voice_distances::HOOK,
    &merge_spot_voices::HOOK,
    &merge_spot_voices_all::HOOK,
    &thin_spot_voices::HOOK,
    &thin_spot_voices_all::HOOK,
    &exclusive_spot_voices::HOOK,
    &repeat_spot_voices::HOOK,
    &play_spot_voices::HOOK,
    &spot_sounds::HOOK,
    &send_end_of_turn::HOOK,
    &race_turn::HOOK,
    &race_turn_if::HOOK,
    &race_sound_tick::HOOK,
    &channels_in_use::HOOK,
    &send_command_2::HOOK,
    &send_command_8::HOOK,
    &send_music_command::HOOK,
    &send_command_7::HOOK,
    &commands_pending::HOOK,
    &open_cd_audio::HOOK,
    &stop_cd_track::HOOK,
    &stop_cd_audio::HOOK,
];
