//! The queue of commands for the audio thread (`rc2k_core::audio::commands`): the senders
//! queue a `Command`, the audio thread's `audio::take_commands` takes them oldest first. A
//! command counts as waiting until the audio thread has acted on it.
//!
//! The audio thread takes commands once a turn and pauses between turns. A thread that needs
//! the queue taken (`wait_a_while`) does not poll: it tells the audio thread, which cuts its
//! pause short while commands are queued (`pause_between_turns`), and sleeps until the audio
//! thread says a turn is over. Commands sent without such a wait are taken at the audio
//! thread's own pace, a turn's length apart, as ever.

use crate::audio::owner::Audio;
use rc2k_core::audio::commands::{Command, Queue};
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::Duration;

/// The turn's end, as the game sends it after a round of commands.
pub const END_OF_TURN: Command = Command::EndOfTurn;

/// The queue, locked for a moment.
pub fn locked(audio: &Audio) -> MutexGuard<'_, Queue> {
    audio
        .commands
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}

/// `sound::send_command`. Without audio the queue is emptied and nothing is queued; a full queue
/// takes nothing.
pub fn send_command_in(audio: &Audio, command: Command) {
    send_with_eax_in(audio, command, None);
}

/// `send_command`, for a sender whose first word the audio thread does not read: only the
/// sound hash records it (a held race's commands are compared turn by turn through it).
pub fn send_with_eax_in(audio: &Audio, command: Command, first_word: Option<u32>) {
    #[cfg(feature = "remote")]
    crate::audio::mix_hash::set_racing(audio.racing.is_set());
    crate::platform::sound::set_race_sounds(audio.racing.is_set());
    let (number, mut registers) = command.record_words();
    if let Some(word) = first_word {
        registers[0] = word;
    }
    hash_if_asked(audio, number, registers);
    if audio.state.no_audio.is_set() {
        locked(audio).clear();
    } else {
        locked(audio).send(command);
    }
}

/// With `RC2K_SOUND_HASH` (a check, as `RC2K_FAN_HASH` for the drawing): the commands of each
/// turn in the race hashed and logged at its end (`END_OF_TURN`), so that two runs of a
/// replay can be compared turn by turn.
fn hash_if_asked(audio: &Audio, command: u32, registers: [u32; 7]) {
    use std::hash::{Hash, Hasher};
    use std::sync::OnceLock;
    static ON: OnceLock<bool> = OnceLock::new();
    /// The turn's hash so far, its commands and the turns logged.
    static TURN: Mutex<Option<(crate::debug::check_hash::CheckHasher, u32, u32)>> =
        Mutex::new(None);
    if !*ON.get_or_init(|| std::env::var_os("RC2K_SOUND_HASH").is_some()) || !audio.racing.is_set()
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
        crate::logfile::program_log::log(&format!(
            "sound hash {turns}: {commands} commands, {:016x}",
            hasher.finish()
        ));
        *hasher = Default::default();
        *commands = 0;
        *turns += 1;
    }
}

/// Whether the audio thread has taken every command and ended the turn it took the last in.
fn all_taken(audio: &Audio, queue: &Queue) -> bool {
    queue.is_empty() && !audio.turns.in_turn.load(Relaxed)
}

/// Waits for the audio thread to take every command and end that turn, at most `longest`:
/// true when it has. The audio thread is told, so it does not pause while commands are
/// queued. Without audio the queue is emptied and there is nothing to wait for.
pub fn wait_a_while(audio: &Audio, longest: Duration) -> bool {
    let mut queue = locked(audio);
    if audio.state.no_audio.is_set() {
        queue.clear();
        return true;
    }
    audio.turns.waiters.fetch_add(1, Relaxed);
    audio.turns.wanted.notify_all();
    let (queue, _) = audio
        .turns
        .over
        .wait_timeout_while(queue, longest, |queue| !all_taken(audio, queue))
        .unwrap_or_else(PoisonError::into_inner);
    audio.turns.waiters.fetch_sub(1, Relaxed);
    all_taken(audio, &queue)
}

