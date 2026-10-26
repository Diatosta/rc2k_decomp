//! The queue of commands for the audio thread: 0x400 records of 0x20 bytes at
//! `unk_9566F8`, written at `dword_506A24` (which wraps), counted in `dword_506A1C`; the
//! audio thread's `sub_40A8A4` takes them from `dword_506A20` on and counts them down.
//!
//! A record is the command's number and the sender's registers: eax, ebx, ecx, edx, esi,
//! edi, ebp. The commands, as the audio thread acts on them (read 2026-10-12, the
//! functions under them not read): 1 `sub_40A9D1` with eax as a signed word (a sound);
//! 2 `dword_506C78` = eax; 3 `dword_50695C` = eax and `sub_40AEB9`; 4 `sub_40AF4B` and
//! `dword_50695C` = 0; 5 `sub_40A426` with eax; 6 `sub_40AFD6`; 7 `sub_40AFD9`; 8
//! `sub_40B016` with eax; 9 `sub_40C194` with eax, ecx, edx; 0xB `sub_40C4DD` with eax,
//! ebx, ecx, edx; 0xA ends the thread's taking for this turn.

use crate::framework::{Regs, game_addr, rva};
use std::sync::atomic::{AtomicU32, Ordering};

pub const RECORDS: u32 = 0x400;
pub const RECORD_LEN: usize = 0x20;
pub const NO_AUDIO_RVA: usize = rva(0x4F82B4); // g_noAudio
pub const COUNT_RVA: usize = rva(0x506A1C); // dword_506A1C
pub const WRITE_RVA: usize = rva(0x506A24); // dword_506A24
pub const RECORDS_RVA: usize = rva(0x9566F8); // unk_9566F8

/// Command numbers used by the ported senders.
pub const START_3: u32 = 3;
pub const END_OF_TURN: u32 = 0xA;
pub const COMMAND_B: u32 = 0xB;

/// Where a queue is: the game's own, or a test's.
#[derive(Clone, Copy)]
pub struct Queue {
    pub no_audio: *const u32,
    pub count: *mut u32,
    pub write: *mut u32,
    pub records: *mut u32,
}

impl Queue {
    pub fn game() -> Self {
        Self {
            no_audio: game_addr(NO_AUDIO_RVA) as *const u32,
            count: game_addr(COUNT_RVA) as *mut u32,
            write: game_addr(WRITE_RVA) as *mut u32,
            records: game_addr(RECORDS_RVA) as *mut u32,
        }
    }
}

/// The registers a record holds, in its order.
pub fn sent(r: &Regs) -> [u32; 7] {
    [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp]
}

/// `sound::send_command` (0x409C45). Without audio the count is cleared and nothing is
/// queued; a full queue takes nothing. The count is shared with the audio thread, which
/// counts it down: it is added to atomically here (the original's `inc` is not).
pub unsafe fn send_command(queue: &Queue, command: u32, registers: [u32; 7]) {
    // Converted: `rc2k_core::audio::commands::put` and `record`.
    use rc2k_core::audio::commands::{Put, put, record};
    unsafe {
        match put(
            queue.no_audio.read_unaligned() != 0,
            queue.count.read_unaligned(),
            queue.write.read_unaligned(),
        ) {
            Put::NoAudio => queue.count.write_unaligned(0),
            Put::Full => {}
            Put::At { slot, next } => {
                let at = queue.records.byte_add(slot as usize * RECORD_LEN);
                (at as *mut [u32; 8]).write_unaligned(record(command, registers));
                (*(queue.count as *const AtomicU32)).fetch_add(1, Ordering::SeqCst);
                queue.write.write_unaligned(next);
            }
        }
    }
}

/// `sub_409C01`: true if commands are still waiting. Without audio the count is cleared.
pub unsafe fn pending(queue: &Queue) -> bool {
    unsafe {
        if queue.no_audio.read_unaligned() != 0 {
            queue.count.write_unaligned(0);
        }
        queue.count.read_unaligned() != 0
    }
}
