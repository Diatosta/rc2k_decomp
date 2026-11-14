//! The network messages as values: `Message::parse` sorts the bytes of a message by their first
//! byte and `encode` gives the same bytes back, for every byte string.

use crate::Invalid;
use crate::net::inbox::{LONGEST, SEND_ROOM};
use crate::net::messages::{ALL_DONE, ALL_READY, CHAT, DONE_ANSWER, RACE_BIT, READY_ANSWER};
use crate::net::players::System;
use crate::net::race::Kind;

/// A race message's bytes after its sequence byte.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Payload {
    /// A kind's tag and its dwords; `trailing` holds the bytes past the last dword.
    Fields {
        kind: Kind,
        values: Vec<u32>,
        trailing: Vec<u8>,
    },
    /// A first dword that is none of the tags.
    Unknown(Vec<u8>),
    /// Fewer than four bytes, or a tag without all its kind's dwords.
    Truncated(Vec<u8>),
}

impl Payload {
    /// A kind's message with all its dwords and nothing after them.
    pub fn of(kind: Kind, values: Vec<u32>) -> Payload {
        Payload::Fields {
            kind,
            values,
            trailing: Vec::new(),
        }
    }

    pub fn parse(bytes: &[u8]) -> Payload {
        let Some((tag, rest)) = bytes.split_first_chunk::<4>() else {
            return Payload::Truncated(bytes.to_vec());
        };
        let Some(kind) = Kind::of(u32::from_le_bytes(*tag)) else {
            return Payload::Unknown(bytes.to_vec());
        };
        let used = 4 * kind.dwords();
        if rest.len() < used {
            return Payload::Truncated(bytes.to_vec());
        }
        let (words, trailing) = rest.split_at(used);
        Payload::Fields {
            kind,
            values: words
                .as_chunks::<4>()
                .0
                .iter()
                .map(|w| u32::from_le_bytes(*w))
                .collect(),
            trailing: trailing.to_vec(),
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        match self {
            Payload::Fields {
                kind,
                values,
                trailing,
            } => {
                let mut out = kind.tag().to_le_bytes().to_vec();
                out.extend(values.iter().flat_map(|v| v.to_le_bytes()));
                out.extend(trailing);
                out
            }
            Payload::Unknown(bytes) | Payload::Truncated(bytes) => bytes.clone(),
        }
    }
}

/// A message as it goes between machines.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    /// A chat line: the text after the byte `"`.
    Chat(Vec<u8>),
    /// The host's call, two bytes: `ready` for "everyone ready", else "everyone done"; the
    /// round.
    Call { ready: bool, round: u8 },
    /// A call's answer, two bytes: of the ready call or the done call; the round.
    Answer { ready: bool, round: u8 },
    /// A race message: the sequence byte (top bit set) and what follows it, at most
    /// `LONGEST` bytes.
    Race { sequence: u8, payload: Payload },
    /// A call, an answer or a race message of the wrong length.
    Malformed(Vec<u8>),
    /// A first byte of none of the kinds, or no bytes at all.
    Unknown(Vec<u8>),
}

