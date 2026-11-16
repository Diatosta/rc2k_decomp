//! Frozen bytes of the network messages the core builds and reads: each case builds a message
//! from fixed inputs, checks the exact bytes, and reads them back to the same values.

use crate::net::inbox::{self, Inbox, RECORD, RECORDS_LEN};
use crate::net::messages::{self, Kind};
use crate::net::race;
use crate::net::record;

/// Guards the first-byte classification of all 256 values: one chat, two calls, two answers, the
/// 128 race values and 123 others.
#[test]
fn the_first_byte_classes_are_frozen() {
    let mut counts = [0u32; 5];
    for first in 0..=255u8 {
        let class = match Kind::of(first) {
            Kind::Chat => 0,
            Kind::Call { .. } => 1,
            Kind::Answer => 2,
            Kind::Race => 3,
            Kind::Other => 4,
        };
        counts[class] += 1;
        // The race messages are exactly those with the top bit.
        assert_eq!(class == 3, first & 0x80 != 0, "{first:#x}");
    }
    assert_eq!(counts, [1, 2, 2, 128, 123]);
    assert_eq!(
        Kind::of(0x40),
        Kind::Call {
            reply: 0x41,
            ready: true
        }
    );
    assert_eq!(
        Kind::of(0x50),
        Kind::Call {
            reply: 0x51,
            ready: false
        }
    );
}

/// Guards the bytes of the host's calls and the answers: two bytes, the kind and the round;
/// the answer's word as it goes on the wire (little-endian: the reply, then the round).
#[test]
fn a_call_and_its_answer_are_two_bytes() {
    let ready = [messages::ALL_READY, 7];
    let done = [messages::ALL_DONE, 7];
    assert_eq!(ready, [0x40, 7]);
    assert_eq!(done, [0x50, 7]);
    for call in [ready, done] {
        let Kind::Call { reply, .. } = Kind::of(call[0]) else {
            panic!("a call");
        };
        assert!(messages::this_round(2, || (call[1], 7)));
        assert_eq!(
            messages::answer(reply, call[1]).to_le_bytes(),
            [call[0] + 1, 7]
        );
        assert_eq!(Kind::of(call[0] + 1), Kind::Answer);
    }
    // A call of another round, or of another length, is not of this round.
    assert!(!messages::this_round(2, || (8, 7)));
    assert!(!messages::this_round(1, || (7, 7)));
}

/// Guards a chat message's first byte and the longest race message: the chat byte is the
/// quotation mark the sender uses and the receiver classifies it as chat.
#[test]
fn the_chat_byte_is_the_quotation_mark() {
    assert_eq!(messages::CHAT, b'"');
    assert_eq!(Kind::of(b'"'), Kind::Chat);
}

/// Guards a race message's framing from sender to receiver: the sequence byte (one more than
/// the last, top bit set), the bytes after it, and what the receiver's inbox holds: the
/// sequence doubled (top bit gone), the length and the bytes.
#[test]
fn a_race_message_goes_from_the_sender_to_the_inbox() {
    let sequence = inbox::next_sequence(5);
    assert_eq!(sequence, 0x86);
    let payload = [0x4A, 0x47, 0x53, 0x4D, 1, 0, 0, 0];
    let sent = inbox::outgoing(sequence, &payload).unwrap();
    assert_eq!(sent, [0x86, 0x4A, 0x47, 0x53, 0x4D, 1, 0, 0, 0]);
    // The receiver: the first byte tells a race message and the rest is the payload.
    assert_eq!(Kind::of(sent[0]), Kind::Race);
    assert!(messages::race_fits(sent.len() as u32 - 1));
    let mut inbox = Inbox {
        count: 0,
        records: vec![0; RECORDS_LEN],
        oldest: 0,
        newest: 0,
        time: 0,
    };
    inbox::store(&mut inbox, sent[0], 99, &sent[1..]).unwrap();
    assert_eq!(inbox.count, 1);
    assert_eq!(
        inbox.records[..RECORD][..10],
        [0x0C, 8, 0x4A, 0x47, 0x53, 0x4D, 1, 0, 0, 0]
    );
    assert_eq!((inbox.oldest, inbox.newest, inbox.time), (0, 0x0C, 99));
    // The longest message that fits both sides, and the first that does not.
    assert!(inbox::outgoing(sequence, &[0; 0xFF]).is_ok());
    assert!(inbox::outgoing(sequence, &[0; 0x100]).is_err());
    assert!(messages::race_fits(0xFE) && !messages::race_fits(0xFF));
}

