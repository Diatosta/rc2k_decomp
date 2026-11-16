//! The game's threads: what a wait says, and the shutdown's rules.

/// `platform::sync`'s wait answers that are not "signalled".
pub const WAIT_TIMEOUT: u32 = 0x102;
pub const WAIT_FAILED: u32 = 0xFFFF_FFFF;

/// What a wait ended in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Waited {
    Signalled,
    TimedOut,
    Failed,
}

impl Waited {
    /// From the wait's answer.
    pub fn of(result: u32) -> Self {
        match result {
            WAIT_FAILED => Waited::Failed,
            WAIT_TIMEOUT => Waited::TimedOut,
            _ => Waited::Signalled,
        }
    }
}

/// The quit switch once the game is shutting down.
pub const QUITTING: u32 = 1;

/// `threading::request_quit`: the shutdown thread starts only on the first request.
pub fn starts_shutdown(was: u32) -> bool {
    was != QUITTING
}

/// The threads the shutdown waits for (input, audio, the network's receiving thread), and
/// for how long at most (all of them, milliseconds).
pub const WAITED: usize = 3;
pub const WAIT_MS: u32 = 3000;

/// The handles as the wait gets them: the slots' handles pushed in order, so the last first;
/// closed in that order too.
pub fn wait_order(handles: [u32; WAITED]) -> [u32; WAITED] {
    let [a, b, c] = handles;
    [c, b, a]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn answers() {
        assert_eq!(Waited::of(0), Waited::Signalled);
        assert_eq!(Waited::of(0x80), Waited::Signalled);
        assert_eq!(Waited::of(0x102), Waited::TimedOut);
        assert_eq!(Waited::of(u32::MAX), Waited::Failed);
    }

    #[test]
    fn shutdown() {
        assert!(starts_shutdown(0) && starts_shutdown(2) && !starts_shutdown(1));
        assert_eq!(wait_order([1, 2, 3]), [3, 2, 1]);
    }
}
