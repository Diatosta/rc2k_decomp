//! The audio thread's routine, from its start to the thread's end. Already running goes straight
//! to the end. Else: "AUDIOTDB:START"; unless audio is off, the sample buffer taken, `audio::start`
//! (failing: the end), the splash read, the menu sound mode set, priority raised; then a turn every
//! 20 ms: sleep what is left of them, the streams fed, commands taken, voices updated, lost
//! buffers restored, until the game quits. The end: audio off, everything closed, the splash
//! written, "AUDIOTDB:QUIT".

use crate::errors::Refused;

const START_TEXT: &std::ffi::CStr = c"AUDIOTDB:START"; // "AUDIOTDB:START"
const QUIT_TEXT: &std::ffi::CStr = c"AUDIOTDB:QUIT"; // "AUDIOTDB:QUIT"

/// Runs to the thread's end, which does not come back. Its slot's flag
/// (`threading::thread_slot::AUDIO`) says it runs.
pub fn audio_thread(
    owner: &'static crate::audio::owner::Audio,
    sound: &crate::sound::owner::Sound,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    use crate::audio::*;
    use crate::system::tick_count::tick_count;
    use crate::threading::set_priority::set_priority;
    // The thread's own variables (`audio::own`): made here, so that no other thread has them.
    let audio = &crate::audio::own::AudioThread::new(owner);
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
        set_mode::set_mode(audio, system, audio.state.menu_sound_mode.get())?;
        set_priority(0xF);
        audio.state.audio_turn_tick.set(tick_count());
        loop {
            let last = audio.state.audio_turn_tick.get();
            let now = tick_count();
            audio.state.audio_turn_tick.set(now);
            crate::system::sleep::sleep(rc2k_core::audio::device::turn_sleep(last, now));
            let begun = crate::debug::timing::start();
            // The turn's feeds (action 1, which reads nothing else).
            stream_commands::stream_command(audio, 0, 0, 1)?;
            stream_commands::music_command(&audio.state, 0, 0, 0, 1)?;
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
    close_all::close_all(audio, system)?;
    thread_parts::free_sample_buffer(&audio.state)?;
    a3d::splash(&audio.state)?;
    sound_device::log_text(QUIT_TEXT, true);
    set_priority(0);
    running.set(0);
    Ok(())
}
