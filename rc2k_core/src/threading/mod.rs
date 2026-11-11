//! The game's threads: their slots and what its waits and handles say
//! (from `threading::find_thread_slot` 0x410CC5, `threading::wait_for_object`
//! 0x411063, `threading::create_event` 0x41103C and `threading::create_mutex` 0x41107F's
//! results; the same day the shutdown's rules from `threading::request_quit` 0x410BF9 and
//! `threading::wait_for_threads` 0x410BA8).
//!
//! The game keeps four thread slots (0x110 bytes, +4 the thread's id); a thread finds its
//! own by its id. Its Win32 calls answer in the flags: a handle by `cmp eax, 1`, a wait
//! by carry (failed) and zero (failed or timed out).

/// The thread slots.
pub const SLOTS: usize = 4;

/// The slot of the thread `id`: the first whose id (`id_of(n)`, asked in order) is it.
pub fn slot_of(id_of: impl Fn(usize) -> u32, id: u32) -> Option<usize> {
    (0..SLOTS).find(|&n| id_of(n) == id)
}

/// `WaitForSingleObject`'s answers that are not "signalled".
pub const WAIT_TIMEOUT: u32 = 0x102;
pub const WAIT_FAILED: u32 = 0xFFFF_FFFF;

/// What a wait ended in, as the flags say it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Waited {
    Signalled,
    TimedOut,
    Failed,
}

impl Waited {
    /// From `WaitForSingleObject`'s answer.
    pub fn of(result: u32) -> Self {
        match result {
            WAIT_FAILED => Waited::Failed,
            WAIT_TIMEOUT => Waited::TimedOut,
            _ => Waited::Signalled,
        }
    }

    /// (carry, zero).
    pub fn flags(self) -> (bool, bool) {
        match self {
            Waited::Signalled => (false, false),
            Waited::TimedOut => (false, true),
            Waited::Failed => (true, true),
        }
    }
}

/// `cmp eax, 1` on a handle: (carry, zero); the carry says there is none.
pub fn handle_flags(handle: u32) -> (bool, bool) {
    (handle < 1, handle == 1)
}

/// The quit switch (`dword_51837C`) once the game is shutting down.
pub const QUITTING: u32 = 1;

/// `threading::request_quit`: the shutdown thread is started only by the first request
/// (the switch, swapped for `QUITTING`, was not that already).
pub fn starts_shutdown(was: u32) -> bool {
    was != QUITTING
}

/// A slot's flag byte (+0xC) of a suspended thread, and what resuming makes it.
pub const SUSPENDED: u8 = 0xFF;
pub const RESUMED: u8 = 1;

/// `threading::resume_slot_thread` (0x410D1D): only a suspended thread is resumed.
pub fn resumes(flag: u8) -> bool {
    flag == SUSPENDED
}

/// The threads the shutdown waits for (input, audio, the network's receiving thread), and
/// for how long at most (all of them, milliseconds).
pub const WAITED: usize = 3;
pub const WAIT_MS: u32 = 3000;

/// The handles as the wait gets them: the slots' handles pushed in order, so the last
/// first; closed in that order too.
pub fn wait_order(handles: [u32; WAITED]) -> [u32; WAITED] {
    let [a, b, c] = handles;
    [c, b, a]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn answers() {
        let ids = [5, 7, 7, 9];
        assert_eq!(slot_of(|n| ids[n], 7), Some(1));
        assert_eq!(slot_of(|n| ids[n], 8), None);
        assert_eq!(Waited::of(0).flags(), (false, false));
        assert_eq!(Waited::of(0x80).flags(), (false, false));
        assert_eq!(Waited::of(0x102).flags(), (false, true));
        assert_eq!(Waited::of(u32::MAX).flags(), (true, true));
        assert_eq!(handle_flags(0), (true, false));
        assert_eq!(handle_flags(1), (false, true));
        assert_eq!(handle_flags(0x44), (false, false));
    }

    #[test]
    fn shutdown() {
        assert!(starts_shutdown(0) && starts_shutdown(2) && !starts_shutdown(1));
        assert!(resumes(0xFF) && !resumes(1));
        assert_eq!(wait_order([1, 2, 3]), [3, 2, 1]);
    }
}
