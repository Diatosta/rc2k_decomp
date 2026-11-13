//! A player's race messages as the receiving thread keeps them: a count, 0x40 records of 0x100
//! bytes (+0 the sequence byte doubled, +1 the length, +2 the bytes), the oldest sequence still
//! wanted, the newest seen and the time of the last message; kept in sequence order. A race
//! message's sequence byte has its top bit set. Each race step takes the messages over: the count
//! and the records copied for the race, the inbox emptied.

use crate::Invalid;

/// The records' room.
pub const RECORDS: u32 = 0x40;
/// A record's bytes.
pub const RECORD: usize = 0x100;
/// The longest message a record holds.
pub const LONGEST: u32 = 0xFE;

/// The records' bytes.
pub const RECORDS_LEN: usize = RECORDS as usize * RECORD;

/// A player's inbox.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inbox {
    /// The records held.
    pub count: u32,
    /// The records, `RECORDS_LEN` bytes.
    pub records: Vec<u8>,
    /// The oldest sequence still wanted.
    pub oldest: u8,
    /// The newest sequence seen.
    pub newest: u8,
    /// The time of the last message.
    pub time: u32,
}

/// What a message put into a full inbox writes past the last record (the game's slip, kept): its
/// sequence byte over the oldest sequence wanted, its length over the newest seen, its bytes over
/// what follows (the time, the copy's count and records).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spill {
    pub sequence: u8,
    pub length: u8,
    pub bytes: Vec<u8>,
}

/// True if sequence `a` is older than `b` (wrapping, as a signed byte).
pub fn older(a: u8, b: u8) -> bool {
    (a.wrapping_sub(b) as i8) < 0
}

/// A message received (`sequence` as sent, doubled here) put into the inbox at `time`: one older
/// than the oldest wanted is dropped; the newest seen is kept. Into a full inbox the records move
/// up one (the oldest gone) and it goes in past the last record (the `Spill` given back, to write
/// after the time is set); else it goes in after the last record not newer, those newer moved down
/// one. Moving one leaves the count 0, so the message goes in with length 0 and none of its bytes
/// (the game's slip, kept). Refused, before anything: a count above 0x40 or a message longer than
/// 0xFE bytes.
pub fn store(
    inbox: &mut Inbox,
    sequence: u8,
    time: u32,
    message: &[u8],
) -> Result<Option<Spill>, Invalid> {
    let sequence = sequence.wrapping_add(sequence);
    let count = inbox.count;
    if count > RECORDS {
        return Err(Invalid("store_received: more than 0x40 records"));
    }
    if message.len() as u32 > LONGEST {
        return Err(Invalid("store_received: a message longer than 0xFE bytes"));
    }
    inbox.time = time;
    if older(sequence, inbox.oldest) {
        return Ok(None);
    }
    if !older(sequence, inbox.newest) {
        inbox.newest = sequence;
    }
    let mut length = message.len();
    let at = if count == 0 {
        inbox.count = 1;
        0
    } else if count == RECORDS {
        // The records 1 .. 0x3F to 0 .. 0x3E.
        inbox.records.copy_within(RECORD.., 0);
        return Ok(Some(Spill {
            sequence,
            length: length as u8,
            bytes: message.to_vec(),
        }));
    } else {
        let mut at = count as usize * RECORD;
        loop {
            if !older(sequence, inbox.records[at - RECORD]) {
                break;
            }
            inbox.records.copy_within(at - RECORD..at, at);
            length = 0;
            at -= RECORD;
            if at == 0 {
                break;
            }
        }
        inbox.count = count + 1;
        at
    };
    inbox.records[at] = sequence;
    inbox.records[at + 1] = length as u8;
    inbox.records[at + 2..at + 2 + length].copy_from_slice(&message[..length]);
    Ok(None)
}

/// The inbox's messages handed to the race: the count and the records to apply; the inbox's
/// count becomes 0 and the oldest wanted the newest seen.
pub fn hand_over(inbox: &mut Inbox) -> (u32, Vec<u8>) {
    let handed = (inbox.count, inbox.records.clone());
    inbox.count = 0;
    inbox.oldest = inbox.newest;
    handed
}