impl Message {
    pub fn parse(bytes: &[u8]) -> Message {
        let Some((&first, rest)) = bytes.split_first() else {
            return Message::Unknown(Vec::new());
        };
        let pair = |ready: bool, answer: bool| match *rest {
            [round] if answer => Message::Answer { ready, round },
            [round] => Message::Call { ready, round },
            _ => Message::Malformed(bytes.to_vec()),
        };
        match first {
            CHAT => Message::Chat(rest.to_vec()),
            ALL_READY => pair(true, false),
            ALL_DONE => pair(false, false),
            READY_ANSWER => pair(true, true),
            DONE_ANSWER => pair(false, true),
            b if b & RACE_BIT != 0 => {
                if rest.len() as u32 <= LONGEST {
                    Message::Race {
                        sequence: b,
                        payload: Payload::parse(rest),
                    }
                } else {
                    Message::Malformed(bytes.to_vec())
                }
            }
            _ => Message::Unknown(bytes.to_vec()),
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        match self {
            Message::Chat(text) => [&[CHAT][..], text].concat(),
            Message::Call { ready, round } => {
                vec![if *ready { ALL_READY } else { ALL_DONE }, *round]
            }
            Message::Answer { ready, round } => {
                vec![if *ready { READY_ANSWER } else { DONE_ANSWER }, *round]
            }
            Message::Race { sequence, payload } => [&[*sequence][..], &payload.encode()].concat(),
            Message::Malformed(bytes) | Message::Unknown(bytes) => bytes.clone(),
        }
    }

    /// The bytes to send: refused when they do not fit the send buffer.
    pub fn encode_for_send(&self) -> Result<Vec<u8>, Invalid> {
        let bytes = self.encode();
        if bytes.len() as u32 > SEND_ROOM {
            return Err(Invalid("a message longer than the buffer's 0x100 bytes"));
        }
        Ok(bytes)
    }
}

/// A DirectPlay system message (sender 0): its kind and the two dwords after it, 0 where the
/// bytes end.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SystemMessage {
    pub kind: System,
    /// The first dword: the system message's own kind number.
    pub number: u32,
    /// The player type of a destroyed player or group (`A_PLAYER` for a player).
    pub player_type: u32,
    /// The id of what was destroyed.
    pub id: u32,
}

