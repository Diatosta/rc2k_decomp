//! Frozen bytes of the network messages the game side builds and reads, on state of the tests'
//! own (a boxed `Net`) or restored after: each case builds from fixed inputs, checks the exact
//! bytes, and reads them back.

use crate::net::apply_message::{apply_message, place, wanted};
use crate::net::owner::Net;
use crate::net::send_inputs::{Message, fields, send_fields};
use crate::net::send_message::send_message;
use crate::net::store::PlayerRecord;
use rc2k_core::net::race::Kind;

fn dwords(words: &[u32]) -> Vec<u8> {
    words.iter().flat_map(|w| w.to_le_bytes()).collect()
}

/// A hosted DirectPlay object for `net` to send through (the messages go to its own player).
fn live(net: &Net) {
    let (object, player, _) = crate::net::dp_objects::hosted_for_tests();
    net.link.set_dplay(object);
    net.link.set_player_id(player);
    net.link.set_send_to(player);
}

fn sent(net: &Net, len: usize) -> Vec<u8> {
    net.lobby()
        .send_buffer
        .read_at(0, len, "the send buffer")
        .unwrap()
}

/// Guards the framing of a message sent to everyone: the sequence byte goes first (one more
/// than the last, top bit set, wrapping from 0xFF to 0x80), then the bytes; one that fills the
/// 0x100-byte buffer is refused and writes nothing.
#[test]
fn a_message_is_sent_as_its_sequence_byte_and_its_bytes() {
    let _lock = crate::test_support::test_lock();
    let net: Box<Net> = Box::new(Net::new());
    live(&net);
    assert!(send_message(&net, b"abc").is_ok());
    assert_eq!(sent(&net, 4), [0x81, b'a', b'b', b'c']);
    assert!(send_message(&net, &[0x22, b'h']).is_ok());
    assert_eq!(sent(&net, 3), [0x82, 0x22, b'h']);
    net.lobby().sequence.set(0xFF);
    assert!(send_message(&net, &[7]).is_ok());
    assert_eq!(sent(&net, 2), [0x80, 7]);
    assert_eq!(net.lobby().sequence.get(), 0x80);
    assert!(send_message(&net, &[9; 0x100]).is_err());
    assert_eq!(sent(&net, 2), [0x80, 7]);
    assert!(send_message(&net, &[9; 0xFF]).is_ok());
    assert_eq!(net.link.release_dplay(), Some(0));
}

/// The work car and car 2, as they are, to put back.
fn snapshot() -> (
    crate::physics::car_store::Car,
    Option<crate::physics::car_store::Car>,
) {
    let race = crate::test_support::own::race();
    (
        race.shared.car().clone(),
        race.shared.race_car(2).ok().map(|c| c.clone()),
    )
}

fn restore(
    saved: &(
        crate::physics::car_store::Car,
        Option<crate::physics::car_store::Car>,
    ),
) {
    let race = crate::test_support::own::race();
    *race.shared.car_mut() = saved.0.clone();
    if let Some(car) = &saved.1
        && let Ok(mut block) = race.shared.race_car_mut(2)
    {
        *block = car.clone();
    }
}

/// FNV-1a of `bytes`.
fn hash(bytes: &[u8]) -> u64 {
    crate::test_support::hash(bytes)
}