/// A message's room in the send buffer (0x100 bytes, the sequence byte first).
pub const SEND_ROOM: u32 = 0x100;

/// The sequence byte of the next message sent: one on, the top bit set.
pub fn next_sequence(last: u8) -> u8 {
    last.wrapping_add(1) | 0x80
}

/// A message as sent: the sequence byte and the bytes. Refused: one that does not fit the
/// buffer.
pub fn outgoing(sequence: u8, bytes: &[u8]) -> Result<Vec<u8>, Invalid> {
    if bytes.len() as u32 >= SEND_ROOM {
        return Err(Invalid("a message longer than the buffer's 0x100 bytes"));
    }
    let mut out = vec![sequence];
    out.extend(bytes);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty() -> Inbox {
        Inbox {
            count: 0,
            records: vec![0; RECORDS_LEN],
            oldest: 0,
            newest: 0,
            time: 0,
        }
    }

    fn record(inbox: &Inbox, k: usize) -> (u8, u8, &[u8]) {
        let at = k * RECORD;
        let r = &inbox.records;
        (r[at], r[at + 1], &r[at + 2..at + 2 + r[at + 1] as usize])
    }

    #[test]
    fn storing() {
        let mut inbox = empty();
        assert_eq!(store(&mut inbox, 0x81, 7, b"ab"), Ok(None));
        assert_eq!(inbox.count, 1);
        assert_eq!(record(&inbox, 0), (2, 2, b"ab".as_slice()));
        assert_eq!((inbox.newest, inbox.time), (2, 7));
        store(&mut inbox, 0x83, 8, b"cd").unwrap();
        assert_eq!(record(&inbox, 1), (6, 2, b"cd".as_slice()));
        // An older one goes in between, with nothing of it (the slip).
        store(&mut inbox, 0x82, 9, b"xy").unwrap();
        assert_eq!(inbox.count, 3);
        assert_eq!(record(&inbox, 1), (4, 0, b"".as_slice()));
        assert_eq!(record(&inbox, 2), (6, 2, b"cd".as_slice()));
        assert_eq!(inbox.newest, 6);
        // Too old.
        inbox.oldest = 0x10;
        store(&mut inbox, 0x84, 10, b"z").unwrap();
        assert_eq!(inbox.count, 3);
        assert_eq!(inbox.time, 10);
        // Full: the records move up and the message spills past them.
        inbox.oldest = 0;
        inbox.count = RECORDS;
        let spill = store(&mut inbox, 0x10, 11, b"q").unwrap();
        assert_eq!(record(&inbox, 0), (4, 0, b"".as_slice()));
        assert_eq!(
            spill,
            Some(Spill {
                sequence: 0x20,
                length: 1,
                bytes: b"q".to_vec()
            })
        );
        assert!(store(&mut inbox, 0x10, 11, &[0; 0xFF]).is_err());
        inbox.count = RECORDS + 1;
        assert!(store(&mut inbox, 0x10, 11, b"").is_err());
    }

    #[test]
    fn handing_over() {
        let mut inbox = empty();
        inbox.count = 2;
        inbox.records[0] = 0x84;
        inbox.oldest = 2;
        inbox.newest = 6;
        let (count, records) = hand_over(&mut inbox);
        assert_eq!((count, records[0]), (2, 0x84));
        assert_eq!((inbox.count, inbox.oldest, inbox.newest), (0, 6, 6));
    }

    #[test]
    fn sending() {
        assert_eq!(next_sequence(0x7F), 0x80);
        assert_eq!(next_sequence(0xFF), 0x80);
        assert_eq!(next_sequence(3), 0x84);
        assert_eq!(outgoing(0x81, b"ab"), Ok(vec![0x81, b'a', b'b']));
        assert!(outgoing(0x81, &[0; 0x100]).is_err());
    }
}
