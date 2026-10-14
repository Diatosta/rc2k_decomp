//! The audio thread (`audio::thread`, 0x409CCE): the side that carries out what `sound::`
//! sends. See `voices.rs`.

use crate::framework::GameHook;

pub mod buffer_play;
pub mod buffer_settings;
pub mod clear_channels;
pub mod com;
pub mod commit_3d;
pub mod describe_wave_format;
pub mod fill_buffer;
pub mod fill_voice;
pub mod find_shared_voice;
pub mod free_voices;
pub mod is_playing;
pub mod keep_sample;
pub mod load_sounds;
pub mod make_buffer;
pub mod mix;
pub mod mix_channels;
pub mod mixer;
pub mod mixer_harness;
pub mod new_voice;
pub mod play_buffer;
pub mod prepare_sample;
pub mod read_sample;
pub mod read_sample_of;
pub mod release_eax;
pub mod release_listener;
pub mod release_voice;
pub mod rewind_buffer;
pub mod sample_mode;
pub mod samples;
pub mod set_buffer_frequency;
pub mod set_buffer_pan;
pub mod set_buffer_place;
pub mod set_buffer_volume;
pub mod set_master_volume;
pub mod start_voice;
pub mod stop_all_voices;
pub mod stop_buffer;
pub mod test_copy;
pub mod test_sound;
pub mod update_mixer_voices;
pub mod update_voices;
pub mod voices;
pub mod volume_command;
pub mod write_mix;

pub static HOOKS: &[&dyn GameHook] = &[
    &start_voice::HOOK,
    &stop_all_voices::HOOK,
    &clear_channels::HOOK,
    &release_voice::HOOK,
    &release_listener::HOOK,
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
];
