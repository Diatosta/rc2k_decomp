//! `audio::thread` (0x409CCE): the audio thread's routine, from its start to `ExitThread`.
//!
//! In:  ebx, ecx, edx, edi, ebp = as the thread starter leaves them (carried into the
//!      commands: `audio::take_commands`)
//! Out: does not return (`ExitThread(0)`)
//!
//! Its slot (`stru_518604`, +0xC running): already running goes straight to the end. Else
//! running; "AUDIOTDB:START"; `CoInitialize` (gone in phase 11: no COM objects are
//! left); `g_noAudio` set: the end. Else
//! `audio::take_sample_buffer`, `audio::start` (the carry: the end), `audio::read_splash`,
//! `audio::set_mode` with `g_menuSoundMode`, `threading::set_priority` 0xF; then a turn
//! every 20 ms: sleep what is left of them since the last (`g_audioTurnTick`; at least 1),
//! `audio::stream_command` and `audio::music_command` with edx 1 (ebx the last turn's
//! tick), `audio::take_commands`, `audio::update_voices`, `audio::restore_lost` when
//! `g_soundBufferLost`, until `g_quitting` is 1. The end: `g_noAudio` 1, `audio::close_all`,
//! `audio::free_sample_buffer`, `audio::write_splash`, `CoUninitialize`, "AUDIOTDB:QUIT",
//! priority 0, not running, `ExitThread(0)` (the routine's return since phase 11:
//! `sync::spawn`).
//! ecx (and edx before the first turn) handed on to `take_commands` and `close_all` are, in
//! the original, what `CoInitialize` and the clock's Win32 calls leave; the Rust version
//! hands on the entry's. Neither is read there (ecx is a volume for action 1, which does not
//! read it; edx reaches `close_all` only when nothing plays yet, where every action stops).

use crate::audio::store::AUDIO;
use crate::errors::Refused;
use crate::pointers::GamePtr;
use crate::session::FLAGS;

const START_TEXT: &std::ffi::CStr = c"AUDIOTDB:START"; // "AUDIOTDB:START"
const QUIT_TEXT: &std::ffi::CStr = c"AUDIOTDB:QUIT"; // "AUDIOTDB:QUIT"

/// Runs to the `ExitThread`, which does not come back. `slot` is the thread's slot
/// (`threading::thread_slot`), whose +0xC says it runs.
pub fn audio_thread(slot: u32) -> Result<(), Refused> {
    use crate::audio::*;
    use crate::system::tick_count::tick_count;
    use crate::threading::set_priority::set_priority;
    let running = GamePtr::new(slot + 0xC);
    let started = running.read::<u8>("the audio thread's slot")? != 1
        && {
            running.write(1u8, "the audio thread's slot")?;
            sound_device::log_text(crate::pointers::game_address(START_TEXT.as_ptr()), true);
            AUDIO.no_audio.get() == 0
        }
        && {
            thread_parts::take_sample_buffer()?;
            start::start()?
        };
    if started {
        a3d::splash()?;
        set_mode::set_mode(AUDIO.menu_sound_mode.get())?;
        set_priority(0xF);
        AUDIO.audio_turn_tick.set(tick_count());
        loop {
            let last = AUDIO.audio_turn_tick.get();
            let now = tick_count();
            AUDIO.audio_turn_tick.set(now);
            // Converted: `rc2k_core::audio::device::turn_sleep`.
            crate::system::sleep::sleep(rc2k_core::audio::device::turn_sleep(last, now));
            // The turn's feeds (action 1, which reads nothing else).
            stream_commands::stream_command(0, 0, 1)?;
            stream_commands::music_command(0, 0, 0, 1)?;
            take_commands::take_commands()?;
            update_voices::update_voices()?;
            stray_voices::check();
            if AUDIO.sound_buffer_lost.get() != 0 {
                restore::restore_lost()?;
            }
            if FLAGS.quitting.get() == 1 {
                break;
            }
        }
    }
    AUDIO.no_audio.set(1);
    close_all::close_all()?;
    thread_parts::free_sample_buffer()?;
    a3d::splash()?;
    sound_device::log_text(crate::pointers::game_address(QUIT_TEXT.as_ptr()), true);
    set_priority(0);
    running.write(0u8, "the audio thread's slot")?;
    // The routine's end ends its thread (the original's `ExitThread`).
    Ok(())
}
