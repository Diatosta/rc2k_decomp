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
//! The thunk is `ungated` (the thread is in it for the whole game). A refusal from a callee
//! hands the call to the original from its start: it finds the slot running and goes to
//! the end, so the sound stops (hook.log's "refused" line says why).
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`). Plain since 2026-10-23 (phase 3): the registers above are the original's,
//! for reference; the feeds and `close_all` get only what they read.

use crate::audio::voices::{read, write};
use crate::errors::Refused;
use crate::framework::{game_addr, rva};

const TURN_TICK_RVA: usize = rva(0x4F82B8); // g_audioTurnTick
const START_TEXT: usize = rva(0x4DA5F2); // "AUDIOTDB:START"
const QUIT_TEXT: usize = rva(0x4DA601); // "AUDIOTDB:QUIT"

#[derive(Clone, Copy)]
pub struct ThreadVars {
    pub slot: u32,
    pub no_audio: u32,
    pub menu_mode: u32,
    pub turn_tick: u32,
    pub lost: u32,
    pub quitting: u32,
}

impl ThreadVars {
    pub fn game() -> Self {
        use crate::audio::voices::*;
        let a = |r| game_addr(r) as u32;
        Self {
            slot: crate::threading::thread_slot::slot(crate::threading::thread_slot::AUDIO) as u32,
            no_audio: a(crate::sound::queue::NO_AUDIO_RVA),
            menu_mode: a(crate::audio::choose_modes::MENU_MODE_RVA),
            turn_tick: a(TURN_TICK_RVA),
            lost: a(BUFFER_LOST_RVA),
            quitting: a(rva(0x51837C)),
        }
    }
}

/// Runs to the `ExitThread`, which does not come back.
pub unsafe fn audio_thread(v: &ThreadVars) -> Result<(), Refused> {
    use crate::audio::*;
    use crate::system::tick_count::tick_count;
    use crate::threading::set_priority::set_priority;
    use windows_sys::Win32::System::Com::{CoInitialize, CoUninitialize};
    let splash = || unsafe { a3d::splash(game_addr(a3d::SPLASH_KEPT_RVA) as u32) };
    unsafe {
        let running = (v.slot + 0xC) as *mut u8;
        let started = running.read() != 1
            && {
                running.write(1);
                sound_device::log_text(game_addr(START_TEXT) as u32, true);
                CoInitialize(std::ptr::null());
                read(v.no_audio) == 0
            }
            && {
                thread_parts::take_sample_buffer(&thread_parts::SampleBufferVars::game())?;
                start::start(&start::StartVars::game())?
            };
        if started {
            splash()?;
            set_mode::set_mode(read(v.menu_mode), &set_mode::ModeVars::game())?;
            set_priority(0xF);
            write(v.turn_tick, tick_count());
            loop {
                let last = read(v.turn_tick);
                let now = tick_count();
                write(v.turn_tick, now);
                // Converted: `rc2k_core::audio::device::turn_sleep`.
                crate::system::sleep::sleep(rc2k_core::audio::device::turn_sleep(last, now));
                // The turn's feeds (action 1, which reads nothing else).
                stream_commands::stream_command(
                    0,
                    0,
                    1,
                    game_addr(stream::SOUND_STREAM_RVA) as u32,
                )?;
                stream_commands::music_command(0, 0, 0, 1, &stream_commands::MusicVars::game())?;
                take_commands::take_commands(&take_commands::Vars::game())?;
                update_voices::update_voices(&update_voices::Vars::game())?;
                stray_voices::check();
                if read(v.lost) != 0 {
                    restore::restore_lost(&restore::RestoreVars::game())?;
                }
                if read(v.quitting) == 1 {
                    break;
                }
            }
        }
        write(v.no_audio, 1);
        close_all::close_all()?;
        thread_parts::free_sample_buffer(&thread_parts::SampleBufferVars::game())?;
        splash()?;
        CoUninitialize();
        sound_device::log_text(game_addr(QUIT_TEXT) as u32, true);
        set_priority(0);
        running.write(0);
        windows_sys::Win32::System::Threading::ExitThread(0)
    }
}