/// Guards the three race messages as sent: the tag dword ('JGSM', 'PGSM', 'XGSM'), then the
/// car's dwords in the table's order (here dword n of a kind holds `0x1000 + n`), behind the
/// sequence byte. The 'JGSM' and 'XGSM' bytes are literal; the 'PGSM' one is frozen as a hash.
#[test]
fn the_race_messages_are_built_from_the_car() {
    let _lock = crate::test_support::test_lock();
    let saved = snapshot();
    let net: Box<Net> = Box::new(Net::new());
    live(&net);
    let mut built = Vec::new();
    for (kind, text) in [
        (Kind::Inputs, c"send j"),
        (Kind::Position, c"send p"),
        (Kind::Extra, c"send x"),
    ] {
        let mut car = crate::test_support::own::race().shared.car_mut();
        for (n, (field, _)) in fields(kind).iter().enumerate() {
            field(&mut car).set_bits(0x1000 + n as u32);
        }
        drop(car);
        let m = Message { text, kind };
        assert!(send_fields(crate::test_support::own::race(), &net, &m).is_ok());
        let len = 1 + 4 * (1 + fields(kind).len());
        built.push(sent(&net, len));
    }
    restore(&saved);
    assert_eq!(net.link.release_dplay(), Some(0));
    assert_eq!(
        built[0],
        [
            vec![0x81],
            dwords(&[0x4D53_474A, 0x1000, 0x1001, 0x1002, 0x1003])
        ]
        .concat()
    );
    assert_eq!(built[0][1..5], *b"JGSM");
    assert_eq!(built[1][..5], [0x82, b'P', b'G', b'S', b'M']);
    assert_eq!(built[1].len(), 1 + 4 * 41);
    assert_eq!(built[1][5..9], 0x1000u32.to_le_bytes());
    assert_eq!(built[1][5 + 4 * 39..], 0x1027u32.to_le_bytes());
    let extra: Vec<u32> = (0..9).map(|n| 0x1000 + n).collect();
    assert_eq!(
        built[2],
        [vec![0x83], dwords(&[0x4D53_4758]), dwords(&extra)].concat()
    );
    assert_eq!(hash(&built[1]), POSITION_HASH);
}

/// The 'PGSM' message of `the_race_messages_are_built_from_the_car`.
const POSITION_HASH: u64 = 10_747_184_649_910_218_868;

/// Guards what a received race message does to a car: 'JGSM' sets the four input words, 'XGSM'
/// the nine extra dwords, 'PGSM' glides to a place under ten metres off (the 32nd of the way
/// kept, 32 steps armed) and jumps to a far one after the first thousand steps; a message with a
/// value no car holds, an unknown tag, and a player past the eighth change nothing.
#[test]
fn a_received_message_is_put_into_the_senders_car() {
    let _lock = crate::test_support::test_lock();
    let r = crate::test_support::own::race();
    let net = crate::test_support::own::net();
    if r.shared.race_car(2).is_err() {
        crate::program_log::log("SKIPPED a_received_message_is_put_into_the_senders_car: no car 2");
        return;
    }
    let saved = snapshot();
    let bits = |kind: Kind| -> Vec<u32> {
        let mut own = r.shared.race_car_mut(2).expect("car 2");
        fields(kind)
            .iter()
            .map(|(f, _)| f(&mut own).bits())
            .collect()
    };

    let inputs = dwords(&[0x4D53_474A, 11, 12, 13, 14]);
    assert!(apply_message(r, net, 2, &[vec![0x81], inputs.clone()].concat()[1..]).is_ok());
    assert_eq!(bits(Kind::Inputs), [11, 12, 13, 14]);

    let extras: Vec<u32> = (0..9).map(|n| 0x3F00_0000 + n).collect();
    let message = [dwords(&[0x4D53_4758]), dwords(&extras)].concat();
    assert!(apply_message(r, net, 2, &message).is_ok());
    assert_eq!(bits(Kind::Extra), extras);

    // A place 3 metres off (units of 1/65536 metre), the steps past 1000: glided to.
    let metre = 65536u32;
    {
        let mut own = r.shared.race_car_mut(2).expect("car 2");
        own.step_clock = 5000;
        own.position = ([1000 * metre, 0, 1000 * metre]).map(f32::from_bits);
    }
    let position = |at: [u32; 3]| -> Vec<u8> {
        let values: Vec<u32> = (0..40)
            .map(|n| match n {
                0..=2 => at[n as usize],
                26 => 3,
                3..=14 => 0,
                _ => 0x10,
            })
            .collect();
        [dwords(&[0x4D53_4750]), dwords(&values)].concat()
    };
    let near = [1000 * metre + 3 * metre, 0, 1000 * metre];
    assert!(apply_message(r, net, 2, &position(near)).is_ok());
    let own_car = || r.shared.race_car(2).expect("car 2");
    assert_eq!(own_car().glide_steps_left, 0x20);
    assert_eq!(
        wanted(&own_car()).map(f32::to_bits),
        [(3 * metre) >> 5, 0, 0]
    );
    // A place 11 metres off: jumped to, the glide ended.
    let far = [1000 * metre + 11 * metre, 0, 1000 * metre];
    assert!(apply_message(r, net, 2, &position(far)).is_ok());
    assert_eq!(place(&own_car()).map(f32::to_bits), far);
    assert_eq!(own_car().glide_steps_left, 0);
    // Gear 9 is no gear: nothing is applied.
    let mut bad = position(near);
    bad[4 + 4 * 26..][..4].copy_from_slice(&9u32.to_le_bytes());
    {
        let mut own = r.shared.race_car_mut(2).expect("car 2");
        own.step_clock = 5000;
        own.glide_steps_left = 0;
    }
    assert!(apply_message(r, net, 2, &bad).is_ok());
    assert_eq!(own_car().glide_steps_left, 0);
    // An unknown tag and a player past the eighth change nothing.
    let before = bits(Kind::Inputs);
    assert!(apply_message(r, net, 2, &dwords(&[0x4D53_474B, 1, 2, 3, 4])).is_ok());
    assert!(apply_message(r, net, 8, &inputs).is_ok());
    assert_eq!(bits(Kind::Inputs), before);
    restore(&saved);
}