/// The audio thread's pause between two turns: the turn before is over (a waiting thread is
/// told), then `longest` passes, or less when a thread waits for commands that are queued;
/// the next turn has begun when this returns.
pub fn pause_between_turns(audio: &Audio, longest: Duration) {
    let queue = locked(audio);
    audio.turns.in_turn.store(false, Relaxed);
    audio.turns.over.notify_all();
    let asked_for = |queue: &Queue| audio.turns.waiters.load(Relaxed) != 0 && !queue.is_empty();
    let (queue, _) = audio
        .turns
        .wanted
        .wait_timeout_while(queue, longest, |queue| !asked_for(queue))
        .unwrap_or_else(PoisonError::into_inner);
    audio.turns.in_turn.store(true, Relaxed);
    drop(queue);
}

#[cfg(test)]
mod tests {
    use super::*;
    use rc2k_core::audio::commands::Command;
    use std::sync::atomic::{AtomicBool, AtomicU32};
    use std::time::Instant;

    /// A stand-in for the audio thread's loop: a pause, then the commands up to a turn's end.
    fn turns(audio: &Audio, pause: Duration, stop: &AtomicBool, taken: &AtomicU32) {
        while !stop.load(Relaxed) {
            pause_between_turns(audio, pause);
            loop {
                let Some(command) = locked(audio).front() else {
                    break;
                };
                // Acting on it takes a while, during which it still counts as waiting.
                std::thread::sleep(Duration::from_millis(2));
                taken.fetch_add(1, Relaxed);
                locked(audio).pop();
                if command == END_OF_TURN {
                    break;
                }
            }
        }
        // As the audio thread at its end.
        audio.state.no_audio.set(1);
    }

    #[test]
    fn a_wait_is_over_when_the_turn_that_took_the_commands_is() {
        let audio = crate::test_support::own::audio();
        let (stop, taken) = (AtomicBool::new(false), AtomicU32::new(0));
        std::thread::scope(|scope| {
            // The pause is far longer than the test may take: only the wait can cut it short.
            scope.spawn(|| turns(audio, Duration::from_secs(20), &stop, &taken));
            for round in 1..=3 {
                send_command_in(audio, Command::Volume(round));
                send_command_in(audio, Command::StopAll);
                send_command_in(audio, END_OF_TURN);
                let begun = Instant::now();
                while !wait_a_while(audio, Duration::from_millis(5)) {
                    assert!(begun.elapsed() < Duration::from_secs(10), "the wait hangs");
                }
                assert!(locked(audio).is_empty());
                assert!(!audio.turns.in_turn.load(Relaxed));
                assert_eq!(taken.load(Relaxed), 3 * round);
            }
            // Ends the stand-in: a last wait wakes it from its pause.
            stop.store(true, Relaxed);
            send_command_in(audio, END_OF_TURN);
            while !wait_a_while(audio, Duration::from_millis(5)) {}
        });
    }

    #[test]
    fn commands_alone_do_not_cut_the_pause_short() {
        let audio = crate::test_support::own::audio();
        send_command_in(audio, Command::StopAll);
        let begun = Instant::now();
        pause_between_turns(audio, Duration::from_millis(30));
        assert!(begun.elapsed() >= Duration::from_millis(30));
        assert!(audio.turns.in_turn.load(Relaxed));
        // Nothing takes the queue: a wait gives up after its stretch.
        assert!(!wait_a_while(audio, Duration::from_millis(5)));
    }

    #[test]
    fn without_audio_there_is_nothing_to_wait_for() {
        let audio = crate::test_support::own::audio();
        send_command_in(audio, Command::StopAll);
        audio.state.no_audio.set(1);
        assert!(wait_a_while(audio, Duration::from_secs(20)));
        assert!(locked(audio).is_empty());
    }
}