impl SystemMessage {
    pub fn parse(bytes: &[u8]) -> SystemMessage {
        let dword = |at: usize| {
            let mut word = [0u8; 4];
            for (slot, b) in word.iter_mut().zip(bytes.iter().skip(at)) {
                *slot = *b;
            }
            u32::from_le_bytes(word)
        };
        SystemMessage {
            kind: System::of(dword(0)),
            number: dword(0),
            player_type: dword(4),
            id: dword(8),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Bytes from a fixed generator, so every run sees the same ones.
    fn noise(len: usize, seed: u32) -> Vec<u8> {
        let mut state = seed.wrapping_mul(2_654_435_761).wrapping_add(1);
        (0..len)
            .map(|_| {
                state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                (state >> 24) as u8
            })
            .collect()
    }

    fn dwords(words: &[u32]) -> Vec<u8> {
        words.iter().flat_map(|w| w.to_le_bytes()).collect()
    }

    #[test]
    fn every_byte_string_encodes_back_to_itself() {
        let mut seed = 0;
        for first in 0..=255u8 {
            // Lengths around every boundary: none, the pairs, the longest race message
            // (0xFE bytes after the first), the receive buffer's 0x100.
            for len in [0, 1, 2, 3, 4, 5, 0x10, 0xFD, 0xFE, 0xFF, 0x100] {
                seed += 1;
                let mut bytes = noise(len, seed);
                if len > 0 {
                    bytes[0] = first;
                }
                let m = Message::parse(&bytes);
                assert_eq!(m.encode(), bytes, "{first:#x} {len}");
                assert_eq!(Message::parse(&m.encode()), m);
            }
        }
    }

    #[test]
    fn the_classes_of_the_first_byte() {
        assert_eq!(Message::parse(b"\"hi"), Message::Chat(b"hi".to_vec()));
        assert_eq!(Message::parse(b"\""), Message::Chat(vec![]));
        let call = |ready| Message::Call { ready, round: 7 };
        let answer = |ready| Message::Answer { ready, round: 7 };
        assert_eq!(Message::parse(&[0x40, 7]), call(true));
        assert_eq!(Message::parse(&[0x50, 7]), call(false));
        assert_eq!(Message::parse(&[0x41, 7]), answer(true));
        assert_eq!(Message::parse(&[0x51, 7]), answer(false));
        for bytes in [&[0x40][..], &[0x40, 1, 2], &[0x51], &[0x41, 1, 2]] {
            assert_eq!(Message::parse(bytes), Message::Malformed(bytes.to_vec()));
        }
        assert_eq!(Message::parse(&[0x7F, 1]), Message::Unknown(vec![0x7F, 1]));
        assert_eq!(Message::parse(&[]), Message::Unknown(vec![]));
    }

    #[test]
    fn a_race_message_is_at_most_0xfe_bytes_after_its_sequence_byte() {
        let longest = [vec![0x81], vec![0; 0xFE]].concat();
        assert!(matches!(
            Message::parse(&longest),
            Message::Race { sequence: 0x81, .. }
        ));
        let too_long = [vec![0x81], vec![0; 0xFF]].concat();
        assert_eq!(
            Message::parse(&too_long),
            Message::Malformed(too_long.clone())
        );
        assert_eq!(
            Message::parse(&[0x80]),
            Message::Race {
                sequence: 0x80,
                payload: Payload::Truncated(vec![])
            }
        );
    }

    #[test]
    fn the_race_payloads_by_tag_and_size() {
        for (kind, count) in [(Kind::Inputs, 4), (Kind::Position, 40), (Kind::Extra, 9)] {
            assert_eq!(kind.dwords(), count);
            let values: Vec<u32> = (0..count as u32).map(|n| 0x1000 + n).collect();
            let exact = [dwords(&[kind.tag()]), dwords(&values)].concat();
            assert_eq!(Payload::parse(&exact), Payload::of(kind, values.clone()));
            let longer = [exact.clone(), vec![9, 8]].concat();
            assert_eq!(
                Payload::parse(&longer),
                Payload::Fields {
                    kind,
                    values,
                    trailing: vec![9, 8]
                }
            );
            let short = exact[..exact.len() - 1].to_vec();
            assert_eq!(Payload::parse(&short), Payload::Truncated(short.clone()));
            assert_eq!(Payload::parse(&short).encode(), short);
            assert_eq!(
                Payload::parse(&exact[..4]),
                Payload::Truncated(exact[..4].to_vec())
            );
        }
        assert_eq!(
            Payload::parse(b"JGSNabcd"),
            Payload::Unknown(b"JGSNabcd".to_vec())
        );
        assert_eq!(Payload::parse(b"JGS"), Payload::Truncated(b"JGS".to_vec()));
    }

    #[test]
    fn a_message_built_from_values_parses_back() {
        let sent = [
            Message::Chat(b"hello".to_vec()),
            Message::Call {
                ready: true,
                round: 255,
            },
            Message::Answer {
                ready: false,
                round: 0,
            },
            Message::Race {
                sequence: 0xFF,
                payload: Payload::of(Kind::Extra, (0..9).collect()),
            },
            Message::Race {
                sequence: 0x80,
                payload: Payload::of(Kind::Position, vec![u32::MAX; 40]),
            },
        ];
        for m in sent {
            assert_eq!(Message::parse(&m.encode()), m);
        }
    }

    #[test]
    fn a_system_message_is_three_dwords() {
        let m = SystemMessage::parse(&dwords(&[5, 1, 77, 99]));
        assert_eq!(
            m,
            SystemMessage {
                kind: System::Destroyed,
                number: 5,
                player_type: 1,
                id: 77
            }
        );
        assert_eq!(SystemMessage::parse(&dwords(&[0x101])).kind, System::Host);
        assert_eq!(SystemMessage::parse(&dwords(&[0x101])).id, 0);
        assert_eq!(
            SystemMessage::parse(&[0x31, 0, 0]).kind,
            System::SessionLost
        );
        assert_eq!(SystemMessage::parse(&[]).kind, System::Other);
    }

    #[test]
    fn a_message_for_the_send_buffer_is_at_most_0x100_bytes() {
        let race = |n: usize| Message::Race {
            sequence: 0x81,
            payload: Payload::Unknown(vec![0; n]),
        };
        assert_eq!(race(0xFF).encode_for_send().unwrap().len(), 0x100);
        assert!(race(0x100).encode_for_send().is_err());
    }
}