/// Guards the sequence byte's run: after 0xFF it wraps to 0x80 and every one has its top bit.
#[test]
fn the_sequence_bytes_run_0x80_to_0xff() {
    let mut last = 0u8;
    let mut seen = Vec::new();
    for _ in 0..0x100 {
        last = inbox::next_sequence(last);
        assert!(last & 0x80 != 0);
        seen.push(last);
    }
    assert_eq!(&seen[..3], &[0x81, 0x82, 0x83]);
    assert_eq!(&seen[0x7E..0x81], &[0xFF, 0x80, 0x81]);
}

/// Guards the three race payloads' tags (the ASCII of 'JGSM', 'PGSM' and 'XGSM' as the first
/// dword, little-endian), and that each tag reads back to its kind.
#[test]
fn the_race_tags_are_their_letters() {
    assert_eq!(race::Kind::Inputs.tag().to_le_bytes(), *b"JGSM");
    assert_eq!(race::Kind::Position.tag().to_le_bytes(), *b"PGSM");
    assert_eq!(race::Kind::Extra.tag().to_le_bytes(), *b"XGSM");
    for kind in race::SENT {
        assert_eq!(race::Kind::of(kind.tag()), Some(kind));
    }
    assert_eq!(race::Kind::of(u32::from_le_bytes(*b"JGSN")), None);
}

/// Guards the cadence: over 0x300 own steps the inputs go out 192 times (every fourth step),
/// the place 24 times (at count & 0x1F == 2) and the extra dwords 3 times (count & 0xFF == 1),
/// and the first steps that send each.
#[test]
fn the_cadence_of_the_three_messages_is_frozen() {
    let mut sent = [Vec::new(), Vec::new(), Vec::new()];
    for count in 0..0x300u32 {
        for (n, kind) in race::SENT.into_iter().enumerate() {
            if kind.due(count).0 {
                sent[n].push(count);
            }
        }
    }
    assert_eq!([sent[0].len(), sent[1].len(), sent[2].len()], [192, 24, 3]);
    assert_eq!(sent[0][..3], [0, 4, 8]);
    assert_eq!(sent[1][..3], [2, 0x22, 0x42]);
    assert_eq!(sent[2], [1, 0x101, 0x201]);
}

/// Guards the player record's bytes as the core builds them for this machine's player: the
/// state, car, colours, nationality, stage, direction and race number; the default stage
/// (0x29, direction 0) when none is chosen; the host's bit.
#[test]
fn the_players_record_bytes_are_frozen() {
    assert_eq!(
        record::own_record(0x0B, 0x22, 0x33, 0x14, 1, 0x0C),
        [0, 0x0B, 0x22, 0x33, 0x14, 1, 0x0C]
    );
    assert_eq!(
        record::own_record(0x0B, 0x22, 0x33, 0, 1, 0x0C),
        [0, 0x0B, 0x22, 0x33, 0x29, 0, 0x0C]
    );
    assert_eq!(record::DEFAULT_STAGE, 0x29);
    assert!(record::is_host(0x10) && !record::is_host(0x0F));
    // The offsets of the record's fields.
    assert_eq!(
        [
            record::STATE,
            record::CAR,
            record::COLOURS,
            record::NATIONALITY,
            record::STAGE,
            record::DIRECTION,
            record::RACE_NUMBER,
            record::RANDOM
        ],
        [0, 1, 2, 3, 4, 5, 6, 7]
    );
    // Read back as a seat: from the host's record the stage (low seven bits) and direction.
    let bytes = [0x10, 0x0B, 0x22, 0x33, 0x94, 3, 0x0C, 0];
    let seat = record::seat(|n| bytes[n]);
    assert_eq!(
        (seat.car, seat.colours, seat.race_number, seat.stage),
        (0x0B, 0x22, 0x0C, Some((0x14, 1)))
    );
}

/// Guards a compound address's element: the data type's GUID, the data's size and the data's
/// address as little-endian dwords, 0x18 bytes; and the sizes of the elements TCP/IP's address
/// holds (provider GUID 16, host text and its NUL, port 2).
#[test]
fn an_address_element_is_twenty_four_bytes() {
    use crate::net::address;
    let e = address::element(&[0xC0; 16], 0x0A, 0x0040_1000);
    let mut expected = vec![0xC0; 16];
    expected.extend([0x0A, 0, 0, 0, 0x00, 0x10, 0x40, 0]);
    assert_eq!(e.as_slice(), expected.as_slice());
    assert_eq!(address::ELEMENT_LEN, 0x18);
    assert_eq!(address::host_address_size(9), 10);
    assert_eq!((address::PROVIDER_SIZE, address::PORT_SIZE), (16, 2));
}
