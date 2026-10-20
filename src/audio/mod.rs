//! The audio thread (`audio::thread`, 0x409CCE): the side that carries out what `sound::`
//! sends. See `voices.rs`.

use crate::framework::GameHook;

pub mod a3d;
pub mod buffer_play;
pub mod buffer_settings;
pub mod choose_modes;
pub mod clear_channels;
pub mod close_a3d2;
pub mod close_all;
pub mod close_primary_mixer;
pub mod close_sound;
pub mod close_way;
pub mod com;
pub mod commit_3d;
pub mod describe_wave_format;
pub mod feed_stream;
pub mod fill_buffer;
pub mod fill_stream_part;
pub mod fill_voice;
pub mod find_shared_voice;
pub mod free_sample_buffer;
pub mod free_voices;
pub mod is_playing;
pub mod keep_sample;
pub mod load_sounds;
pub mod make_buffer;
pub mod make_primary;
pub mod mix;
pub mod mix_channels;
pub mod mixer;
pub mod mixer_harness;
pub mod mixer_set_up;
pub mod music_command;
pub mod new_voice;
pub mod open_a3d;
pub mod open_a3d2;
pub mod open_eax;
pub mod open_listener;
pub mod open_sound;
pub mod open_voice_manager;
pub mod open_way;
pub mod play_buffer;
pub mod play_primary;
pub mod prepare_sample;
pub mod primary;
pub mod property_sets;
pub mod read_caps;
pub mod read_sample;
pub mod read_sample_of;
pub mod read_splash;
pub mod release_a3d;
pub mod release_eax;
pub mod release_primary;
pub mod release_sound_listener;
pub mod release_voice;
pub mod release_voice_manager;
pub mod reopen_sound;
pub mod replay_looping;
pub mod restore;
pub mod restore_buffer;
pub mod restore_lost;
pub mod rewind_buffer;
pub mod sample_mode;
pub mod samples;
pub mod set_buffer_frequency;
pub mod set_buffer_min_distance;
pub mod set_buffer_pan;
pub mod set_buffer_place;
pub mod set_buffer_volume;
pub mod set_eax_room;
pub mod set_master_volume;
pub mod set_mode;
pub mod set_primary_format;
pub mod set_stream_volume;
pub mod set_up_primary_mixer;
pub mod sound_device;
pub mod start;
pub mod start_stream;
pub mod start_voice;
pub mod stop_all_voices;
pub mod stop_buffer;
pub mod stop_primary;
pub mod stop_stream;
pub mod stop_streams;
pub mod stray_voices;
pub mod stream;
pub mod stream_command;
pub mod stream_commands;
pub mod take_commands;
pub mod take_sample_buffer;
pub mod test_copy;
pub mod test_sound;
pub mod thread;
pub mod thread_parts;
pub mod update_mixer_voices;
pub mod update_voices;
pub mod voices;
pub mod volume_command;
pub mod volume_index;
pub mod write_mix;
pub mod write_splash;

pub static HOOKS: &[&dyn GameHook] = &[
    &start_voice::HOOK,
    &stop_all_voices::HOOK,
    &clear_channels::HOOK,
    &release_voice::HOOK,
    &release_voice_manager::HOOK,
    &release_eax::HOOK,
    &set_master_volume::HOOK,
    &volume_command::HOOK,
    &free_voices::HOOK,
    &read_sample::HOOK,
    &read_sample_of::HOOK,
    &find_shared_voice::HOOK,
    &new_voice::HOOK,
    &fill_voice::HOOK,
    &sample_mode::HOOK,
    &prepare_sample::HOOK,
    &keep_sample::HOOK,
    &describe_wave_format::HOOK,
    &set_buffer_volume::HOOK,
    &set_buffer_pan::HOOK,
    &set_buffer_frequency::HOOK,
    &set_buffer_place::HOOK,
    &set_buffer_min_distance::HOOK,
    &open_voice_manager::HOOK,
    &open_eax::HOOK,
    &set_eax_room::HOOK,
    &make_buffer::HOOK,
    &fill_buffer::HOOK,
    &load_sounds::HOOK,
    &is_playing::HOOK,
    &rewind_buffer::HOOK,
    &play_buffer::HOOK,
    &stop_buffer::HOOK,
    &commit_3d::HOOK,
    &update_voices::HOOK,
    &mix_channels::HOOK,
    &write_mix::HOOK,
    &mix::HOOK,
    &update_mixer_voices::HOOK,
    &fill_stream_part::HOOK,
    &stop_stream::HOOK,
    &set_stream_volume::HOOK,
    &start_stream::HOOK,
    &feed_stream::HOOK,
    &stream_command::HOOK,
    &music_command::HOOK,
    &stop_streams::HOOK,
    &take_commands::HOOK,
    &take_sample_buffer::HOOK,
    &free_sample_buffer::HOOK,
    &play_primary::HOOK,
    &stop_primary::HOOK,
    &release_sound_listener::HOOK,
    &release_primary::HOOK,
    &read_caps::HOOK,
    &release_a3d::HOOK,
    &close_sound::HOOK,
    &volume_index::HOOK,
    &set_primary_format::HOOK,
    &open_listener::HOOK,
    &make_primary::HOOK,
    &restore_buffer::HOOK,
    &replay_looping::HOOK,
    &restore_lost::HOOK,
    &close_primary_mixer::HOOK,
    &set_up_primary_mixer::HOOK,
    &open_way::HOOK,
    &close_way::HOOK,
    &open_a3d::HOOK,
    &open_a3d2::HOOK,
    &close_a3d2::HOOK,
    &read_splash::HOOK,
    &write_splash::HOOK,
    &close_all::HOOK,
    &open_sound::HOOK,
    &reopen_sound::HOOK,
    &choose_modes::HOOK,
    &start::HOOK,
    &set_mode::HOOK,
    &thread::HOOK,
];