/// Guards the player record's 16 bytes (DirectPlay's player data): the head's eight bytes, the
/// race time and the total, little-endian; and that `set_bytes` reads them back (a shorter
/// slice sets only its part).
#[test]
fn the_player_record_is_sixteen_bytes() {
    let mut record = PlayerRecord::zero();
    let bytes: [u8; 16] = [
        0x10, 0x0B, 0x22, 0x33, 0x14, 1, 0x0C, 0x5A, 0x78, 0x56, 0x34, 0x12, 0xEF, 0xCD, 0xAB, 0x90,
    ];
    record.set_bytes(&bytes);
    assert_eq!(record.bytes(), bytes);
    assert_eq!(record.state(), 0x10);
    assert_eq!(record.race_time(), 0x1234_5678);
    assert_eq!(record.total(), 0x90AB_CDEF);
    record.set_bytes(&[0, 0xFF]);
    assert_eq!(record.bytes()[..3], [0, 0xFF, 0x22]);
    assert_eq!(record.bytes()[8..], bytes[8..]);
}

/// Guards the options file's network fields in their places: the machine's name (0x40 bytes),
/// the most players (a dword), the host's address (0x40), the port (a dword) and the saved
/// connection's GUID (0x10), in this order and width, as pieces 12 to 16 of the file.
#[test]
fn the_options_network_fields_have_their_bytes() {
    let _lock = crate::test_support::test_lock();
    let pieces = crate::saves::options_state::pieces();
    let ours = &pieces[12..=16];
    let before: Vec<Vec<u8>> = ours.iter().map(|p| p.read().unwrap()).collect();
    let text = |s: &[u8], n: usize| {
        let mut v = s.to_vec();
        v.resize(n, 0);
        v
    };
    let wanted: Vec<Vec<u8>> = vec![
        text(b"Machine", 0x40),
        4u32.to_le_bytes().to_vec(),
        text(b"192.168.1.5", 0x40),
        47624u32.to_le_bytes().to_vec(),
        (0..16).map(|n| 0xA0 + n).collect(),
    ];
    for (piece, bytes) in ours.iter().zip(&wanted) {
        piece.write(bytes).unwrap();
    }
    let net = crate::test_support::own::net();
    let read: Vec<Vec<u8>> = ours.iter().map(|p| p.read().unwrap()).collect();
    let (name, players, host, port, guid) = (
        net.setup().machine_name.read(),
        net.setup().most_players.get(),
        net.setup().host_address.read(),
        net.setup().port.get(),
        net.setup().saved_connection.read(),
    );
    for (piece, bytes) in ours.iter().zip(&before) {
        piece.write(bytes).unwrap();
    }
    assert_eq!(read, wanted);
    assert_eq!(name.as_slice(), wanted[0].as_slice());
    assert_eq!((players, port), (4, 47624));
    assert_eq!(host.as_slice(), wanted[2].as_slice());
    assert_eq!(guid.as_slice(), wanted[4].as_slice());
}
