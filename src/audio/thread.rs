//! `audio::thread` (0x409CCE): the audio thread's routine, from its start to `ExitThread`.
//!
//! It does not return to its caller (`ExitThread(0)`).
//!
//! Its slot (`stru_518604`, +0xC running): already running goes straight to the end. Else
//! running; "AUDIOTDB:START"; `CoInitialize` (nothing to do here: no COM objects are
//! used); `g_noAudio` set: the end. Else
//! `audio::take_sample_buffer`, `audio::start` (the carry: the end), `audio::read_splash`,
//! `audio::set_mode` with `g_menuSoundMode`, `threading::set_priority` 0xF; then a turn
//! every 20 ms: sleep what is left of them since the last (`g_audioTurnTick`; at least 1),
//! `audio::stream_command` and `audio::music_command` with action 1 (a feed), `audio::take_commands`, `audio::update_voices`, `audio::restore_lost` when
//! `g_soundBufferLost`, until `g_quitting` is 1. The end: `g_noAudio` 1, `audio::close_all`,
//! `audio::free_sample_buffer`, `audio::write_splash`, `CoUninitialize`, "AUDIOTDB:QUIT",
//! priority 0, not running, `ExitThread(0)` (the routine's return, in `sync::spawn`).

use crate::errors::Refused;
use crate::system::owner::system;

const START_TEXT: &std::ffi::CStr = c"AUDIOTDB:START"; // "AUDIOTDB:START"
const QUIT_TEXT: &std::ffi::CStr = c"AUDIOTDB:QUIT"; // "AUDIOTDB:QUIT"

/// Runs to the `ExitThread`, which does not come back. Its slot's flag
/// (`threading::thread_slot::AUDIO`) says it runs.
pub fn audio_thread(audio: &crate::audio::owner::Audio) -> Result<(), Refused> {
    // The thread's root takes the system it works with.
    let system = system();
    let sound = crate::sound::owner::sound();
    use crate::audio::*;
    use crate::system::tick_count::tick_count;
    use crate::threading::set_priority::set_priority;
    let running =
        &crate::threading::thread_slot::slot_ref(crate::threading::thread_slot::AUDIO)?.flag;
    let started = running.get() != 1
        && {
            running.set(1);
            sound_device::log_text(START_TEXT, true);
            audio.state.no_audio.get() == 0
        }
        && {
            thread_parts::take_sample_buffer(&audio.state);
            start::start(audio, system)?
        };
    if started {
        a3d::splash(&audio.state)?;
        set_mode::set_mode(
            audio,
            system,
            &sound.menu.options.tune_turn,
            audio.state.menu_sound_mode.get(),
        )?;
        set_priority(0xF);
        audio.state.audio_turn_tick.set(tick_count());
        loop {
            let last = audio.state.audio_turn_tick.get();
            let now = tick_count();
            audio.state.audio_turn_tick.set(now);
            // Converted: `rc2k_core::audio::device::turn_sleep`.
            crate::system::sleep::sleep(rc2k_core::audio::device::turn_sleep(last, now));
            let begun = crate::debug::timing::start();
            // The turn's feeds (action 1, which reads nothing else).
            stream_commands::stream_command(audio, 0, 0, 1)?;
            stream_commands::music_command(
                &audio.state,
                &sound.menu.options.tune_turn,
                0,
                0,
                0,
                1,
            )?;
            take_commands::take_commands(audio, system, sound)?;
            update_voices::update_voices(audio)?;
            stray_voices::check(&audio.state);
            if begun.is_some() {
                crate::debug::timing::note_voices(stray_voices::on_lists(&audio.state));
            }
            crate::debug::timing::stop(crate::debug::timing::Stage::AudioTurn, begun);
            if audio.state.sound_buffer_lost.is_set() {
                restore::restore_lost(audio, system)?;
            }
            if system.session.flags.quitting.is_set() {
                break;
            }
        }
    }
    audio.state.no_audio.set(1);
    close_all::close_all(audio, system, &sound.menu.options.tune_turn)?;
    thread_parts::free_sample_buffer(&audio.state)?;
    a3d::splash(&audio.state)?;
    sound_device::log_text(QUIT_TEXT, true);
    set_priority(0);
    running.set(0);
    // The routine's end ends its thread (the original's `ExitThread`).
    Ok(())
}
