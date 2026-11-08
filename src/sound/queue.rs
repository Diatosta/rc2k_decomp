//! The queue of commands for the audio thread: 0x400 records of 0x20 bytes at
//! `unk_9566F8`, written at `dword_506A24` (which wraps), counted in `dword_506A1C`; the
//! audio thread's `audio::take_commands` takes them from `dword_506A20` on and counts them down.
//!
//! A record is the command's number and the sender's registers: eax, ebx, ecx, edx, esi,
//! edi, ebp. The commands, as the audio thread acts on them (read 2026-10-12, the
//! functions under them not read): 1 `audio::start_voice` with eax as a signed word (a sound);
//! 2 `dword_506C78` = eax; 3 `dword_50695C` = eax and `audio::load_sounds`; 4 `audio::free_voices`
//! and
//! `dword_50695C` = 0; 5 `audio::set_mode` with eax; 6 0x40AFD6; 7 `audio::stop_all_voices`; 8
//! `audio::volume_command` with eax; 9 `audio::stream_command` with eax, ecx, edx; 0xB
//! `audio::music_command` with eax, ebx, ecx, edx; 0xA ends the thread's taking for this turn.

use crate::audio::owner::audio;
use std::sync::atomic::Ordering;

/// Command numbers used by the ported senders.
pub const START_3: u32 = 3;
pub const END_OF_TURN: u32 = 0xA;

/// `sound::send_command` (0x409C45). Without audio the count is cleared and nothing is
/// queued; a full queue takes nothing. The count is shared with the audio thread, which
/// counts it down: it is added to atomically here (the original's `inc` is not).
pub fn send_command(command: u32, registers: [u32; 7]) {
    let audio = audio();
    // Converted: `rc2k_core::audio::commands::put` and `record`.
    use rc2k_core::audio::commands::{Put, put, record};
    match put(
        audio.state.no_audio.get() != 0,
        audio.state.command_count.get(),
        audio.state.command_write.get(),
    ) {
        Put::NoAudio => audio.state.command_count.set(0),
        Put::Full => {}
        Put::At { slot, next } => {
            // A slot past the queue ends the program, as an overrun would.
            audio.commands.records[slot as usize].set(record(command, registers));
            audio
                .state
                .command_count
                .atomic()
                .fetch_add(1, Ordering::SeqCst);
            audio.state.command_write.set(next);
        }
    }
}

/// 0x409C01: true if commands are still waiting. Without audio the count is cleared.
pub fn pending() -> bool {
    let audio = audio();
    if audio.state.no_audio.get() != 0 {
        audio.state.command_count.set(0);
    }
    audio.state.command_count.get() != 0
}
