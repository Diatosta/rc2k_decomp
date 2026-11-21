//! The audio thread's routine, from its start to the thread's end. Already running goes straight
//! to the end. Else: "AUDIOTDB:START"; unless audio is off, the sample buffer taken, `audio::start`
//! (failing: the end), the menu sound mode set, priority raised; then a turn every
//! 20 ms: a pause of what is left of them (cut short when the game's thread waits for the
//! commands it queued), the streams fed, commands taken, voices updated, until the game quits. The end: audio off, everything closed,
//! "AUDIOTDB:QUIT".

use crate::errors::Refused;
use rc2k_core::audio::commands::StreamAction;

const START_TEXT: &std::ffi::CStr = c"AUDIOTDB:START";
const QUIT_TEXT: &std::ffi::CStr = c"AUDIOTDB:QUIT";

/// Everything the audio thread reads from outside its own state, built once where the thread is
/// started (`threading::thread_slot`).
///
/// What crosses threads: `audio` (atomic cells, `Shared` records and the command queue's mutex;
/// the game's thread writes the modes, volumes, sample keys and `environment`, the audio thread
/// the voices, streams and device facts), `sets` and `menu_sounds` (`SoundForThreads`' lists, atomic cells the game's
/// thread fills before it queues the load command that makes the audio thread read them) and
/// `system` (only the quitting flag, an atomic cell, is read). No `GameThread` state is
/// reachable from here, and the thread reads none of the sound options.
#[derive(Clone, Copy)]
pub struct AudioInputs<'a> {
    pub audio: &'static crate::audio::owner::Audio,
    pub system: &'static crate::system::owner::System,
    /// The sets of sounds a race loads (the lists of the load command).
    pub sets: &'a crate::sound::sets_store::SoundSets,
    /// The settings page's menu sounds (the menu list of the load command).
    pub menu_sounds: &'a crate::sound::sets_store::MenuSounds,
}

/// Runs to the thread's end. Its slot (`threading::thread_slot`) says while it runs.
pub fn audio_thread(inputs: AudioInputs<'_>) -> Result<(), Refused> {
    let AudioInputs {
        audio: owner,
        system,
        sets,
        menu_sounds,
    } = inputs;
    use crate::audio::*;
    use crate::clock::tick_count::tick_count;
    use crate::platform::sync::set_priority;
    use std::sync::atomic::Ordering;
    // The thread's own variables (`audio::own`): made here, so that no other thread has them.
    let audio = &crate::audio::own::AudioThread::new(owner);
    let running = &crate::threading::thread_slot::SlotThread::Audio
        .slot()
        .running;
    let started = !running.swap(true, Ordering::SeqCst)
        && {
            sound_device::log_text(START_TEXT, true);
            audio.state.no_audio.get() == 0
        }
        && {
            thread_parts::take_sample_buffer(&audio.state);
            start::start(audio, system)?
        };
    if started {
        set_mode::set_mode(audio, system, audio.state.menu_sound_mode.get())?;
        set_priority(0xF);
        audio
            .state
            .audio_turn_tick
            .set(tick_count(&system.session.picture_clock));
        loop {
            let last = audio.state.audio_turn_tick.get();
            let now = tick_count(&system.session.picture_clock);
            audio.state.audio_turn_tick.set(now);
            // What is left of the turn's length, or less when the game's thread waits for the
            // commands it queued.
            crate::sound::queue::pause_between_turns(
                owner,
                crate::clock::sleep::duration(rc2k_core::audio::device::turn_sleep(last, now)),
            );
            let begun = crate::debug::timing::start();
            // The turn's feeds (action 1, which reads nothing else).
            stream_commands::stream_command(audio, 0, 0, StreamAction::Turn)?;
            stream_commands::music_command(&audio.state, 0, 0, 0, StreamAction::Turn)?;
            take_commands::take_commands(audio, system, sets, menu_sounds)?;
            update_voices::update_voices(audio)?;
            stray_voices::check(&audio.state);
            crate::remote::audio_turn_done();
            if begun.is_some() {
                crate::debug::timing::note_voices(stray_voices::on_lists(&audio.state));
            }
            crate::debug::timing::stop(crate::debug::timing::Stage::AudioTurn, begun);
            if system.session.flags.quitting.is_set() {
                break;
            }
        }
    }
    audio.state.no_audio.set(1);
    close_all::close_all(audio, system)?;
    thread_parts::free_sample_buffer(&audio.state)?;
    sound_device::log_text(QUIT_TEXT, true);
    set_priority(0);
    running.store(false, Ordering::SeqCst);
    Ok(())
}
