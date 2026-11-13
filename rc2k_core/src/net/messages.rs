//! The messages the receiving thread takes, by their first byte: a chat line, the host's two calls
//! to wait for everyone (with the round) and the other machines' answers to them, and
//! the race's messages (top bit set) that go into the sender's inbox.

use crate::net::inbox::LONGEST;
use crate::net::record::is_host;

/// A chat line: the text follows the byte.
pub const CHAT: u8 = 0x22;
/// The host's call: everyone ready for the race (answered `READY_ANSWER`).
pub const ALL_READY: u8 = 0x40;
pub const READY_ANSWER: u8 = 0x41;
/// The host's call: everyone done with the race (answered `DONE_ANSWER`).
pub const ALL_DONE: u8 = 0x50;
pub const DONE_ANSWER: u8 = 0x51;
/// A race message's bit.
pub const RACE_BIT: u8 = 0x80;

/// A message's kind, by its first byte.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Chat,
    /// `ALL_READY` / `ALL_DONE`, answered with `reply`; `ready` for the first.
    Call {
        reply: u8,
        ready: bool,
    },
    /// `READY_ANSWER` / `DONE_ANSWER`.
    Answer,
    Race,
    Other,
}

impl Kind {
    pub fn of(first: u8) -> Kind {
        match first {
            CHAT => Kind::Chat,
            ALL_READY => Kind::Call {
                reply: READY_ANSWER,
                ready: true,
            },
            ALL_DONE => Kind::Call {
                reply: DONE_ANSWER,
                ready: false,
            },
            READY_ANSWER | DONE_ANSWER => Kind::Answer,
            b if b & RACE_BIT != 0 => Kind::Race,
            _ => Kind::Other,
        }
    }
}

/// Whether a call or an answer (`length` bytes) is of this round: two bytes, the second
/// the round (`round_and_sync`, asked only then: the second byte, the round now).
pub fn this_round(length: u32, round_and_sync: impl FnOnce() -> (u8, u8)) -> bool {
    length == 2 && {
        let (round, sync) = round_and_sync();
        round == sync
    }
}

/// Whether a call is answered: of this round, on a machine not the host (`own_state` its
/// record's state).
pub fn answers_call(this_round: bool, own_state: u8) -> bool {
    this_round && !is_host(own_state)
}

/// Whether an answer is noted: of this round, on the host.
pub fn notes_answer(this_round: bool, own_state: u8) -> bool {
    this_round && is_host(own_state)
}

/// A call's answer as sent: the reply byte, then the round.
pub fn answer(reply: u8, sync: u8) -> u16 {
    (sync as u16) << 8 | reply as u16
}

/// Whether a race message's bytes after its first (`length`, unsigned: none wraps) fit the
/// inbox's record.
pub fn race_fits(length: u32) -> bool {
    length <= LONGEST
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds() {
        assert_eq!(Kind::of(0x22), Kind::Chat);
        assert_eq!(
            Kind::of(0x50),
            Kind::Call {
                reply: 0x51,
                ready: false
            }
        );
        assert_eq!(Kind::of(0x41), Kind::Answer);
        assert_eq!(Kind::of(0x81), Kind::Race);
        assert_eq!(Kind::of(0x7F), Kind::Other);
        assert!(this_round(2, || (5, 5)));
        assert!(!this_round(3, || unreachable!()));
        assert!(!this_round(2, || (4, 5)));
        assert!(answers_call(true, 0) && !answers_call(true, 0x10));
        assert!(notes_answer(true, 0x10) && !notes_answer(false, 0x10));
        assert_eq!(answer(0x41, 7), 0x0741);
        assert!(race_fits(0xFE) && !race_fits(0xFF) && !race_fits(u32::MAX));
    }
}
