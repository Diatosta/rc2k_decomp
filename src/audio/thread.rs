//! `audio::thread` (0x409CCE): the audio thread's routine, from its start to `ExitThread`.
//!
//! In:  ebx, ecx, edx, edi, ebp = as the thread starter leaves them (carried into the
//!      commands: `audio::take_commands`)
//! Out: does not return (`ExitThread(0)`)
//!
//! Its slot (`stru_518604`, +0xC running): already running goes straight to the end. Else
//! running; "AUDIOTDB:START"; `CoInitialize`; `g_noAudio` set: the end. Else
//! `audio::take_sample_buffer`, `audio::start` (the carry: the end), `audio::read_splash`,
//! `audio::set_mode` with `g_menuSoundMode`, `threading::set_priority` 0xF; then a turn
//! every 20 ms: sleep what is left of them since the last (`g_audioTurnTick`; at least 1),
//! `audio::stream_command` and `audio::music_command` with edx 1 (ebx the last turn's
//! tick), `audio::take_commands`, `audio::update_voices`, `audio::restore_lost` when
//! `g_soundBufferLost`, until `g_quitting` is 1. The end: `g_noAudio` 1, `audio::close_all`,
//! `audio::free_sample_buffer`, `audio::write_splash`, `CoUninitialize`, "AUDIOTDB:QUIT",
//! priority 0, not running, `ExitThread(0)`.
//! ecx (and edx before the first turn) handed on to `take_commands` and `close_all` are, in
//! the original, what `CoInitialize` and the clock's Win32 calls leave; the Rust version
//! hands on the entry's. Neither is read there (ecx is a volume for action 1, which does not
//! read it; edx reaches `close_all` only when nothing plays yet, where every action stops).

use crate::audio::store::AUDIO;
use crate::errors::Refused;
use crate::framework::rva;
use crate::session::FLAGS;
use crate::store::game_addr;

const START_TEXT: usize = rva(0x4DA5F2); // "AUDIOTDB:START"
const QUIT_TEXT: usize = rva(0x4DA601); // "AUDIOTDB:QUIT"

/// Runs to the `ExitThread`, which does not come back. `slot` is the thread's slot
/// (`threading::thread_slot`), whose +0xC says it runs.
pub unsafe fn audio_thread(slot: u32) -> Result<(), Refused> {
    use crate::audio::*;
    use crate::system::tick_count::tick_count;
    use crate::threading::set_priority::set_priority;
    use windows_sys::Win32::System::Com::{CoInitialize, CoUninitialize};
    unsafe {
        let running = (slot + 0xC) as *mut u8;
        let started = running.read() != 1
            && {
                running.write(1);
                sound_device::log_text(game_addr(START_TEXT) as u32, true);
                CoInitialize(std::ptr::null());
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
        CoUninitialize();
        sound_device::log_text(game_addr(QUIT_TEXT) as u32, true);
        set_priority(0);
        running.write(0);
        windows_sys::Win32::System::Threading::ExitThread(0)
    }
}
