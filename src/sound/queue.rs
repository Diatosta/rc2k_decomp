//! The queue of commands for the audio thread (`rc2k_core::audio::commands`): the senders
//! queue a `Command`, the audio thread's `audio::take_commands` takes them oldest first. A
//! command counts as waiting until the audio thread has acted on it.

use crate::audio::owner::audio;
use rc2k_core::audio::commands::{Command, Queue};
use std::sync::{Mutex, MutexGuard, PoisonError};

/// The turn's end, as the game sends it after a round of commands.
pub const END_OF_TURN: Command = Command::EndOfTurn;

/// The queue, locked for a moment.
pub fn locked() -> MutexGuard<'static, Queue> {
    queue().lock().unwrap_or_else(PoisonError::into_inner)
}

fn queue() -> &'static Mutex<Queue> {
    &audio().commands
}

/// `sound::send_command` (0x409C45). Without audio the queue is emptied and nothing is
/// queued; a full queue takes nothing.
pub fn send_command(command: Command) {
    send_with_eax(command, None);
}

/// `send_command`, for a sender that left an eax the audio thread does not read: the sound
/// hash records it.
pub fn send_with_eax(command: Command, eax: Option<u32>) {
    let (number, mut registers) = command.record_words();
    if let Some(eax) = eax {
        registers[0] = eax;
    }
    hash_if_asked(number, registers);
    if audio().state.no_audio.is_set() {
        locked().clear();
    } else {
        locked().send(command);
    }
}

/// With `RC2K_SOUND_HASH` (a check, as `RC2K_FAN_HASH` for the drawing): the commands of each
/// turn in the race hashed and logged at its end (`END_OF_TURN`), so that two runs of a
/// replay can be compared turn by turn.
fn hash_if_asked(command: u32, registers: [u32; 7]) {
    use std::hash::{Hash, Hasher};
    use std::sync::OnceLock;
    static ON: OnceLock<bool> = OnceLock::new();
    /// The turn's hash so far, its commands and the turns logged.
    static TURN: Mutex<Option<(std::collections::hash_map::DefaultHasher, u32, u32)>> =
        Mutex::new(None);
    if !*ON.get_or_init(|| std::env::var_os("RC2K_SOUND_HASH").is_some())
        || !crate::race::owner::race().course.numbers.in_race.is_set()
    {
        return;
    }
    let Ok(mut turn) = TURN.lock() else {
        return;
    };
    let (hasher, commands, turns) = turn.get_or_insert_with(Default::default);
    (command, registers).hash(hasher);
    *commands += 1;
    if command == END_OF_TURN.record_words().0 {
        crate::framework::log(&format!(
            "sound hash {turns}: {commands} commands, {:016x}",
            hasher.finish()
        ));
        *hasher = Default::default();
        *commands = 0;
        *turns += 1;
    }
}

/// 0x409C01: true if commands are still waiting. Without audio the queue is emptied.
pub fn pending() -> bool {
    let mut queue = locked();
    if audio().state.no_audio.is_set() {
        queue.clear();
    }
    !queue.is_empty()
}
