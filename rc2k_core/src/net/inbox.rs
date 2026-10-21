//! A player's race messages as the receiving thread keeps them (converted 2026-10-21 from
//! `net::store_received` 0x419677 and `net::send_message` 0x41736D): from the player's
//! block +0x34, a count, 0x40 records of 0x100 bytes (+0 the sequence byte doubled, +1 the
//! length, +2 the bytes), at +0x4004 the oldest sequence still wanted, +0x4005 the newest
//! seen, +0x4008 the time of the last message; kept in sequence order. A race message's
//! sequence byte has its top bit set. Each race step takes the messages over
//! (`net::copy_received` 0x4173A0, converted the same day): the inbox's first 0x400C bytes
//! copied after it for the race, the inbox emptied.

use crate::Invalid;

/// The records' room.
pub const RECORDS: u32 = 0x40;
/// A record's bytes.
pub const RECORD: usize = 0x100;
/// The longest message a record holds.
pub const LONGEST: u32 = 0xFE;
/// The inbox's bytes from the count on that a message stored may reach: past the 0x40th
/// record (see `store`), the longest message's.
pub const INBOX: usize = 0x4000 + 6 + LONGEST as usize;

const OLDEST: usize = 0x4004;
const NEWEST: usize = 0x4005;
const TIME: usize = 0x4008;

/// True if sequence `a` is older than `b` (`cmp a, b; js`).
pub fn older(a: u8, b: u8) -> bool {
    (a.wrapping_sub(b) as i8) < 0
}

fn get(inbox: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(inbox[at..at + 4].try_into().unwrap())
}

fn set(inbox: &mut [u8], at: usize, v: u32) {
    inbox[at..at + 4].copy_from_slice(&v.to_le_bytes());
}

/// A message received (`sequence` as sent, doubled here: `add al, al`) put into the inbox
/// (`INBOX` bytes) at `time`: one older than the oldest still wanted is dropped; the
/// newest seen kept; the first goes first; into a full inbox the records move up one
/// (the oldest gone) and it goes in past the last record, over the oldest and newest
/// sequences, the time and on (`imul edx, 100h` of 0x40: the game's slip, IDA-checked);
/// else it goes in after the last record not newer,
/// those newer moved down one. Moving one leaves the original's count 0: the message then
/// goes in with length 0 and none of its bytes (the game's slip). Refused, before
/// anything: a count above 0x40 or a message longer than 0xFE bytes (the original writes
/// past the block or the record).
pub fn store(inbox: &mut [u8], sequence: u8, time: u32, message: &[u8]) -> Result<(), Invalid> {
    let sequence = sequence.wrapping_add(sequence);
    let count = get(inbox, 0);
    if count > RECORDS {
        return Err(Invalid("store_received: more than 0x40 records"));
    }
    if message.len() as u32 > LONGEST {
        return Err(Invalid("store_received: a message longer than 0xFE bytes"));
    }
    set(inbox, TIME, time);
    if older(sequence, inbox[OLDEST]) {
        return Ok(());
    }
    if !older(sequence, inbox[NEWEST]) {
        inbox[NEWEST] = sequence;
    }
    let mut length = message.len();
    let at = if count == 0 {
        set(inbox, 0, 1);
        0
    } else if count == RECORDS {
        // rep movsd forwards: the records 1 .. 0x3F to 0 .. 0x3E.
        inbox.copy_within(4 + RECORD..4 + RECORDS as usize * RECORD, 4);
        RECORDS as usize * RECORD
    } else {
        let mut at = count as usize * RECORD;
        loop {
            if !older(sequence, inbox[at - 0xFC]) {
                break;
            }
            inbox.copy_within(at - 0xFC..at + 4, at + 4);
            // `mov ecx, 40h; rep movsd` leaves ecx 0.
            length = 0;
            at -= RECORD;
            if at == 0 {
                break;
            }
        }
        set(inbox, 0, count + 1);
        at
    };
    inbox[at + 4] = sequence;
    inbox[at + 5] = length as u8;
    inbox[at + 6..at + 6 + length].copy_from_slice(&message[..length]);
    Ok(())
}

