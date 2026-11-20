//! A player's race messages as the receiving thread keeps them: a count, 0x40 records of 0x100
//! bytes (+0 the sequence byte doubled, +1 the length, +2 the bytes), the oldest sequence still
//! wanted, the newest seen and the time of the last message; kept in sequence order. A race
//! message's sequence byte has its top bit set. Each race step takes the messages over: the count
//! and the records copied for the race, the inbox emptied.

use crate::Invalid;
use crate::net::message::{Message, Payload};

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

/// True if sequence `a` is older than `b` (wrapping, as a signed byte).
pub fn older(a: u8, b: u8) -> bool {
    (a.wrapping_sub(b) as i8) < 0
}

/// A message received (`sequence` as sent, doubled here) put into the inbox at `time`: one older
/// than the oldest wanted is dropped; the newest seen is kept. Into a full inbox the oldest
/// record goes to make room; the message goes in after the last record not newer, those newer
/// moved down one. Refused, before anything: a count above 0x40 or a message longer than 0xFE
/// bytes.
pub fn store(inbox: &mut Inbox, sequence: u8, time: u32, message: &[u8]) -> Result<(), Invalid> {
    let sequence = sequence.wrapping_add(sequence);
    let mut count = inbox.count;
    if count > RECORDS {
        return Err(Invalid("store_received: more than 0x40 records"));
    }
    if message.len() as u32 > LONGEST {
        return Err(Invalid("store_received: a message longer than 0xFE bytes"));
    }
    inbox.time = time;
    if older(sequence, inbox.oldest) {
        return Ok(());
    }
    if !older(sequence, inbox.newest) {
        inbox.newest = sequence;
    }
    if count == RECORDS {
        // The records 1 .. 0x3F to 0 .. 0x3E: the oldest makes room.
        inbox.records.copy_within(RECORD.., 0);
        count -= 1;
    }
    let mut at = count as usize * RECORD;
    while at > 0 && older(sequence, inbox.records[at - RECORD]) {
        inbox.records.copy_within(at - RECORD..at, at);
        at -= RECORD;
    }
    inbox.count = count + 1;
    inbox.records[at] = sequence;
    inbox.records[at + 1] = message.len() as u8;
    inbox.records[at + 2..at + 2 + message.len()].copy_from_slice(message);
    // What an earlier, longer message left after this one's end goes.
    inbox.records[at + 2 + message.len()..at + RECORD].fill(0);
    Ok(())
}

/// The message a record holds: its bytes up to the length byte (never past the record's room).
pub fn record_message(record: &[u8]) -> &[u8] {
    let len = (record[1] as usize).min(LONGEST as usize);
    &record[2..2 + len]
}

/// The inbox's messages handed to the race: the count and the records to apply; the inbox's
/// count becomes 0 and the oldest wanted the newest seen. Only the records that arrived are
/// copied (none when nothing did).
pub fn hand_over(inbox: &mut Inbox) -> (u32, Vec<u8>) {
    let used = (inbox.count as usize).min(RECORDS as usize) * RECORD;
    let handed = (inbox.count, inbox.records[..used].to_vec());
    inbox.count = 0;
    inbox.oldest = inbox.newest;
    handed
}

/// A message's room in the send buffer (0x100 bytes, the sequence byte first).
pub(crate) const SEND_ROOM: u32 = 0x100;

/// The sequence byte of the next message sent: one on, the top bit set.
pub fn next_sequence(last: u8) -> u8 {
    last.wrapping_add(1) | 0x80
}

/// A message as sent: the sequence byte and the bytes. Refused: one that does not fit the
/// buffer.
pub fn outgoing(sequence: u8, bytes: &[u8]) -> Result<Vec<u8>, Invalid> {
    Message::Race {
        sequence,
        payload: Payload::parse(bytes),
    }
    .encode_for_send()
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
    fn a_record_holds_its_own_length_and_nothing_after_it() {
        let mut inbox = empty();
        store(&mut inbox, 0x81, 0, &[7; 0xFE]).unwrap();
        // the full-length message is read whole
        assert_eq!(record_message(&inbox.records[..RECORD]), &[7; 0xFE][..]);
        // a shorter one in the same place leaves no stale bytes and reads as short
        inbox.count = 0;
        inbox.oldest = 0;
        store(&mut inbox, 0x81, 1, b"JGSM").unwrap();
        assert_eq!(record_message(&inbox.records[..RECORD]), b"JGSM");
        assert!(inbox.records[6..RECORD].iter().all(|&b| b == 0));
    }

    #[test]
    fn storing() {
        let mut inbox = empty();
        assert_eq!(store(&mut inbox, 0x81, 7, b"ab"), Ok(()));
        assert_eq!(inbox.count, 1);
        assert_eq!(record(&inbox, 0), (2, 2, b"ab".as_slice()));
        assert_eq!((inbox.newest, inbox.time), (2, 7));
        store(&mut inbox, 0x83, 8, b"cd").unwrap();
        assert_eq!(record(&inbox, 1), (6, 2, b"cd".as_slice()));
        // An older one goes in between, with its bytes.
        store(&mut inbox, 0x82, 9, b"xy").unwrap();
        assert_eq!(inbox.count, 3);
        assert_eq!(record(&inbox, 1), (4, 2, b"xy".as_slice()));
        assert_eq!(record(&inbox, 2), (6, 2, b"cd".as_slice()));
        assert_eq!(inbox.newest, 6);
        // Too old.
        inbox.oldest = 0x10;
        store(&mut inbox, 0x84, 10, b"z").unwrap();
        assert_eq!(inbox.count, 3);
        assert_eq!(inbox.time, 10);
        // Full: the oldest goes and the message is the newest.
        inbox.oldest = 0;
        inbox.count = RECORDS;
        inbox.records[RECORD] = 0x30;
        inbox.records[RECORD * 0x3F] = 0x50;
        store(&mut inbox, 0x30, 11, b"q").unwrap();
        assert_eq!(inbox.count, RECORDS);
        assert_eq!(record(&inbox, 0), (0x30, 2, b"xy".as_slice()));
        assert_eq!(record(&inbox, 0x3F), (0x60, 1, b"q".as_slice()));
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
        assert_eq!(records.len(), 2 * RECORD);
        assert_eq!((inbox.count, inbox.oldest, inbox.newest), (0, 6, 6));
    }

    #[test]
    fn nothing_arrived_nothing_is_copied() {
        let mut inbox = empty();
        let (count, records) = hand_over(&mut inbox);
        assert_eq!((count, records.len()), (0, 0));
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