/// The inbox's bytes the race's copy takes (the count, the records, the two sequences and
/// the time); the copy follows them.
pub const COPIED: usize = 0x400C;

/// The inbox's messages handed to the race (`inbox` the inbox and its copy, 2 x `COPIED`
/// bytes from the count on): copied after it, the count made 0, the oldest sequence still
/// wanted made the newest seen (which it gives back).
pub fn hand_over(inbox: &mut [u8]) -> u8 {
    inbox.copy_within(..COPIED, COPIED);
    set(inbox, 0, 0);
    let newest = inbox[NEWEST];
    inbox[OLDEST] = newest;
    newest
}

/// A message's room in the send buffer (0x100 bytes, the sequence byte first).
pub const SEND_ROOM: u32 = 0x100;

/// The sequence byte of the next message sent: one on, the top bit set.
pub fn next_sequence(last: u8) -> u8 {
    last.wrapping_add(1) | 0x80
}

/// A message as sent: the sequence byte and the bytes. Refused: one that does not fit the
/// buffer (the original writes on).
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

    fn record(inbox: &[u8], k: usize) -> (u8, u8, &[u8]) {
        let at = 4 + k * RECORD;
        (
            inbox[at],
            inbox[at + 1],
            &inbox[at + 2..at + 2 + inbox[at + 1] as usize],
        )
    }

    #[test]
    fn storing() {
        let mut inbox = vec![0u8; INBOX];
        store(&mut inbox, 0x81, 7, b"ab").unwrap();
        assert_eq!(get(&inbox, 0), 1);
        assert_eq!(record(&inbox, 0), (2, 2, b"ab".as_slice()));
        assert_eq!((inbox[NEWEST], get(&inbox, TIME)), (2, 7));
        store(&mut inbox, 0x83, 8, b"cd").unwrap();
        assert_eq!(record(&inbox, 1), (6, 2, b"cd".as_slice()));
        // An older one goes in between, with nothing of it (the slip).
        store(&mut inbox, 0x82, 9, b"xy").unwrap();
        assert_eq!(get(&inbox, 0), 3);
        assert_eq!(record(&inbox, 1), (4, 0, b"".as_slice()));
        assert_eq!(record(&inbox, 2), (6, 2, b"cd".as_slice()));
        assert_eq!(inbox[NEWEST], 6);
        // Too old.
        inbox[OLDEST] = 0x10;
        store(&mut inbox, 0x84, 10, b"z").unwrap();
        assert_eq!(get(&inbox, 0), 3);
        assert_eq!(get(&inbox, TIME), 10);
        // Full.
        inbox[OLDEST] = 0;
        set(&mut inbox, 0, RECORDS);
        store(&mut inbox, 0x10, 11, b"q").unwrap();
        assert_eq!(record(&inbox, 0), (4, 0, b"".as_slice()));
        assert_eq!(record(&inbox, 0x40), (0x20, 1, b"q".as_slice()));
        assert_eq!(inbox[OLDEST], 0x20);
        assert!(store(&mut inbox, 0x10, 11, &[0; 0xFF]).is_err());
        set(&mut inbox, 0, RECORDS + 1);
        assert!(store(&mut inbox, 0x10, 11, b"").is_err());
    }

    #[test]
    fn handing_over() {
        let mut both = vec![0xAAu8; 2 * COPIED];
        set(&mut both, 0, 2);
        both[4] = 0x84;
        both[OLDEST] = 2;
        both[NEWEST] = 6;
        assert_eq!(hand_over(&mut both), 6);
        assert_eq!(get(&both, 0), 0);
        assert_eq!((both[OLDEST], both[NEWEST]), (6, 6));
        assert_eq!(get(&both, COPIED), 2);
        assert_eq!(both[COPIED + 4], 0x84);
        assert_eq!(both[COPIED + OLDEST], 2);
        assert_eq!(both[COPIED + COPIED - 1], 0xAA);
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
